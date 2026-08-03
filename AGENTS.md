# Kyle Monorepo — AI Agent Context

> Single entry-point for AI agents and team members working on the Kyle codebase.
> Current focus: **Backend package ecosystem** (see [`BACKEND_PACKAGES_PLAN.md`](docs/11-project/backend-packages-plan.md))
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
for f in tests/std_*.ky; do ky run "$f"; done

# Check package
ky check packages/http/src/lib.ky

# Run package tests
ky test packages/http/tests/test_http.ky

# Install package
ky add http

# Create project
ky new myapp
```

## Architecture Overview

```
kl/
├── crates/                  ← Rust compiler & tooling
│   ├── kyc_frontend/        → Lexer + parser
│   ├── kyc_hir/             → HIR desugaring
│   ├── kyc_semantic/        → Type checker, borrow analysis
│   ├── kyc_mir/             → MIR lowering, SSA, optimizations
│   ├── kyc_backend/         → LLVM codegen, runtime linkage
│   ├── kyc_driver/          → Compilation pipeline
│   ├── kyc_cli/             → CLI binary (`ky`)
│   ├── kyc_runtime/         → Runtime library (Rust) — 190+ extern "C" functions
│   └── kyc_tools/           → LSP, formatter, package manager
│
├── packages/                → Kyle installable packages
├── vscode-extension/        → VS Code extension
├── docs/                    → Language docs, specs, plans
│   ├── 04-standard-library/ → 21 std modules (native)
│   ├── packages/            → 9 package docs
│   └── 11-project/          → Roadmaps, plans
├── tests/                   → Syntax tests, package tests
├── examples/                → .ky + .kyx example programs
└── scripts/                 → install.sh, dev-install.sh
```

## Backend Ecosystem

### Standard Library (native — `use std.<module>`)

| Module | File | Status |
|--------|------|--------|
| `result` | `docs/04-standard-library/result.md` | ✅ Runtime (language) |
| `json` | `docs/04-standard-library/json.md` | ✅ Runtime (`ky_json_*`) |
| `time` | `docs/04-standard-library/time.md` | ✅ Runtime (`ky_datetime_*`) |
| `fs` | `docs/04-standard-library/fs.md` | ✅ Runtime (`ky_fs_*`) |
| `path` | `docs/04-standard-library/path.md` | ✅ Runtime (`ky_path_*`) |
| `str` | `docs/04-standard-library/str.md` | ✅ Runtime (`ky_str_*`) |
| `math` | `docs/04-standard-library/math.md` | ✅ Runtime (`ky_pow`, etc.) |
| `io` | `docs/04-standard-library/io.md` | ✅ Runtime (`ky_print*`) |
| `net` | `docs/04-standard-library/net.md` | ✅ Runtime (`ky_tcp_*`) |
| `random` | `docs/04-standard-library/random.md` | ✅ Runtime (`ky_random_bytes`) |
| `regex` | `docs/04-standard-library/regex.md` | ✅ Runtime (`ky_regex_*`) |
| `thread` | `docs/04-standard-library/thread.md` | ✅ Runtime (`ky_spawn_thread`) |
| `sync` | `docs/04-standard-library/sync.md` | ✅ Runtime (`ky_mutex_*`) |
| `crypto` | `docs/04-standard-library/crypto.md` | ✅ Runtime (sha256, base64, uuid) |
| `process` | `docs/04-standard-library/process.md` | ✅ Runtime (`ky_getenv`, libc) |
| `testing` | `docs/04-standard-library/testing.md` | ✅ Runtime (`ky_assert*`) |
| `log` | `docs/04-standard-library/log.md` | ✅ Runtime (`ky_log_*` + wrapper) |
| `cli` | `docs/04-standard-library/cli.md` | ✅ Runtime (`ky_cli_*`) |
| `csv` | `docs/04-standard-library/csv.md` | ✅ Runtime (`ky_csv_*`, handle-based) |
| `url` | `docs/04-standard-library/url.md` | 🟡 Extend existing runtime |
| `bytes` | `docs/04-standard-library/bytes.md` | 🟡 Extend existing runtime |

### Packages (installable — `ky add`)

| Package | Doc | Status |
|---------|-----|--------|
| `http` | `docs/packages/http.md` | 🟡 Rewrite client (TCP direct), sub-modules |
| `postgres` | `docs/packages/postgres.md` | 🟡 Add typed params, transactions |
| `sqlite` | `docs/packages/sqlite.md` | 🟡 Add Database/Statement classes |
| `env` | `docs/packages/env.md` | 🟡 Add .env loader, typed accessors |
| `crypto` | `docs/packages/crypto.md` | ❌ New (password hash, JWT, HMAC) |
| `config` | `docs/packages/config.md` | ❌ New (YAML/TOML loader) |
| `compress` | `docs/packages/compress.md` | ❌ New (gzip) |
| `mail` | `docs/packages/mail.md` | ❌ New (SMTP) |

---

## Documentation Map

| Document | Location | Content |
|----------|----------|---------|
| **AGENTS.md** (this) | `AGENTS.md` | Main entry point |
| **Build guide** | `BUILD.md` | Build, install, test |
| **Packages** | `PACKAGES.md` | Library development guide |
| **Backend plan** | `docs/11-project/backend-packages-plan.md` | Complete implementation plan with tasks |
| **Std library docs** | `docs/04-standard-library/` | 21 native module docs |
| **Package docs** | `docs/packages/` | 9 installable package docs |
| **Syntax reference** | `docs/15-kyle-syntax-reference.md` | .ky language reference |
| **Roadmap** | `docs/11-project/roadmap.md` | Overall project state |
| **Remaining work** | `docs/11-project/remaining-work.md` | Bugs & features |
| **Runtime functions** | `crates/kyc_runtime/src/` | 190+ `extern "C"` functions |
| **KYUI docs** | `docs/03-language/ui/` | UI framework (paused) |

---

## Current State

### ✅ Core Language — Stable

| Component | Status |
|-----------|--------|
| Lexer + Parser | ✅ |
| Type system (generics, enums, classes) | ✅ |
| Borrow checker | ✅ |
| MIR (lowering, SSA, optimizations) | ✅ |
| LLVM codegen | ✅ |
| Collections: `[T]`, `{K:V}`, `set{T}`, `queue{T}`, `stack{T}`, `deque{T}` | ✅ |
| Orthogonal types: `^` `&` `?` `!` | ✅ |
| Module system: `use X.Y` | ✅ |
| Package manager: `ky add`, `ky.toml` | ✅ |
| Syntax tests: 13/13 passing | ✅ |
| Workspace tests: 117+ | ✅ |

### 🟢 Runtime — Rich FFI Surface

190+ `extern "C"` functions available in `kyc_runtime/src/`:
- `ky_tcp_*` — TCP networking
- `ky_fs_*` — File system
- `ky_str_*` — String manipulation
- `ky_list_*`, `ky_dict_*`, `ky_set_*`, `ky_queue_*`, `ky_stack_*`, `ky_deque_*`
- `ky_json_*` — JSON parse/stringify
- `ky_datetime_*`, `ky_date_*`, `ky_time_*`, `ky_duration_*`
- `ky_regex_*` — Regular expressions
- `ky_sha256`, `ky_base64_encode`, `ky_uuid_v4` — Crypto
- `ky_mutex_*`, `ky_atomic_*`, `ky_channel_*` — Sync
- `ky_spawn_thread`, `ky_join_thread` — Threading
- `ky_ws_*` — WebSocket frames

### ⏸️ KYUI — Paused

UI framework development is paused. Existing code at `packages/ui/`, `crates/kyc_ui/`.
All 30+ components, web backend, JS runtime are functional but not in active development.

---

## Implementation Plan

See [`docs/11-project/backend-packages-plan.md`](docs/11-project/backend-packages-plan.md) for complete task breakdown.

### Phase 1: Std Core Wrappers (16 modules)
Create `use std.X` Kyle wrappers for existing runtime functions.

### Phase 2: Std New Runtime (5 modules)
Implement `ky_log_*`, `ky_cli_*`, `ky_csv_*`, extend `url.rs` and `bytes.rs`.

### Phase 3: Package Improvements (4 packages)
Rewrite `http` (TCP client), improve `postgres`, `sqlite`, `env`.

### Phase 4: New Packages (4 packages)
Implement `crypto`, `config` (YAML/TOML), `compress` (gzip), `mail` (SMTP).

---

## Development Workflow

### Backend Package Development

1. **Check runtime** — Verify `kyc_runtime/src/` has needed extern fns, or add them
2. **Register codegen** — Add new extern fns to `kyc_backend/src/codegen/function.rs`
3. **Create Kyle wrapper** — Write `packages/<name>/src/lib.ky` or `packages/std/<name>.ky`
4. **Build** — `./scripts/dev-install.sh`
5. **Test** — `ky test packages/<name>/tests/`

### Adding a New Runtime Function

1. Add Rust function in `kyc_runtime/src/<module>.rs` with `#[no_mangle] pub extern "C" fn ky_*`
2. Export it in `kyc_runtime/src/lib.rs`
3. In Kyle: `@link "c" extern fn ky_*(...) <type>`
4. Rebuild: `cargo build --release --bin ky`

---

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
