# compress — Gzip compression

> Compress and decompress data using gzip.
> Installation: `ky add compress`
> Import: `use compress`

## Compress string

```ky
use compress

data: str = "hello world hello world hello world "
compressed: bytes = compress.gzip(data)!
println("original: " + data.len().to_str())
println("compressed: " + compressed.len().to_str())
```

## Decompress

```ky
original: str = compress.gunzip(compressed)!
println(original)  # "hello world hello world hello world "
```

## Compress bytes

```ky
raw: bytes = bytes.from_str("hello world")
compressed: bytes = compress.gzip_bytes(raw)!
```

## File compression

```ky
compress.gzip_file("data.txt", "data.txt.gz")!
compress.gunzip_file("data.txt.gz", "data.txt")!
```

## HTTP response compression

```ky
use http.server
use compress

app: http.router = http.router()

app.get("/data", fn(req: http.request, res: http.response):
    data: str = """{"users": [{"name": "Kyle"}, {"name": "Ana"}]}"""
    if req.header("Accept-Encoding").contains("gzip"):
        compressed: bytes = compress.gzip(data)!
        res.set_header("Content-Encoding", "gzip")
        res.set_header("Content-Type", "application/json")
        res.set_header("Content-Length", compressed.len().to_str())
        # send compressed bytes
    else:
        res.json(data, 200)
)

app.listen(8080)
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gzip(data)` | `fn(data: &str) bytes!` | Compress string to gzip bytes |
| `gzip_bytes(data)` | `fn(data: &bytes) bytes!` | Compress bytes to gzip bytes |
| `gunzip(data)` | `fn(data: &bytes) str!` | Decompress gzip to string |
| `gunzip_bytes(data)` | `fn(data: &bytes) bytes!` | Decompress gzip to bytes |
| `gzip_file(input, output)` | `fn(input: &str, output: &str)!` | Compress file |
| `gunzip_file(input, output)` | `fn(input: &str, output: &str)!` | Decompress file |

## Example

```ky
use compress
use std.fs

# Compress a log file
compress.gzip_file("server.log", "server.log.gz")!
println("compressed: " + fs.size("server.log.gz").to_str() + " bytes")

# Decompress
compress.gunzip_file("server.log.gz", "server.log")!
```
