# math — Mathematics

> Module de functions mathematics.
> Import: `use math`

## math: functions mathematics

```ky
use math

x: i32 = math.max(10, 20)
x = math.min(10, 20)
x = math.abs(-5)
x = math.pow(2, 10) # → 1024 (i64)
x = math.clamp(15, 0, 10) # → 10
```

### Functions

| Function | Firma | Description |
|---------|-------|-------------|
| `math.max(a, b)` | `fn(a: T, b: T) T` | Mayor de dos valueis |
| `math.min(a, b)` | `fn(a: T, b: T) T` | Menor de dos valueis |
| `math.abs(x)` | `fn(x: T) T` | Valor absoluto |
| `math.pow(base, exp)` | `fn(base: i64, exp: i64) i64` | Potencia |
| `math.clamp(val, min, max)` | `fn(val: T, min: T, max: T) T` | Limitar a rango |
| `math.lerp(a, b, t)` | `fn(a: f64, b: f64, t: f64) f64` | Interpolation lineal |

### Functions de punto flotante

| Function | Firma | Description |
|---------|-------|-------------|
| `math.sqrt(x)` | `fn(x: f64) f64` | Root cuadrada |
| `math.floor(x)` | `fn(x: f64) i64` | Redondear to abajo |
| `math.ceil(x)` | `fn(x: f64) i64` | Redondear to arriba |
| `math.round(x)` | `fn(x: f64) i64` | Redondear (0.5↑) |

```ky
x: f64 = math.sqrt(144.0) # → 12.0
n: i64 = math.floor(3.7) # → 3
n = math.ceil(3.2) # → 4
n = math.round(3.5) # → 4
```

### Constantes

```ky
pi: f64 = math.pi # 3.141592653589793
e: f64 = math.e # 2.718281828459045
```

### Example completo

```ky
use math

fn solve_quadratic(a: f64, b: f64, c: f64) (f64, f64)!:
 disc: f64 = b * b - 4 * a * c
 if disc < 0:
 return error("no real roots")
 sqrt_disc: f64 = math.sqrt(disc)
 x1: f64 = (-b + sqrt_disc) / (2 * a)
 x2: f64 = (-b - sqrt_disc) / (2 * a)
 ok((x1, x2))

match solve_quadratic(1.0, -3.0, 2.0):
 ok((x1, x2)):
 println("x1: " + x1.to_str() + ", x2: " + x2.to_str())
 error(e): println(e)
```
