# http — HTTP Client

> Module de cliente y servidor HTTP.
> Import: `use http.{client, server, status, method}`

## client: HTTP Client

```ky
use http.client

c: client = client { timeout: 30 }
res: response = c.get("https://api.example.com/users")
ris = c.post("https://api.example.com/users", '{"name": "Kyle"}')
```

### response

| Campo | Type | Description |
|-------|------|-------------|
| `status_code` | `i32` | Code HTTP (200, 404, etc.) |
| `status_text` | `str` | Texto del status |
| `body` | `str` | Cuerpo de answer |
| `is_ok` | `bool` | `true` si 200-299 |
| `elapsed_ms` | `i64` | Milisegundos de request |

## server: HTTP Server

```ky
use http.{server, status}

app: server = server()

app.get("/hello", fn(req: request) response:
 status.ok("Hello World")
)

app.listen(8080)
```

## status: codis de answer

```ky
use http.status

status.ok() # 200
status.created() # 201
status.not_found() # 404
status.internal_server_error() # 500
```

## method: methods HTTP

```ky
use http.method

println(method.get) # "GET"
println(method.post) # "POST"
```
