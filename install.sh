#!/bin/sh
set -e

# mcp-stama One-Liner Installer for Linux & macOS
REPO="mcp-stama/mcp-stama"
INSTALL_DIR="$HOME/.local/bin"

echo "⚡ Installing mcp-stama..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    TARGET="x86_64-unknown-linux-gnu"
    ;;
  Darwin)
    if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
      TARGET="aarch64-apple-darwin"
    else
      TARGET="x86_64-apple-darwin"
    fi
    ;;
  *)
    echo "Error: Unsupported operating system: $OS"
    exit 1
    ;;
esac

TARBALL="mcp-stama-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${TARBALL}"

mkdir -p "$INSTALL_DIR"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Downloading ${URL}..."
curl -fsSL "$URL" -o "$TMP_DIR/$TARBALL"

tar -xzf "$TMP_DIR/$TARBALL" -C "$TMP_DIR"
mv "$TMP_DIR/mcp-stama" "$INSTALL_DIR/mcp-stama"
chmod +x "$INSTALL_DIR/mcp-stama"

echo "✅ mcp-stama successfully installed to ${INSTALL_DIR}/mcp-stama"

# Attempt automatic client configuration if binary runs
if "$INSTALL_DIR/mcp-stama" --install-cursor --install-claude 2>/dev/null; then
  echo "🚀 MCP clients (Cursor / Claude Desktop) auto-configured successfully!"
else
  echo "Notice: Add ${INSTALL_DIR} to your PATH to run mcp-stama from any directory."
fi
