# math — Mathematical functions

> Common mathematical operations.
> Import: `use std.math`

## Comparison

```ky
use std.math

x: i32 = math.max(10, 20)       # 20
x = math.min(10, 20)            # 10
x = math.clamp(15, 0, 10)       # 10
```

## Arithmetic

```ky
x: i64 = math.pow(2, 10)        # 1024
x: i32 = math.abs(-5)           # 5
```

## Floating point

```ky
x: f64 = math.sqrt(144.0)       # 12.0
n: i64 = math.floor(3.7)        # 3
n = math.ceil(3.2)              # 4
n = math.round(3.5)             # 4
n = math.trunc(3.7)             # 3
```

## Interpolation

```ky
t: f64 = math.lerp(0.0, 10.0, 0.5)   # 5.0
```

## Constants

```ky
pi: f64 = math.pi    # 3.141592653589793
e: f64 = math.e      # 2.718281828459045
```

## Trigonometric

```ky
x: f64 = math.sin(math.pi / 2)    # 1.0
x = math.cos(0.0)                 # 1.0
x = math.tan(0.0)                 # 0.0
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `max(a, b)` | `fn(a: T, b: T) T` | Larger of two values |
| `min(a, b)` | `fn(a: T, b: T) T` | Smaller of two values |
| `abs(x)` | `fn(x: T) T` | Absolute value |
| `pow(base, exp)` | `fn(base: i64, exp: i64) i64` | Exponentiation |
| `clamp(val, min, max)` | `fn(val: T, min: T, max: T) T` | Constrain to range |
| `lerp(a, b, t)` | `fn(a: f64, b: f64, t: f64) f64` | Linear interpolation |
| `sqrt(x)` | `fn(x: f64) f64` | Square root |
| `floor(x)` | `fn(x: f64) i64` | Round down |
| `ceil(x)` | `fn(x: f64) i64` | Round up |
| `round(x)` | `fn(x: f64) i64` | Round (0.5 up) |
| `trunc(x)` | `fn(x: f64) i64` | Truncate decimal part |
| `sin(x)` | `fn(x: f64) f64` | Sine (radians) |
| `cos(x)` | `fn(x: f64) f64` | Cosine (radians) |
| `tan(x)` | `fn(x: f64) f64` | Tangent (radians) |

## Example

```ky
use std.math

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
    error(e):
        println(e)
```
