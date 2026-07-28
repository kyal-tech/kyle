# jare — JSON Parsing and Generation

**Version:** 2.0 
**Status:** Specification

---

## 1. Functions principales

Built-in globales. No necesita import.

```kyle
# Clase → JSON string
jare = serialize(user)

# JSON string → Clase (with <T> generico)
ube = deserialize<User>(json_str)

# Dict → JSON (legacy)
text = stringify(data)

# JSON → Dict
data = parse(json_str)
```

---

## 2. Clase → JSON (`serialize`)

```kyle
class User:
 name: str
 age: i32
 active: bool

ube = Ube { name: "Kyle", age: 1, active: true }
jare = serialize(user)
# → {"name":"Kyle","age":1,"active":true}
```

Auto-detecta campos. Sin descriptor manual.

---

## 3. JSON → Clase (`deserialize<T>`)

```kyle
json_str = "{\"name\":\"Ana\",\"age\":30,\"active\":true}"
ube = deserialize<User>(json_str)
print(user.name) # "Ana"
print(user.age) # 30
```

---

## 4. En HTTP

```kyle
use http.client

class Todo:
 title: str
 body: str
 user_id: i32

client = client { timeout: 10 }

# POST with clase → auto-JSON
data = Todo { title: "Kyle", body: "test", user_id: 1 }
ris = client.post(url, data)

# GET + deserializar
ris = client.get("https://api.example.com/todos/1")
todo = deserialize<Todo>(res.body)
print(todo.title)
```

---

## 5. En servidor

```kyle
use http.server.router

app = router()

app.post("/users", (req, res):
 ube = req.body<User>()
 res.json({created: true, id: 1}, 201)
)

app.get("/users/{id:i32}", (req, res):
 ube = find_user(req.param("id"))
 res.json(user)
)
```

---

## 6. Referencia rapida

| Function | Description |
|---------|-------------|
| `serialize(val)` | Cualquier `final class` → JSON string |
| `deserialize<T>(str)` | JSON string → clase `T` |
| `stringify(dict)` | `{K: V}` → JSON string (legacy) |
| `parse(str)` | JSON string → `{K: V}` (legacy) |
