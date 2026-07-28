# Building Kyle

## Requirements

- **Rust** 1.81+ (`rustup install stable`)
- **LLVM 18** runtime libraries
- **make**, **gcc**/**clang** (for linking)

### Install LLVM 18

```bash
# macOS
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

# Linux (Debian/Ubuntu)
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev

# Windows (PowerShell as Admin)
choco install llvm --version=18.1.8
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"
```

## Build the Compiler

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE

# Release build (recommended)
cargo build --release --bin ky

# Debug build
cargo build --bin ky
```

## Quick Test

```bash
# Run a .ky file
echo 'print("Hello from Kyle!")' > hello.ky
./target/release/ky run hello.ky

# Type-check only
./target/release/ky check hello.ky

# Build UI app (.kyx)
./target/release/ky build examples/counter.kyx

# Build for desktop
./target/release/ky build examples/counter.kyx desktop

# Build for iOS
./target/release/ky build examples/counter.kyx --target ios
```

## Run Tests

```bash
# All workspace tests
cargo test --workspace --exclude kyc_runtime_wasm

# Syntax tests
for f in tests/syntax/*.ky; do
    ./target/release/ky run "$f" && echo "PASS: $f" || echo "FAIL: $f"
done

# Package type-checks
./target/release/ky check packages/http/src/lib.ky
./target/release/ky check packages/sqlite/src/lib.ky
./target/release/ky check packages/postgres/src/lib.ky
./target/release/ky check packages/env/src/lib.ky
./target/release/ky check packages/ui/src/lib.kyx
```

## Install

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.sh | sh

# Windows (PowerShell)
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.ps1" | iex
```

## Install VS Code Extension

```bash
# From marketplace
code --install-extension kynera.ky

# Or from .vsix
code --install-extension ky-0.8.7.vsix
```

> Extension source at [`vscode-extension/`](vscode-extension/)
