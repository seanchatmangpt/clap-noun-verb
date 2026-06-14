# Release Automation & Deployment Guide

**Phase**: Phase 6 (final phase - after all other work)
**Duration**: 1-2 days
**Goal**: Automated releases with proper versioning, testing, and distribution
**Status**: Complete implementation guide with scripts

---

## Release Strategy Overview

```
Feature Development
    ↓
Commit to main/develop
    ↓
[Automated Testing via CI]
    ↓
Feature Merged to main
    ↓
Tag Release (v1.2.3)
    ↓
[GitHub Actions Release Workflow]
    ├→ Run full CI suite
    ├→ Build release artifacts
    ├→ Create release notes
    ├→ Publish to crates.io
    ├→ Generate binaries (macOS, Linux, Windows)
    └→ Publish to package managers (Homebrew, etc.)
    ↓
Release Published
    ↓
Post-Release Monitoring
```

---

## Version Numbering Strategy

**Semantic Versioning**: MAJOR.MINOR.PATCH (X.Y.Z)

| Type | When | Example | Changelog |
|------|------|---------|-----------|
| **MAJOR** | Breaking changes (API/CLI) | v1.0.0 → v2.0.0 | Migration guide required |
| **MINOR** | New features (backward compatible) | v1.0.0 → v1.1.0 | Feature highlights |
| **PATCH** | Bug fixes, perf improvements | v1.0.0 → v1.0.1 | Fix descriptions |

**Current Version**: v0.2.0
**Next Release**: v0.3.0 (minor feature release)
**Pre-Release**: Use alpha.N, beta.N, rc.N for pre-releases

---

## 1. GitHub Actions Release Workflow

**File**: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
      version: ${{ env.VERSION }}
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Extract version
        run: |
          VERSION=$(echo ${GITHUB_REF#refs/tags/v})
          echo "VERSION=$VERSION" >> $GITHUB_ENV

      - name: Generate changelog
        run: |
          git log v$(git describe --tags --abbrev=0 HEAD^)..HEAD --oneline > /tmp/changelog.txt
          echo "## Changes in v$VERSION" > /tmp/release_notes.md
          cat /tmp/changelog.txt >> /tmp/release_notes.md

      - name: Create GitHub Release
        id: create_release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            target/release/htf
            target/release/htf.exe
          body_path: /tmp/release_notes.md
          draft: false
          prerelease: ${{ contains(env.VERSION, 'alpha') || contains(env.VERSION, 'beta') || contains(env.VERSION, 'rc') }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  test-release:
    name: Test Release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build release
        run: cargo build --release

      - name: Run tests
        run: cargo test --all --release

      - name: Run clippy
        run: cargo clippy --all -- -D warnings

      - name: Security audit
        run: cargo audit

  build-binaries:
    name: Build ${{ matrix.target }}
    needs: test-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: htf-linux-x64

          - target: x86_64-apple-darwin
            os: macos-latest
            name: htf-macos-intel

          - target: aarch64-apple-darwin
            os: macos-latest
            name: htf-macos-arm64

          - target: x86_64-pc-windows-msvc
            os: windows-latest
            name: htf-windows-x64

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Create checksum
        run: |
          cd target/${{ matrix.target }}/release
          if [ "$OS" = "Windows" ]; then
            certutil -hashfile htf.exe SHA256 > htf.sha256
          else
            sha256sum htf > htf.sha256
          fi

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.name }}
          path: target/${{ matrix.target }}/release/htf*

  publish-crates:
    name: Publish to crates.io
    needs: build-binaries
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Publish to crates.io
        run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}

  publish-homebrew:
    name: Publish to Homebrew
    needs: publish-crates
    runs-on: ubuntu-latest
    steps:
      - name: Trigger Homebrew formula update
        run: |
          curl -X POST \
            https://api.github.com/repos/seanchatmangpt/homebrew-playground/dispatches \
            -H "Accept: application/vnd.github.v3+raw" \
            -H "Authorization: token ${{ secrets.HOMEBREW_TOKEN }}" \
            -d '{"event_type":"release","client_payload":{"version":"'${GITHUB_REF#refs/tags/v}'"}}'
```

---

## 2. Release Script

**File**: `scripts/release.sh`

```bash
#!/bin/bash
set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
MAIN_BRANCH="main"
DEVELOP_BRANCH="develop"

# Functions
print_help() {
    cat <<EOF
Usage: ./scripts/release.sh [COMMAND] [OPTIONS]

Commands:
  start <version>     Start new release (e.g., 0.3.0)
  finish              Finish release and create tag
  publish             Publish to all channels
  hotfix <version>    Create hotfix release (e.g., 0.2.1)
  rollback <version>  Rollback release

Options:
  -h, --help         Show this help message
EOF
}

log_info() {
    echo -e "${GREEN}ℹ${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Validate version format
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?$ ]]; then
        log_error "Invalid version format: $version. Use: X.Y.Z or X.Y.Z-alpha.N"
    fi
}

