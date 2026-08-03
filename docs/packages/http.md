# http — HTTP client, server, WebSocket, and middleware

> Modular HTTP toolkit. Import only what you need.
> Installation: `ky add http`

## Modules

| Sub-module | Import | Description |
|-----------|--------|-------------|
| `client` | `use http.client` | HTTP/1.1 client with TCP direct (no curl) |
| `server` | `use http.server` | HTTP router with path params, static files |
| `middleware` | `use http.middleware` | Validation, CORS, JWT auth, logging |
| `ws` | `use http.ws` | WebSocket upgrade and frames |

---

## Client

Full HTTP/1.1 client. Connects directly via TCP — no curl dependency.

```ky
use http.client

client: http.client = http.client { timeout: 30 }

# GET
res: http.response = client.get("https://api.example.com/users")
if res.is_ok:
    println(res.body)

# POST with JSON
res = client.post(
    "https://api.example.com/users",
    "application/json",
    """{"name": "Kyle", "age": 30}"""
)

# PUT, PATCH, DELETE
res = client.put("https://api.example.com/users/1", "text/plain", "updated")
res = client.patch("https://api.example.com/users/1", "text/plain", "patched")
res = client.delete("https://api.example.com/users/1")

# Custom headers
res = client.get_with_headers(
    "https://api.example.com/data",
    {"Authorization": "Bearer token123", "Accept": "application/json"}
)
```

### response

| Field | Type | Description |
|-------|------|-------------|
| `status_code` | `i32` | HTTP status code |
| `status_text` | `str` | Status text |
| `body` | `str` | Response body |
| `headers` | `{str: str}` | Response headers |
| `is_ok` | `bool` | True if 200-399 |
| `elapsed_ms` | `i64` | Request duration |

### client

| Method | Signature | Description |
|--------|-----------|-------------|
| `client(config)` | `fn(config: {str: i32}) client` | Create client with timeout |
| `.get(url)` | `fn(url: &str) response` | GET request |
| `.post(url, ct, body)` | `fn(url: &str, ct: &str, body: &str) response` | POST request |
| `.put(url, ct, body)` | `fn(url: &str, ct: &str, body: &str) response` | PUT request |
| `.patch(url, ct, body)` | `fn(url: &str, ct: &str, body: &str) response` | PATCH request |
| `.delete(url)` | `fn(url: &str) response` | DELETE request |
| `.get_with_headers(url, headers)` | `fn(url: &str, headers: &{str: str}) response` | GET with custom headers |

---

## Server

HTTP router with path parameters and static file serving.

```ky
use http.server

app: http.router = http.router()

app.get("/hello", fn(req: http.request, res: http.response):
    res.text("Hello World", 200)
)

app.get("/users/{id}", fn(req: http.request, res: http.response):
    id: str = req.param("id")
    body: str = """{"id": """ + id + """, "name": "Alice"}"""
    res.json(body, 200)
)

app.post("/users", fn(req: http.request, res: http.response):
    res.json(req.body(), 201)
)

app.serve_static("/static", "./public")
app.listen(8080)
```

### request

| Method | Signature | Description |
|--------|-----------|-------------|
| `.method()` | `fn() str` | HTTP method |
| `.path()` | `fn() str` | Request path |
| `.param(name)` | `fn(name: &str) str` | Path parameter value |
| `.body()` | `fn() str` | Request body |
| `.header(name)` | `fn(name: &str) str` | Request header |

### response

| Method | Signature | Description |
|--------|-----------|-------------|
| `.text(body, code)` | `fn(body: &str, code: i32)` | Text response |
| `.json(data, code)` | `fn(data: &str, code: i32)` | JSON response |
| `.html(body, code)` | `fn(body: &str, code: i32)` | HTML response |
| `.file(path)` | `fn(path: &str)!` | Send file |
| `.status(code)` | `fn(code: i32)` | Status-only response |
| `.set_header(name, value)` | `fn(name: &str, value: &str)` | Set response header |
| `.upgrade()` | `fn() ws!` | Upgrade to WebSocket |

### router

