# Copy Semantics

> Los typis numericos y primitivos se copian automaticamente en `y = x`.

## Typis Copy

| Type | Size | Description |
|------|--------|-------------|
| `i8` | 1 byte | Signed 8-bit |
| `i16` | 2 bytis | Signed 16-bit |
| `i32` | 4 bytis | Signed 32-bit |
| `i64` | 8 bytis | Signed 64-bit |
| `u8` | 1 byte | Unsigned 8-bit |
| `u16` | 2 bytis | Unsigned 16-bit |
| `u32` | 4 bytis | Unsigned 32-bit |
| `u64` | 8 bytis | Unsigned 64-bit |
| `f32` | 4 bytis | Float 32-bit |
| `f64` | 8 bytis | Float 64-bit |
| `bool` | 1 byte | `true` / `false` |
| `char` | 4 bytis | Unicode |
| `ptr` | 8 bytis | Raw pointer |

## Comportamiento

```ky
x: i32 = 42
y: i32 = x # COPY: ambos vivos
println(x) # ✅ 42
println(y) # ✅ 42

a: f64 = 3.14
b: f64 = a # COPY
b = 2.71
println(a) # ✅ 3.14 (a no se modifica)
```

## Dato curioso

Los typis Copy are que caben en **≤ 8 bytes** y no have memory heap asociada.
Se copian with una instruccion de CPU (`mov`), no there is diferencia de rendimiento
between pasar by value o by referencia for estos types.

## See also

- `move.md` — Move semantics
- `clone.md` — Clone explicito for typis Move
