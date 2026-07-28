# core — Typis Fundamentales

> Module base with typis `option` (`T?`) y `result` (`T!`).
> Import: `use core.{option, result}`

## option: `option<T>` / `T?`

Representa un value opcional: `some(val: T)` o `none`.

```ky
use core.option

name: option<str> = option.some("Kyle")
name = option.none

match name:
 option.some(v): println(v)
 option.none: println("no name")
```

### Syntax sugar `T?`

```ky
name: str? = "Kyle"
name = none

match name:
 some(v): println(v)
 none: println("no name")
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `is_some` | `fn() bool` | `true` si has value |
| `is_none` | `fn() bool` | `true` si is none |
| `unwrap` | `fn() T` | Retorna value o panic |
| `unwrap_or` | `fn(default: T) T` | Valor o default |

```ky
name: str? = get_user_name()
if name.is_some():
 println(name.unwrap())
```

## result: `result<T, E>` / `T!`

Representa una operation que can failsr: `ok(val: T)` o `error(msg: E)`.

```ky
use core.result

fn divide(a: i32, b: i32) result<i32, str>:
 if b == 0:
 return result.error("division by zero")
 result.ok(a / b)
```

### Syntax sugar `T!`

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

| Method | Firma | Description |
|--------|-------|-------------|
| `is_ok` | `fn() bool` | `true` si is ok |
| `is_error` | `fn() bool` | `true` si is error |
| `unwrap` | `fn() T` | Retorna value o panic |
| `unwrap_or` | `fn(default: T) T` | Valor o default |

## See also

- `03-language/error-handling/option.md`
- `03-language/error-handling/result.md`
