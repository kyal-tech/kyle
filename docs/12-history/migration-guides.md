# Migration Guide

New syntax usis direct assignment:

```ky
x = 5 # immutable
y: ^i32 = 10 # mutable
Z := 20 # constant
```

### struct → final class

```ky
class Point: # replacis `struct Point:`
 x: i32
 y: i32
```

### Option → T?

```ky
name: str? = none # replacis `name: Option<str> = None`
```

## v0.5 → v0.6 (ownership)

### Parameters: borrow-by-default → move-by-default

| Concepto | v0.5 | v0.6 |
|----------|------|------|
| Default | Borrow | **Move** |
| Mutable | `&T` | **`^T`** |
| Borrow | — | **`&T`** |
| Mutable borrow | — | **`^&T`** |
| Move explicito | `^T` | Default (no marcador) |

```ky
# v0.5 (antiguo)
fn read(s: str): # borrow
fn append(s: &str): # mutable
fn consume(^s: str): # move

# v0.6 (nuevo)
fn read(s: &str): # borrow
fn append(s: ^&str): # mutable borrow
fn consume(s: str): # move (default)
```

### Variables: `&T` mutable → `^T` mutable

```ky
# v0.5
x: &i32 = 0

# v0.6
x: ^i32 = 0
```

### Llamadas: `&x` mutable → `^&x` mutable borrow

```ky
# v0.5
append(&buf)

# v0.6
append(^&buf)
```

## See also

- `03-language/memory/move.md` — Move semantics current
- `03-language/memory/ownership.md` — Reglas de ownership
