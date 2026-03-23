#!/bin/sh
# Jed — JSON Editor installer
# Usage: curl -fsSL https://github.com/caoergou/jed/releases/latest/download/install.sh | sh

set -e

REPO="caoergou/jed"
BIN_NAME="jed"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  ASSET="jed-linux-x86_64" ;;
      aarch64) ASSET="jed-linux-aarch64" ;;
      *)       echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  ASSET="jed-macos-x86_64" ;;
      arm64)   ASSET="jed-macos-aarch64" ;;
      *)       echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "For Windows, download jed-windows-x86_64.exe from:"
    echo "  https://github.com/$REPO/releases/latest"
    exit 1
    ;;
esac

URL="https://github.com/$REPO/releases/latest/download/$ASSET"

echo "Downloading $ASSET..."
curl -fsSL "$URL" -o "/tmp/$BIN_NAME"
chmod +x "/tmp/$BIN_NAME"

# Install (try with sudo if needed)
if [ -w "$INSTALL_DIR" ]; then
  mv "/tmp/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo mv "/tmp/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

echo "Installed: $(which $BIN_NAME)"
"$BIN_NAME" --version
