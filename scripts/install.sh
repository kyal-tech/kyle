#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="${KY_VERSION:-v0.8.6}"

usage() {
    echo "Usage: curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.sh | sh"
    echo ""
    echo "Environment:"
    echo "  KY_VERSION=v0.8.6     Version (default: latest)"
    echo "  KY_PREFIX=/custom     Install dir (default: ~/.ky or /usr/local)"
    echo ""
    echo "  install.sh uninstall   Remove Kyle"
    exit 0
}

detect_platform() {
    local os arch
    case "$(uname -s)" in
        Darwin) os="macos" ;;
        Linux)  os="linux" ;;
        *)
            echo "Error: Windows? Use PowerShell:"
            echo "  iwr -Uri \"https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.ps1\" | iex"
            exit 1 ;;
    esac
    case "$(uname -m)" in
        arm64|aarch64) arch="arm64" ;;
        x86_64|amd64)  arch="x64" ;;
        *) echo "Error: unsupported arch ($(uname -m))"; exit 1 ;;
    esac
    echo "${os}-${arch}"
}

cleanup() { rm -rf "/tmp/ky-install" 2>/dev/null || true; }

# Uninstall
if [ "${1:-}" = "uninstall" ]; then
    echo "Removing Kyle..."
    for f in /usr/local/bin/ky "$HOME/.ky/bin/ky" "$HOME/.kl/bin/ky"; do
        [ -f "$f" ] && rm -f "$f" && echo "  Removed $f"
    done
    for f in /usr/local/lib/libkyc_runtime.a /usr/local/lib/ky/libkyc_runtime.a "$HOME/.ky/lib/libkyc_runtime.a" "$HOME/.kl/lib/libkyc_runtime.a"; do
        [ -f "$f" ] && rm -f "$f" && echo "  Removed $f"
    done
    for d in "$HOME/.ky" "$HOME/.kl" /usr/local/lib/ky; do
        [ -d "$d" ] && rm -rf "$d" && echo "  Removed $d"
    done
    for rc in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.profile"; do
        [ -f "$rc" ] && sed -i '' '/\.ky\/bin/d' "$rc" 2>/dev/null || true
        [ -f "$rc" ] && sed -i '/\.ky\/bin/d' "$rc" 2>/dev/null || true
    done
    echo "ky uninstalled. Close and reopen terminal."
    exit 0
fi

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then usage; fi

# Download
PLATFORM=$(detect_platform)
echo "Detected: $PLATFORM"
BUNDLE="ky-${PLATFORM}.tar.gz"
BUNDLE_URL="https://github.com/$REPO/releases/download/$VERSION/$BUNDLE"
SHA256_URL="$BUNDLE_URL.sha256"

cleanup
mkdir -p /tmp/ky-install
cd /tmp/ky-install

echo "Downloading Kyle $VERSION for $PLATFORM..."
curl -fsSL "$BUNDLE_URL" -o "$BUNDLE" || {
    echo "Error: failed to download $BUNDLE"
    echo "Check release at: https://github.com/$REPO/releases"
    exit 1
}

if curl -fsSL "$SHA256_URL" -o "${BUNDLE}.sha256" 2>/dev/null; then
    echo "Verifying checksum..."
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -c "${BUNDLE}.sha256" || { echo "Checksum failed"; exit 1; }
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c "${BUNDLE}.sha256" || { echo "Checksum failed"; exit 1; }
    fi
else
    echo "Warning: no checksum file, skipping verification"
fi

# Pre-install cleanup
[ -f /usr/local/lib/ky/libkyc_runtime.a ] && rm -f /usr/local/lib/ky/libkyc_runtime.a && rmdir /usr/local/lib/ky 2>/dev/null || true

# Extract
tar xzf "$BUNDLE"

if [ ! -f "ky" ]; then
    echo "Error: 'ky' not found in archive"
    ls -la; exit 1
fi

