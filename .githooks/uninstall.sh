#!/bin/bash
# Uninstall git hooks and reset configuration
# Usage: ./.githooks/uninstall.sh

set -e

HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GIT_HOOKS_DIR="$HOOKS_DIR/../.git/hooks"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo "${BLUE}  🗑️  Git Hooks Uninstallation${NC}"
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${RED}⚠️  This will remove all git hooks from .git/hooks/${NC}"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""

# Remove hooks
HOOKS=("pre-commit" "commit-msg" "pre-push" "post-commit")

for hook in "${HOOKS[@]}"; do
    HOOK_PATH="$GIT_HOOKS_DIR/$hook"
    if [ -f "$HOOK_PATH" ]; then
        rm -f "$HOOK_PATH"
        echo -e "${GREEN}✅ Removed: $hook${NC}"
    fi

    # Also remove backups
    if [ -f "${HOOK_PATH}.bak" ]; then
        rm -f "${HOOK_PATH}.bak"
        echo -e "${GREEN}✅ Removed backup: ${hook}.bak${NC}"
    fi
done

echo ""

# Unset core.hooksPath
echo -e "${BLUE}Resetting git configuration...${NC}"
git config --local --unset core.hooksPath || true
echo -e "${GREEN}✅ Git configuration reset${NC}"

echo ""
echo "${BLUE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Uninstallation complete${NC}"
echo ""
echo "Git hooks have been removed."
echo "To reinstall: ${BLUE}./.githooks/install.sh${NC}"
echo ""
