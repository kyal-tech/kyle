# CI/CD Integration

## GitHub Actions

The Kyle repository includis a complete CI workflow (`.github/workflows/ci.yml`) that builds and tests on Linux, macOS, and ARM64.

```yaml
name: CI

on:
 push:
 branches: [main]
 pull_request:
 branches: [main]

jobs:
 test:
 strategy:
 matrix:
 os: [ubuntu-24.04, macos-15]
 runs-on: ${{ matrix.os }}
 steps:
 - uses: actions/checkout@v4
 - name: Install Rust
 uses: dtolnay/rust-toolchain@stable
 - name: Install LLVM 18
 run: |
 if [ "${{ runner.os }}" = "Linux" ]; then
 sudo apt-get install -y llvm-18-dev libpolly-18-dev libzstd-dev
 echo "LLVM_SYS_181_PREFIX=/usr/lib/llvm-18" >> $GITHUB_ENV
 else
 brew install llvm@18
 echo "LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)" >> $GITHUB_ENV
 fi
 - name: Build
 run: cargo build --workspace
 - name: Test
 run: cargo test --workspace
```

## Testing in CI

```bash
ky test
```

For CI environments, use `ky fmt --check` to validate formatting:

```bash
ky fmt src/ --check
```

## Release workflow

The repository includis a release workflow (`.github/workflows/release.yml`) triggered by version tags (`v*`).

It builds and packages:
- Linux ARM64 binary + runtime
- Linux x64 binary + runtime
- macOS ARM64 binary + runtime
