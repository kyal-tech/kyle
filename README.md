<div align="center">

# Kyle

**A compiled, statically-typed language for backend systems and CLI tools.**

Readable like Python · Typed like Rust · Simple like Go · **Fast like C**

[![License: MIT](https://img.shields.io/badge/license-MIT-6C3FC5?style=for-the-badge)](LICENSE)
[![Release](https://img.shields.io/badge/release-v0.8.9-6C3FC5?style=for-the-badge)](https://github.com/kyal-tech/kyle/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS%20ARM/x64%20%7C%20Linux%20ARM/x64%20%7C%20Windows%20x64-6C3FC5?style=for-the-badge)](#install)
[![VS Code](https://img.shields.io/badge/VS%20Code-ky-6C3FC5?style=for-the-badge)](vscode-extension/)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-6C3FC5?style=for-the-badge)](https://www.rust-lang.org)

</div>

---

## Why Kyle?

Kyle compiles directly to **native machine code** via LLVM. You get the ergonomics of a high-level language with the performance of C — without garbage collection pauses, without a heavyweight runtime, and without giving up type safety.

- ⚡ **Native speed** — compiles to machine code, benchmarks within **1.0–1.3× of C**
- 🔒 **Statically typed** — deep type system: `?` optional, `!` fallible, `^` mutable, `&` borrow
- 🧵 **Batteries included** — threads, channels, mutexes, async, TCP, HTTP, JSON, regex, crypto
- 📦 **Packages** — install and reuse libs with `ky add`
- 🧹 **Predictable** — no GC, deterministic ownership, low memory footprint

---

## Quick Start

```bash
# One-command install (macOS / Linux)
curl -fsSL https://raw.githubusercontent.com/kyal-tech/kyle/main/scripts/install.sh | sh

# Create a project and run it
ky new myapp && cd myapp
ky run
```

Or compile a single file straight to a native binary:

```bash
echo 'print("Hello from Kyle!")' > hello.ky
ky run hello.ky
ky build hello.ky && ./hello   # native binary
```

---

## Hello, Kyle

```kyle
print("Hello, World!")
```

### Variables — immutable by default, mutable with `^`

```kyle
name = "Kyle"          # immutable (default)
count: ^i32 = 0        # mutable
count += 1
```

### Fallible results with `!` — no more exceptions

```kyle
fn parse(s: &str) i32!:
    n = int(s)?
    if n < 0: return error("negative")
    n

x: i32! = parse("42")
y = x!                 # propagates the error automatically
```

### Possibilities with `?` — no more null checks

```kyle
fn find_user(id: i32) User?:
    ...
user = find_user(1)?
if user == nil: ...
```

### Orthogonal types

```kyle
x: i32            # primitive
x: i32?           # optional
x: i32!           # fallible (Result)
x: ^i32           # mutable
x: &str           # borrow
x: ^&[i32]!       # mutable borrow of a list, may error
```

### Collections

```kyle
items = [1, 2, 3]                    # list [i32]
nums = set{1, 2, 3}                  # set <i32>
dict = {"name": "kyle", "ver": 1}    # dict {str: i32}
q = queue{1, 2, 3}                   # queue <i32>
s = stack{"a", "b"}                  # stack <str>
```

---

## Performance

Kyle compiles to native code and sits squarely next to C. Benchmarks on an **Apple M5**, release mode, median of 15 runs:

| Benchmark | C | C++ | Rust | **Kyle** | **Kyle vs C** |
| :--- | ---: | ---: | ---: | ---: | ---: |
| Fibonacci (500M iters) | 118ms | 121ms | 122ms | **128ms** | **1.08×** |
| Prime Sieve (3M) | 8.5ms | 8.3ms | 8.6ms | **10.6ms** | **1.25×** |
| String Concat (500k) | 8.2ms | 8.1ms | 1.9ms | **10.3ms** | **1.26×** |
| MatMul (100×100) | 6.6ms | 6.8ms | 7.2ms | **6.7ms** | **1.02×** |

> Full benchmark runner and history: [`BENCHMARKS.md`](BENCHMARKS.md)

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

Syntax highlighting, LSP integration, snippets, debugging UI, and a color theme.

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

> **Source**: The extension source is at [`vscode-extension/`](vscode-extension/) in this repository.

---

## Packages

The standard library ships with the language. Install extra libraries with `ky add`:

```bash
ky add http        # HTTP client/server
ky add postgres    # PostgreSQL
ky add sqlite      # SQLite
ky add env         # .env loader
```

| Package | Description |
| :------- | :---------- |
| [`http`](docs/packages/http.md) | HTTP client and server |
| [`postgres`](docs/packages/postgres.md) | PostgreSQL driver |
| [`sqlite`](docs/packages/sqlite.md) | SQLite driver |
| [`env`](docs/packages/env.md) | .env loader |

```kyle
use std.http
use std.json
use http.server.*
```

---

## Commands

```bash
ky new <project>      # create new project
ky run <file.ky>      # compile and run
ky build <file.ky>    # compile to native binary
ky check <file.ky>    # type-check only
ky parse <file.ky>    # dump AST
ky mir <file.ky>      # dump MIR
ky fmt <file.ky>      # format source
ky test               # run project tests
ky add <package>      # install a package
```

---

## Documentation

| Resource | Location |
| :------- | :------- |
| Language Syntax | `docs/03-language/syntax/` |
| Type System | `docs/09-specification/type-system.md` |
| Collections | `docs/03-language/syntax/collections.md` |
| Modules & Imports | `docs/03-language/syntax/modules.md` |
| Syntax Reference | `docs/15-kyle-syntax-reference.md` |
| VS Code Extension | `vscode-extension/` |
| Benchmarks | `BENCHMARKS.md` |
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
cd kyle
cargo build --release --bin ky
```

---

## License

[MIT](LICENSE) — Copyright (c) 2026 [Kynera](https://github.com/IT-KYNERA)
