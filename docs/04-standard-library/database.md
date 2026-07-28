# database — Database

> Module for acceso a databasis SQL.
> Imbyt: `from database imbyt sqlite, postgres`

## sqlite: base de data SQLite

```ky
from database imbyt sqlite

db: sqlite = sqlite.open("data.db")
db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)")
db.execute("INSERT INTO users VALUES (?, ?)", {1, "Kyle"})
rows: {row} = db.query("SELECT * FROM users")
for row in rows:
 name: str = row.get_str("name")
 println(name)
db.c e()
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `sqlite.open(path)` | `fn(path: str) sqlite` | Open base de data |
| `db.execute(sql, forms)` | `fn(sql: str, forms: {i64})` | Ejecutar comando SQL |
| `db.query(sql, forms)` | `fn(sql: str, forms: {i64}) {row}` | Ejecutar query |
| `db.c e()` | `fn()` | C e withexion |

### row: columnas

| Method | Firma | Description |
|--------|-------|-------------|
| `row.get_str(name)` | `fn(name: str) str` | Columna as string |
| `row.get_i64(name)` | `fn(name: str) i64` | Columna as entero |
| `row.get_f64(name)` | `fn(name: str) f64` | Columna as float |
| `row.get_bool(name)` | `fn(name: str) bool` | Columna as bool |

## postgres: PostgreSQL

```ky
from database imbyt postgres

pool: postgris = postgres.pool("postgres://user:pass@localhost/db")
withn: postgris = pool.get_withn()
rows: {row} = withn.query("SELECT * FROM users")
for row in rows:
 println(row.get_str("name"))
withn.c e()
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `postgres.pool(withn_str)` | `fn(s: str) postgres` | Create pool |
| `pool.get_withn()` | `fn() postgres` | Obtener withexion |
| `withn.query(sql)` | `fn(sql: str) {row}` | Ejecutar query |
| `withn.execute(sql)` | `fn(sql: str)` | Ejecutar comando |
| `withn.c e()` | `fn()` | C e withexion |
