# Backend Package Development Plan

> Implementation plan for Kyle's backend ecosystem.
> Phases: Std (nativos) → Packages (instalables).

---

## Phase 1: Std Core — Ya existen (16 módulos)

Estos módulos ya tienen implementación en el runtime (`kyc_runtime/`).
Solo falta verificar que los wrappers en Kyle funcionen correctamente.

| Módulo | Runtime | Kyle Wrapper | Tests | Bugs |
|--------|---------|-------------|-------|------|
| `result` | ❌ (sintaxis del lenguaje) | ✅ | ❌ | — |
| `json` | ✅ `ky_json_*`, `ky_struct_to_json` | ❌ | ❌ | — |
| `time` | ✅ `ky_datetime_*`, `ky_date_*`, `ky_duration_*` | ❌ | ❌ | — |
| `fs` | ✅ `ky_fs_*` | ❌ | ❌ | — |
| `path` | ✅ `ky_path_*` | ❌ | ❌ | — |
| `str` | ✅ `ky_str_*`, `ky_str_builder_*` | ❌ (built-in) | ❌ | — |
| `math` | ✅ `ky_pow`, etc. | ❌ | ❌ | — |
| `io` | ✅ `ky_print*`, `ky_input*` | ❌ (built-in) | ❌ | — |
| `net` | ✅ `ky_tcp_*` | ❌ | ❌ | — |
| `random` | ✅ `ky_random_bytes` | ❌ | ❌ | — |
| `regex` | ✅ `ky_regex_*` | ❌ | ❌ | — |
| `thread` | ✅ `ky_spawn_thread`, `ky_join_thread` | ❌ | ❌ | — |
| `sync` | ✅ `ky_mutex_*`, `ky_atomic_*`, `ky_channel_*` | ❌ | ❌ | — |
| `crypto` | ✅ `ky_sha256`, `ky_base64_*`, `ky_uuid_v4` | ❌ | ❌ | — |
| `process` | ✅ `ky_getenv`, `ky_setenv` (+ libc: `system`, `popen`) | ❌ | ❌ | — |
| `testing` | ✅ `ky_assert*` | ❌ | ❌ | — |

**Acción:** Crear wrappers Kyle (`use std.X`) y tests para cada uno.

---

## Phase 2: Std Nuevos — Implementar runtime + tests (5 módulos)

### 2.1 `std::log`

**Runtime:** Nuevo módulo en `kyc_runtime/src/log.rs`

| Extern Function | Status | Descripción |
|----------------|--------|-------------|
| `ky_log_init(level, output)` | ❌ | Inicializar logger con nivel mínimo y destino |
| `ky_log_write(level, msg, len)` | ❌ | Escribir entrada de log con timestamp |
| `ky_log_set_level(level)` | ❌ | Cambiar nivel mínimo en runtime |
| `ky_log_set_output(path)` | ❌ | Redirigir output a archivo |

**Dependencias Rust:** `log`, `env_logger` o implementación manual simple

**Kyle wrapper:** `std/log.ky`

```
Tasks:
[ ] Crear kyc_runtime/src/log.rs
[ ] Registrar extern fns en backend codegen
[ ] Crear packages/std/log.ky con use std.log
[ ] Test: log levels, file output, format
[ ] Test: structured fields with_fields()
```

---

### 2.2 `std::cli` — ✅ COMPLETADO

**Runtime:** `kyc_runtime/src/cli.rs`

Implementación manual (sin clap). Parsea argv (`--name=val`, `--name`, `-n val`, posicionales).

| Extern Function | Status | Descripción |
|----------------|--------|-------------|
| `ky_cli_parse()` | ✅ | Parsear argv → dict handle de flags |
| `ky_cli_argc()` | ✅ | Número de args posicionales |
| `ky_cli_arg(i)` | ✅ | Arg posicional |
| `ky_cli_has(name)` | ✅ | Verificar si flag existe |
| `ky_cli_get(name)` | ✅ | Obtener valor de flag |
| `ky_cli_get_int(name)` | ✅ | Flag como entero (usa default) |
| `ky_cli_get_bool(name)` | ✅ | Flag como bool (usa default) |
| `ky_cli_define(name, short, desc, default)` | ✅ | Definir flag con metadatos |
| `ky_cli_help()` | ✅ | String de ayuda auto-generado |