# macOS: sign binary + bundled dylibs (ad-hoc, no Team ID)
if [ "$(uname -s)" = "Darwin" ]; then
    echo "  Signing binary for macOS..."
    xattr -d com.apple.quarantine ky 2>/dev/null || true
    if [ -d "lib" ]; then
        for f in lib/*.dylib; do
            [ -f "$f" ] && codesign -f -s - "$f" 2>/dev/null || true
        done
    fi
    codesign -f -s - ky 2>/dev/null && echo "  Code signature applied"
fi

chmod +x ky

# Determine install dir
INSTALL_TO_USR=false
if [ "${KY_PREFIX:-}" = "" ]; then
    if [ -w /usr/local/bin ]; then
        INSTALL_TO_USR=true
    elif command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
        INSTALL_TO_USR=true
    fi
fi

if [ "$INSTALL_TO_USR" = true ] && [ "${KY_PREFIX:-}" = "" ]; then
    echo "Installing to /usr/local..."
    if [ "$(id -u)" -eq 0 ]; then
        mkdir -p /usr/local/bin /usr/local/lib
        cp ky /usr/local/bin/ky
        [ -f libkyc_runtime.a ] && cp libkyc_runtime.a /usr/local/lib/libkyc_runtime.a
        [ -d lib ] && cp -f lib/*.dylib /usr/local/lib/ 2>/dev/null || true
    else
        sudo mkdir -p /usr/local/bin /usr/local/lib
        sudo cp ky /usr/local/bin/ky
        [ -f libkyc_runtime.a ] && sudo cp libkyc_runtime.a /usr/local/lib/libkyc_runtime.a
        [ -d lib ] && sudo cp -f lib/*.dylib /usr/local/lib/ 2>/dev/null || true
    fi
    INSTALL_DIR="/usr/local/bin"
    KY_PREFIX="/usr/local"
elif [ -n "${KY_PREFIX:-}" ]; then
    echo "Installing to $KY_PREFIX..."
    mkdir -p "$KY_PREFIX/bin" "$KY_PREFIX/lib"
    cp ky "$KY_PREFIX/bin/ky"
    [ -f libkyc_runtime.a ] && cp libkyc_runtime.a "$KY_PREFIX/lib/libkyc_runtime.a"
    [ -d lib ] && cp -f lib/*.dylib "$KY_PREFIX/lib/" 2>/dev/null || true
    INSTALL_DIR="$KY_PREFIX/bin"
else
    echo "Installing to $HOME/.ky..."
    mkdir -p "$HOME/.ky/bin" "$HOME/.ky/lib"
    cp ky "$HOME/.ky/bin/ky"
    [ -f libkyc_runtime.a ] && cp libkyc_runtime.a "$HOME/.ky/lib/libkyc_runtime.a"
    [ -d lib ] && cp -f lib/*.dylib "$HOME/.ky/lib/" 2>/dev/null || true
    INSTALL_DIR="$HOME/.ky/bin"
    KY_PREFIX="$HOME/.ky"
fi

# Add to PATH
SHELL_NAME=$(basename "${SHELL:-}")
case "$SHELL_NAME" in
    zsh)  SHELL_CONFIG="$HOME/.zshrc" ;;
    bash) SHELL_CONFIG="$HOME/.bashrc" ;;
    *)    SHELL_CONFIG="" ;;
esac

if [ -n "$SHELL_CONFIG" ] && [ -w "$SHELL_CONFIG" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG" 2>/dev/null; then
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
        echo "  Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
    else
        echo "  PATH already configured in $(basename "$SHELL_CONFIG")"
    fi
elif [ -n "$SHELL_CONFIG" ]; then
    echo "  Warning: $SHELL_CONFIG not writable."
    echo "  Run: export PATH=\"$INSTALL_DIR:\$PATH\""
fi

export PATH="$INSTALL_DIR:$PATH"

# Verify
echo ""
if ky --version 2>/dev/null; then
    echo ""
    echo "✅ Kyle $VERSION installed successfully!"
    echo ""
    echo "  Binary:  $INSTALL_DIR/ky"
    [ -f "$KY_PREFIX/lib/libkyc_runtime.a" ] && echo "  Runtime: installed"
    echo ""
    echo "  Use: source ${SHELL_CONFIG:-~/.profile} && ky -v"
    echo "  Or:  export PATH=\"$INSTALL_DIR:\$PATH\" && ky -v"
    echo "  Try: ky run examples/hello.ky"
else
    echo "⚠️  Installation completed but 'ky --version' failed."
    echo "   Check PATH: $INSTALL_DIR"
fi

cleanup
