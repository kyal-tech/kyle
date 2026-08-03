# sqlite — SQLite database

> Pure Kyle FFI bindings to SQLite3.
> Installation: `ky add sqlite`
> Import: `use sqlite`

## Opening a database

```ky
use sqlite

db: sqlite.database = sqlite.open("data.db")!
```

## Creating tables

```ky
db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")!
```

## Inserting data

```ky
db.execute("INSERT INTO users (name, age) VALUES ('Kyle', 30)")!
```

## Parameterized queries (safe from SQL injection)

```ky
stmt: sqlite.statement = db.prepare("INSERT INTO users (name, age) VALUES (?, ?)")!
stmt.bind_text(1, "Alice")
stmt.bind_int(2, 25)
stmt.step()!
stmt.finalize()!
```

## Querying data

```ky
stmt: sqlite.statement = db.prepare("SELECT id, name, age FROM users WHERE age > ?")!
stmt.bind_int(1, 20)

while stmt.step() == sqlite.row:
    id: i32 = stmt.column_int(0)
    name: str = stmt.column_text(1)
    age: i32 = stmt.column_int(2)
    println(name + " is " + age.to_str())

stmt.finalize()!
```

## Transaction support

```ky
db.execute("BEGIN")!
db.execute("INSERT INTO users (name, age) VALUES ('Bob', 35)")!
db.execute("INSERT INTO users (name, age) VALUES ('Carol', 28)")!
db.execute("COMMIT")!
```

## database

| Method | Signature | Description |
|--------|-----------|-------------|
| `open(path)` | `fn(path: &str) database!` | Open or create database |
| `.execute(sql)` | `fn(sql: &str)!` | Execute SQL statement |
| `.prepare(sql)` | `fn(sql: &str) statement!` | Prepare statement |
| `.close()` | `fn()` | Close database |
| `.error()` | `fn() str` | Last error message |

## statement

| Method | Signature | Description |
|--------|-----------|-------------|
| `.bind_int(index, value)` | `fn(index: i32, value: i32)` | Bind integer parameter |
| `.bind_text(index, value)` | `fn(index: i32, value: &str)` | Bind text parameter |
| `.bind_float(index, value)` | `fn(index: i32, value: f64)` | Bind float parameter |
| `.bind_null(index)` | `fn(index: i32)` | Bind NULL |
| `.step()` | `fn() i32` | Execute step (returns row/done) |
| `.column_count()` | `fn() i32` | Number of columns |
| `.column_type(index)` | `fn(index: i32) i32` | Column type (int, text, float, null) |
| `.column_int(index)` | `fn(index: i32) i32` | Get integer column |
| `.column_text(index)` | `fn(index: i32) str` | Get text column |
| `.column_float(index)` | `fn(index: i32) f64` | Get float column |
| `.column_name(index)` | `fn(index: i32) str` | Column name |
| `.reset()` | `fn()` | Reset for re-execution |
| `.finalize()` | `fn()` | Destroy statement |

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `sqlite.ok` | `0` | Success |
| `sqlite.row` | `100` | Row available |
| `sqlite.done` | `101` | No more rows |
| `sqlite.int_type` | `1` | Column is integer |
| `sqlite.text_type` | `3` | Column is text |
| `sqlite.float_type` | `2` | Column is float |
| `sqlite.null_type` | `5` | Column is NULL |

## Example

```ky
use sqlite

db: sqlite.database = sqlite.open("test.db")!

db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")!

# Insert with parameters
stmt: sqlite.statement = db.prepare("INSERT INTO users (name, age) VALUES (?, ?)")!
stmt.bind_text(1, "Kyle")
stmt.bind_int(2, 30)
stmt.step()!
stmt.finalize()!

# Query
stmt = db.prepare("SELECT id, name, age FROM users")!
while stmt.step() == sqlite.row:
    id: i32 = stmt.column_int(0)
    name: str = stmt.column_text(1)
    age: i32 = stmt.column_int(2)
    println("#" + id.to_str() + ": " + name + " (" + age.to_str() + ")")
stmt.finalize()!

db.close()
```