**Kyle wrapper:** `packages/std/cli.ky` (`use std.cli.{Cli}`)

**Test:** `tests/std_cli.ky` ✅

```
Tasks:
[x] Crear kyc_runtime/src/cli.rs
[x] Parsear argv manualmente (flags --name=val, -n val, args posicionales)
[x] Registrar extern fns + pub mod
[x] Crear packages/std/cli.ky con use std.cli
[x] Test: flags, args posicionales, --help, defaults
[x] Test: get_int, get_bool
```

---

### 2.3 `std::csv` — ✅ COMPLETADO

**Runtime:** `kyc_runtime/src/csv.rs`

Parsing/serialization en Rust (sin crate externo). API handle-based (fila 0 = header). Nota: el `[csv.row]` del spec no es viable aún — clases con campos `str` en listas no se leen bien (slot `i64` de 8 bytes pierde el puntero).

| Extern Function | Status | Descripción |
|----------------|--------|-------------|
| `ky_csv_parse(data, delim)` | ✅ | Parsear CSV string → handle |
| `ky_csv_free(h)` | ✅ | Liberar handle |
| `ky_csv_row_count(h)` | ✅ | Número de filas (sin header) |
| `ky_csv_col_count(h)` | ✅ | Número de columnas |
| `ky_csv_get(h, r, c)` | ✅ | Celda por índice (fila 0 = header) |
| `ky_csv_get_col(h, col, r)` | ✅ | Celda por nombre de columna |
| `ky_csv_to_str(h)` | ✅ | Serializar handle a CSV string |

**Kyle wrapper:** `packages/std/csv.ky` (`use std.csv.{Csv}`)

**Test:** `tests/std_csv.ky` ✅ (parse, get, get_col, to_str, parse_file/to_file roundtrip)

```
Tasks:
[x] Crear kyc_runtime/src/csv.rs
[x] Registrar extern fns + pub mod
[x] Crear packages/std/csv.ky con use std.csv
[x] Test: parse, to_str, file read/write
[ ] Test: class serialization csv.from_str<T>() — requiere fix de clases en listas
[x] Test: custom delimiters (parse_delim)
```

---

### 2.4 `std::url`

**Runtime:** Extender `kyc_runtime/src/url.rs` (ya existe)

| Extern Function | Status | Descripción |
|----------------|--------|-------------|
| `ky_url_scheme(url)` | ✅ | Scheme |
| `ky_url_host(url)` | ✅ | Hostname |
| `ky_url_port(url)` | ✅ | Port |
| `ky_url_path(url)` | ✅ | Path |
| `ky_url_query(url)` | ✅ | Query string |
| `ky_url_fragment(url)` | ❌ | Fragment |
| `ky_url_encode(s)` | ❌ | Percent-encode |
| `ky_url_decode(s)` | ❌ | Percent-decode |
| `ky_url_query_get(url, key)` | ❌ | Query param por key |
| `ky_url_query_all(url)` | ❌ | Todos los query params |
| `ky_url_new()` | ❌ | Crear URL vacía |
| `ky_url_set_*(url, val)` | ❌ | Setters para construir URLs |

**Dependencias Rust:** `url` crate (ya está en Cargo.toml)

**Kyle wrapper:** `std/url.ky`

```
Tasks:
[ ] Extender kyc_runtime/src/url.rs con funciones faltantes
[ ] Registrar nuevas extern fns
[ ] Crear packages/std/url.ky con use std.url
[ ] Test: parse, components, query params
[ ] Test: encode/decode, build URL
```

---

### 2.5 `std::bytes`

**Runtime:** Extender `kyc_runtime/src/bytes.rs` (ya existe)

