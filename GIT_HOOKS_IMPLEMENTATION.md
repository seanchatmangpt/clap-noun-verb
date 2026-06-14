# Git Hooks Implementation Guide

This document provides implementation details, code snippets, and best practices for the comprehensive git hooks in `clap-noun-verb`.

---

## Implementation Overview

The hooks system consists of:
1. **4 hook scripts** in `.githooks/` directory
2. **Installation helper** (`.githooks/install.sh`)
3. **Uninstall helper** (`.githooks/uninstall.sh`)
4. **Git configuration** (`core.hooksPath = .githooks`)

**Key design decisions**:
- Hooks are bash scripts (portable across Linux/macOS)
- Store in `.githooks/` (versioned with code)
- Use git config `core.hooksPath` (Git 2.9+)
- Color-coded output (human-friendly)
- Clear error messages with fix suggestions

---

## 1. Pre-Commit Hook Implementation

### File: `.githooks/pre-commit`

**Purpose**: Fast quality gates (fail fast on formatting/clippy)  
**Strategy**: Run only checks that take <2s total  
**Speed hierarchy**: Format (500ms) → Clippy (1s) → Deny (100ms) → Check (800ms)

### Key Implementation Features

#### 1.1 Fail-Fast Error Handling

```bash
set -e
# This causes the script to exit immediately on first error
# But we want to track all failures, so we use:
FAILED=0
# And check exit codes manually:
if ! command_here; then
    FAILED=1
fi
```

**Why**: Allows script to continue checking and report all failures, not just the first one.

#### 1.2 Color Output with Constants

```bash
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'  # No Color - always reset

echo -e "${RED}Error message${NC}"
```

**Why**: Makes output human-readable, easy to skim for errors/passes.

#### 1.3 Directory Detection

```bash
HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$HOOKS_DIR")"
```

**Why**: Works regardless of how hook is invoked (relative path, symlink, etc.)

#### 1.4 Optional Tool Handling

```bash
# Example: cargo-deny is optional
if [ -f "$PROJECT_ROOT/deny.toml" ]; then
    if command -v cargo-deny &> /dev/null; then
        if cargo deny check licenses advisories bans --log-level off >/dev/null 2>&1; then
            echo -e "${GREEN}✅ PASS: Dependency checks${NC}"
        else
            echo -e "${RED}❌ FAILED: Dependency check issues${NC}"
            FAILED=1
        fi
    else
        echo -e "${YELLOW}⚠️  SKIP: cargo-deny not installed (optional)${NC}"
        echo "   Install with: ${BLUE}cargo install cargo-deny${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  SKIP: No deny.toml configuration${NC}"
fi
```

**Why**: Allows optional tools without breaking the hook.

#### 1.5 Check Ordering

**Order is important for UX**:
1. **Format check first** (fastest, ~500ms) - catches 90% of issues
2. **Clippy next** (medium speed, ~1s) - catches logic errors
3. **Deny** (fast, ~100ms) - catches security issues
4. **Cargo check** (medium, ~800ms) - catches compilation errors
5. **Policy checks last** (very fast, ~50ms) - project-specific

**Rationale**: Fail fast on the most common issues.

#### 1.6 Check Numbering

```bash
echo "${BLUE}🔍 Check 1/5: Code formatting (rustfmt)${NC}"
# Later:
echo "${BLUE}🔍 Check 2/5: Linting (clippy)${NC}"
```

**Why**: Gives developers confidence something is happening; shows progress.

### Error Message Quality

Each failure includes:
1. **What failed**: "Code formatting issues found"
2. **How to fix**: `cargo make format`
3. **Why it matters**: (implicit in check name)

### Exit Behavior

```bash
if [ $FAILED -ne 0 ]; then
    echo -e "${RED}❌ PRE-COMMIT FAILED - Commit blocked${NC}"
    # ... show next steps and suggestions ...
    exit 1
else
    echo -e "${GREEN}✅ PRE-COMMIT PASSED - Ready for commit${NC}"
    # ... show next steps (full test suite in pre-push) ...
    exit 0
fi
```

---

## 2. Commit-Msg Hook Implementation

