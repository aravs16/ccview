#!/bin/sh
set -e

# ccview installer
# Usage: curl -fsSL https://raw.githubusercontent.com/aravs16/ccview/main/install.sh | sh

REPO="aravs16/ccview"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux)  OS="unknown-linux-gnu" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  arm64|aarch64) ARCH="aarch64" ;;
  x86_64|amd64)  ARCH="x86_64" ;;
  *)             echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

BINARY="ccview-${ARCH}-${OS}"
URL="https://github.com/${REPO}/releases/latest/download/${BINARY}"

echo "Downloading ccview for ${ARCH}-${OS}..."
curl -fsSL "$URL" -o /tmp/ccview
chmod +x /tmp/ccview

if [ -w "$INSTALL_DIR" ]; then
  mv /tmp/ccview "$INSTALL_DIR/ccview"
else
  echo "Need sudo to install to $INSTALL_DIR"
  sudo mv /tmp/ccview "$INSTALL_DIR/ccview"
fi

# Create pages directory
mkdir -p "$HOME/.ccview/pages/_inbox"

# Install CC skill if Claude Code is present
if [ -d "$HOME/.claude" ]; then
  mkdir -p "$HOME/.claude/skills/view"
  SKILL_URL="https://raw.githubusercontent.com/${REPO}/main/skill/SKILL.md"
  curl -fsSL "$SKILL_URL" -o "$HOME/.claude/skills/view/SKILL.md" 2>/dev/null || true
  echo "Installed Claude Code skill at ~/.claude/skills/view/"
fi

echo ""
echo "ccview installed successfully!"
echo ""
echo "  Run:  ccview"
echo "  Dir:  ~/.ccview/pages/"
echo ""
