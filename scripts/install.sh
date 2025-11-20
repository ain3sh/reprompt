#!/bin/bash
# reprompt install script

set -e

REPO="ain3sh/reprompt"
VERSION="latest"

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux)
        ASSET="reprompt-linux-amd64"
        ;;
    Darwin)
        ASSET="reprompt-macos-amd64"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        echo "Windows detected. Please download the .exe manually from GitHub Releases."
        exit 1
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Detect Arch
ARCH="$(uname -m)"
if [ "$ARCH" != "x86_64" ]; then
    echo "Unsupported architecture: $ARCH. Only x86_64 is currently supported."
    exit 1
fi

# Construct Download URL
if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$ASSET"
else
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
fi

echo "Downloading reprompt ($ASSET)..."
echo "URL: $DOWNLOAD_URL"

# Create a temporary directory
TMP_DIR=$(mktemp -d)
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Download to temp dir
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/reprompt"

# Make executable
chmod +x "$TMP_DIR/reprompt"

# Install
INSTALL_DIR="/usr/local/bin"
echo "Installing to $INSTALL_DIR ..."

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/reprompt" "$INSTALL_DIR/reprompt"
else
    echo "Sudo permission required to move binary to $INSTALL_DIR"
    sudo mv "$TMP_DIR/reprompt" "$INSTALL_DIR/reprompt"
fi

echo "Successfully installed reprompt!"
echo "Run 'reprompt' to sanitize your clipboard."