### File: `.githooks/commit-msg`

**Purpose**: Validate commit message format  
**Strategy**: Check length, capitalization, format (not content)

### Key Implementation Features

#### 2.1 Reading Commit Message

```bash
COMMIT_MSG_FILE="$1"
COMMIT_SOURCE="${2:-message}"

# Read the commit message file
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Trim leading/trailing whitespace
COMMIT_MSG=$(echo "$COMMIT_MSG" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')

# Extract first line for length/capitalization checks
FIRST_LINE=$(echo "$COMMIT_MSG" | head -n 1)
```

**Why**: Git passes message filename as arg, need to parse and trim for validation.

#### 2.2 Merge Commit Skip

```bash
if [ "$COMMIT_SOURCE" = "merge" ] || [ "$COMMIT_SOURCE" = "squash" ]; then
    echo -e "${YELLOW}⚠️  Skipping validation for merge commit${NC}"
    exit 0
fi

if echo "$FIRST_LINE" | grep -q "^Merge\|^Revert"; then
    echo -e "${YELLOW}⚠️  Skipping validation for merge/revert commit${NC}"
    exit 0
fi
```

**Why**: Merges and reverts have auto-generated messages; don't validate those.

#### 2.3 Length Validation

```bash
FIRST_LINE_LENGTH=${#FIRST_LINE}
if [ "$FIRST_LINE_LENGTH" -gt 72 ]; then
    echo -e "${RED}❌ FAILED: First line is $FIRST_LINE_LENGTH chars (max 72)${NC}"
    echo "   Current: $FIRST_LINE"
    FAILED=1
else
    echo -e "${GREEN}✅ PASS: Length is $FIRST_LINE_LENGTH chars${NC}"
fi
```

**Why**: 72 chars is GitHub's standard (fits in console, displays well in UI).

#### 2.4 Capitalization Check

```bash
if ! echo "$FIRST_LINE" | grep -q '^[A-Z]'; then
    echo -e "${RED}❌ FAILED: First character must be capitalized${NC}"
    FAILED=1
else
    echo -e "${GREEN}✅ PASS: First character is capitalized${NC}"
fi
```

**Why**: Consistency; looks professional in commit logs.

#### 2.5 Conventional Commit Detection

```bash
if echo "$FIRST_LINE" | grep -qE '^(feat|fix|refactor|perf|docs|test|chore|ci|style|revert)(\(.+\))?: .+'; then
    echo -e "${GREEN}✅ PASS: Valid conventional commit${NC}"
else
    echo -e "${YELLOW}ℹ️  INFO: Not conventional commit format${NC}"
    echo "   Recommended format: type(scope): description"
fi
```

**Why**: Conventional commits enable automatic changelog generation, but aren't mandatory.

#### 2.6 Issue Reference Detection

```bash
if echo "$COMMIT_MSG" | grep -qE '#[0-9]+|Fixes #|Closes #|Resolves #|Related to #'; then
    echo -e "${GREEN}✅ PASS: Contains issue/PR reference${NC}"
else
    echo -e "${YELLOW}ℹ️  INFO: No issue/PR reference found${NC}"
    echo "   Consider adding: Fixes #123 or Related to #456"
fi
```

**Why**: Best practice to link commits to issues for traceability.

### Severity Levels

```
FAILED=1  → Block commit (empty, >72 chars, uncapitalized)
WARNING   → Don't block, just warn (trailing period)
INFO      → Just inform (missing issue reference)
```

---

## 3. Pre-Push Hook Implementation

### File: `.githooks/pre-push`

**Purpose**: Prevent pushing code that fails tests  
**Strategy**: Run full test suite before allowing push  
**Timeout**: 30-60 seconds (acceptable since it gates network activity)

### Key Implementation Features

#### 3.1 Test Ordering

```bash
# Step 1: Quick sanity check (compilation)
# Step 2: Library tests (deterministic, single-threaded)
# Step 3: Integration tests (deterministic)
# Step 4: All features test (comprehensive)
# Step 5: Critical features test (subset)
```

