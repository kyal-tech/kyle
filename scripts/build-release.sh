#!/bin/bash
set -euo pipefail

# build-release.sh — Build Kyle bundles locally
#
# IMPORTANT: The `ky` binary links against LLVM C libraries (via inkwell).
# LLVM static libs (.a) are architecture-specific. Cross-compiling `ky`
# from ARM → x86_64 requires x86_64 LLVM libraries.
#
# Only `kyc_runtime` (pure Rust) can be freely cross-compiled.
# For non-native `ky` builds, use GitHub Actions CI (native runners):
#   .github/workflows/release.yml
#
# Native builds on this host:     ✅ always works
# kyc_runtime cross-compile:      ✅ works (pure Rust)
# ky cross-compile (non-native):  ❌ needs LLVM libs for target arch
#
# Prerequisites (one-time):
#   rustup target add x86_64-apple-darwin
#   rustup target add x86_64-unknown-linux-gnu
#   rustup target add x86_64-pc-windows-gnu
#   brew install mingw-w64

VERSION="${KY_VERSION:-$(grep '^version =' Cargo.toml | head -1 | cut -d'"' -f2)}"
PROFILE="${CARGO_PROFILE:-release}"
DIST="dist"
HOST_TARGET="$(rustc -vV | grep host | cut -d' ' -f2)"

target_bundle() {
    case "$1" in
        aarch64-apple-darwin)     echo "ky-macos-arm64" ;;
        x86_64-apple-darwin)      echo "ky-macos-x64" ;;
        x86_64-unknown-linux-gnu) echo "ky-linux-x64" ;;
        aarch64-unknown-linux-gnu) echo "ky-linux-arm64" ;;
        x86_64-pc-windows-gnu)    echo "ky-windows-x64" ;;
        *)                        echo "unknown" ;;
    esac
}

build_target() {
    local target="$1"
    local bundle="$2"
    local ext="${3:-tar.gz}"

    echo "==> [$bundle] Building for $target..."

    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target"; then
        echo "    SKIP: target '$target' not installed"
        echo "    Run: rustup target add $target"
        return 1
    fi

    # Build runtime (pure Rust — always cross-compilable)
    echo "    Compiling kyc_runtime..."
    if [ "$target" = "$HOST_TARGET" ]; then
        cargo build --profile "$PROFILE" -p kyc_runtime 2>&1 | sed 's/^/      /' || {
            echo "    FAIL: kyc_runtime build failed"
            return 1
        }
    elif command -v cargo-zigbuild &>/dev/null; then
        cargo zigbuild --profile "$PROFILE" --target "$target" -p kyc_runtime 2>&1 | sed 's/^/      /' || {
            echo "    FAIL: kyc_runtime build failed"
            echo "    Tip: install cargo-zigbuild (pip3 install cargo-zigbuild)"
            return 1
        }
    else
        echo "    SKIP: need cargo-zigbuild for cross-compilation"
        echo "    Run: pip3 install cargo-zigbuild"
        return 1
    fi

    # Build compiler (only native target — LLVM dependency)
    if [ "$target" = "$HOST_TARGET" ]; then
        echo "    Compiling ky (native)..."
        cargo build --profile "$PROFILE" --bin ky 2>&1 | sed 's/^/      /' || {
            echo "    FAIL: ky build failed"
            return 1
        }
        local src_dir="target/$PROFILE"
    else
        echo "    SKIP: ky binary (non-native target — use CI for $target)"
        local src_dir="target/$target/$PROFILE"
    fi

    # Package (flat structure — no top-level dir)
    mkdir -p "$DIST"
    local abs_dist
    abs_dist="$(cd "$DIST" && pwd)"
    local bundle_dir="$abs_dist/_tmp_$bundle"
    local archive="$abs_dist/$bundle.$ext"

    mkdir -p "$bundle_dir"
    if [ "$target" = "$HOST_TARGET" ]; then
        cp "$src_dir/ky" "$bundle_dir/"
    fi
    if [ -f "$src_dir/libkyc_runtime.a" ]; then
        cp "$src_dir/libkyc_runtime.a" "$bundle_dir/"
    fi
    cp LICENSE "$bundle_dir/"

    # Create archive from within the temp dir (flat)
    (
        cd "$bundle_dir"
        if [ "$ext" = "zip" ]; then
            zip -r "$archive" .
        else
            tar czf "$archive" .
        fi
    )

    # SHA256
    (cd "$abs_dist" && shasum -a 256 "$bundle.$ext" > "$bundle.$ext.sha256")

    echo "    OK: $archive"
    rm -rf "$bundle_dir"
}

echo "=== Kyle v$VERSION — Build Release ==="
echo "Host:    $HOST_TARGET"
echo "Profile: $PROFILE"
echo ""

TARGET_LIST="aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-gnu"

BUILT=0
FAILED=0
for target in $TARGET_LIST; do
    bundle="$(target_bundle "$target")"
    ext="tar.gz"
    if [[ "$target" == *windows* ]]; then
        ext="zip"
    fi

    if build_target "$target" "$bundle" "$ext"; then
        BUILT=$((BUILT + 1))
    else
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=== Done: $BUILT built, $FAILED skipped ==="
echo ""
echo "Artifacts in $DIST/:"
ls -lh "$DIST/"*.{tar.gz,zip} 2>/dev/null || true
echo ""
echo "Checksums:"
cat "$DIST/"*.sha256 2>/dev/null || true
