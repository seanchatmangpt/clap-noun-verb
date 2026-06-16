#!/bin/bash
# Install git hooks from .githooks directory
# Usage: ./.githooks/install.sh

set -e

HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_HOOKS_DIR="$HOOKS_DIR/../.git/hooks"
PROJECT_ROOT="$(dirname "$HOOKS_DIR")"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo "${BLUE}  🔧 Git Hooks Installation${NC}"
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if .git/hooks exists
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo -e "${YELLOW}Creating .git/hooks directory...${NC}"
    mkdir -p "$GIT_HOOKS_DIR"
fi

# Copy hooks
HOOKS=("pre-commit" "commit-msg" "pre-push" "post-commit")
INSTALLED=0

for hook in "${HOOKS[@]}"; do
    SRC="$HOOKS_DIR/$hook"
    DEST="$GIT_HOOKS_DIR/$hook"

    if [ -f "$SRC" ]; then
        # Back up existing hook if it differs
        if [ -f "$DEST" ] && ! cmp -s "$SRC" "$DEST"; then
            echo -e "${YELLOW}Backing up existing $hook to ${DEST}.bak${NC}"
            cp "$DEST" "${DEST}.bak"
        fi

        # Copy and make executable
        cp "$SRC" "$DEST"
        chmod +x "$DEST"
        echo -e "${GREEN}✅ Installed: $hook${NC}"
        INSTALLED=$((INSTALLED + 1))
    else
        echo -e "${YELLOW}⚠️  Skipping: $hook (not found in .githooks/)${NC}"
    fi
done

echo ""

# Configure git to use .githooks directory
echo -e "${BLUE}Configuring git to use .githooks directory...${NC}"
if git config --local core.hooksPath >/dev/null 2>&1; then
    CURRENT=$(git config --local core.hooksPath)
    if [ "$CURRENT" != ".githooks" ]; then
        echo -e "${YELLOW}  Updating core.hooksPath from '$CURRENT' to '.githooks'${NC}"
    fi
fi

git config --local core.hooksPath .githooks
echo -e "${GREEN}✅ Git configured${NC}"

echo ""
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "Installed hooks ($INSTALLED):"
echo "  • pre-commit  - Fast quality gates (format, clippy, compile)"
echo "  • commit-msg  - Commit message validation"
echo "  • pre-push    - Full test suite validation"
echo "  • post-commit - Helpful reminders and tips"
echo ""
echo "Commands:"
echo "  Verify setup:   ${BLUE}git config --local core.hooksPath${NC}"
echo "  Test a hook:    ${BLUE}git commit --allow-empty -m 'Test message'${NC}"
echo "  Bypass hooks:   ${BLUE}git commit --no-verify${NC} (NOT RECOMMENDED)"
echo "  Uninstall:      ${BLUE}./.githooks/uninstall.sh${NC}"
echo ""
