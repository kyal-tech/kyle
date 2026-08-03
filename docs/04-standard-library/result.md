# result — option and result types

> Core types: `option<T>` / `T?` for optional values, `result<T, E>` / `T!` for fallible operations.
> Import: `use std.result`

## option

Represents a value that may or may not be present: `some(value)` or `none`.

```ky
use std.result

name: option<str> = some("Kyle")
name = none

match name:
    some(v): println(v)
    none: println("no name")
```

### Sugar syntax: `T?`

```ky
name: str? = "Kyle"
name = none

match name:
    some(v): println(v)
    none: println("no name")
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_some` | `fn() bool` | True if contains a value |
| `is_none` | `fn() bool` | True if contains nothing |
| `unwrap` | `fn() T` | Returns value or panics |
| `unwrap_or` | `fn(default: T) T` | Returns value or default |

```ky
name: str? = get_user_name()
if name.is_some():
    println(name.unwrap())
```

### Pattern binding in if/while

```ky
if name = optional_value:
    println(name)  # name is available inside this block

while line = read_line():
    println(line)
```

## result

Represents an operation that can succeed or fail: `ok(value)` or `error(message)`.

```ky
use std.result

fn divide(a: i32, b: i32) result<i32, str>:
    if b == 0:
        return error("division by zero")
    ok(a / b)
```

### Sugar syntax: `T!`

```ky
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    a / b

res: i32! = divide(10, 2)
match res:
    ok(v): println(v.to_str())
    error(e): println("error: " + e)
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_ok` | `fn() bool` | True if result is ok |
| `is_error` | `fn() bool` | True if result is error |
| `unwrap` | `fn() T` | Returns value or panics |
| `unwrap_or` | `fn(default: T) T` | Returns value or default |
| `unwrap_error` | `fn() E` | Returns error message or panics |

### Error propagation: `!`

```ky
use std.fs
use std.json

fn read_config(path: &str) Config!:
    content: str = fs.read(path)!   # propagates on error
    config: Config = json.from_str<Config>(content)!
    ok(config)
```

### Null-coalescing: `??`

```ky
name: str = user_name ?? "anonymous"
```

## Design notes

- `result<T, E>` replaces exception-based error handling. Errors are values, not control flow.
- The `!` operator propagates errors upward, similar to Rust's `?` operator.
- `option<T>` replaces null pointers. No null reference errors in Kyle.