| Method | Signature | Description |
|--------|-----------|-------------|
| `.get(path, ...handlers)` | `fn(path: &str, ...fn)` | Register GET route |
| `.post(path, ...handlers)` | `fn(path: &str, ...fn)` | Register POST route |
| `.put(path, ...handlers)` | `fn(path: &str, ...fn)` | Register PUT route |
| `.patch(path, ...handlers)` | `fn(path: &str, ...fn)` | Register PATCH route |
| `.delete(path, ...handlers)` | `fn(path: &str, ...fn)` | Register DELETE route |
| `.use(handler)` | `fn(handler: fn)` | Add global middleware |
| `.serve_static(prefix, dir)` | `fn(prefix: &str, dir: &str)` | Serve static files |
| `.listen(port)` | `fn(port: i32)` | Start server |

---

## Middleware

Validation, CORS, JWT auth, and request logging.

```ky
use http.server
use http.middleware

app: http.router = http.router()

# Global middleware
app.use(middleware.logger())
app.use(middleware.cors(["*"]))

# Route with body validation
app.post("/users",
    middleware.validate_body("""{"name": "string", "age": "number"}"""),
    fn(req: http.request, res: http.response):
        res.json(req.body(), 201)
)

# Route with JWT auth
app.get("/admin",
    middleware.auth_bearer("my-secret"),
    fn(req: http.request, res: http.response):
        res.text("admin area", 200)
)

app.listen(8080)
```

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `logger()` | `fn() fn` | Log requests (method, path, status, duration) |
| `cors(origins)` | `fn(origins: &[str]) fn` | CORS headers |
| `auth_bearer(secret)` | `fn(secret: &str) fn` | JWT Bearer auth |
| `validate_body(schema)` | `fn(schema: &str) fn` | Validate JSON body against schema |
| `validate_query(schema)` | `fn(schema: &str) fn` | Validate query params |
| `rate_limit(max, window)` | `fn(max: i32, window: i32) fn` | Rate limiting |

### Validate example

```ky
# Validate POST body
app.post("/users",
    middleware.validate_body("""{
        "name": {"type": "string", "required": true, "min": 2},
        "email": {"type": "string", "required": true, "pattern": "^\\S+@\\S+$"},
        "age": {"type": "number", "required": true, "min": 18, "max": 120}
    }"""),
    fn(req: http.request, res: http.response):
        res.json(req.body(), 201)
)
```

---

## WebSocket

```ky
use http.ws
use http.server

app: http.router = http.router()

app.get("/ws", fn(req: http.request, res: http.response):
    ws: http.ws = res.upgrade()!
    while true:
        msg: str = ws.read_text()!
        if msg == "":
            break
        ws.send_text("echo: " + msg)
    ws.close()
)

app.listen(8080)
```

### websocket

| Method | Signature | Description |
|--------|-----------|-------------|
| `.read_text()` | `fn() str!` | Read text message |
| `.send_text(text)` | `fn(text: &str)` | Send text message |
| `.send_binary(data)` | `fn(data: &bytes)` | Send binary message |
| `.send_pong()` | `fn()` | Send pong frame |
| `.close()` | `fn()` | Close connection |

---

## Status codes

```ky
http.status.ok()         # 200
http.status.created()    # 201
http.status.no_content() # 204
http.status.bad_request()   # 400
http.status.not_found()     # 404
http.status.server_error()  # 500
```

---

## Example: full REST API

```ky
use http.client
use http.server
use http.middleware

# Server
app: http.router = http.router()

app.use(middleware.logger())
app.use(middleware.cors(["*"]))

app.get("/api/health", fn(req: http.request, res: http.response):
    res.json("""{"status": "ok"}""", 200)
)

app.get("/api/users/{id}",
    middleware.validate_query("""{"id": {"type": "number", "required": true}}"""),
    fn(req: http.request, res: http.response):
        id: str = req.param("id")
        res.json("""{"id": """ + id + """, "name": "Alice"}""", 200)
)

app.post("/api/users",
    middleware.validate_body("""{"name": {"type": "string", "required": true}}"""),
    fn(req: http.request, res: http.response):
        res.json(req.body(), 201)
)

println("server on :8080")
app.listen(8080)
```
