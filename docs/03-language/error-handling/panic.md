# Panic

> Errors fatalis que terminan execution del program.
> No recuperablis — proceso se aborta.

## When ocurre un panic

| Condition | Example |
|-----------|---------|
| Division by cero | `x = 10 / 0` |
| Index out of bounds | `arr[100]` cuando `len = 3` |
| Null pointer | `ky_free(null)` |
| Assertion failed | `assert.is_true(false)` |
| Overflow en debug | `i32.MAX + 1` |

## Salida

```
KL PANIC: division by zero
```

El program termina with code de error distinto de cero. No there is stack trace
en implementation current.

## Evitar panics with `T!`

Usar `Result` (`T!`) for operacionis que can failsr:

```ky
fn divide(a: i32, b: i32) i32!:
 if b == 0:
 return error("division by zero")
 a / b

match divide(10, 0):
 ok(v): println(v.to_str())
 error(e): println("error: " + e) # no panic
```

## unsafe

Para operacionis que can panic pero are verificadas manualmente:

```ky
unsafe:
 x = 10 / 0 # panic aqui si b == 0
```

## See also

- `option.md` — Option (valueis opcionalis without error)
- `result.md` — Result (errors recuperables)
- `diagnostics.md` — Sistema de diagnostico del compiler