| Extern Function | Status | Descripción |
|----------------|--------|-------------|
| `ky_bytes_new(size)` | ✅ | Crear buffer |
| `ky_bytes_free(ptr)` | ✅ | Liberar buffer |
| `ky_bytes_get(ptr, i)` | ✅ | Leer byte |
| `ky_bytes_set(ptr, i, val)` | ✅ | Escribir byte |
| `ky_bytes_to_hex(ptr)` | ✅ | Hex encode |
| `ky_bytes_from_hex(s)` | ✅ | Hex decode |
| `ky_bytes_to_base64(ptr)` | ✅ | Base64 encode |
| `ky_bytes_from_base64(s)` | ❌ | Base64 decode |
| `ky_bytes_copy(dst, src)` | ❌ | Copiar bytes |
| `ky_bytes_slice(ptr, start, end)` | ❌ | Slice |
| `ky_bytes_to_be_i32(ptr)` | ❌ | Big-endian i32 |
| `ky_bytes_to_le_i32(ptr)` | ❌ | Little-endian i32 |
| `ky_bytes_from_be_i32(val)` | ❌ | i32 → big-endian bytes |
| `ky_bytes_from_le_i32(val)` | ❌ | i32 → little-endian bytes |
| `ky_bytes_concat(a, b)` | ❌ | Concatenar |

**Dependencias Rust:** Ninguna extra (ya tiene `base64` crate)

**Kyle wrapper:** `std/bytes.ky`

```
Tasks:
[ ] Extender kyc_runtime/src/bytes.rs
[ ] Agregar from_base64, endian conversion, slice, concat
[ ] Crear buffer struct con write_be_i32, etc.
[ ] Registrar extern fns
[ ] Crear packages/std/bytes.ky con use std.bytes
[ ] Test: hex, base64, endian conversion
[ ] Test: buffer building
```

---

## Phase 3: Packages Existentes — Mejorar (4 packages)

### 3.1 `http` — Reescribir

**Estado actual:** Usa `system("curl...")` para cliente. Server usa TCP directo. WebSocket separado.

**Objetivo:** Cliente TCP directo (sin curl). Sub-módulos: client, server, middleware, ws.

```
Tasks:
[ ] Crear packages/http/src/client.ky — TCP directo, parse HTTP response
[ ] Crear packages/http/src/server.ky — router + path params + static files
[ ] Crear packages/http/src/middleware.ky — validate, cors, auth, logger
[ ] Crear packages/http/src/ws.ky — WebSocket upgrade + frames
[ ] Crear packages/http/src/lib.ky — re-export
[ ] Test: GET/POST requests against a test server
[ ] Test: server routing + params
[ ] Test: middleware validation
[ ] Test: WebSocket echo
```

### 3.2 `postgres` — Mejorar

**Estado actual:** FFI a libpq. Sin queries parametrizados. Sin helpers tipados.

```
Tasks:
[ ] Agregar PQexecParams para queries parametrizados
[ ] Agregar bind_str, bind_int, bind_float en statement
[ ] Agregar helpers row.get_int, row.get_float, row.get_bool
[ ] Agregar transacciones (BEGIN/COMMIT/ROLLBACK)
[ ] Test: SELECT con parámetros
[ ] Test: INSERT/UPDATE/DELETE
[ ] Test: transacciones
```

### 3.3 `sqlite` — Mejorar

**Estado actual:** Solo raw FFI (open/close/exec). Sin classes.

```
Tasks:
[ ] Crear clase Database con open, execute, prepare, close
[ ] Crear clase Statement con bind_int, bind_text, step, column_*
[ ] Agregar constantes sqlite.ok, sqlite.row, sqlite.done
[ ] Test: CREATE + INSERT + SELECT
[ ] Test: parámetros bind
[ ] Test: transacciones
```

### 3.4 `env` — Mejorar

**Estado actual:** Solo env() y env_or(). Sin setenv, sin .env loader.

```
Tasks:
[ ] Agregar env.set(key, value)
[ ] Agregar env.unset(key)
[ ] Agregar env.get_int, env.get_float, env.get_bool
[ ] Agregar env.load_file(".env")
[ ] Test: get/set env vars
[ ] Test: .env file parsing
[ ] Test: typed accessors
```

---

## Phase 4: Packages Nuevos (5 packages)

### 4.1 `crypto`

**Runtime:** Usa extern fns existentes + nuevas.

| Extern Function | Status |
|----------------|--------|
| `ky_sha256` | ✅ |
| `ky_random_bytes` | ✅ |
| `ky_uuid_v4` | ✅ |
| `ky_base64_encode` | ✅ |
| `ky_pbkdf2(password, salt, iter, dklen)` | ❌ |
| `ky_hmac_sha256(key, data)` | ❌ |
| `ky_constant_time_compare(a, b)` | ❌ |