**Rationale**:
- Compilation first (fails fastest)
- Unit tests next (most valuable for quick feedback)
- Integration tests after (more expensive)
- Feature tests last (only if basic tests pass)

#### 3.2 Deterministic Testing

```bash
# Single-threaded testing prevents race conditions and flakiness
cargo test --lib --quiet -- --test-threads=1
```

**Why**: 
- Tests run sequentially, preventing concurrency bugs from hiding
- No test pollution (shared state issues surface)
- Results are reproducible

#### 3.3 Feature Matrix Testing

```bash
FEATURE_TESTS=(
    "federated-network"
    "otel"
)

FEATURES_FAILED=0
for feature in "${FEATURE_TESTS[@]}"; do
    if cargo test --features "$feature" --quiet >/dev/null 2>&1; then
        echo -e "${GREEN}  ✅ Feature '$feature' tests passed${NC}"
    else
        echo -e "${RED}  ❌ Feature '$feature' tests FAILED${NC}"
        FEATURES_FAILED=1
    fi
done

if [ $FEATURES_FAILED -ne 0 ]; then
    FAILED=1
fi
```

**Why**:
- Feature-gated code can have bugs that only appear with specific features
- Early detection prevents merged bugs

#### 3.4 Progress Reporting

```bash
echo ""
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo "${BLUE}  🧪 PRE-PUSH HOOK: Full Test Validation${NC}"
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Running comprehensive tests before push..."
echo "(This may take 30-60 seconds)"
```

**Why**: 
- Sets expectations (hook is slow on purpose)
- Prevents developers from thinking terminal is hung
- Shows progress markers

#### 3.5 Bypass Instructions

```bash
if [ $FAILED -ne 0 ]; then
    echo -e "${RED}❌ PRE-PUSH VALIDATION FAILED - Push blocked${NC}"
    echo ""
    echo "Options:"
    echo "  1. Fix test failures and retry"
    echo "  2. Push anyway (NOT RECOMMENDED):"
    echo "     ${YELLOW}git push --no-verify${NC}"
fi
```

**Why**: Developers need to know how to override, but warnings discourage it.

---

## 4. Post-Commit Hook Implementation

### File: `.githooks/post-commit`

**Purpose**: Provide helpful reminders (never blocks)  
**Strategy**: Analyze commit, show relevant next steps

### Key Implementation Features

#### 4.1 Commit Metadata Extraction

```bash
COMMIT_HASH=$(git rev-parse --short HEAD)
COMMIT_MSG=$(git log -1 --pretty=format:"%s")
```

**Why**: Shows what was just committed; confirms success.

#### 4.2 Commit Type Detection

```bash
if echo "$COMMIT_MSG" | grep -qiE "^(feat|feature)"; then
    COMMIT_TYPE="Feature"
    SUGGESTION="Consider writing tests for this feature"
elif echo "$COMMIT_MSG" | grep -qiE "^(fix|bugfix)"; then
    COMMIT_TYPE="Bug fix"
    SUGGESTION="Verify fix with: cargo test test_name"
elif echo "$COMMIT_MSG" | grep -qiE "^(test|tests)"; then
    COMMIT_TYPE="Test"
    SUGGESTION="Run: cargo make test to ensure all tests pass"
# ... more types ...
else
    COMMIT_TYPE="Changes"
    SUGGESTION="Next: git push to send to remote"
fi
```

**Why**: 
- Shows developers what kind of change was made
- Provides tailored suggestions based on type

#### 4.3 Branch Status

```bash
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
COMMITS_AHEAD=$(git rev-list --count origin/main..HEAD 2>/dev/null || echo "?")
UNPUSHED=$(git rev-list --count @{u}..HEAD 2>/dev/null || echo "?")

echo "${CYAN}📍 Branch: $CURRENT_BRANCH${NC}"
if [ "$COMMITS_AHEAD" != "?" ]; then
    echo "${CYAN}📊 Commits ahead of main: $COMMITS_AHEAD${NC}"
fi
```

**Why**: 
- Developers see their current branch and progress
- Handles edge cases (detached HEAD, no upstream)

#### 4.4 Context-Aware Next Steps

