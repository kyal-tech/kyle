# Kyle Monorepo — AI Agent Context

> Single entry-point for AI agents and team members working on the Kyle codebase.
> See also: [`BUILD.md`](BUILD.md) · [`BENCHMARKS.md`](BENCHMARKS.md) · [`VSCODE.md`](VSCODE.md) · [`PACKAGES.md`](PACKAGES.md) · [`tests/SYNTAX_CHECKLIST.md`](tests/SYNTAX_CHECKLIST.md)

## Quick Reference

```bash
# Build
cargo build --release --bin ky

# Run
ky run examples/hello.ky

# Test
cargo test --workspace --exclude kyc_runtime_wasm
for f in tests/syntax/*.ky; do ky run "$f"; done
```

## Architecture Overview

```
kl/                          ← Monorepo root
├── crates/                  ← Rust compiler & tooling
│   ├── kyc_frontend/        → Lexer + parser
│   ├── kyc_hir/             → HIR desugaring
│   ├── kyc_semantic/        → Type checker, borrow analysis
│   ├── kyc_mir/             → MIR lowering, SSA, optimizations
│   ├── kyc_backend/         → LLVM codegen, runtime linkage
│   ├── kyc_driver/          → Compilation pipeline
│   ├── kyc_cli/             → CLI binary (`ky`)
│   ├── kyc_ui/              → .kyx parser → UI-IR → backends
│   ├── kyc_runtime/         → Runtime library (Rust)
│   └── kyc_tools/           → LSP, formatter, package manager
│
├── packages/                → Kyle libraries (see [`PACKAGES.md`](PACKAGES.md))
├── vscode-extension/        → VS Code extension (see [`VSCODE.md`](VSCODE.md))
├── kyle-benchmarks/         → Multi-language benchmarks (see [`BENCHMARKS.md`](BENCHMARKS.md))
├── docs/                    → Language docs, specs, RFCs, roadmap
├── tests/                   → 13 syntax tests (see [`SYNTAX_CHECKLIST.md`](tests/SYNTAX_CHECKLIST.md))
├── examples/                → .ky + .kyx example programs
└── scripts/                 → install.sh, install.ps1
```

- **`.ky`** (core language) → Rust compiler (`crates/kyc_*`)
- **`.kyx`** (UI markup) → Rust parser + backends + Kyle component library
- **Packages** (`packages/*/`) → Pure Kyle libraries

## Compilation Pipeline

```
source (.ky) → kyc_frontend (lexer+parser) → kyc_hir (desugar)
  → kyc_semantic (type check) → kyc_mir (lower+SSA+optimize)
  → kyc_backend (LLVM codegen) → binary

.kyx → kyc_ui parser → UI-IR → (web: JS | desktop: native | ios: Swift)
```

## Documentation Map

| Document | Location | Content |
|----------|----------|---------|
| **AGENTS.md** (this) | [`AGENTS.md`](AGENTS.md) | Main entry point |
| **Build guide** | [`BUILD.md`](BUILD.md) | Build, install, test instructions |
| **Benchmarks** | [`BENCHMARKS.md`](BENCHMARKS.md) | Multi-language benchmark runner |
| **VS Code Extension** | [`VSCODE.md`](VSCODE.md) | Extension features, install, development |
| **Packages** | [`PACKAGES.md`](PACKAGES.md) | Library development guide |
| **Syntax reference** | `docs/15-kyle-syntax-reference.md` | Complete .ky language reference |
| **UI syntax** | `docs/03-language/syntax/ui-syntax.md` | .kyx component markup spec |
| **SYNTAX_CHECKLIST** | [`tests/SYNTAX_CHECKLIST.md`](tests/SYNTAX_CHECKLIST.md) | 244/247 features verified |
| **Remaining work** | `docs/11-project/remaining-work.md` | Bugs & features status |
| **UI design** | `docs/03-language/ui/*.md` | Routing, styles, a11y, i18n, SSR |
| **Type system** | `docs/09-specification/` | Type system, ABI, memory model |
| **RFCs** | `docs/10-design/rfc/` | 0005 UI architecture |

