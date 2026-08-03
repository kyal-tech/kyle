# crypto — Extended cryptography

> Password hashing, HMAC, JWT, and higher-level cryptographic utilities.
> Installation: `ky add crypto`
> Import: `use crypto`

## Password hashing

```ky
use crypto

# Hash a password (automatic salt + work factor)
hash: str = crypto.password_hash("my_secret_password")!

# Verify a password against a hash
valid: bool = crypto.password_verify("my_secret_password", hash)
if valid:
    println("password correct")
```

## HMAC

```ky
key: str = "my-secret-key"
data: str = "message-to-authenticate"

hmac: str = crypto.hmac_sha256(key, data)
println(hmac)   # hex-encoded HMAC-SHA256
```

## JWT

```ky
# Create a JWT token
secret: str = "my-secret-key"
token: str = crypto.jwt_encode(
    {"sub": "123", "name": "Kyle", "admin": true},
    secret
)!
println(token)

# Verify and decode
payload: {str: str}? = crypto.jwt_decode(token, secret)
match payload:
    some(data): println("user: " + data["name"])
    none: println("invalid token")
```

## Key derivation

```ky
derived: str = crypto.pbkdf2_sha256("password", "salt", 100000, 32)
println(derived)   # hex string
```

## Utility functions

```ky
# Compare in constant time (prevents timing attacks)
equal: bool = crypto.constant_time_compare("abc123", "abc123")

# Generate cryptographically secure random string
token: str = crypto.random_string(32)
println(token)
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `password_hash(password)` | `fn(password: &str) str!` | Hash password with bcrypt-style algorithm |
| `password_verify(password, hash)` | `fn(password: &str, hash: &str) bool` | Verify password against hash |
| `hmac_sha256(key, data)` | `fn(key: &str, data: &str) str` | HMAC-SHA256 hex |
| `jwt_encode(payload, secret)` | `fn(payload: &{str: str}, secret: &str) str!` | Create JWT token |
| `jwt_decode(token, secret)` | `fn(token: &str, secret: &str) {str: str}?` | Decode and verify JWT |
| `pbkdf2_sha256(password, salt, iterations, dklen)` | `fn(password: &str, salt: &str, iterations: i32, dklen: i32) str` | PBKDF2 key derivation |
| `constant_time_compare(a, b)` | `fn(a: &str, b: &str) bool` | Constant-time string comparison |
| `random_string(length)` | `fn(length: i32) str` | Cryptographically random string |

## Example

```ky
use crypto

# User registration
fn register_user(username: str, password: str):
    hash: str = crypto.password_hash(password)!
    # store username + hash in database
    println("user " + username + " registered")

# User login
fn login_user(username: str, password: str, stored_hash: str) bool:
    crypto.password_verify(password, stored_hash)

# JWT authentication
fn create_session_token(user_id: str, secret: str) str!:
    crypto.jwt_encode(
        {"user_id": user_id, "role": "user"},
        secret
    )
```