```
Tasks:
[ ] Agregar ky_pbkdf2, ky_hmac_sha256, ky_constant_time_compare al runtime
[ ] Crear packages/crypto/src/lib.ky
[ ] Implementar password_hash/verify (usando PBKDF2)
[ ] Implementar jwt_encode/jwt_decode (HMAC-SHA256 + base64)
[ ] registrar extern fns nuevas
[ ] Test: password hash + verify
[ ] Test: JWT encode + decode
[ ] Test: HMAC
```

### 4.2 `config`

**Runtime:** YAML/TOML parser. Dependencias: `serde_yaml`, `toml`.

| Extern Function | Status |
|----------------|--------|
| `ky_config_load_yaml(path)` | ❌ |
| `ky_config_load_toml(path)` | ❌ |
| `ky_config_to_struct(config, descriptor)` | ❌ |
| `ky_config_from_str_yaml(s)` | ❌ |
| `ky_config_from_str_toml(s)` | ❌ |

```
Tasks:
[ ] Agregar serde_yaml + toml a Cargo.toml
[ ] Crear kyc_runtime/src/config.rs
[ ] Implementar parse YAML + TOML → struct Kyle
[ ] Implementar merge: defaults + file + env vars
[ ] Crear packages/config/src/lib.ky
[ ] Test: TOML load
[ ] Test: YAML load
[ ] Test: merge defaults
[ ] Test: env var override
```

### 4.3 `compress`

**Runtime:** Gzip compress/decompress. Dependencia: `flate2`.

| Extern Function | Status |
|----------------|--------|
| `ky_gzip_compress(data, len)` | ❌ |
| `ky_gzip_decompress(data, len)` | ❌ |
| `ky_gzip_compress_file(input, output)` | ❌ |
| `ky_gzip_decompress_file(input, output)` | ❌ |

```
Tasks:
[ ] Agregar flate2 a Cargo.toml
[ ] Crear kyc_runtime/src/compress.rs
[ ] Implementar gzip compress/decompress para string y bytes
[ ] Implementar compress/decompress file
[ ] Crear packages/compress/src/lib.ky
[ ] Test: compress + decompress string
[ ] Test: compress + decompress file
```

### 4.4 `mail`

**Runtime:** SMTP client. Dependencia: `lettre`.

| Extern Function | Status |
|----------------|--------|
| `ky_mail_send(config, msg)` | ❌ |
| `ky_mail_send_tls(config, msg)` | ❌ |
| `ky_mail_build_msg(from, to, subject, body)` | ❌ |
| `ky_mail_add_attachment(msg, path)` | ❌ |

**Alternativa:** Implementar SMTP directamente sobre `ky_tcp_*` (sin dependencia externa).

```
Tasks:
[ ] Decidir: crate lettre o SMTP raw sobre TCP
[ ] Si lettre: agregar a Cargo.toml
[ ] Crear kyc_runtime/src/mail.rs
[ ] Implementar send, send_tls, send_plain
[ ] Implementar message building + attachments
[ ] Crear packages/mail/src/lib.ky
[ ] Test: build message
[ ] Test: (requiere servidor SMTP de prueba)
```

---

## Milestones & Tracking

### Fase 1: Std Core Wrappers (16 módulos)
```
[ ] result     [ ] json     [ ] time     [ ] fs
[ ] path       [ ] str      [ ] math     [ ] io
[ ] net        [ ] random   [ ] regex    [ ] thread
[ ] sync       [ ] crypto   [ ] process  [ ] testing
```

### Fase 2: Std Nuevos Runtime (5 módulos)
```
[ ] log    [ ] cli    [ ] csv    [ ] url    [ ] bytes
```

### Fase 3: Packages Existentes (4 packages)
```
[ ] http      [ ] postgres    [ ] sqlite    [ ] env
```

### Fase 4: Packages Nuevos (5 packages)
```
[ ] crypto    [ ] config    [ ] compress    [ ] mail
```

---

## Dependencias entre módulos

```
result ─┬─ json ──── config
         ├─ testing
         └─ (todos)

net ─┬─ http
     ├─ mail
     └─ postgres

crypto ─┬─ http (JWT middleware)
         └─ mail (TLS)

fs ─┬─ csv
    ├─ compress
    └─ config

bytes ─┬─ crypto
       ├─ compress
       └─ mail
```