```bash
if echo "$CURRENT_BRANCH" | grep -qE "^(claude|feat|fix|refactor)"; then
    # Feature branch
    echo "${YELLOW}📋 Next steps:${NC}"
    echo "  1. Continue development or push to remote"
    echo "  2. When ready: ${BLUE}git push${NC} (pre-push hook validates tests)"
    echo "  3. Create PR from: ${BLUE}gh pr create${NC} or GitHub UI"
else
    # Main or other branch
    echo "${YELLOW}📋 Next steps:${NC}"
    echo "  1. Push to remote: ${BLUE}git push${NC}"
    echo "  2. Start a PR if needed: ${BLUE}gh pr create${NC}"
fi
```

**Why**: 
- Different workflows for different branches
- Guides developers on what to do next

#### 4.5 No Blocking

```bash
exit 0  # Always exit with 0 (never block)
```

**Why**: Hook is informational only; never prevents commits.

---

## 5. Installation Script Implementation

### File: `.githooks/install.sh`

**Purpose**: Automated hook installation and configuration

### Key Features

#### 5.1 Directory Creation

```bash
HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_HOOKS_DIR="$HOOKS_DIR/../.git/hooks"

if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo -e "${YELLOW}Creating .git/hooks directory...${NC}"
    mkdir -p "$GIT_HOOKS_DIR"
fi
```

**Why**: `.git/hooks` might not exist on first run.

#### 5.2 Hook Backup

```bash
for hook in "${HOOKS_DIR[@]}"; do
    SRC="$HOOKS_DIR/$hook"
    DEST="$GIT_HOOKS_DIR/$hook"

    if [ -f "$SRC" ]; then
        # Back up existing hook if it differs
        if [ -f "$DEST" ] && ! cmp -s "$SRC" "$DEST"; then
            echo -e "${YELLOW}Backing up existing $hook to ${DEST}.bak${NC}"
            cp "$DEST" "${DEST}.bak"
        fi

        cp "$SRC" "$DEST"
        chmod +x "$DEST"
    fi
done
```

**Why**: 
- Preserves user's customizations (backed up as `.bak`)
- Makes hooks executable
- Idempotent (safe to run multiple times)

#### 5.3 Git Configuration

```bash
git config --local core.hooksPath .githooks
```

**Why**: 
- Modern approach (Git 2.9+)
- `--local` = repository-specific, no global pollution
- Allows different hooks per project

#### 5.4 Verification Instructions

```bash
echo "Commands:"
echo "  Verify setup:   ${BLUE}git config --local core.hooksPath${NC}"
echo "  Test a hook:    ${BLUE}git commit --allow-empty -m 'Test message'${NC}"
echo "  Bypass hooks:   ${BLUE}git commit --no-verify${NC} (NOT RECOMMENDED)"
echo "  Uninstall:      ${BLUE}./.githooks/uninstall.sh${NC}"
```

**Why**: Developers know how to verify and test the installation.

---

## 6. Uninstall Script Implementation

### File: `.githooks/uninstall.sh`

**Purpose**: Clean removal of hooks without breaking `.git/` directory

### Key Features

```bash
#!/bin/bash
# Remove hook files

HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_HOOKS_DIR="$HOOKS_DIR/../.git/hooks"

rm -f "$GIT_HOOKS_DIR/pre-commit"
rm -f "$GIT_HOOKS_DIR/commit-msg"
rm -f "$GIT_HOOKS_DIR/pre-push"
rm -f "$GIT_HOOKS_DIR/post-commit"

# Restore any backups (optional)
# for hook_backup in "$GIT_HOOKS_DIR"/*.bak; do
#     [ -e "$hook_backup" ] && mv "$hook_backup" "${hook_backup%.bak}"
# done

# Unset git config
git config --local --unset core.hooksPath

echo "Hooks uninstalled"
```

**Why**:
- Clean removal
- Doesn't break repository
- Safe to re-run

---

## 7. Error Handling Best Practices

### Pattern 1: Conditional Execution

```bash
if command_that_might_fail; then
    echo "Success"
else
    echo "Failed"
    FAILED=1
fi
# Continue to next check (don't exit)
```

