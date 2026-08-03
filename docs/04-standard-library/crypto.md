# crypto — Cryptography

> Cryptographic hash functions, encoding, and random values.
> Import: `use std.crypto`

## Hashing

```ky
use std.crypto

hash: str = crypto.sha256("hello world")
println(hash)   # hex string

hash: str = crypto.sha1("hello world")
println(hash)   # hex string
```

## Encoding

```ky
encoded: str = crypto.base64_encode("hello world")
decoded: str = crypto.base64_decode(encoded)
```

## Random data

```ky
bytes: bytes = crypto.random_bytes(32)
uuid: str = crypto.uuid_v4()
println(uuid)   # "550e8400-e29b-41d4-a716-446655440000"
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `sha256(data)` | `fn(data: &str) str` | SHA-256 hash as hex string |
| `sha1(data)` | `fn(data: &str) str` | SHA-1 hash as hex string |
| `base64_encode(data)` | `fn(data: &str) str` | Base64 encode |
| `base64_decode(str)` | `fn(str: &str) str` | Base64 decode |
| `random_bytes(count)` | `fn(count: i32) bytes` | Cryptographically secure random bytes |
| `uuid_v4()` | `fn() str` | Generate UUID v4 |

## Example

```ky
use std.crypto

token: str = crypto.sha256("user:password:123")
println("token: " + token)

id: str = crypto.uuid_v4()
println("id: " + id)
```
