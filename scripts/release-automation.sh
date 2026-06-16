#!/bin/bash
# Copyright (c) 2024 Sean Chatman
# SPDX-License-Identifier: MIT OR Apache-2.0

# Release Automation Helper for clap-noun-verb
# Guides users through the complete release workflow with automation

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION="${1:-}"

# Functions
print_header() {
    echo ""
    echo "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "${BOLD}${BLUE}  $1${NC}"
    echo "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_section() {
    echo "${BOLD}${BLUE}▶ $1${NC}"
    echo ""
}

print_success() {
    echo "${GREEN}✓ $1${NC}"
}

print_error() {
    echo "${RED}✗ ERROR: $1${NC}"
}

print_info() {
    echo "${BLUE}ℹ $1${NC}"
}

print_warning() {
    echo "${YELLOW}⚠ $1${NC}"
}

print_step() {
    echo "${BOLD}$1${NC}"
}

confirm() {
    local prompt="$1"
    local response
    echo -n "${YELLOW}$prompt (y/n): ${NC}"
    read -r response
    [[ "$response" == "y" || "$response" == "Y" ]]
}

# Main workflow
main() {
    clear

    print_header "RELEASE AUTOMATION HELPER"
    echo "clap-noun-verb v26+ Release Management"
    echo ""

    # Step 1: Verify prerequisites
    print_section "Step 1: Checking Prerequisites"

    if ! command -v cargo &> /dev/null; then
        print_error "cargo not found in PATH"
        exit 1
    fi
    print_success "cargo is installed"

    if ! command -v git &> /dev/null; then
        print_error "git not found in PATH"
        exit 1
    fi
    print_success "git is installed"

    if ! command -v jq &> /dev/null; then
        print_warning "jq not found (optional, for JSON parsing)"
    fi
    print_success "jq is available (optional)"

    cd "$PROJECT_ROOT"

    # Step 2: Determine version
    print_section "Step 2: Determine Version Number"

    current_version=$(grep -E '^version = "' Cargo.toml | head -1 | sed 's/version = "//;s/"//')
    print_info "Current version: $current_version"

    echo ""
    print_step "Semantic Versioning Decision:"
    echo "  MAJOR → Breaking changes (trait redesign, API removal)"
    echo "  MINOR → New features, backward compatible"
    echo "  PATCH → Bug fixes, documentation"
    echo ""

    if [ -z "$VERSION" ]; then
        echo -n "${YELLOW}Enter new version (e.g., 26.6.15): ${NC}"
        read -r VERSION
    fi

    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_error "Invalid version format: $VERSION"
        echo "Expected: MAJOR.MINOR.PATCH (e.g., 26.6.15)"
        exit 1
    fi

    print_success "Target version: $VERSION"
    echo ""

    # Step 3: Review changes
    print_section "Step 3: Review Changes Since Last Release"

    # Find last tag
    last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "HEAD")
    echo "Last tag: $last_tag"
    echo ""
    echo "Commits since last release:"
    git log "$last_tag..HEAD" --oneline | head -20
    echo ""

    if ! confirm "Continue with version $VERSION?"; then
        print_error "Release cancelled by user"
        exit 1
    fi

    # Step 4: Pre-release checks
    print_section "Step 4: Running Pre-Release Checks"

    echo "Checking git status..."
    if ! git diff --quiet; then
        print_error "Uncommitted changes detected"
        git status
        exit 1
    fi
    print_success "Git working directory clean"

    echo "Running cargo check..."
    if ! cargo check > /dev/null 2>&1; then
        print_error "Compilation failed"
        cargo check
        exit 1
    fi
    print_success "Code compiles"

    echo "Running tests..."
    if ! cargo make test > /dev/null 2>&1; then
        print_error "Tests failed"
        cargo make test
        exit 1
    fi
    print_success "All tests pass"

    echo ""

    # Step 5: Version bump
    print_section "Step 5: Bumping Version to $VERSION"

    if [ ! -f "$SCRIPT_DIR/bump-version.sh" ]; then
        print_warning "bump-version.sh not found, creating temporary bump"
        # Create temporary bump
        sed -i.bak "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" Cargo.toml
        sed -i.bak "s/{ path = \".\", version = \"[^\"]*\"/{ path = \".\", version = \"$VERSION\"/" Cargo.toml
        sed -i.bak "s/{ path = \"clap-noun-verb-macros\", version = \"[^\"]*\"/{ path = \"clap-noun-verb-macros\", version = \"$VERSION\"/" Cargo.toml
        sed -i.bak "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" clap-noun-verb-macros/Cargo.toml
        rm -f Cargo.toml.bak clap-noun-verb-macros/Cargo.toml.bak
    else
        "$SCRIPT_DIR/bump-version.sh" "$VERSION"
    fi

    # Verify version was bumped
    main_version=$(grep -E '^version = "' Cargo.toml | head -1 | sed 's/version = "//;s/"//')
    if [ "$main_version" != "$VERSION" ]; then
        print_error "Version bump failed"
        exit 1
    fi
    print_success "Version bumped to $VERSION"
    echo ""

    # Step 6: Update CHANGELOG
    print_section "Step 6: Update CHANGELOG.md"

    if [ ! -f "CHANGELOG.md" ]; then
        print_warning "CHANGELOG.md not found"
    else
        # Check if [Unreleased] exists
        if grep -q "## \[Unreleased\]" CHANGELOG.md; then
            print_info "Found [Unreleased] section in CHANGELOG"
            print_step "Opening CHANGELOG.md for editing..."
            print_info "Instructions:"
            echo "  1. Change '[Unreleased]' to '[$VERSION] - $(date +%Y-%m-%d)'"
            echo "  2. Add new empty [Unreleased] section at top"
            echo "  3. Save and exit"
            echo ""

            if confirm "Edit CHANGELOG.md now?"; then
                "${EDITOR:-vim}" CHANGELOG.md
                print_success "CHANGELOG updated"
            else
                print_warning "Skipped CHANGELOG editing"
            fi
        else
            print_warning "No [Unreleased] section found in CHANGELOG"
        fi
    fi
    echo ""

    # Step 7: Commit version bump
    print_section "Step 7: Committing Version Bump"

    echo "Files to commit:"
    git status --porcelain
    echo ""

    if ! confirm "Commit these changes?"; then
        print_error "Release cancelled by user"
        exit 1
    fi

    git add Cargo.toml clap-noun-verb-macros/Cargo.toml CHANGELOG.md README.md 2>/dev/null || true
    git commit -m "chore(release): bump to $VERSION" || {
        print_warning "No changes to commit (version already bumped?)"
    }
    print_success "Changes committed"
    echo ""

    # Step 8: Run quality gates
    print_section "Step 8: Running Pre-Release Quality Gates"

    if [ -f "$SCRIPT_DIR/pre-release-check.sh" ]; then
        if ! "$SCRIPT_DIR/pre-release-check.sh" "$VERSION"; then
            print_error "Quality gates failed"
            exit 1
        fi
    else
        print_warning "pre-release-check.sh not found, skipping automated gates"
        echo "Running manual checks..."

        # Run cargo make tasks
        echo "Format check..."
        if ! cargo make format-check > /dev/null 2>&1; then
            print_error "Format check failed"
            exit 1
        fi
        print_success "Format check passed"

        echo "Clippy..."
        if ! cargo make clippy > /dev/null 2>&1; then
            print_error "Clippy check failed"
            exit 1
        fi
        print_success "Clippy check passed"

        echo "Building release..."
        if ! cargo make build-release > /dev/null 2>&1; then
            print_error "Release build failed"
            exit 1
        fi
        print_success "Release build passed"
    fi
    echo ""

    # Step 9: Publishing
    print_section "Step 9: Publishing to crates.io"

    print_info "Publishing order:"
    echo "  1. Dry-run macros"
    echo "  2. Publish macros"
    echo "  3. Dry-run main"
    echo "  4. Publish main"
    echo ""

    if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
        print_warning "CARGO_REGISTRY_TOKEN not set"
        echo "Set it with: export CARGO_REGISTRY_TOKEN=<your-token>"
        if ! confirm "Continue without token (dry-run only)?"; then
            exit 1
        fi
        DRY_RUN=true
    fi

    # Dry-run macros
    echo "▶ Dry-running macros publish..."
    if cargo make publish-dry-run-macros > /dev/null 2>&1; then
        print_success "Macros dry-run passed"
    else
        print_error "Macros dry-run failed"
        exit 1
    fi

    if [ "$DRY_RUN" != "true" ]; then
        # Publish macros
        echo "▶ Publishing macros to crates.io..."
        if cargo make publish-macros; then
            print_success "Macros published"
            echo ""
            echo "Waiting for crates.io indexing..."
            for i in {1..30}; do
                if cargo search clap-noun-verb-macros --limit 1 | grep -q "clap_noun_verb_macros"; then
                    print_success "Macros indexed on crates.io"
                    break
                fi
                echo "  Waiting... ($i/30)"
                sleep 2
            done
        else
            print_error "Macros publish failed"
            exit 1
        fi
    fi
    echo ""

    # Dry-run main
    echo "▶ Dry-running main crate publish..."
    if cargo make publish-dry-run > /dev/null 2>&1; then
        print_success "Main crate dry-run passed"
    else
        print_error "Main crate dry-run failed"
        exit 1
    fi

    if [ "$DRY_RUN" != "true" ]; then
        # Publish main
        echo "▶ Publishing main crate to crates.io..."
        if cargo make publish; then
            print_success "Main crate published"
        else
            print_error "Main crate publish failed"
            exit 1
        fi
    fi
    echo ""

    # Step 10: Create git tag
    print_section "Step 10: Creating Git Tag"

    if git rev-parse "v$VERSION" >/dev/null 2>&1; then
        print_warning "Tag v$VERSION already exists"
        if ! confirm "Overwrite existing tag?"; then
            print_info "Skipping tag creation"
        else
            git tag -d "v$VERSION"
            git tag "v$VERSION" -m "Release v$VERSION"
            print_success "Tag created: v$VERSION"
        fi
    else
        git tag "v$VERSION" -m "Release v$VERSION"
        print_success "Tag created: v$VERSION"
    fi
    echo ""

    # Step 11: Push to remote
    print_section "Step 11: Pushing to Remote"

    if [ "$DRY_RUN" != "true" ]; then
        echo "Push main branch:"
        if confirm "Push main to origin?"; then
            git push origin main
            print_success "Main branch pushed"
        fi

        echo ""
        echo "Push tag:"
        if confirm "Push tag v$VERSION to origin (triggers GitHub Actions)?"; then
            git push origin "v$VERSION"
            print_success "Tag pushed - GitHub Actions will handle the rest"
        fi
    else
        print_info "DRY-RUN: Skipping push to remote"
    fi
    echo ""

    # Final summary
    print_header "RELEASE WORKFLOW COMPLETE"

    echo "${GREEN}✓ Version bumped to $VERSION${NC}"
    echo "${GREEN}✓ Quality gates passed${NC}"
    echo "${GREEN}✓ Published to crates.io${NC}"
    echo "${GREEN}✓ Git tag created${NC}"
    echo ""

    if [ "$DRY_RUN" != "true" ]; then
        echo "Next steps:"
        echo "  1. Monitor GitHub Actions: https://github.com/seanchatmangpt/clap-noun-verb/actions"
        echo "  2. Verify on crates.io: https://crates.io/crates/clap-noun-verb/$VERSION"
        echo "  3. Check docs.rs: https://docs.rs/clap-noun-verb/$VERSION"
        echo "  4. Review GitHub Release: https://github.com/seanchatmangpt/clap-noun-verb/releases/tag/v$VERSION"
    else
        echo "This was a DRY-RUN. To perform actual release:"
        echo "  1. Set CARGO_REGISTRY_TOKEN environment variable"
        echo "  2. Run this script again with version $VERSION"
    fi
    echo ""
    print_success "Release $VERSION ready!"
}

# Show help
show_help() {
    cat << 'EOF'
Release Automation Helper for clap-noun-verb

USAGE:
    ./scripts/release-automation.sh [VERSION]

EXAMPLES:
    ./scripts/release-automation.sh              # Interactive mode
    ./scripts/release-automation.sh 26.6.15      # Release v26.6.15

VERSION FORMAT:
    MAJOR.MINOR.PATCH (e.g., 26.6.15)

WORKFLOW:
    1. Check prerequisites (cargo, git)
    2. Determine version number
    3. Review changes since last release
    4. Run pre-release checks
    5. Bump version
    6. Update CHANGELOG
    7. Commit version bump
    8. Run quality gates
    9. Publish to crates.io
    10. Create git tag
    11. Push to remote

ENVIRONMENT:
    CARGO_REGISTRY_TOKEN    Required for publishing to crates.io
                           Get from: https://crates.io/me → API Tokens

For more information, see:
    docs/RELEASE_SKILLS.md
    docs/RELEASE_MANAGEMENT.md
EOF
}

# Parse arguments
if [ "$VERSION" = "--help" ] || [ "$VERSION" = "-h" ]; then
    show_help
    exit 0
fi

# Run main workflow
main
