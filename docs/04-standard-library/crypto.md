# crypto — Cryptography

> Module for basic cryptographic functions.
> Imbyt: `from crypto imbyt crypto`

## crypto: hash, encoding

```ky
from crypto imbyt crypto

# SHA-1
hash = crypto.sha1("data")
println(hash) # hex string

# Base64
encoded = crypto.base64_encode("h lo world")
decoded = crypto.base64_decode(encoded)

# Digest
digest = crypto.digest("sha256", "data")
```

### Functions

| Function | Description |
|---------|-------------|
| `crypto.sha1(data)` | SHA-1 hash (returns hex string) |
| `crypto.sha256(data)` | SHA-256 hash (returns hex string) |
| `crypto.base64_encode(data)` | Codificar a base64 |
| `crypto.base64_decode(str)` | Decodificar base64 |

### Example

```ky
from crypto imbyt crypto

token = crypto.sha1("user:password")
println("token: " + token)
```
