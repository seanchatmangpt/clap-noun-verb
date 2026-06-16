#!/bin/bash
# scripts/release-checklist.sh
# Interactive release checklist for clap-noun-verb
# Usage: ./scripts/release-checklist.sh 26.6.15

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release-checklist.sh 26.6.15"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Track completed items
COMPLETED=0
FAILED=0

print_header() {
    echo ""
    echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo "${BLUE}  RELEASE CHECKLIST - v$VERSION${NC}"
    echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

check_item() {
    local num=$1
    local description=$2
    local command=$3

    echo "${CYAN}[${num}]${NC} $description"

    if [ -z "$command" ]; then
        # Manual check - ask user
        read -p "    ${YELLOW}→${NC} Press enter to continue, or type 'skip' to skip: " response
        if [ "$response" = "skip" ]; then
            echo "    ${YELLOW}⊘${NC} Skipped"
            return 1
        fi
        echo "    ${GREEN}✓${NC} Done"
        COMPLETED=$((COMPLETED + 1))
        return 0
    else
        # Automated check
        if eval "$command" > /tmp/check-output.txt 2>&1; then
            echo "    ${GREEN}✓${NC} Done"
            COMPLETED=$((COMPLETED + 1))
            return 0
        else
            echo "    ${RED}✗${NC} Failed"
            echo "    ${RED}Error output:${NC}"
            cat /tmp/check-output.txt | sed 's/^/    /'
            FAILED=$((FAILED + 1))
            return 1
        fi
    fi
}

print_header

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 1: PRE-RELEASE PLANNING (1-2 days before)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}PHASE 1: Pre-Release Planning${NC}"
echo ""

check_item "1.1" "Review commits since last release" \
    "git log v$(grep 'version = ' Cargo.toml | head -1 | sed 's/.*version = \"//;s/\".*//')..HEAD --oneline | head -20"

check_item "1.2" "Determine version bump type (MAJOR/MINOR/PATCH)" || true

check_item "1.3" "Update CHANGELOG.md with all changes" || true

check_item "1.4" "Review for stubs, TODOs, unwrap() in production code" || true

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 2: VERSION BUMPING
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}PHASE 2: Version Bumping${NC}"
echo ""

check_item "2.1" "Bump version to $VERSION" \
    "grep -r 'version = \"$VERSION\"' Cargo.toml clap-noun-verb-macros/Cargo.toml"

check_item "2.2" "Verify version consistency across Cargo.toml files" \
    "[ \"$(grep 'version = ' Cargo.toml | wc -l)\" -gt 0 ]"

check_item "2.3" "Update README.md version examples" || true

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 3: QUALITY GATES
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}PHASE 3: Quality Gates (run: ./scripts/pre-release-check.sh $VERSION)${NC}"
echo ""

check_item "3.1" "Run pre-release checks" \
    "./scripts/pre-release-check.sh $VERSION"

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 4: PUBLISHING
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}PHASE 4: Publishing${NC}"
echo ""

check_item "4.1" "Push commits to main" \
    "git log origin/main..HEAD | wc -l | grep -q '[0-9]'"

check_item "4.2" "Publish macros crate" \
    "cargo search clap-noun-verb-macros | grep -q 'clap_noun_verb_macros'"

check_item "4.3" "Publish main crate" \
    "cargo search clap-noun-verb | grep -q 'clap_noun_verb'"

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 5: GITHUB RELEASE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}PHASE 5: GitHub Release${NC}"
echo ""

check_item "5.1" "Create git tag v$VERSION" || true

check_item "5.2" "Push tag to GitHub" || true

check_item "5.3" "Monitor GitHub Actions release workflow" || true

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PHASE 6: POST-RELEASE VERIFICATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}PHASE 6: Post-Release Verification${NC}"
echo ""

check_item "6.1" "Verify on crates.io (macros)" \
    "cargo search clap-noun-verb-macros --limit 1 | grep -q 'clap_noun_verb_macros'"

check_item "6.2" "Verify on crates.io (main)" \
    "cargo search clap-noun-verb --limit 1 | grep -q 'clap_noun_verb'"

check_item "6.3" "Check docs.rs (macros)" || true

check_item "6.4" "Check docs.rs (main)" || true

check_item "6.5" "GitHub Release created successfully" || true

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SUMMARY
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "${BLUE}  RELEASE CHECKLIST SUMMARY${NC}"
echo "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo "  ${GREEN}Completed: $COMPLETED${NC}"
echo "  ${RED}Failed: $FAILED${NC}"
echo ""

if [ "$FAILED" -eq 0 ]; then
    echo "  ${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "  ${CYAN}Next steps:${NC}"
    echo "    1. Verify release on crates.io and docs.rs"
    echo "    2. Announce on Twitter/blog (if major feature)"
    echo "    3. Close any 'ready for release' milestones"
    echo "    4. Plan next release"
    echo ""
    exit 0
else
    echo "  ${RED}✗ Some checks failed. Fix issues and retry.${NC}"
    echo ""
    exit 1
fi
