#!/bin/bash
# Developer Experience Setup Script
# Sets up VS Code, IDE extensions, and automation for clap-noun-verb
# Usage: ./scripts/setup-dx.sh

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "=================================================="
echo "clap-noun-verb Developer Experience Setup"
echo "=================================================="
echo ""

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check Rust installation
echo -e "${BLUE}[1/6]${NC} Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo -e "${YELLOW}Rust not found. Installing rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    echo -e "${GREEN}✓ Rust ${RUST_VERSION}${NC}"
fi
echo ""

# Install cargo-make
echo -e "${BLUE}[2/6]${NC} Checking cargo-make..."
if ! command -v cargo-make &> /dev/null; then
    echo "Installing cargo-make..."
    cargo install cargo-make
else
    echo -e "${GREEN}✓ cargo-make installed${NC}"
fi
echo ""

# Install cargo-watch (optional but recommended)
echo -e "${BLUE}[3/6]${NC} Checking cargo-watch..."
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch (for auto-recompile on changes)..."
    cargo install cargo-watch
else
    echo -e "${GREEN}✓ cargo-watch installed${NC}"
fi
echo ""

# Setup VS Code if found
echo -e "${BLUE}[4/6]${NC} Configuring VS Code..."
if command -v code &> /dev/null; then
    # Create .vscode directory if needed
    mkdir -p "$REPO_ROOT/.vscode"

    # Create settings.json
    cat > "$REPO_ROOT/.vscode/settings.json" << 'EOF'
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.hover.documentation.enable": true,
  "editor.codeActionsOnSave": {
    "source.organizeImports": true
  },
  "search.exclude": {
    "**/target": true,
    "**/.git": true
  },
  "terminal.integrated.defaultProfile.linux": "bash",
  "terminal.integrated.defaultProfile.osx": "zsh"
}
EOF

    echo -e "${GREEN}✓ Created .vscode/settings.json${NC}"

    # Create launch.json for debugging
    cat > "$REPO_ROOT/.vscode/launch.json" << 'EOF'
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tutorial Basic",
      "cargo": {
        "args": ["build", "--example", "tutorial_basic"],
        "filter": { "name": "tutorial_basic", "kind": "bin" }
      },
      "args": ["services", "status"],
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Tests",
      "cargo": {
        "args": ["test", "--lib", "--no-run"],
        "filter": { "kind": "lib" }
      },
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    }
  ]
}
EOF

    echo -e "${GREEN}✓ Created .vscode/launch.json${NC}"

    # Create tasks.json
    cat > "$REPO_ROOT/.vscode/tasks.json" << 'EOF'
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo make test",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "test"],
      "problemMatcher": ["$rustc"],
      "group": { "kind": "test", "isDefault": true }
    },
    {
      "label": "cargo make clippy",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "clippy"],
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo make format",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "format"]
    },
    {
      "label": "cargo make lint",
      "type": "shell",
      "command": "cargo",
      "args": ["make", "lint"],
      "problemMatcher": ["$rustc"]
    }
  ]
}
EOF

    echo -e "${GREEN}✓ Created .vscode/tasks.json${NC}"

    # Create keybindings.json
    cat > "$REPO_ROOT/.vscode/keybindings.json" << 'EOF'
[
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo make test"
  },
  {
    "key": "ctrl+shift+l",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo make clippy"
  },
  {
    "key": "ctrl+shift+f",
    "command": "workbench.action.tasks.runTask",
    "args": "cargo make format"
  }
]
EOF

    echo -e "${GREEN}✓ Created .vscode/keybindings.json${NC}"

    # Recommended extensions
    echo ""
    echo "Recommended VS Code extensions (install manually or via CLI):"
    echo "  - rust-lang.rust-analyzer"
    echo "  - vadimcn.vscode-lldb"
    echo "  - tamasfe.even-better-toml"
    echo "  - serayuzgur.crates"
    echo ""
    echo "Install with:"
    echo "  code --install-extension rust-lang.rust-analyzer"
    echo "  code --install-extension vadimcn.vscode-lldb"
    echo "  code --install-extension tamasfe.even-better-toml"
    echo ""
else
    echo -e "${YELLOW}VS Code not found. Skipping VS Code setup.${NC}"
fi
echo ""

# Setup git hooks
echo -e "${BLUE}[5/6]${NC} Setting up git hooks..."
mkdir -p "$REPO_ROOT/.git/hooks"

