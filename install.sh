#!/bin/sh
set -e

# ccview installer
# Usage: curl -fsSL https://raw.githubusercontent.com/aravs16/ccview/main/install.sh | sh

REPO="aravs16/ccview"

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

# Install to user-writable location first, fall back to /usr/local/bin
if [ -d "$HOME/.local/bin" ] && echo "$PATH" | grep -q "$HOME/.local/bin"; then
  INSTALL_DIR="$HOME/.local/bin"
elif [ -w "/usr/local/bin" ]; then
  INSTALL_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
  INSTALL_DIR="$HOME/.local/bin"
else
  mkdir -p "$HOME/.local/bin"
  INSTALL_DIR="$HOME/.local/bin"
fi

mv /tmp/ccview "$INSTALL_DIR/ccview"

# Create pages directory
mkdir -p "$HOME/.ccview/pages/_inbox"

# Install CC skill if Claude Code is present
if [ -d "$HOME/.claude" ]; then
  mkdir -p "$HOME/.claude/skills/view"
  SKILL_URL="https://raw.githubusercontent.com/${REPO}/main/skill/SKILL.md"
  curl -fsSL "$SKILL_URL" -o "$HOME/.claude/skills/view/SKILL.md" 2>/dev/null || true
  echo "  Installed Claude Code skill at ~/.claude/skills/view/"
fi

echo ""
echo "  ccview installed to $INSTALL_DIR/ccview"
echo ""
echo "  Run:  ccview"
echo "  Dir:  ~/.ccview/pages/"

# Warn if install dir not in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo ""
  echo "  Note: $INSTALL_DIR is not in your PATH."
  echo "  Add it:  export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo ""
