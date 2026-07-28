# Kyle Monorepo — AI Agent Context

> Single entry-point for AI agents and team members working on the Kyle codebase.
> See also: [`BUILD.md`](BUILD.md) · [`BENCHMARKS.md`](BENCHMARKS.md) · [`VSCODE.md`](VSCODE.md) · [`PACKAGES.md`](PACKAGES.md) · [`tests/SYNTAX_CHECKLIST.md`](tests/SYNTAX_CHECKLIST.md) · [`KYUI_ROADMAP.md`](KYUI_ROADMAP.md)

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
| **Kyle UI Roadmap** | [`KYUI_ROADMAP.md`](KYUI_ROADMAP.md) | Implementation plan & priorities |

## Kyle UI Documentation

Complete documentation for Kyle UI framework at `docs/03-language/ui/`:

### Core Documentation

| Document | Description |
|----------|-------------|
| [README.md](docs/03-language/ui/README.md) | Main index & quick start |
| [architecture.md](docs/03-language/ui/architecture.md) | Multi-platform architecture, anti-patterns |
| [style-system.md](docs/03-language/ui/style-system.md) | Typed styles (color, spacing, layout, theme) |
| [state-events.md](docs/03-language/ui/state-events.md) | State, events, binding, forms, validation |
| [events.md](docs/03-language/ui/events.md) | Complete event system (click, hover, touch, keyboard) |
| [lifecycle.md](docs/03-language/ui/lifecycle.md) | Component lifecycle hooks (on_mounted, on_unmounted, etc.) |
| [animation.md](docs/03-language/ui/animation.md) | Animations & transitions |
| [routing.md](docs/03-language/ui/routing.md) | Routing, navigation, guards |
| [accessibility.md](docs/03-language/ui/accessibility.md) | WCAG 2.1 AA, ARIA, keyboard, screen readers |
| [anti-patterns.md](docs/03-language/ui/anti-patterns.md) | Anti-patterns from other frameworks |
| [framework-comparison.md](docs/03-language/ui/framework-comparison.md) | Comparison with React, Vue, SwiftUI, Compose, Flutter |

### Component Documentation

All components documented at `docs/03-language/ui/components/`:

| Category | Components |
|----------|-----------|
| **Layout** | view, card |
| **Text** | text, link |
| **Input** | button, text_field, text_area, checkbox, radio, switch, slider, select, file_picker, form |
| **Media** | img, video, audio |
| **Feedback** | progress, spinner, skeleton |
| **Overlay** | modal, alert, tooltip, toast |
| **Navigation** | app_bar, sidebar, tab_bar, bottom_nav |
| **Data** | list, table, grid |

### Advanced Patterns

| Document | Description |
|----------|-------------|
| [composition.md](docs/03-language/ui/composition.md) | Slots, render props, compound components |
| [context-patterns.md](docs/03-language/ui/context-patterns.md) | Context, selectors, reducers |
| [portals.md](docs/03-language/ui/portals.md) | Portals for modals, tooltips |
| [error-boundaries.md](docs/03-language/ui/error-boundaries.md) | Error handling, fallback UI |

### Infrastructure

| Document | Description |
|----------|-------------|
| [ssr.md](docs/03-language/ui/ssr.md) | Server-Side Rendering |
| [i18n.md](docs/03-language/ui/i18n.md) | Internationalization |
| [testing.md](docs/03-language/ui/testing.md) | Testing (unit, integration, E2E) |
| [file-picker.md](docs/03-language/ui/file-picker.md) | Native file picker |

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
| **Complete documentation** (35+ components, events, lifecycle) | ✅ |

### ✅ VS Code Extension — Included in Monorepo

| Component | Location | Status |
|-----------|----------|--------|
| Extension manifest | `vscode-extension/package.json` | ✅ v0.8.8 |
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
| **Web** | ✅ Functional (see [`KYUI_ROADMAP.md`](KYUI_ROADMAP.md) for remaining features) |
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
# Build compiler
cargo build --release --bin ky

# Development install (compiler + runtime)
./scripts/dev-install.sh

# Development install (VS Code extension)
./scripts/dev-ext.sh

# Run tests
cargo test --workspace

# Run Kyle program
ky run examples/hello.ky

# Build UI web app
ky build examples/counter.kyx

# Type-check UI package
ky check packages/ui/src/lib.kyx

# Run package test
ky test packages/http/tests/test_http.ky

# Install package
ky add http

# Create new project
ky new myapp

# Open docs
ky doc
```

## Development Workflow

### Quick Development Cycle

1. **Make changes** to compiler or UI components
2. **Build and install**: `./scripts/dev-install.sh`
3. **Test**: `ky run examples/hello.ky`
4. **Repeat**

### VS Code Extension Development

1. **Make changes** to extension code
2. **Build and install**: `./scripts/dev-ext.sh`
3. **Reload VS Code** window (Cmd+Shift+P → Reload Window)
4. **Test** extension features

### UI Component Development

1. **Edit component** in `packages/ui/src/components/`
2. **Build**: `./scripts/dev-install.sh`
3. **Test in browser**: `ky run web` in a kyui project
4. **Verify** component renders correctly

## Implementation Priorities

See [`KYUI_ROADMAP.md`](KYUI_ROADMAP.md) for detailed implementation plan.

### Current Focus: Phase 1 - Core Web Backend

**Priority:** 🔴 CRITICAL

- [x] Implement touch events (touch_start, touch_end, touch_move)
- [x] Implement lifecycle hooks (on_created, on_mounted, on_updated, on_unmounted)
- [x] Implement image lazy loading
- [x] Implement list virtualization
- [x] Create unit tests
- [x] Implement CSS transitions
- [ ] Create integration tests

### Next: Phase 2 - Testing & Validation

**Priority:** 🔴 CRITICAL

- [ ] Unit tests (>80% coverage)
- [ ] Integration tests
- [ ] Browser testing (Chrome, Firefox, Safari, Edge)
- [ ] Mobile browser testing
- [ ] Accessibility testing

### Future: Phase 3-6

- Phase 3: Desktop backend (SDL2/Skia)
- Phase 4: iOS backend (SwiftUI)
- Phase 5: Android backend (Jetpack Compose)
- Phase 6: Advanced features (SSR, DevTools)
