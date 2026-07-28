# postgris — PostgreSQL Driver

**Version:** 1.0 
**Status:** Specification

---

## 1. Philosophy

Driver PostgreSQL nativo via FFI a `libpq`. Minimum overhead, tipado fuerte, cero magia.

```kyle
use postgris.{pool, Row}
```

---

## 2. Connection

```kyle
use postgris.pool

pool = pool.new("postgresql://user:pass@localhost:5432/mydb")

# Una consulta
rows = pool.query("SELECT * FROM users")
for row in rows:
 print(row["name"])
```

### pool vs Connection

```kyle
# pool (recommended) — reusa connections
pool = pool.new(conn_string, max_size=10)

# Connection directa (una sola)
conn = pool.get_conn()
rows = conn.query("SELECT 1")
conn.close()
```

---

## 3. Queries

### Query simple (SELECT)

```kyle
rows = pool.query("SELECT id, name, age FROM users")

for row in rows:
 print(row["id"]) # i64
 print(row["name"]) # str
 print(row["age"]) # i64 (NULL → 0)
```

### Con parameters

```kyle
rows = pool.query("SELECT * FROM users WHERE age > $1", [18])
```

### Insert/Update/Delete (Execute)

```kyle
n = pool.execute("INSERT INTO users (name, age) VALUES ($1, $2)", {"Ana", 30})
print(n) # numero de filas afectadas
```

### Transacciones

```kyle
conn = pool.get_conn()
conn.begin()
conn.execute("UPDATE users SET age = $1 WHERE id = $2", {25, 1})
conn.execute("UPDATE users SET age = $1 WHERE id = $2", {30, 2})
conn.commit()
# conn.rollback() si algo sale mal
conn.close()
```

---

## 4. Tipado

### Row — acceso by name

```kyle
final class Row:
 fn get<T>(name: str) T # with type explicito
 fn get(name: str) i64 # default i64
 fn get_str(name: str) str
 fn get_i64(name: str) i64
 fn get_f64(name: str) f64
 fn get_bool(name: str) bool
 fn keys() {str} # columnas disponibles
```

### Mapeo a clasis (with deserialize)

```kyle
use postgris.pool
use jare.deserialize

class User:
 id: i64
 name: str
 email: str

rows = pool.query("SELECT id, name, email FROM users WHERE id = $1", [1])
if len(rows) > 0:
 ube = deserialize<User>(rows[0].json())
 print(user.name)
```

### Valoris NULL

```kyle
age = row.get("age") # 0 si NULL (i64 default)
age = row.get("age") as i64? # 0 si NULL with Option
name = row.get_str("name") # "" si NULL
```

---

## 5. Migracionis (simple)

```kyle
use postgris.pool

pool = pool.new(conn_string)

pool.execute("CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE, age INTEGER DEFAULT 0)")

pool.execute("CREATE TABLE IF NOT EXISTS posts (id SERIAL PRIMARY KEY, user_id INTEGER REFERENCES users(id), title TEXT NOT NULL, body TEXT)")
```

---

## 6. API completa

| Function | Description |
|---------|-------------|
| `pool.new(conn_str)` | Crear pool de conexionis |
| `pool.query(sql)` | SELECT → list de Rows |
| `pool.query(sql, params)` | SELECT with parameters |
| `pool.execute(sql)` | INSERT/UPDATE/DELETE → filas afectadas |
| `pool.execute(sql, params)` | Con parameters |
| `pool.get_conn()` | Obtener connection del pool |
| `conn.begin()` | Iniciar transaccion |
| `conn.commit()` | Confirmar transaccion |
| `conn.rollback()` | Revertir transaccion |
| `conn.close()` | Cerrar connection |
| `row.get(name)` | Obtener value by name (i64) |
| `row.get_str(name)` | Obtener string |
| `row.get_i64(name)` | Obtener i64 |
| `row.get_f64(name)` | Obtener f64 |
| `row.get_bool(name)` | Obtener bool |
| `row.json()` | Serializar row a JSON string |
| `row.keys()` | List de columnas |

---

## 7. Implementation

El driver usa `extern fn` + `@link "pq"` for llamar a libpq:

```kyle
@link "pq"

extern fn PQconnectdb(conninfo: ptr) ptr
extern fn PQexec(conn: ptr, query: ptr) ptr
extern fn PQntuples(result: ptr) i32
extern fn PQgetvalue(result: ptr, row: i32, col: i32) ptr
extern fn PQfinish(conn: ptr)
```

La implementation completa is en `packages/postgres/src/lib.ky`.

---

## 8. Dependencias del sistema

```bash
# macOS
brew install libpq

# Linux (Debian/Ubuntu)
apt install libpq-dev

# Linux (RHEL/Fedora)
dnf install libpq-devel
```

El package linkea automaticamente with `@link "pq"`.
