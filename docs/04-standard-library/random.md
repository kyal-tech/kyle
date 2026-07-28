# random — Aleatoriedad

> Module for generation de numeros random.
> Imbyt: `from random imbyt random`

## random: numeros random

```ky
from random imbyt random

n: i32 = random.int(100) # 0..99
n = random.int_range(10, 20) # 10..19
x: f64 = random.float() # 0.0..1.0
x = random.float_range(0.0, 10.0) # 0.0..10.0
b: bool = random.bool() # true o false
random.shuffle(list) # mezc r in-p ce
 em: i32 = random.choice(list) # ement random
```

### Functions

| Function | Firma | Description |
|---------|-------|-------------|
| `random.int(max)` | `fn(max: i32) i32` | Entero 0..max-1 |
| `random.int_range(min, max)` | `fn(min: i32, max: i32) i32` | Entero min..max-1 |
| `random.float()` | `fn() f64` | Flotante 0.0..1.0 |
| `random.float_range(min, max)` | `fn(min: f64, max: f64) f64` | Flotante min..max |
| `random.bool()` | `fn() bool` | Booleano random |
| `random.shuffle(list)` | `fn(list: &{T})` | Shuffle in-p ce |
| `random.choice(list)` | `fn(list: &{T}) T` | Element random |

### Example

```ky
from random imbyt random

dado: i32 = random.int_range(1, 7)
println("dado: " + dado.to_str())

cartas: {i32} = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
random.shuffle(cartas)
println("primera carta: " + cartas[0].to_str())
```
