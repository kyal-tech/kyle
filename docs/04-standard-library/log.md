# log — Logging

> Structured logging with levels, timestamps, and configurable output.
> Import: `use std.log`

## Basic usage

```ky
use std.log

log.info("server started")
log.info("server started on port " + port.to_str())
log.warn("disk space low: " + free.to_str() + " bytes")
log.error("failed to connect: " + err)
```

## Log levels

```ky
log.debug("connecting to database")  # development details
log.info("server started")           # general information
log.warn("high memory usage")        # warning, not critical
log.error("connection failed")       # error, needs attention
```

## Level filtering

```ky
log.set_level(log.warn)  # only warn and error shown
log.debug("hidden")      # not printed
log.warn("visible")      # printed
log.error("visible")     # printed
```

## With structured fields

```ky
log.with_fields({"user": user_id, "ip": ip}).info("user logged in")
log.with_fields({"duration_ms": elapsed}).warn("slow request")
```

## Output to file

```ky
log.set_output("app.log")!
log.info("this goes to file")

log.set_output(std.io.console)  # back to console
```

## Format

Default format: `[LEVEL] YYYY-MM-DD HH:MM:SS message`

```
[INFO] 2024-01-15 10:30:00 server started on port 8080
[ERROR] 2024-01-15 10:30:01 failed to connect: timeout
[WARN]  2024-01-15 10:30:02 disk space low: 512 bytes
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `debug(msg)` | `fn(msg: &str)` | Log at debug level |
| `info(msg)` | `fn(msg: &str)` | Log at info level |
| `warn(msg)` | `fn(msg: &str)` | Log at warn level |
| `error(msg)` | `fn(msg: &str)` | Log at error level |
| `set_level(level)` | `fn(level: i32)` | Filter by minimum level |
| `set_output(path)` | `fn(path: &str)!` | Write logs to file |
| `set_output(console)` | `fn()` | Write logs to stdout |
| `with_fields(fields)` | `fn(fields: &{str: str}) logger` | Create logger with structured fields |

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `log.debug` | `0` | Debug level |
| `log.info` | `1` | Info level |
| `log.warn` | `2` | Warn level |
| `log.error` | `3` | Error level |

## Example

```ky
use std.log

log.set_level(log.debug)
log.set_output("server.log")!

log.with_fields({"version": "1.0"}).info("starting server")
log.debug("loading config")
log.info("listening on :8080")
log.warn("ssl certificate expires in 30 days")
```
