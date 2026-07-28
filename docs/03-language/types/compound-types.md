# Compound Types

> Tipos compuestos de Kyle: arrays, listas, tuplas, diccionarios.

## Array: `[T, N]`

Array nativo en stack, tamaño fijo en compile-time.
**Los arrays trabajan por índice** — acceso directo y óptimo vía GEP.

### Unidimensional

```ky
arr: [i32, 3] = [1, 2, 3]
arr = [0; 100]    # repetir valor
x: i32 = arr[0]   # get por índice (GEP + load)
arr[0] = 99       # set por índice (GEP + store)

for i in 0..arr.len():
    println(arr[i].to_str())
```

### Multidimensional

Sintaxis `[T, N]` anidada para arrays de 2+ dimensiones:

```ky
# 2D: matriz 3×4
matriz: [[i32, 4], 3] = [
    [1, 2, 3, 4],
    [5, 6, 7, 8],
    [9, 10, 11, 12]
]
x = matriz[1][2]  # → 7

# 3D: cubo 2×3×2
cubo: [[[i32, 2], 3], 2] = [
    [[1, 2], [3, 4], [5, 6]],
    [[7, 8], [9, 10], [11, 12]]
]
y = cubo[1][2][0]  # → 11
```

### Pasaje por borrow vs mutable borrow

```ky
fn fn_borrow(arr: &[i32, 100]):     # solo lectura
    println(arr[0].to_str())

fn fn_mut(arr: ^&[i32, 100]):       # permite modificar
    arr[0] = 99
```

## List: `{T}`

Lista dinámica en heap.
**Las listas trabajan por valor** — búsqueda, inserción, eliminación son por valor, no por posición.

```ky
v: {i32} = {1, 2, 3}
v.push(4)

# Operaciones por valor
if v.contains(2):
    println("está")
v.remove(2)           # eliminar por valor (no por índice)

# Indexar solo si sabes la posición exacta
x = v.get(0)
v.set(0, 99)
```

## Tuple: `(T1, T2, ...)`

```ky
t: (i32, str, f64) = (1, "hello", 3.14)
x: i32 = t.0
y: str = t.1
```

## Dict: `{K: V}`

```ky
d: {str: i32} = {"key": 42}
val: i32 = d["key"]
```

## Slice: `&[T]`

```ky
arr: [i32, 5] = [10, 20, 30, 40, 50]
s: &[i32] = arr[1..4]    # slice: [20, 30, 40]
println(s.len().to_str()) # → 3
println(s[0].to_str())    # → 20
```

Fat pointer `{ptr, len}`. Copy semantics. No ownership.

```ky
d: {str: i32} = {"key": 42}
val: i32 = d["key"]
```

## Comparison

| Type | Heap/Stack | Size | Acceso primario |
|------|-----------|--------|-----------------|
| `[T, N]` | Stack | Fijo | Por índice (GEP) |
| `&[T]` | Stack (fat ptr) | 16 (ptr+len) | Por índice + len |
| `{T}` | Heap | Dinámico | Por valor (push/pop/remove/contains) |
| `(T1, T2)` | Stack | Fijo | Por campo (.0, .1) |
| `{K: V}` | Heap | Dinámico | Por clave (set/get) |

## See also

- `collections.md` — API completa con ejemplos de ownership, for loops, modificar durante iteración
- `primitive-types.md` — Tipos primitivos
- `structs.md` — Tipos usuario (class)
- `generics.md` — Tipos genéricos
- `ownership.md` — Reglas de ownership
