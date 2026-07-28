# Installation

> How to install the Kyle compiler on any platform.

## Requirements

**No external dependencies needed.** The `ky` binary is self-contained — it does NOT require LLVM, Rust, or any runtime libraries to run.

LLVM 18 + Rust are only needed to **build** the compiler from source.

## Quick Install (one command)

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.sh | sh
```

### Windows (PowerShell)

```powershell
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.ps1" | iex
```

Both scripts:
1. Detect OS + architecture automatically
2. Download the correct bundle from GitHub Releases
3. Verify SHA-256 checksum
4. Install `ky` binary
5. Configure PATH automatically

---

## Build from Source

Only needed if you want to modify the compiler. Requires **LLVM 18** and **Rust 1.81+**.

### Install LLVM 18

**macOS:**
```bash
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev
```

**Windows (PowerShell as Admin):**
```powershell
choco install llvm --version=18.1.8
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"
```

### Build

```bash
git clone https://github.com/IT-KYNERA/KYLE.git
cd KYLE
cargo build --release --bin ky

# Binary at: target/release/ky (or ky.exe on Windows)
```

---

## Verify Installation

```bash
ky --version    # Should show v0.8.6
ky check --help # Should show help
ky run examples/hello.ky  # Should print "Hello, World!"
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `LLVM_SYS_181_PREFIX` | Path to LLVM 18 (only for building from source) |
| `KY_VERSION` | Version to install (for install scripts) |
| `KY_PREFIX` | Custom install directory |

---

## See also

- `first-program.md` — First program
- `build.md` — Building projects
- `testing.md` — Writing and running tests