## Current State

### ✅ Core Language (.ky) — Stable

| Component | Status |
|-----------|--------|
| Lexer + Parser | ✅ |
| Type system (generics, enums, classes) | ✅ |
| Borrow checker | ✅ |
| MIR (lowering, SSA, optimizations) | ✅ |
| LLVM codegen | ✅ |
| Collections: `[T]`, `{K:V}`, `set{T}`, `queue{T}`, `stack{T}`, `deque{T}` | ✅ |
| Orthogonal types: `^` `&` `?` `!` on all types | ✅ |
| Module system: `use X.Y` | ✅ |
| Package manager: `ky add`, `ky.toml` | ✅ |
| Syntax tests: 13/13 passing | ✅ |
| Workspace tests: 117+ passing | ✅ |

### ✅ UI Framework (.kyx) — Web Working

| Component | Status |
|-----------|--------|
| `.kyx` parser → UI-IR | ✅ |
| Web backend (JS generation) | ✅ Functional |
| JS runtime (9 files: glue, router, reactivity, a11y, i18n, ssr, portal, error boundary, testing) | ✅ |
| 30+ UI components (packages/ui/src/components/) | ✅ |
| Module resolver (views/, components/, src/) | ✅ |
| Routing: `<router>` + `<route>` + `<layout>` + `<slot>` | ✅ |
| Theming (light/dark) | ✅ |
| Styles, animations, state/events | ✅ |
| File picker, form models | ✅ |

### ✅ VS Code Extension — Included in Monorepo

| Component | Location | Status |
|-----------|----------|--------|
| Extension manifest | `vscode-extension/package.json` | ✅ v0.8.7 |
| .ky grammar | `vscode-extension/syntaxes/ky.tmLanguage.json` | ✅ Updated |
| .kyx grammar | `vscode-extension/syntaxes/kyx.tmLanguage.json` | ✅ Updated |
| Snippets (60+) | `vscode-extension/snippets/ky.json` + `kyx.json` | ✅ Complete |
| LSP client | `vscode-extension/src/extension.ts` | ✅ |
| Debugger UI | `vscode-extension/src/debugger.ts` | ✅ |
| Testing UI | `vscode-extension/src/testUI.ts` | ✅ |
| Theme | `vscode-extension/themes/` | 🟡 Needs color refinements |

### 🟡 UI Backends — Needing Work

| Backend | Status |
|---------|--------|
| **Web** | ✅ Functional |
| **Desktop (SDL2/Skia)** | 🟡 WIP — SDL_PollEvent + RenderFillRect added |
| **iOS (SwiftUI)** | 🟡 Broken — invalid Swift output |
| **WASM** | ❌ Untested |
| **Android** | ❌ Does not exist |
| **TUI (Terminal)** | 📅 Pending |

### ✅ Packages (publishable Kyle code)

| Package | Status | Notes |
|---------|--------|-------|
| `ui/` | ✅ | 30 .kyx components, themes, desktop renderer |
| `http/` | ✅ | Client + server + websocket |
| `json/` | ✅ | In registry (docs/packages/json.json) |
| `sqlite/` | ✅ | SQLite bindings |
| `env/` | ✅ | Environment variables |
| `postgres/` | ✅ | PostgreSQL client (WIP, needs testing) |
| `webapp/` | ✅ | Project template for `ky new` |
| `std/` | ✅ | Standard library modules |

### ⏸️ Self-Hosting — Paused

The self-hosting compiler (`runtimes/ky/`) works for transpiling simple Kyle to C, but has an architectural limitation: `kyle_main` can't be a real C function due to the two-stage transpilation pipeline (Kyle→C→clang→binary). Not a priority. See `docs/11-project/self-hosting.md`.

## ✅ All Known Compiler Bugs Fixed

