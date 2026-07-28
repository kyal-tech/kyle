#!/bin/bash
set -eu

export PATH="/opt/homebrew/opt/rustup/bin:/opt/homebrew/opt/llvm@18/bin:$PATH"
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

echo "→ Building ky compiler..."
cargo build --release --bin ky

echo "→ Building runtime..."
cargo build --release -p kyc_runtime

echo "→ Installing to ~/.ky/..."
mkdir -p ~/.ky/bin ~/.ky/lib
cp target/release/ky ~/.ky/bin/ky
cp target/release/libkyc_runtime.a ~/.ky/lib/libkyc_runtime.a
chmod +x ~/.ky/bin/ky

if [ "$(uname -s)" = "Darwin" ]; then
    xattr -d com.apple.quarantine ~/.ky/bin/ky 2>/dev/null || true
    codesign -f -s - ~/.ky/bin/ky 2>/dev/null || true
fi

echo "→ Verifying..."
~/.ky/bin/ky --version

echo ""
echo "✅ Installed. Try:"
echo "   ~/.ky/bin/ky run examples/hello.ky"
echo "   export PATH=\"\$HOME/.ky/bin:\$PATH\" && ky run examples/hello.ky"
