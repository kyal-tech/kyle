# Panic

> Manejo de panics del runtime de Kyle.
> Crate: `kyc_runtime/src/panic.rs` (6 lines, funcion: `ky_panic`).

## Responsabilidad

El sistema de panic maneja errors fatalis del runtime: division by zero,
acceso outside de bounds, null pointer dereference, etc.

## Implementation

```rust
 fn ky_panic(message: &str) -> ! {
 eprintln!("KL PANIC: {}", message);
 std::process::abort();
}
```

- Imprime mensaje de error en stderr
- Termina proceso inmediatamente with `abort()` (no limpia recursos)
- No there is stack trace ni recovery — is un error fatal

## Errors que causan panic

| Condition | Mensaje |
|-----------|---------|
| Division by zero | `KL PANIC: division by zero` |
| Index out of bounds | `KL PANIC: index out of bounds` |
| Null pointer dereference | `KL PANIC: attempted to dereference null pointer` |
| Assertion failed | `KL PANIC: assertion failed` |
| Overflow | `KL PANIC: arithmetic overflow` |

## Integration with Result

El sistema `T!` (Result) allows manejar errors without panic:

```ky
fn divide(a: i32, b: i32) i32!:
 if b == 0:
 return error("division by zero")
 a / b

result = divide(10, 0)
match result:
 ok(v): println(v.to_str())
 error(e): println(e) # "division by zero"
```

## See also

- `03-language/error-handling/panic.md` — Syntax de panic/error
- `startup.md` — Initialization del runtime