| Bug | Status |
|-----|--------|
| `return ok(val)` / `return error(msg)` fallible return | ✅ FIXED |
| `match result: ok(n): n error(e): 0` (T! from function) | ✅ FIXED |
| `^[Token].push()` inside loops → `.len()==0` | ✅ WORKS |
| `^[str].pop()` / `.first()` / `.last()` / `.get()` returned garbage | ✅ FIXED |
| `^[Token].pop()` returned garbage | ✅ FIXED |
| String `>=` `<=` `>` `<` comparison wrong | ✅ FIXED |
| `.find()` → link error | ✅ FIXED |
| `.split()` returned wrong substrings | ✅ FIXED |
| `ky_clone_str` → link error crash | ✅ FIXED |
| `_name`/`__name` scope resolver | ✅ FIXED |
| `&[Token]`/`^[Token]` function params → LLVM type mismatch | ✅ FIXED |
| `&[T]` slice creation `arr[0..2]` → returns garbage | ✅ FIXED |
| Field access on classes returns `i32` | ✅ FIXED |
| `str_builder_free` crash | ✅ Fixed |
| `_call` SSA verify error in large files | ✅ FIXED |
| `!` error propagation operator | ✅ FIXED |
| `&str` prelude functions (fs, uuid, etc.) SIGSEGV | ✅ FIXED |

## Unimplemented Features (no priority)

- `unsafe` block (`as_ptr`)
- Macros (`macro_rules!`, derive, proc macros)
- Async/await (runtime exists, no compiler lowering)

### 🟡 MEDIUM — UI (.kyx) Backend Fixes

| Task | Files | Status |
|------|-------|--------|
| Fix desktop backend (SDL2 events, rendering) | `kyc_ui/src/backend/desktop.rs` | 🟡 |
| Fix iOS backend (Swift output) | `kyc_ui/src/backend/ios.rs` | 🟡 |
| Create .kyx integration tests | `tests/ui/` | ❌ |
| Test WASM target | `kyc_ui/src/backend/web.rs` | ❌ |

### 🟢 LOW — Packages & Documentation

| Task | Files |
|------|-------|
| Test all packages compile with current syntax | ✅ done — all pass `ky check` |
| Create more .kyx examples | `examples/*.kyx` |
| Update docs/11-project/ui-roadmap.md | `docs/11-project/` |
| Deprecate old `kyle-packages` repo | README there |

## Documentation Map

| Document | Content | Location |
|----------|---------|----------|
| Language syntax | Complete .ky reference | `docs/15-kyle-syntax-reference.md` |
| .kyx UI syntax | Component markup spec v2.0 | `docs/03-language/syntax/ui-syntax.md` |
| UI design docs | Styles, routing, a11y, i18n, SSR, etc. | `docs/03-language/ui/*.md` |
| UI architecture RFCs | 0002 (original), 0003 (translation), 0005 (v2) | `docs/10-design/rfc/` |
| Roadmap | Overall project state | `docs/11-project/roadmap.md` |
| Test checklist | What to test before release | `docs/11-project/test-checklist.md` |
| VS Code extension | Grammar, snippets, themes, LSP client | `vscode-extension/` |
| Extension package.json | Manifest, commands, activation events | `vscode-extension/package.json` |
| Extension grammar (.ky) | TextMate grammar for `.ky` files | `vscode-extension/syntaxes/ky.tmLanguage.json` |
| Extension grammar (.kyx) | TextMate grammar for `.kyx` files | `vscode-extension/syntaxes/kyx.tmLanguage.json` |
| Extension snippets | 60+ code snippets for .ky and .kyx | `vscode-extension/snippets/` |
| Extension theme | "Kyle Pastel" dark theme | `vscode-extension/themes/` |

## Key Commands

```bash
cargo build --release --bin ky         # Build
cargo test --workspace                  # Run all Rust tests
ky run examples/hello.ky                # Run Kyle program
ky build examples/counter.kyx           # Build UI web app
ky check packages/ui/src/lib.kyx        # Type-check UI package
ky test packages/http/tests/test_http.ky  # Run package test
ky add http                             # Install package
ky new myapp                            # Create new project
ky doc                                  # Open docs
```
