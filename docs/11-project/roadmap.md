# Kyle — Roadmap de Desarrollo

> Current focus: **Backend package ecosystem** (see [`backend-packages-plan.md`](backend-packages-plan.md))
> UI Framework: ⏸️ **Paused** (see `docs/03-language/ui/`)

---

## Estado Actual

| Área | Estado |
|------|--------|
| **Compilador** (Fases 1-17) | ✅ Completo |
| **Runtime** (memoria, strings, colecciones, TCP, JSON, crypto, regex, threads, sync) | ✅ Completo (190+ extern fns) |
| **Borrow checker** | ✅ Completo |
| **Cross-platform** (macOS, Linux, Windows) | ✅ Completo |
| **Tooling** (LSP, formatter, VS Code, package manager) | ✅ Completo |
| **Core language** (generics, enums, classes, modules) | ✅ Completo |
| **Error handling** (`T!`, `T?`, `!` operator) | ✅ Completo |
| **FFI** (`extern fn`, `@link`, `ptr`) | ✅ Completo |

---

## Fases de Desarrollo Backend

### 🔴 FASE 1: Std Core Wrappers (19 módulos) — ✅ 21 wrappers creados en `packages/std/`

Crear wrappers Kyle para las 190+ funciones runtime existentes. Todos compilan (`ky check`); smoke-tested en conjunto.

| Módulo | Estado | Notas |
|--------|:------:|-------|
| `std::result` | ✅ Wrapper | `ok`/`error`/`some`/`none` son builtins |
| `std::json` | ✅ Wrapper | `ky_json_*`, `ky_struct_to_json` |
| `std::time` | ✅ Wrapper | `ky_datetime_*`, `ky_date_*`, `ky_duration_*` |
| `std::fs` | ✅ Wrapper | `ky_fs_*` |
| `std::path` | ✅ Wrapper | `ky_path_*` |
| `std::str` | ✅ Wrapper | `ky_str_*`, `Str` class |
| `std::math` | ✅ Wrapper | `ky_pow`, etc. |
| `std::io` | ✅ Wrapper | `ky_print*`, `ky_input*` |
| `std::net` | ✅ Wrapper | `ky_tcp_*` |
| `std::random` | ✅ Wrapper | `Random.bytes`, `Random.int` |
| `std::regex` | ✅ Wrapper | `ky_regex_*` |
| `std::thread` | ✅ Wrapper | `ky_spawn_thread` |
| `std::sync` | ✅ Wrapper | `Mutex`/`AtomicI64`/`AtomicBool`/`Channel` (+ `tests/std_sync.ky`) |
| `std::crypto` | ✅ Wrapper | sha256, base64, uuid |
| `std::process` | ✅ Wrapper | `ky_getenv`, libc |
| `std::testing` | ✅ Wrapper | `ky_assert*`, `Assert` |
| `std::bytes` | ✅ Implementado | `ky_bytes_*`, `ky_buffer_*` `tests/std_bytes.ky` |
| `std::cli` | ✅ Implementado | `ky_cli_*`, `std/cli.ky` `tests/std_cli.ky` |
| `std::csv` | ✅ Implementado | `ky_csv_*`, `std/csv.ky` `tests/std_csv.ky` |
| `std::url` | ✅ Wrapper | `ky_url_*` |
| `std::log` | ✅ Wrapper | `ky_log_*`, `Log` class |

**Total:** 21/21 std modules documented. Implementados con wrapper+test: result, json, time, fs, path, str, math, io, net, random, regex, thread, sync, crypto, process, testing, bytes, cli, csv.

### 🟡 FASE 2: Std New Runtime (5 módulos)

Implementar runtime functions + wrappers Kyle.

| Módulo | Runtime | Kyle Wrapper | Tests | Prioridad |
|--------|:-------:|:------------:|:-----:|:---------:|
| `std::log` | 🟡 `log.rs` | ❌ | ❌ | 🔴 Alta |
| `std::cli` | ✅ `ky_cli_*` | ✅ `std/cli.ky` | ✅ `tests/std_cli.ky` | 🔴 Alta |
| `std::csv` | ✅ `ky_csv_*` | ✅ `std/csv.ky` | ✅ `tests/std_csv.ky` | 🟡 Media |
| `std::url` | 🟡 Extender | ❌ | ❌ | 🟡 Media |
| `std::bytes` | ✅ `ky_bytes_*` | ✅ `std/bytes.ky` | ✅ `tests/std_bytes.ky` | 🟡 Media |

### 🟡 FASE 3: Package Improvements (4 packages)

Mejorar packages existentes con funcionalidades faltantes.

| Package | Estado | Pendiente |
|---------|:------:|-----------|
| `http` | 🟡 | Cliente TCP directo (sin curl), sub-módulos, middleware |
| `postgres` | 🟡 | Queries parametrizados, binds tipados, transacciones |
| `sqlite` | 🟡 | Database/Statement/ResultSet classes |
| `env` | 🟡 | .env loader, typed accessors, setenv/unset |

### 🟢 FASE 4: New Packages (4 packages)

Crear packages desde cero con runtime Rust + Kyle wrapper.

| Package | Runtime | Kyle Wrapper | Tests | Prioridad |
|---------|:-------:|:------------:|:-----:|:---------:|
| `crypto` | ❌ | ❌ | ❌ | 🟡 Media |
| `config` (YAML/TOML) | ❌ | ❌ | ❌ | 🟡 Media |
| `compress` (gzip) | ❌ | ❌ | ❌ | 🟢 Baja |
| `mail` (SMTP) | ❌ | ❌ | ❌ | 🟢 Baja |

---

## ⏸️ UI Framework — Paused

UI development (`.kyx`, web/desktop/iOS/TUI backends) is paused.
Existing code at `packages/ui/` and `crates/kyc_ui/` remains functional but not in active development.

See `docs/03-language/ui/README.md` for full UI documentation.

---

## Testing

```bash
# Rust tests
cargo test --workspace --exclude kyc_runtime_wasm

# Build release binary
cargo build --release --bin ky

# Syntax tests
for f in tests/syntax/*.ky; do ky run "$f"; done

# Package tests
ky test packages/http/tests/test_http.ky

# Check package
ky check packages/http/src/lib.ky
```

---

*Última actualización: 2026-07-30 · Focus: Backend packages*
