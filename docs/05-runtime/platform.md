# Platform Abstraction

> Capa de abstraccion de plataforma: interaccion with sistema operativo.
> Crate: `kyc_platform` (en desarrollo).

## Responsabilidad

La capa Platform abstrae diferencias between sistemas operativos, proporcionando
una API unificada for runtime de Kyle. Es unica capa que conoce SO subyacente.

## Status current

| Module | Status | Crate | Notis |
|--------|--------|-------|-------|
| File I/O | ✅ | `kyc_runtime/src/io.rs` | open/read/write/close |
| Time | ✅ | `kyc_runtime/src/datetime.rs` | now/parse/format |
| Networking | ✅ | `kyc_runtime/src/net.rs` | TCP listen/accept/read/write |
| Threads | ✅ | `kyc_runtime/src/thread.rs` | spawn/join |
| Environment | ✅ | `kyc_runtime/src/string.rs` | getenv/setenv |
| Filesystem (dirs) | ❌ | — | Pending |
| Process | ❌ | — | Pending |
| Signals | ❌ | — | Pending |
| Sockets (UDP) | ❌ | — | Pending |
| USB/Serial | ❌ | — | Pending |

## I/O Module

El module I/O current (`io.rs`) proporciona:

```rust
// File operations
ky_open(path, mode) -> i32 fd // Open file
ky_read_str(fd, len) -> ptr // Read as string
ky_write_str(fd, buf) -> i32 // Write string
ky_close(fd) -> i32 // Close fd
```

## Networking Module

```rust
// TCP operations
ky_tcp_listen(port) -> i32 fd
ky_tcp_accept(fd) -> i32 client_fd
ky_tcp_read(fd, len) -> ptr
ky_tcp_write(fd, buf, len) -> i32
ky_tcp_close(fd) -> i32

// WebSocket
ky_ws_accept(request) -> ptr
ky_ws_read_frame(fd) -> ptr
ky_ws_send_frame(fd, opcode, data, len) -> i32

// Crypto helpers
ky_sha1(data, len, output) -> i32
ky_base64_encode(data, len) -> ptr

// Pointer operations (for FFI)
ky_ptr_read_i32(ptr) -> i32
ky_ptr_read_ptr(ptr) -> ptr
ky_ptr_write_i32(ptr, i32)
```

## Time Module

```rust
ky_now() -> i64 // Current timestamp (ms)
ky_sleep(ms) // Sleep milliseconds
```

## Platform Adapters

Cada SO has su propia implementation de plataforma:

| SO | Status |
|----|--------|
| macOS (Apple Silicon) | ✅ Production |
| Linux (ARM64) | ✅ CI tested |
| Linux (x86_64) | ✅ CI tested |
| WASM | 🔶 Experimental |
| Windows | 📅 Planned |
| KYOS | 📅 Aspirational |

## See also

- `06-compiler/linker.md` — How se linkea with libraries de plataforma
- `04-standard-library/io.md` — API de I/O for usuario
- `04-standard-library/fs.md` — API de filesystem
