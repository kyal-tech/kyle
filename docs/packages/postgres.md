# postgres — PostgreSQL client

> Pure Kyle FFI bindings to libpq for PostgreSQL.
> Installation: `ky add postgres`
> Import: `use postgres`

## Connecting

```ky
use postgres

db: postgres.db = postgres.connect("host=localhost port=5432 dbname=mydb user=user password=pass")!
```

## Simple query

```ky
rows: postgres.result = db.query("SELECT id, name, age FROM users")!
for i in 0..rows.len():
    row: postgres.row = rows.get(i)
    println(row.get_str("name"))
    println(row.get_int("age").to_str())
```

## Parameterized query

Parameters are passed as a list of string values. PostgreSQL converts them to the target column type.

```ky
rows: postgres.result = db.query_params(
    "SELECT * FROM users WHERE age > $1 AND active = $2",
    ["30", "true"]
)!
println("found " + rows.len().to_str() + " users")
```

## Typed row access

```ky
rows: postgres.result = db.query("SELECT id, name, email, age, active FROM users")!
for i in 0..rows.len():
    row: postgres.row = rows.get(i)
    id: i32 = row.get_int("id")
    name: str = row.get_str("name")
    email: str = row.get_str("email")
    age: i32 = row.get_int("age")
    active: bool = row.get_bool("active")
    println(name + " (" + email + ")")
```

## INSERT, UPDATE, DELETE

```ky
db.execute("CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name TEXT)")!
db.execute_params("INSERT INTO users (name) VALUES ($1)", ["Kyle"])!
db.execute_params("UPDATE users SET name = $1 WHERE id = $2", ["Updated", "1"])!
db.execute_params("DELETE FROM users WHERE id = $1", ["1"])!
```

## Prepared statements with typed bindings

For full type safety, use prepared statements with explicit bind methods.

```ky
stmt: postgres.statement = db.prepare("INSERT INTO users (name, age) VALUES ($1, $2)")!
stmt.bind_str(1, "Kyle")?
stmt.bind_int(2, 30)?
stmt.execute()!
stmt.finalize()!

# Query with typed binds
stmt = db.prepare("SELECT name, age FROM users WHERE age > $1")!
stmt.bind_int(1, 25)?
while stmt.step():
    name: str = stmt.column_str(0)
    age: i32 = stmt.column_int(1)
    println(name + " is " + age.to_str())
stmt.finalize()!
```

## Transactions

```ky
tx: postgres.transaction = db.begin()!
tx.execute_params("INSERT INTO users (name) VALUES ($1)", ["Alice"])!
tx.execute_params("INSERT INTO users (name) VALUES ($1)", ["Bob"])!
tx.commit()!
# or tx.rollback()! on error
```

## db

| Method | Signature | Description |
|--------|-----------|-------------|
| `connect(conninfo)` | `fn(conninfo: &str) db!` | Connect to PostgreSQL |
| `.query(sql)` | `fn(sql: &str) result!` | Execute query |
| `.query_params(sql, params)` | `fn(sql: &str, params: &[str]) result!` | Parameterized query |
| `.execute(sql)` | `fn(sql: &str)!` | Execute statement |
| `.execute_params(sql, params)` | `fn(sql: &str, params: &[str])!` | Parameterized statement |
| `.prepare(sql)` | `fn(sql: &str) statement!` | Create prepared statement |
| `.begin()` | `fn() transaction!` | Start transaction |
| `.close()` | `fn()` | Close connection |

## result

| Method | Signature | Description |
|--------|-----------|-------------|
| `.len()` | `fn() i32` | Number of rows |
| `.cols()` | `fn() i32` | Number of columns |
| `.get(row)` | `fn(row: i32) row` | Get row at index |
| `.free()` | `fn()` | Free result memory |

## row

| Method | Signature | Description |
|--------|-----------|-------------|
| `.get_str(name)` | `fn(name: &str) str` | Column as string |
| `.get_int(name)` | `fn(name: &str) i32` | Column as integer |
| `.get_float(name)` | `fn(name: &str) f64` | Column as float |
| `.get_bool(name)` | `fn(name: &str) bool` | Column as boolean |
| `.is_null(name)` | `fn(name: &str) bool` | True if column is NULL |

## statement

| Method | Signature | Description |
|--------|-----------|-------------|
| `.bind_str(index, val)` | `fn(index: i32, val: &str)` | Bind string parameter |
| `.bind_int(index, val)` | `fn(index: i32, val: i32)` | Bind integer parameter |
| `.bind_float(index, val)` | `fn(index: i32, val: f64)` | Bind float parameter |
| `.bind_null(index)` | `fn(index: i32)` | Bind NULL |
| `.execute()` | `fn()!` | Execute statement |
| `.step()` | `fn() bool` | Fetch next row (true = row available) |
| `.column_str(index)` | `fn(index: i32) str` | Get text column |
| `.column_int(index)` | `fn(index: i32) i32` | Get integer column |
| `.column_float(index)` | `fn(index: i32) f64` | Get float column |
| `.finalize()` | `fn()` | Destroy statement |

## transaction

| Method | Signature | Description |
|--------|-----------|-------------|
| `.query(sql)` | `fn(sql: &str) result!` | Query in transaction |
| `.query_params(sql, params)` | `fn(sql: &str, params: &[str]) result!` | Parameterized query |
| `.execute(sql)` | `fn(sql: &str)!` | Execute statement |
| `.execute_params(sql, params)` | `fn(sql: &str, params: &[str])!` | Parameterized execute |
| `.commit()` | `fn()!` | Commit transaction |
| `.rollback()` | `fn()!` | Rollback transaction |

## Example

```ky
use postgres

db: postgres.db = postgres.connect("host=localhost dbname=test")!

db.execute("CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name TEXT, age INTEGER)")!

db.execute_params("INSERT INTO users (name, age) VALUES ($1, $2)", ["Kyle", "30"])!

rows: postgres.result = db.query("SELECT * FROM users ORDER BY id")!
for i in 0..rows.len():
    row: postgres.row = rows.get(i)
    println(row.get_str("name") + " is " + row.get_int("age").to_str())

db.close()
```
