# csv — CSV parsing and serialization

> Parse, serialize, read, and write CSV data.
> Import: `use std.csv`

## Parse CSV string

```ky
use std.csv

data: str = "name,age\nKyle,30\nAna,25\n"
rows: [csv.row] = csv.parse(data)!

for row in rows:
    println(row.get("name") + " is " + row.get("age"))
```

## Serialize to CSV

```ky
rows: [csv.row] = [
    csv.row({"name": "Kyle", "age": "30"}),
    csv.row({"name": "Ana", "age": "25"}),
]

data: str = csv.to_str(rows)
println(data)
# name,age
# Kyle,30
# Ana,25
```

## Read CSV file

```ky
rows: [csv.row] = csv.parse_file("users.csv")!

for row in rows:
    name: str = row.get("name")
    age: i32 = row.get("age").to_i32()
    println(name)
```

## Write CSV file

```ky
rows: [csv.row] = [
    csv.row({"name": "Kyle", "age": "30"}),
    csv.row({"name": "Ana", "age": "25"}),
]

csv.to_file("output.csv", rows)!
```

## Typed serialization with classes

```ky
class User:
    name: str
    age: i32
    active: bool

users: [User] = [
    User { name: "Kyle", age: 30, active: true },
    User { name: "Ana", age: 25, active: false },
]

# Class to CSV
data: str = csv.to_str(users)!
# name,age,active
# Kyle,30,true
# Ana,25,false

# CSV to class list
parsed: [User] = csv.from_str<User>(data)!
for user in parsed:
    println(user.name)
```

## Options

```ky
# Custom delimiter
data: str = csv.parse_with("name;age\nKyle;30\n", ";")!

# With headers row
rows: [csv.row] = csv.parse("a,b\n1,2\n3,4\n")
for row in rows:
    println(row.get("a"))
```

## csv.row

| Method | Signature | Description |
|--------|-----------|-------------|
| `.get(col)` | `fn(col: &str) str` | Get value by column name |
| `.get_at(index)` | `fn(index: i32) str` | Get value by column index |
| `.columns()` | `fn() [str]` | Get all column names |
| `.values()` | `fn() [str]` | Get all values |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse(s)` | `fn(s: &str) [row]!` | Parse CSV string |
| `parse_file(path)` | `fn(path: &str) [row]!` | Parse CSV file |
| `to_str(rows)` | `fn(rows: &[row]) str` | Serialize rows to CSV |
| `to_file(path, rows)` | `fn(path: &str, rows: &[row])!` | Write CSV to file |
| `from_str<T>(s)` | `fn(s: &str) [T]!` | Parse CSV to class list |
| `to_str(objs)` | `fn(objs: &[T]) str` | Serialize class list to CSV |
| `row(fields)` | `fn(fields: &{str: str}) row` | Create CSV row |

## Example

```ky
use std.csv

# Parse and process
rows: [csv.row] = csv.parse_file("users.csv")!
for row in rows:
    name: str = row.get("name")
    age: i32 = row.get("age").to_i32()
    println(name + " is " + age.to_str() + " years old")

# Filter and write
adults: [csv.row] = []
for row in rows:
    if row.get("age").to_i32() >= 18:
        adults.push(row)
csv.to_file("adults.csv", adults)!
```
