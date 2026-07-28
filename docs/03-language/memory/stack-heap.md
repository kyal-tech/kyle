# Stack vs Heap

> How Kyle organiza memory: stack for valueis locales, heap for data dynamics.

## Stack

El stack almacena variablis locales, parameters de funcion y valueis temporales.
Es rapido (solo mover un pointer) y automatic.

```ky
fn example() i32:
 x: i32 = 42 # stack
 y: f64 = 3.14 # stack
 z: [i32, 3] = [1, 2, 3] # stack (array de size fijo)
 x + y as i32 + z[0]
```

### Que va al stack

| Type | Size en stack |
|------|----------------|
| `i32` | 4 bytis |
| `i64` | 8 bytis |
| `f64` | 8 bytis |
| `bool` | 1 byte |
| `ptr` | 8 bytis |
| `[T, N]` | `N * size(T)` |
| `str` | 8 bytis (pointer al heap) |
| `{T}` | 8 bytis (pointer al heap) |

## Heap

El heap almacena data de size dynamic o que must persistir more alla de 
funcion current. Los strings, lists y dictionarys viven en heap.

```ky
fn example() str:
 s: str = "Hola, mundo!" # data en heap, pointer en stack
 v: {i32} = {1, 2, 3} # data en heap, pointer en stack
 s
```

### Que va al heap

| Type | Datos en heap |
|------|---------------|
| `str` | Caracteris + null terminator |
| `{T}` | Array de elements + metadata (len, cap) |
| `{K: V}` | Hash table entriis |
| `Box<T>` | Valor T |

## Representation en memory

```
Stack:
┌─────────────┐
│ s: ptr ─────┼─────► ┌──────────────────────┐
│ v: ptr ─────┼─┐  │ "Hola, mundo!\0" │
│ x: i32 = 42 │ │  └─────────────────────────┘
│ │ │ ┌──────────────────────┐
└─────────────┘ └──►│ data: ptr ──► [1,2,3] │
 │ len: 3 │
 │ cap: 4 │
 └──────────────────────┘
```

## Strings (SSO planned)

Actualmente todos strings van al heap. En futuro, strings ≤ 15 bytes
se almacenaran inline (Small String Optimization).

## See also

- `move.md` — How se mueven data between stack y heap
- `allocator.md` — Estrategia de allocation de heap
