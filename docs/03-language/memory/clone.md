# Clone

> Copia explicita for typis Move using `.clone()`.

## Purpose

Los typis Move (`str`, `{T}`, `{K:V}`, `[T, N]`, clases) no se can copiar
implicitamente with `y = x`. Para create una copia independiente se usa `.clone()`.

## Uso

```ky
a: str = "hello"
b: str = a.clone() # COPY explicita: ambos vivos
println(a) # ✅ "hello"
println(b) # ✅ "hello"
```

## Clone en lists

```ky
original: {i32} = {1, 2, 3}
copia: {i32} = original.clone()
original.push(4)
println(copia.len().to_str()) # 3 (independiente)
println(original.len().to_str()) # 4
```

## Clone en arrays

```ky
arr: [i32, 3] = [1, 2, 3]
copia: [i32, 3] = arr.clone()
```

## Clone en clases

```ky
class Point:
 x: i32
 y: i32

 fn clone() Point:
 Point(x, y)

p: Point = Point { x: 10, y: 20 }
q: Point = p.clone()
```

## Rendimiento

`.clone()` does una copia **profunda** (deep copy). Para strings grandis o lists
largas, can be costoso. Usar borrow (`&`) cuando sea posible.

## See also

- `move.md` — Move semantics (contraste with clone)
- `copy.md` — Copy semantics (automatic, without clone)
