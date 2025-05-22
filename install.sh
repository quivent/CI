#!/bin/bash
# CI Installation Script
# This script builds and installs the CI tool

set -e

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    echo "❌ Error: This script must be run from the CI project root directory."
    exit 1
fi

echo "🔨 Building CI binary..."
cargo build --release

# Determine the installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "📦 Installing CI binary to $INSTALL_DIR..."
cp target/release/CI "$INSTALL_DIR/"

# Make executable
chmod +x "$INSTALL_DIR/CI"

# Create lowercase symlink
ln -sf "$INSTALL_DIR/CI" "$INSTALL_DIR/ci"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "⚠️  Warning: $INSTALL_DIR is not in your PATH"
    
    # Determine the shell configuration file
    SHELL_CONFIG=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        if [[ "$(uname)" == "Darwin" ]]; then
            SHELL_CONFIG="$HOME/.bash_profile"
        else
            SHELL_CONFIG="$HOME/.bashrc"
        fi
    else
        SHELL_CONFIG="$HOME/.profile"
    fi
    
    echo "Add the following line to your $SHELL_CONFIG:"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
fi

# Create completions directory
COMPLETIONS_DIR="$INSTALL_DIR/../share/bash-completion/completions"
if [ -d "completions" ]; then
    mkdir -p "$COMPLETIONS_DIR"
    if [ -f "completions/ci.bash" ]; then
        cp completions/ci.bash "$COMPLETIONS_DIR/ci"
        echo "✅ Installed bash completion"
    fi
    
    # Fish completions
    FISH_COMPLETIONS_DIR="$HOME/.config/fish/completions"
    if [ -f "completions/ci.fish" ]; then
        mkdir -p "$FISH_COMPLETIONS_DIR"
        cp completions/ci.fish "$FISH_COMPLETIONS_DIR/ci.fish"
        echo "✅ Installed fish completion"
    fi
    
    # Zsh completions
    ZSH_COMPLETIONS_DIR="$HOME/.zsh/completions"
    if [ -f "completions/ci.zsh" ]; then
        mkdir -p "$ZSH_COMPLETIONS_DIR"
        cp completions/ci.zsh "$ZSH_COMPLETIONS_DIR/_ci"
        echo "✅ Installed zsh completion"
    fi
fi

echo "✅ CI installation complete!"
echo "You can now run 'ci' to use the Collaborative Intelligence CLI"