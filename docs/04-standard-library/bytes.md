# bytes — Binary data manipulation

> Hex encoding, endian conversion, and binary buffers.
> Import: `use std.bytes`

## Hex encoding

```ky
use std.bytes

data: bytes = bytes.from_hex("48656C6C6F")
println(data as str)  # "Hello"

hex: str = bytes.to_hex(data)
println(hex)          # "48656C6C6F"
```

## Base64 encoding

```ky
encoded: str = bytes.to_base64(data)
decoded: bytes = bytes.from_base64(encoded)
```

## Integer to/from bytes

```ky
# Big endian (network byte order)
b: bytes = bytes.to_be_i32(1024)
val: i32 = bytes.from_be_i32(b)

b = bytes.to_be_i64(123456789)
val = bytes.from_be_i64(b)

# Little endian
b = bytes.to_le_i32(1024)
val = bytes.from_le_i32(b)
```

## Fixed-size byte arrays

```ky
# Read specific types from bytes
b: bytes = bytes.from_hex("0004A200000005")
i1: i32 = bytes.from_be_i32_at(b, 0)  # 1186  (0x0004A2)
i2: i32 = bytes.from_be_i32_at(b, 4)  # 5     (0x00000005)
```

## Buffer for building binary data

```ky
buf: bytes.buffer = bytes.buffer(64)
buf.write_be_i32(1024)
buf.write_be_i64(123456789)
buf.write_str("hello")

data: bytes = buf.to_bytes()
println(data.len().to_str())  # total bytes
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `to_hex(data)` | `fn(data: &bytes) str` | Encode bytes to hex |
| `from_hex(s)` | `fn(s: &str) bytes` | Decode hex string |
| `to_base64(data)` | `fn(data: &bytes) str` | Encode to base64 |
| `from_base64(s)` | `fn(s: &str) bytes` | Decode base64 |
| `to_be_i32(val)` | `fn(val: i32) bytes` | i32 to 4 bytes (big endian) |
| `to_le_i32(val)` | `fn(val: i32) bytes` | i32 to 4 bytes (little endian) |
| `to_be_i64(val)` | `fn(val: i64) bytes` | i64 to 8 bytes (big endian) |
| `to_le_i64(val)` | `fn(val: i64) bytes` | i64 to 8 bytes (little endian) |
| `from_be_i32(bytes)` | `fn(bytes: &bytes) i32` | 4 bytes to i32 (big endian) |
| `from_le_i32(bytes)` | `fn(bytes: &bytes) i32` | 4 bytes to i32 (little endian) |
| `from_be_i64(bytes)` | `fn(bytes: &bytes) i64` | 8 bytes to i64 (big endian) |
| `from_le_i64(bytes)` | `fn(bytes: &bytes) i64` | 8 bytes to i64 (little endian) |
| `from_be_i32_at(bytes, offset)` | `fn(bytes: &bytes, offset: i32) i32` | Read i32 at offset |
| `from_be_i64_at(bytes, offset)` | `fn(bytes: &bytes, offset: i32) i64` | Read i64 at offset |

## buffer

| Method | Signature | Description |
|--------|-----------|-------------|
| `buffer(capacity)` | `fn(capacity: i32) buffer` | Create buffer |
| `.write_be_i32(val)` | `fn(val: i32)` | Write i32 (big endian) |
| `.write_le_i32(val)` | `fn(val: i32)` | Write i32 (little endian) |
| `.write_be_i64(val)` | `fn(val: i64)` | Write i64 (big endian) |
| `.write_le_i64(val)` | `fn(val: i64)` | Write i64 (little endian) |
| `.write_str(s)` | `fn(s: &str)` | Write string bytes |
| `.write_byte(b)` | `fn(b: i8)` | Write single byte |
| `.to_bytes()` | `fn() bytes` | Get all bytes |
| `.len()` | `fn() i32` | Current length |
| `.clear()` | `fn()` | Reset buffer |

## Example

```ky
use std.bytes

# Encode a network packet
buf: bytes.buffer = bytes.buffer(32)
buf.write_be_i32(0xDEADBEEF)  # magic number
buf.write_be_i32(42)          # message length
buf.write_str("hello")        # message body

packet: bytes = buf.to_bytes()
println(bytes.to_hex(packet))

# Decode
magic: i32 = bytes.from_be_i32_at(packet, 0)
length: i32 = bytes.from_be_i32_at(packet, 4)
println("magic: " + magic.to_str())   # 3735928559
```
