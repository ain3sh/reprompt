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
INSTALL_DIR="$HOME/.local/bin"
echo "Installing to $INSTALL_DIR ..."

# Create directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
fi

mv "$TMP_DIR/reprompt" "$INSTALL_DIR/reprompt"

echo "Successfully installed reprompt!"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "WARNING: $INSTALL_DIR is not in your PATH."
    echo "Attempting to add it to your shell configuration..."

    EXPORT_CMD="export PATH=\"\$HOME/.local/bin:\$PATH\""
    UPDATED_CONFIG=0

    # Function to update config file
    update_config() {
        local config="$1"
        if [ -f "$config" ]; then
            if grep -q ".local/bin" "$config"; then
                echo "  - $config: Already contains .local/bin"
            else
                echo "  - $config: Appending to PATH..."
                echo "" >> "$config"
                echo "# Added by reprompt installer" >> "$config"
                echo "$EXPORT_CMD" >> "$config"
                UPDATED_CONFIG=1
            fi
        fi
    }

    # Check common shell configs
    update_config "$HOME/.bashrc"
    update_config "$HOME/.zshrc"

    if [ $UPDATED_CONFIG -eq 1 ]; then
        echo "Successfully updated shell configuration."
        echo "Please restart your terminal or run 'source <your_shell_config>' to apply."
    else
        echo "Could not detect or update .bashrc / .zshrc."
        echo "Please manually add the following line to your shell configuration:"
        echo "  $EXPORT_CMD"
    fi
else
    echo "Run 'reprompt' to sanitize your clipboard."
fi
