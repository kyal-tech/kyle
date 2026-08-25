<div align="center">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · Fast like C

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Release](https://img.shields.io/badge/release-v0.8.9-6C3FC5?style=for-the-badge)](https://github.com/kyal-tech/kyle/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS%20ARM/x64%20%7C%20Linux%20ARM/x64%20%7C%20Windows%20x64-6C3FC5?style=for-the-badge)](#install)
[![VS Code](https://img.shields.io/badge/VS%20Code-ky-6C3FC5?style=for-the-badge)](vscode-extension/)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)

</div>

---

## Install

### 1. Compiler (`ky`)

**macOS / Linux** (one command):

```bash
curl -fsSL https://raw.githubusercontent.com/kyal-tech/kyle/main/scripts/install.sh | sh
```

**Windows** (PowerShell):

```powershell
iwr -Uri "https://raw.githubusercontent.com/kyal-tech/kyle/main/scripts/install.ps1" | iex
```

| Platform | Arch | Direct link |
| :--- | :--- | :--- |
| **macOS** | ARM64 | [ky-macos-arm64.tar.gz](https://github.com/kyal-tech/kyle/releases/download/v0.8.9/ky-macos-arm64.tar.gz) |
| **Linux** | ARM64 | [ky-linux-arm64.tar.gz](https://github.com/kyal-tech/kyle/releases/download/v0.8.9/ky-linux-arm64.tar.gz) |
| **Linux** | x64 | [ky-linux-x64.tar.gz](https://github.com/kyal-tech/kyle/releases/download/v0.8.9/ky-linux-x64.tar.gz) |
| **Windows** | x64 | [ky-windows-x64.zip](https://github.com/kyal-tech/kyle/releases/download/v0.8.9/ky-windows-x64.zip) |

> **Note**: macOS Intel (x64) is no longer supported. Use Apple Silicon (ARM64).

### 2. VS Code Extension (`ky` language support)

The extension provides syntax highlighting, LSP integration, snippets, debugging UI, and a color theme.

**macOS / Linux**:

```bash
# Install from VS Code Marketplace
code --install-extension kynera.ky
```

Or download the `.vsix` from the [releases page](https://github.com/kyal-tech/kyle/releases) and install:

```bash
code --install-extension ky-0.8.9.vsix
```

**Windows** (PowerShell):

```powershell
# Install from VS Code Marketplace
code --install-extension kynera.ky
```

Or download the `.vsix` and install:

```powershell
code --install-extension ky-0.8.9.vsix
```

> **Source**: The extension source is at [`vscode-extension/`](vscode-extension/) in this repository.

---

## Quick Start

```bash
ky new myapp && cd myapp
ky run
```

Or run a single file:

```bash
echo 'print("Hello from Kyle!")' > hello.ky
ky run hello.ky
```

---

## Hello World

```kyle
print("Hello, World!")
```

## Variables

```kyle
name = "Kyle"          # immutable (default)
count: ^i32 = 0        # mutable with ^
count += 1

items: ^[str] = []     # mutable list
```

## Collections

```kyle
items = [1, 2, 3]                    # list [i32]
items: ^[str] = ["a", "b"]           # mutable list
nums = set{1, 2, 3}                  # set set<i32>
dict = {"name": "kyle", "ver": 1}    # dict {str: i32}
q = queue{1, 2, 3}                   # queue queue<i32>
s = stack{"a", "b"}                  # stack stack<str>
```

## Imports

```kyle
use std.io                       # module
use std.io.{print, read}         # selective
use ~utils.helpers               # relative
```

## Functions

```kyle
fn add(a: i32, b: i32) i32:
    a + b

fn greet(name: &str):
    print("Hello, " + name)
```

## Error Handling

```kyle
fn parse(s: &str) i32!:
    n = int(s)?
    if n < 0: return error("negative")
    n

x: i32! = parse("42")
y = x!   # propagate on error
```

## Types

```kyle
x: i32            # primitive
x: i32?           # optional (Option)
x: i32!           # fallible (Result)
x: ^i32           # mutable
x: &str           # borrow
x: ^&[i32]!       # mutable borrow of list, may error
x: ^&[str]?       # mutable borrow of list, optional
x: ^set<i32>!     # mutable set with error
```

---

## Commands

```bash
ky new <project>      # create new project
ky run <file.ky>      # compile and run
ky build <file.ky>    # compile to binary
ky check <file.ky>    # type-check only
ky parse <file.ky>    # dump AST
ky mir <file.ky>      # dump MIR
ky fmt <file.ky>      # format source
ky test               # run project tests
```

---

## Project Structure

```
kl/
├── crates/              → Rust crates (compiler, tooling)
│   ├── kyc_frontend     → Lexer + parser
│   ├── kyc_semantic     → Type checker, borrow analysis
│   ├── kyc_mir          → MIR lowering, SSA, optimizations
│   ├── kyc_backend      → LLVM codegen
│   ├── kyc_ui           → .kyx parser + UI backends
│   ├── kyc_cli          → CLI binary (ky)
│   ├── kyc_runtime      → Runtime library (Rust)
│   └── kyc_driver       → Compilation pipeline
├── packages/            → Kyle libraries (http, sqlite, ui, etc.)
├── vscode-extension/    → VS Code extension (grammar, snippets, LSP)
├── docs/                → Language documentation
├── tests/               → Syntax tests
└── examples/            → Example programs
```

## Documentation

| Resource | Location |
| :------- | :------- |
| Language Syntax | `docs/03-language/syntax/` |
| Type System | `docs/09-specification/type-system.md` |
| Collections | `docs/03-language/syntax/collections.md` |
| Modules & Imports | `docs/03-language/syntax/modules.md` |
| Syntax Reference | `docs/15-kyle-syntax-reference.md` |
| VS Code Extension | `vscode-extension/` |
| Roadmap | `docs/11-project/roadmap.md` |

---

## Build from Source

Requires **LLVM 18** and **Rust 1.81+**.

```bash
# Linux (Debian/Ubuntu)
sudo apt install llvm-18-dev libpolly-18-dev libzstd-dev

# macOS
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)

# Windows (PowerShell as Admin)
choco install llvm --version=18.1.8
$env:LLVM_SYS_181_PREFIX = "C:\Program Files\LLVM"

# Build
git clone https://github.com/kyal-tech/kyle.git
cd KYLE
cargo build --release --bin ky
```

---

## Development

```bash
cargo test --workspace
cargo build --workspace
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 [Kynera](https://github.com/IT-KYNERA)