# Create pre-commit hook
cat > "$REPO_ROOT/.git/hooks/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook for clap-noun-verb
# Runs format check, clippy, and quick tests

set -e

echo "Running pre-commit checks..."

# Format check
if ! cargo make format-check > /dev/null 2>&1; then
    echo "✗ Format check failed. Run: cargo make format"
    exit 1
fi

# Clippy
if ! cargo make clippy > /dev/null 2>&1; then
    echo "✗ Clippy failed. Fix issues and try again."
    exit 1
fi

# Quick tests
if ! cargo make test > /dev/null 2>&1; then
    echo "✗ Tests failed. Fix and try again."
    exit 1
fi

echo "✓ Pre-commit checks passed"
exit 0
EOF

chmod +x "$REPO_ROOT/.git/hooks/pre-commit"
echo -e "${GREEN}✓ Created git pre-commit hook${NC}"
echo ""

# Create development scripts
echo -e "${BLUE}[6/6]${NC} Creating development scripts..."

# dev-loop.sh - Watch mode
cat > "$REPO_ROOT/scripts/dev-loop.sh" << 'EOF'
#!/bin/bash
# Watch mode: auto-recompile and test on file changes
cargo watch -x "make format-check" -x "make clippy" -x "make test"
EOF
chmod +x "$REPO_ROOT/scripts/dev-loop.sh"
echo -e "${GREEN}✓ Created scripts/dev-loop.sh${NC}"

# build-demo.sh - Run tutorial examples
cat > "$REPO_ROOT/scripts/build-demo.sh" << 'EOF'
#!/bin/bash
set -e
echo "Building examples..."
cargo make build-examples

echo ""
echo "Running tutorial examples..."
echo ""

echo "[1/3] services status"
cargo run --example tutorial_basic -- services status

echo ""
echo "[2/3] services logs"
cargo run --example tutorial_basic -- services logs web-server

echo ""
echo "[3/3] services restart"
cargo run --example tutorial_basic -- services restart database

echo ""
echo "✓ All demo commands executed successfully!"
EOF
chmod +x "$REPO_ROOT/scripts/build-demo.sh"
echo -e "${GREEN}✓ Created scripts/build-demo.sh${NC}"

# pr-ready.sh - Full validation before PR
cat > "$REPO_ROOT/scripts/pr-ready.sh" << 'EOF'
#!/bin/bash
set -e

echo "=================================================="
echo "PR Ready Validation"
echo "=================================================="
echo ""

echo "[1/5] Format check..."
cargo make format-check || (echo "✗ Format failed. Run: cargo make format" && exit 1)

echo "[2/5] Clippy lint..."
cargo make clippy || (echo "✗ Clippy failed" && exit 1)

echo "[3/5] Running tests (all features)..."
cargo make test-all || (echo "✗ Tests failed" && exit 1)

echo "[4/5] Building documentation..."
cargo make doc > /dev/null 2>&1 || (echo "✗ Docs build failed" && exit 1)

echo "[5/5] Checking examples..."
cargo make build-examples > /dev/null 2>&1 || (echo "✗ Examples build failed" && exit 1)

echo ""
echo "=================================================="
echo "✓ PR Ready!"
echo "=================================================="
echo ""
echo "Ready to submit PR. Use:"
echo "  git push origin <branch>"
EOF
chmod +x "$REPO_ROOT/scripts/pr-ready.sh"
echo -e "${GREEN}✓ Created scripts/pr-ready.sh${NC}"

echo ""
echo "=================================================="
echo "Setup Complete!"
echo "=================================================="
echo ""
echo -e "${GREEN}✓ Rust toolchain verified${NC}"
echo -e "${GREEN}✓ cargo-make installed${NC}"
echo -e "${GREEN}✓ cargo-watch installed${NC}"
echo -e "${GREEN}✓ VS Code configured (.vscode/)${NC}"
echo -e "${GREEN}✓ Git hooks installed${NC}"
echo -e "${GREEN}✓ Development scripts created${NC}"
echo ""
echo "Next steps:"
echo ""
echo "1. Run a quick build:"
echo "   cargo make test"
echo ""
echo "2. Start development (watch mode):"
echo "   ./scripts/dev-loop.sh"
echo ""
echo "3. Run examples:"
echo "   ./scripts/build-demo.sh"
echo ""
echo "4. Before submitting PR:"
echo "   ./scripts/pr-ready.sh"
echo ""
echo "See DX_GUIDE.md for detailed documentation!"
echo ""
