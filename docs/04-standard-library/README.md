# Standard Library

> Built-in modules available to all Kyle programs without installation.
> Import with `use std.<module>`.

## Modules

| Module | Import | Description |
|--------|--------|-------------|
| `result` | `use std.result` | `option<T>` (`T?`) and `result<T, E>` (`T!`) types |
| `json` | `use std.json` | Class-based JSON serialization |
| `time` | `use std.time` | `datetime`, `date`, `time`, `duration` types |
| `fs` | `use std.fs` | File system read, write, copy, remove, directory operations |
| `path` | `use std.path` | Path manipulation: dirname, basename, extension, join |
| `str` | `use std.str` | String utilities: trim, split, contains, replace, to_upper, to_lower |
| `math` | `use std.math` | Mathematical functions: max, min, abs, pow, sqrt, clamp |
| `io` | `use std.io` | Console I/O: print, println, input |
| `net` | `use std.net` | TCP networking: listen, accept, connect, read, write |
| `random` | `use std.random` | Random number generation: int, float, shuffle, choice |
| `regex` | `use std.regex` | Regular expressions: match, find, replace, split |
| `thread` | `use std.thread` | OS threads: spawn, join, sleep |
| `sync` | `use std.sync` | Synchronization: mutex, atomic, channel |
| `crypto` | `use std.crypto` | Cryptography: sha256, base64, uuid, random_bytes |
| `process` | `use std.process` | OS process: exec, exit, working directory |
| `testing` | `use std.testing` | Test assertions: assert_eq, assert_ne, assert_true |
| `log` | `use std.log` | Structured logging with levels (debug, info, warn, error) |
| `cli` | `use std.cli` | Command-line argument parsing |
| `csv` | `use std.csv` | CSV parsing, serialization, file read/write |
| `url` | `use std.url` | URL parsing, building, query manipulation |
| `bytes` | `use std.bytes` | Hex/base64 encoding, endian conversion, binary buffer |

## Conventions

- All modules imported explicitly: `use std.math`
- Functions called with namespace: `math.max(a, b)`
- snake_case everywhere: functions, types, methods, variables
- Error-returning functions return `T!` (fallible type)
- Functions that may not return a value return `T?` (optional type)
- String parameters use `&str` (borrow) to avoid ownership transfer
