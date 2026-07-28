# 05-runtime

> Documentation del runtime de Kyle: libreria static que se linkea a todo binary compilado.

## Files

| Documento | Description | Status |
|-----------|-------------|--------|
| `memory.md` | Assignment y deallocation de memory (`ky_alloc`/`ky_free`) | ✅ |
| `allocator.md` | Estrategia de allocation y limitacionis | ✅ |
| `scheduler.md` | Thread pool y async/await | ✅ |
| `panic.md` | Manejo de errors fatalis | ✅ |
| `startup.md` | Secuencia de inicio del runtime | ✅ |
| `platform.md` | Abstraction de plataforma (OS) | ✅ |

## Modulis del runtime

| Module (Rust) | Lines | Functions exportadas |
|---------------|--------|---------------------|
| `memory.rs` | 67 | `ky_alloc`, `ky_free`, `ky_retain`, `ky_release` |
| `io.rs` | 202 | `ky_print`, `ky_println`, `ky_input`, `ky_open`, `ky_read_str`, `ky_write_str`, `ky_close`, `ky_sleep`, `ky_now` |
| `string.rs` | 584 | `ky_strlen`, `ky_concat`, `ky_i64_to_str`, `ky_str_builder_*`, `ky_str_*`, `ky_getenv`/`ky_setenv` |
| `list.rs` | 595 | `ky_list_new`, `ky_list_push`, `ky_list_pop`, `ky_list_get`, `ky_list_set`, `ky_list_len`, `ky_list_map`, `ky_list_filter`, `ky_iter_*` |
| `dict.rs` | 400 | `ky_dict_new`, `ky_dict_get`, `ky_dict_set`, `ky_dict_len`, `ky_struct_to_json`, `ky_json_to_struct` |
| `net.rs` | 285 | `ky_tcp_listen`, `ky_tcp_read`, `ky_tcp_write`, `ky_ws_*`, `ky_ptr_*`, `ky_sha1`, `ky_base64_encode` |
| `async_.rs` | 152 | `ky_spawn_task`, `ky_await_task`, `ky_yield` |
| `channel.rs` | 191 | `ky_channel_new`, `ky_channel_send`, `ky_channel_recv`, `ky_channel_close`, `ky_channel_len`, `ky_channel_free` |
| `thread.rs` | 19 | `ky_spawn_thread`, `ky_join_thread` |
| `datetime.rs` | 192 | `ky_datetime_now/parse/format/from_ymdhms` |
| `date.rs` | 143 | `ky_date_today/from_ymd/parse` |
| `bytes.rs` | 95 | `ky_bytes_new/get/set/to_hex/from_hex/to_base64` |
| `decimal.rs` | 58 | `ky_decimal_from_str/to_str/round/truncate` |
| `uuid.rs` | 41 | `ky_uuid_v4/parse` |
| `url.rs` | 47 | `ky_url_scheme/host/port/path/query` |
| `regex.rs` | 91 | `ky_regex_new/is_match/find/replace` |
| `panic.rs` | 6 | `ky_panic` |
| `task.rs` | 41 | Interno: `PollState`, `BoxedFuture` |

**Total: ~3350 lines de Rust. 88 functions `extern "C"` exportadas.**