# Check git status
check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Git working directory is not clean. Commit or stash changes."
    fi
}

# Update version in files
update_version() {
    local new_version=$1
    log_info "Updating version to $new_version..."

    # Update Cargo.toml
    sed -i "" "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml

    # Update examples and docs
    sed -i "" "s/playground = \".*\"/playground = \"$new_version\"/" examples/*.rs 2>/dev/null || true

    log_info "Version updated in Cargo.toml"
}

# Create release commit and tag
create_release_tag() {
    local version=$1

    git add Cargo.toml examples/ docs/CHANGELOG.md 2>/dev/null || true
    git commit -m "chore: bump version to v$version"

    git tag -a "v$version" -m "Release v$version"

    log_info "Created tag: v$version"
}

# Start new release
start_release() {
    local version=$1

    validate_version "$version"
    check_git_status

    # Switch to develop branch
    git checkout "$DEVELOP_BRANCH"
    git pull origin "$DEVELOP_BRANCH"

    log_info "Starting release v$version from $DEVELOP_BRANCH..."

    # Create release branch
    git checkout -b "release/v$version"

    # Update version
    update_version "$version"

    # Commit version bump
    git add Cargo.toml
    git commit -m "chore: start release v$version"
    git push origin "release/v$version"

    log_info "Release branch created: release/v$version"
    log_info "Next: Make any final fixes, then run: $0 finish"
}

# Finish release
finish_release() {
    check_git_status

    # Get current branch
    current_branch=$(git rev-parse --abbrev-ref HEAD)
    if [[ ! $current_branch =~ ^release/v ]]; then
        log_error "Not on release branch. Current: $current_branch"
    fi

    # Extract version from branch name
    version=${current_branch#release/v}

    log_info "Finishing release v$version..."

    # Merge to main
    git checkout "$MAIN_BRANCH"
    git pull origin "$MAIN_BRANCH"
    git merge --no-ff "release/v$version" -m "Merge release/v$version into main"

    # Create release tag
    create_release_tag "$version"

    # Merge back to develop
    git checkout "$DEVELOP_BRANCH"
    git pull origin "$DEVELOP_BRANCH"
    git merge --no-ff "release/v$version" -m "Merge release/v$version back into develop"

    # Delete release branch
    git branch -d "release/v$version"
    git push origin --delete "release/v$version"

    # Push everything
    git push origin "$MAIN_BRANCH" "$DEVELOP_BRANCH" "v$version"

    log_info "Release v$version completed!"
    log_info "GitHub Actions will now build and publish artifacts."
}

# Main script
case "${1:-}" in
    start)
        start_release "${2:-}"
        ;;
    finish)
        finish_release
        ;;
    publish)
        log_info "Publishing is automated via GitHub Actions."
        log_info "Create a tag: git tag -a v<version> && git push origin v<version>"
        ;;
    hotfix)
        start_release "${2:-}"
        ;;
    rollback)
        version="${2:-}"
        if [[ -z $version ]]; then
            log_error "Version required for rollback"
        fi
        log_warn "Rolling back to v$version..."
        git checkout "v$version"
        # Further rollback steps...
        ;;
    -h|--help|help)
        print_help
        ;;
    *)
        print_help
        exit 1
        ;;
esac
```

Make executable:
```bash
chmod +x scripts/release.sh
```

---

## 3. Changelog Generation

**File**: `.github/cliff.toml` (git-cliff configuration)

```toml
# git-cliff configuration for automated changelog generation

[changelog]
# changelog header
header = """
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
"""

# changelog body
body = """
{% if version %}\
    ## [{{ version }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
    ## [Unreleased]
{% endif %}\
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | upper }}
    {% for commit in commits %}
        - {{ commit.message | upper_first }} ([`{{ commit.id | truncate(length=7, end="") }}`]({{ commit.link }})) - ({{ commit.author }})
    {%- endfor %}
{% endfor %}\n
"""

# parse commit messages
[git]
conventional_commits = true
filter_unconventional = true
split_commits = false
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^doc", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactoring" },
    { message = "^test", group = "Testing" },
    { message = "^chore", group = "Miscellaneous Tasks" },
]
```

---

## 4. Dependabot Configuration

**File**: `.github/dependabot.yml`

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: weekly
      day: monday
      time: "03:00"
    open-pull-requests-limit: 5
    allow:
      - dependency-type: "direct"
      - dependency-type: "indirect"
    reviewers:
      - "seanchatmangpt"
    labels:
      - "dependencies"
      - "rust"
    commit-message:
      prefix: "chore"
      include: "scope"

  - package-ecosystem: github-actions
    directory: "/"
    schedule:
      interval: weekly
    reviewers:
      - "seanchatmangpt"
    labels:
      - "ci"
```

---

## 5. Release Checklist

Create `scripts/release_checklist.md`:

```markdown
# Release Checklist

## Pre-Release (1 day before)

- [ ] All PRs merged to main
- [ ] CI/CD passing on main
- [ ] Version number decided (X.Y.Z)
- [ ] Changelog prepared
- [ ] Documentation updated
- [ ] Breaking changes documented
- [ ] Performance SLOs verified
- [ ] Security audit passed
- [ ] Changelog entry written

## Release Day

- [ ] Create release branch: `git checkout -b release/vX.Y.Z`
- [ ] Update version in Cargo.toml
- [ ] Generate changelog: `git-cliff --output CHANGELOG.md`
- [ ] Commit: `git commit -am "chore: v X.Y.Z"`
- [ ] Tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
- [ ] Push: `git push origin --all --tags`

## GitHub Actions Automation

- [ ] Wait for GitHub Actions to complete (usually 5-10 min)
- [ ] Verify release created on GitHub
- [ ] Verify binaries uploaded
- [ ] Verify crates.io updated
- [ ] Verify Homebrew PR created

## Post-Release

- [ ] Verify crates.io package page
- [ ] Test installation: `cargo install playground`
- [ ] Announce release on Twitter/Discord
- [ ] Monitor GitHub issues for problems
- [ ] Monitor analytics for adoption
- [ ] Archive release artifacts

## Rollback (if needed)

- [ ] Delete GitHub release
- [ ] Yank crate from crates.io: `cargo yank --vers X.Y.Z`
- [ ] Delete git tag: `git tag -d vX.Y.Z && git push origin :vX.Y.Z`
- [ ] Notify users
```

---

## 6. Rollback Procedures

### If Critical Bug Found Post-Release

```bash
# 1. Yank from crates.io
cargo yank --vers 0.3.0

# 2. Delete GitHub release
gh release delete v0.3.0

# 3. Delete git tag
git tag -d v0.3.0
git push origin :v0.3.0

# 4. Create hotfix
./scripts/release.sh hotfix 0.2.1
# Fix bug on hotfix branch
# Run: ./scripts/release.sh finish

# 5. Notify users
# Post on GitHub, Twitter, Discord
```

---

## 7. Version Support Policy

```
Version | Release Date | Support Until | Type
--------|-------------|---|----
0.1.x   | 2024-Q1    | EOL           | Alpha
0.2.x   | 2024-Q2    | 6 months      | Beta
0.3.x   | 2024-Q3    | 12 months     | Current
1.0.x   | 2024-Q4    | 24 months     | LTS*
1.1.x   | 2025-Q1    | 12 months     | Current

* LTS: Long-term support with critical patches only
```

---

## 8. Distribution Channels

### Channel 1: Crates.io (Primary)

Already handled by GitHub Actions. Published automatically.

### Channel 2: Homebrew (macOS)

**Setup** (one-time):
1. Create tap repository: `homebrew-playground`
2. Create formula: `homebrew-playground/playground.rb`

```ruby
class Playground < Formula
  desc "Language-agnostic code generation CLI"
  homepage "https://github.com/seanchatmangpt/clap-noun-verb"
  url "https://github.com/seanchatmangpt/clap-noun-verb/releases/download/v#{version}/htf-macos-arm64"
  version "0.3.0"
  sha256 "abc123..."

  def install
    bin.install "htf"
  end

  test do
    system "#{bin}/htf", "--version"
  end
end
```

### Channel 3: Binary Releases

GitHub Actions automatically creates binaries for:
- Linux (x64)
- macOS (Intel + ARM)
- Windows (x64)

Available at: `https://github.com/seanchatmangpt/clap-noun-verb/releases`

---

## 9. Monitoring Post-Release

Create `scripts/post_release_monitor.sh`:

```bash
#!/bin/bash

# Monitor release metrics for 7 days

echo "Post-Release Monitoring"
echo ""

# Check crates.io downloads
echo "Crates.io Downloads:"
curl -s https://crates.io/api/v1/crates/playground | jq '.crate.downloads'

# Check GitHub releases
echo ""
echo "GitHub Release Assets:"
gh release view --json assets -q '.assets[].name'

# Check for critical issues
echo ""
echo "Critical Issues (past 24h):"
gh issue list --label "critical" --state open --created=">=1 day ago"

# Check downloads by version
echo ""
echo "Version Distribution:"
cargo yank --vers 0.2.9 --undo 2>/dev/null || true  # Don't actually yank
# Show download stats
curl -s https://crates.io/api/v1/crates/playground/0.3.0/downloads | jq

echo ""
echo "Monitor GitHub discussions for feedback"
```

---

## 10. Implementation Checklist

- [ ] **GitHub Actions Workflow**
  - [ ] Create `.github/workflows/release.yml`
  - [ ] Configure secrets (CARGO_TOKEN, HOMEBREW_TOKEN)
  - [ ] Test with dry-run tag
  - [ ] Verify all steps execute

- [ ] **Release Script**
  - [ ] Create `scripts/release.sh`
  - [ ] Test: `./scripts/release.sh start 0.3.0`
  - [ ] Test: `./scripts/release.sh finish`
  - [ ] Verify tag created and pushed

- [ ] **Changelog Automation**
  - [ ] Create `.github/cliff.toml`
  - [ ] Install git-cliff: `cargo install git-cliff`
  - [ ] Generate sample: `git-cliff`

- [ ] **Dependabot**
  - [ ] Create `.github/dependabot.yml`
  - [ ] Enable in GitHub settings
  - [ ] Configure PR labels and reviewers

- [ ] **Distribution Channels**
  - [ ] Publish to crates.io
  - [ ] Set up Homebrew tap
  - [ ] Create binary download links
  - [ ] Document installation methods

- [ ] **Post-Release**
  - [ ] Create release checklist
  - [ ] Create post-release monitor
  - [ ] Document rollback procedures
  - [ ] Set up alerts for critical issues

---

## Full Release Flow Example

```bash
# 1. Prepare release
./scripts/release.sh start 0.3.0

# 2. Make any final fixes on release branch
git commit -m "fix: last-minute fix"

# 3. Finish release
./scripts/release.sh finish

# 4. GitHub Actions automatically:
#    - Runs all tests
#    - Builds binaries (Linux, macOS, Windows)
#    - Creates GitHub Release
#    - Publishes to crates.io
#    - Creates Homebrew PR

# 5. Post-release
# - Announce on Twitter/Discord
# - Monitor GitHub for issues
# - Check crates.io stats
```

---

## Expected Timeline

```
T+0:   Tag pushed → GitHub Actions starts
T+2min:  Full CI suite runs (test + clippy + audit)
T+5min:  Binaries built for 4 platforms
T+8min:  GitHub Release created with binaries
T+10min: Crates.io publishes (may take 1-2 min to appear)
T+15min: Homebrew PR created (requires manual approval)
T+30min: Release available on all channels
```

---

## Success Criteria

- [ ] Automated release workflow configured
- [ ] All 4 distribution channels working
- [ ] Release can be created with single command
- [ ] Binaries available for all platforms
- [ ] Changelog generated automatically
- [ ] Dependencies automatically updated
- [ ] Post-release monitoring in place
- [ ] Rollback procedures documented

---

**Status**: Release automation guide complete
**Next**: Execute Phase 6 automation setup
**Expected Duration**: 1-2 days