### Pattern 2: Optional Tool

```bash
if command -v optional_tool &> /dev/null; then
    if optional_tool --check; then
        echo "Pass"
    else
        FAILED=1
    fi
else
    echo "Tool not installed (optional)"
fi
```

### Pattern 3: File Existence

```bash
if [ -f "required.txt" ]; then
    if process_file required.txt; then
        echo "Pass"
    else
        FAILED=1
    fi
else
    echo "File not found"
    FAILED=1
fi
```

---

## 8. Testing the Hooks

### Test Pre-Commit

```bash
# Create a formatting issue (trailing space)
echo "fn main() {  " > src/lib.rs
git add src/lib.rs
git commit -m "Test pre-commit"  # Should fail on formatting
```

### Test Commit-Msg

```bash
# Too long first line
git commit --allow-empty -m "This is a very long commit message that exceeds the 72 character limit and should fail validation"
# Should fail on length

# Not capitalized
git commit --allow-empty -m "this is not capitalized"
# Should fail on capitalization
```

### Test Pre-Push

```bash
# Create a test failure
echo "#[test] fn fail() { panic!(); }" >> tests/lib.rs
git add tests/lib.rs
git commit -m "Add failing test"
git push  # Should fail in pre-push hook
```

### Test Post-Commit

```bash
git commit --allow-empty -m "Test post-commit hook"
# Should see helpful output
```

---

## 9. Performance Optimization

### Pre-Commit Optimization

**Current bottlenecks**:
- `cargo clippy`: ~1s
- `cargo check`: ~800ms
- `cargo fmt --check`: ~500ms

**Optimization opportunities**:
1. Use `--incremental` (cargo default since 1.60)
2. Skip check if only formatting files changed
3. Parallelize independent checks

### Pre-Push Optimization

**Current bottlenecks**:
- Full test suite: ~30-60s
- Feature tests: ~5s per feature

**Optimization opportunities**:
1. Run only tests for changed crates (monorepo)
2. Parallel execution by default (override with `--test-threads=1` only for critical tests)
3. Cache build artifacts

---

## 10. Extending the Hooks

### Adding a New Check to Pre-Commit

```bash
# Add after clippy check
echo ""
echo "${BLUE}🔍 Check 3/5: Documentation check${NC}"
if cargo doc --quiet >/dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS: Documentation builds${NC}"
else
    echo -e "${RED}❌ FAILED: Documentation does not build${NC}"
    FAILED=1
fi
```

### Adding a New Test to Pre-Push

```bash
# Add to feature tests
FEATURE_TESTS=(
    "federated-network"
    "otel"
    "new-feature"  # Add here
)
```

### Adding Custom Validation to Commit-Msg

```bash
# Add validation check
echo ""
echo "${BLUE}🔍 Validation N: Custom check${NC}"
if echo "$COMMIT_MSG" | grep -q "CUSTOM_PATTERN"; then
    echo -e "${GREEN}✅ PASS: Custom validation${NC}"
else
    echo -e "${YELLOW}⚠️  WARNING: Consider adding custom pattern${NC}"
fi
```

---

## 11. CI/CD Integration

### GitHub Actions Example

```yaml
name: Lint & Test
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo deny check

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
```

**Note**: CI runs same commands as hooks but doesn't need to be as fast.

---

## Summary

**Implementation guidelines**:
- Use bash for portability
- Store in `.githooks/` (versioned with code)
- Color-coded output (human-friendly)
- Clear error messages with fixes
- Fail-fast for quick feedback
- Defer heavy tests to pre-push
- Optional tools skip gracefully
- Post-commit provides guidance (never blocks)

**Hook responsibilities**:
| Hook | Speed | Blocks | Purpose |
|------|-------|--------|---------|
| pre-commit | <2s | Yes | Fast quality gates |
| commit-msg | <50ms | Yes | Message validation |
| pre-push | 30-60s | Yes | Full test suite |
| post-commit | <100ms | No | Helpful reminders |

This design balances developer experience (fast feedback) with code quality (comprehensive testing) and reliability (fail-fast on real issues).
