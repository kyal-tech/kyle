# url — URL parsing and manipulation

> Parse, build, and manipulate URLs.
> Import: `use std.url`

## Parse a URL

```ky
use std.url

u: url = url.parse("https://user:pass@api.example.com:8080/path/to/page?q=hello&limit=10#section")!

println(u.scheme())    # "https"
println(u.host())      # "api.example.com"
println(u.port())      # 8080
println(u.path())      # "/path/to/page"
println(u.query())     # "q=hello&limit=10"
println(u.fragment())  # "section"
println(u.userinfo())  # "user:pass"
```

## Query parameters

```ky
u: url = url.parse("/search?q=kyle&page=1&limit=20")!

q: str = u.query_get("q")        # "kyle"
page: str = u.query_get("page")  # "1"
none: str = u.query_get("sort")  # "" (not found)

all: {str: str} = u.query_all()
# {"q": "kyle", "page": "1", "limit": "20"}
```

## Build a URL

```ky
u: url = url.new()
u.set_scheme("https")
u.set_host("api.example.com")
u.set_port(443)
u.set_path("/users")
u.set_query("page=1")
result: str = u.to_str()
# "https://api.example.com:443/users?page=1"
```

## URL from parts

```ky
u: url = url.from_parts(
    "https",
    "api.example.com",
    443,
    "/users",
    "page=1"
)
println(u.to_str())
```

## Encode/decode

```ky
encoded: str = url.encode("hello world")   # "hello%20world"
decoded: str = url.decode("hello%20world") # "hello world"
```

## url

| Method | Signature | Description |
|--------|-----------|-------------|
| `parse(s)` | `fn(s: &str) url!` | Parse URL string |
| `new()` | `fn() url` | Create empty URL |
| `from_parts(scheme, host, port, path, query)` | `fn(...) url` | Build URL from parts |
| `.scheme()` | `fn() str` | URL scheme |
| `.host()` | `fn() str` | Hostname |
| `.port()` | `fn() i32` | Port number (0 if not set) |
| `.path()` | `fn() str` | Path component |
| `.query()` | `fn() str` | Raw query string |
| `.fragment()` | `fn() str` | Fragment identifier |
| `.userinfo()` | `fn() str` | Userinfo (user:pass) |
| `.query_get(key)` | `fn(key: &str) str` | Get query parameter |
| `.query_all()` | `fn() {str: str}` | All query parameters |
| `.set_scheme(s)` | `fn(s: &str)` | Set scheme |
| `.set_host(s)` | `fn(s: &str)` | Set hostname |
| `.set_port(p)` | `fn(p: i32)` | Set port |
| `.set_path(s)` | `fn(s: &str)` | Set path |
| `.set_query(s)` | `fn(s: &str)` | Set query string |
| `.to_str()` | `fn() str` | Full URL string |
| `encode(s)` | `fn(s: &str) str` | Percent-encode |
| `decode(s)` | `fn(s: &str) str` | Percent-decode |

## Example

```ky
use std.url

u: url = url.parse("https://api.github.com/repos/user/repo")!
api: str = u.scheme() + "://" + u.host()
path: str = u.path()

println("api: " + api)
println("endpoint: " + path)
```
