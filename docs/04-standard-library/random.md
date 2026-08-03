# random — Random number generation

> Generate random numbers and shuffle collections.
> Import: `use std.random`

## Integers

```ky
use std.random

n: i32 = random.int(100)         # 0..99
n = random.int_range(10, 20)     # 10..19
```

## Floating point

```ky
x: f64 = random.float()            # 0.0..1.0
x = random.float_range(0.0, 10.0)  # 0.0..10.0
```

## Booleans

```ky
b: bool = random.bool()
```

## Collections

```ky
items: [i32] = [1, 2, 3, 4, 5]
random.shuffle(items)       # shuffle in-place
item: i32 = random.choice(items)  # pick random element
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `int(max)` | `fn(max: i32) i32` | Integer 0..max-1 |
| `int_range(min, max)` | `fn(min: i32, max: i32) i32` | Integer min..max-1 |
| `float()` | `fn() f64` | Float 0.0..1.0 |
| `float_range(min, max)` | `fn(min: f64, max: f64) f64` | Float min..max |
| `bool()` | `fn() bool` | Random boolean |
| `shuffle(list)` | `fn(list: ^&[T])` | Shuffle in-place |
| `choice(list)` | `fn(list: &[T]) T` | Random element |

## Example

```ky
use std.random

# Roll a die
dado: i32 = random.int_range(1, 7)
println("dado: " + dado.to_str())

# Shuffle cards
cartas: [i32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
random.shuffle(cartas)
println("primera carta: " + cartas[0].to_str())
```
