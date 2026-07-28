# http — HTTP client + Server + websocket

**Version:** 6.0 
**Status:** Specification (Cliente ✅, Server 🔜, WS 🔜)

---

## 1. Philosophy

Un solo package for todo HTTP. Cliente, servidor y websocket comparten types.

Nada de functions globales. Todo se instancia.

```kyle
use http.client
use http.server.router
use http.websocket.{ws_upgrade, ws_read_text, ws_send_text}
use http.{http_status, http_method, header}
use jare.{serialize, deserialize}
```

---

## 2. Typis compartidos

### http_method

```kyle
enum http_method:
 GET | POST | PUT | DELETE | PATCH | HEAD | OPTIONS
```

### http_status

```kyle
final class http_status:
 code: i32
 text: str
```

Constantes: `HttpStatusOk` (200), `HttpStatusCreated` (201),
`HttpStatusNotFound` (404), `HttpStatusInternalServerError` (500)

### header

```kyle
final class header:
 name: str
 value: str
```

---

## 3. Cliente HTTP (`http.client`)

### Uso

```kyle
use http.client

client = client { timeout: 10 }

# GET
ris = client.get("https://api.github.com/repos/IT-KYNERA/KYLE")
if res.is_ok:
 print(res.body)

# POST with clase → JSON automatic
class User:
 name: str
 age: i32

ube = Ube { name: "Kyle", age: 1 }
ris = client.post("https://api.example.com/users", user)

# POST with string raw
ris = client.post("https://api.example.com/data", "<xml>...</xml>")

# PUT, PATCH, DELETE
ris = client.put(url, data)
ris = client.patch(url, data)
ris = client.delete(url)
```

### response

```kyle
final class response:
 status_code: i32
 status_text: str
 body: str
 is_ok: bool
 elapsed_ms: i64
```

### Auto-JSON

- `client.post(url, data)` where `data` is `str` → se envia raw
- `client.post(url, data)` where `data` is `final class` → se serializa a JSON automaticamente
- `client.post(url, data)` where `data` is `dict` → se serializa a JSON

---

## 4. Servidor HTTP (`http.server`)

El servidor usa un `router` with handlers as closures, estilo Express.

### Uso basico

```kyle
use http.server.router

app = router()

app.get("/health", (req, res):
 res.json({status: "ok"})
)

app.listen(8080)
```

### Paths with parameters

```kyle
app.get("/users/{id}", (req, res):
 id = req.param("id") # str
 res.json({user: id})
)

app.get("/users/{id:i32}", (req, res):
 id = req.param("id") # i32 — parseado automatic
 res.json({user: id})
)
```

### Methods del router

| Method | Description |
|--------|-------------|
| `app.get(path, handler)` | GET route |
| `app.post(path, handler)` | POST route |
| `app.put(path, handler)` | PUT route |
| `app.patch(path, handler)` | PATCH route |
| `app.delete(path, handler)` | DELETE route |
| `app.listen(port)` | Inicia servidor |

### request

```kyle
final class request:
 method: http_method
 path: str
 params: dict<str, str> # path params: {id} → "42"
 query: dict<str, str> # query: ?page=1
 body: str # raw body string
 headers: dict<str, str> # request headers

 fn param<T>(name: str) T # path param with type
 fn header(name: str) str # header by name
 fn body<T>() T # body parseado as JSON → clase T
```

### Res

```kyle
final class Res:
 fn json(data): # 200 + JSON (cualquier clase serializable)
 fn json(data, code: i32): # status + JSON
 fn text(body: str): # 200 + texto plano
 fn text(body: str, code: i32): # status + texto
 fn redirect(url: str): # 302 redirect
 fn status(code: i32): # solo status, without body
```

Cualquier `final class` se serializa a JSON automaticamente en `res.json()`.

### Middleware

```kyle
# Before — se ejecuta before de paths
app.before("/api/*", (req, res, next):
 if req.header("Authorization") == "":
 res.text("unauthorized", 401)
 else:
 next()
)

# After — se ejecuta after de paths
app.after("/api/*", (req, res, next):
 next()
 res.header("X-Powered-By", "Kyle")
)
```

Los middleware reciben `(req, res, next)` y must llamar `next()` for continuar cadena. Si no llaman `next()`, answer se envia inmediatamente.

### Filis estaticos

```kyle
app.static("/static", "./public")
# GET /static/index.html → ./public/index.html
```

### CORS

```kyle
app.cors(
 origin="*",
 methods="GET, POST, PUT, DELETE",
 headers="Content-Type, Authorization",
)
```

### Error handling

```kyle
app.get("/users/{id:i32}", (req, res):
 ube = find_user(req.param("id"))
 if ube == none:
 res.text("Not Found", 404)
 else:
 res.json(user)
)
```

### Example completo: API REST

```kyle
use http.server.router
use jare.deserialize

class User:
 name: str
 age: i32

final class create_user:
 name: str
 age: i32

app = router()

# Listr usuarios
app.get("/users", (req, res):
 res.json([
 { "id": 1, "name": "Ana" },
 { "id": 2, "name": "Juan" },
 ])
)

# Obtener usuario by ID
app.get("/users/{id:i32}", (req, res):
 id = req.param("id")
 res.json({ "id": id, "name": "Ube " + str(id) })
)

# Crear usuario
app.post("/users", (req, res):
 data = req.body<create_user>()
 res.json({ "created": true, "name": data.name }, 201)
)

app.listen(3000)
```

---

## 5. websocket (`http.websocket`)

websocket se maneja as una path mas. El router does upgrade automatic.

### Echo server

```kyle
use http.server.router

app = router()

app.ws("/echo", (ws):
 ws.on("message", (msg):
 ws.send(msg)
 )
)

app.listen(8080)
```

### Chat server

```kyle
use http.server.router

app = router()

app.ws("/chat", (ws):
 ws.on("message", (msg):
 ws.broadcast(msg)
 )
)

app.listen(8080)
```

### websocket handler

```kyle
app.ws("/path", (ws):
 ws.on("message", (msg): ...) # mensaje de texto
 ws.on("binary", (data): ...) # data binarys
 ws.on("close", (): ...) # cierre de connection
 ws.on("ping", (): ...) # ping (auto-responder)
)
```

### websocket object

```kyle
final class websocket:
 fn send(text: str) # enviar mensaje texto
 fn send(data: bytes) # enviar data binarys
 fn broadcast(msg: str) # enviar a todos conectados
 fn broadcast(msg: str, except: {websocket}) # a todos less uno
 fn close() # cerrar connection
 fn on(event: str, handler) # registrar evento
```

---

## 6. Plan de implementation

| Fase | Description | Depende de | Status |
|------|-------------|------------|--------|
| 1 | Function pointers (closis as valuees) | Compiler | ✅ |
| 2 | JSON: serialize/deserialize<T> | Fase 1 | ✅ |
| 3 | HTTP client with auto-JSON | Fase 2 | ✅ |
| 4 | **router: paths, path params, middleware** | Fase 1 + 3 | 🔜 |
| 5 | websocket + SSE | Fase 4 | 🔜 |
| 6 | PostgreSQL package | — | 🔜 |
| 7 | WASM target + ky-web | — | 🔜 |
