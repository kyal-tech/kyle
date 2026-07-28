# Collections — Lists, Arrays, Dicts, Sets, Iterators

> All collection types in Kyle: dynamic lists, fixed arrays, hash dicts, hash sets, and lazy iterators.
> Every example includes ownership semantics (`&` borrow, `^&` mutable borrow, move).

---

## Array `[T, N]`

Array nativo en **stack**. Tamaño fijo en compile-time.
**Acceso por índice únicamente** — es más rápido y contiguo que una lista.

### Unidimensional

```ky
# Declaración con tipo explícito
arr: [i32, 5] = [1, 2, 3, 4, 5]

# Inferencia de tipo
nums = [1, 2, 3]         # → [i32, 3]

# Repetir valor
repetido = [0; 100]      # → [i32, 100]

# Lectura/escritura por índice (O(1), GEP directo)
x = arr[2]               # load
arr[2] = 99              # store

# Longitud
n = arr.len()            # → 5
```

### Multidimensional

Arrays de arrays vía sintaxis `[T, N]` anidada:

```ky
# 2D: matriz 3×4
matriz: [[i32, 4], 3] = [
    [1, 2, 3, 4],
    [5, 6, 7, 8],
    [9, 10, 11, 12]
]

# Acceso: fila luego columna
x = matriz[0][2]         # → 3 (fila 0, columna 2)
matriz[1][1] = 99        # modificar elemento

# 3D: cubo 2×3×2
cubo: [[[i32, 2], 3], 2] = [
    [[1, 2], [3, 4], [5, 6]],
    [[7, 8], [9, 10], [11, 12]]
]
y = cubo[1][2][0]        # → 11
```

Cada nivel de anidamiento agrega un índice. El acceso es GEP directo en LLVM — cero overhead.

### Iteración

```ky
# Por índice
for i in 0..arr.len():
    println(arr[i].to_str())

# Por valor (elementos copiados si son Copy types)
for val in arr:
    println(val.to_str())

# Por borrow (sin copiar el array)
for val in &arr:
    println(val.to_str())
```

### Ownership y pasaje a funciones

Los arrays son **Copy** porque están en stack. Pero para arrays grandes conviene pasar por borrow para evitar copiar N elementos.

```ky
fn main():
    nums: [i32, 1000] = [0; 1000]
    nums[0] = 1

    # 1. PASAR POR VALOR — COPIA el array completo (1000 × 4 bytes)
    suma1 = sumar_valor(nums)
    # nums sigue accesible (es Copy)

    # 2. PASAR POR BORROW inmutable — sin copia
    suma2 = sumar_borrow(&nums)

    # 3. PASAR POR MUT BORROW — para modificar
    duplicar_todo(^&nums)
    println(nums[1].to_str())  # 2

fn sumar_valor(arr: [i32, 1000]) i64:
    total = 0
    for i in 0..arr.len():
        total = total + arr[i]
    total

fn sumar_borrow(arr: &[i32, 1000]) i64:
    total = 0
    for i in 0..arr.len():
        total = total + arr[i]
    total

fn duplicar_todo(arr: ^&[i32, 1000]):
    for i in 0..arr.len():
        arr[i] = arr[i] * 2
```

### Ownership — Resumen array

| Operación | Firma | Ownership |
|-----------|-------|-----------|
| `arr[i]` | `(&[N]T) i32` → `T` | Borrow inmutable |
| `arr[i] = val` | `(^&[N]T, i32, T)` | Borrow mutable |
| `y = arr` | — | **Copia** (stack, no move) |
| `fn f(a: [N]T)` | — | Copia (por valor) |
| `fn f(a: &[N]T)` | — | Borrow (sin copia) |
| `fn f(a: ^&[N]T)` | — | Mut borrow (permite modificar) |
| `for val in arr` | — | Copia (cada elemento por valor) |
| `for val in &arr` | — | Borrow de cada elemento |

### Cuándo usar array vs lista

| Criterio | Array `[T, N]` | Lista `{T}` |
|----------|----------------|-------------|
| Tamaño | Fijo (compile-time) | Dinámico (crece/decrece) |
| Memoria | Stack | Heap |
| Acceso | Por índice (`arr[i]`) | Por valor (contains, remove) |
| Performance | O(1) GEP directo | O(1) amortizado push/pop |
| Copia | Por defecto (stack) | Move por defecto |
| Iteración | `for i in 0..n` | `for x in &list` |

---

## List `{T}`

Lista dinámica en heap. **Trabaja por valor:** las operaciones buscan, agregan y eliminan por valor, no por posición. Si necesitas acceso por índice, usa arrays.

### Creación

```ky
vacia: {i32} = {0}; vacia.pop()   # inicializar y vaciar
nums: {i32} = {1, 2, 3}           # con valores
nums.push(4)                       # → {1, 2, 3, 4}
```

### Iteración (`for`)

Kyle tiene tres modos de iterar, cada uno con distintas semánticas de ownership:

```ky
libros: {str} = {0 as i64}; libros.pop()
libros.push("El Quijote")
libros.push("Rayuela")
libros.push("Cien Años")

# 1. ITERAR POR VALOR (move) — consume la lista
for libro in libros:
    println(libro)       # cada string se mueve a 'libro'
    # 'libro' se libera al final de cada iteración
# libros ya NO es accesible (move)

# 2. ITERAR POR BORROW (inmutable) — solo lectura
for libro in &libros:
    println(libro)       # prestado, no consumido
# libros sigue siendo accesible

# 3. ITERAR POR MUT BORROW — para modificar elementos
for libro in ^&libros:
    # libro es ^&str — puedes mutarlo
    if libro.contains("Quijote"):
        libro.remove("Quijote")
        libro.push("Quijote de la Mancha")
```

### Modificar durante la iteración

Para modificar (agregar, eliminar, reemplazar) mientras recorres, usa un bucle `while` con índice:

```ky
fn limpiar_nombres(nombres: ^&{str}, suffix: str):
    i = 0
    while i < nombres.len():
        nombre = nombres.get(i)
        if nombre.endswith(suffix):
            nombres.remove_at(i)   # eliminar este, no avanzar
        else:
            i = i + 1              # solo avanzar si no eliminamos

fn duplicar_pares(numeros: ^&{i32}):
    i = 0
    while i < numeros.len():
        val = numeros.get(i)
        if val % 2 == 0:
            numeros.set(i, val * 2)
        i = i + 1
```

### Búsqueda por valor

```ky
fn encontrar_por_titulo(biblioteca: &{str}, titulo: str) bool:
    for libro in &biblioteca:
        if libro == titulo:
            return true
    false

# O directamente:
if biblioteca.contains(titulo):
    println("encontrado")
```

### Eliminar por valor

```ky
fn eliminar_ceros(nums: ^&{i32}):
    nums.remove(0)      # elimina el primer 0 que encuentra
    # Para eliminar TODOS los ceros:
    while nums.contains(0):
        nums.remove(0)
```

### Ownership — Resumen list

| Operación | Firma | Ownership |
|-----------|-------|-----------|
| `list.push(val)` | `(^&{T}, T)` | Borrow mutable + move del valor |
| `list.pop()` | `(^&{T}) T` | Borrow mutable, retorna valor movido |
| `list.get(i)` | `(&{T}, i64) T` | Borrow inmutable |
| `list.set(i, val)` | `(^&{T}, i64, T)` | Borrow mutable |
| `list.contains(val)` | `(&{T}, T) bool` | Borrow inmutable |
| `list.remove(val)` | `(^&{T}, T) i32` | Borrow mutable |
| `for x in list` | — | **Move** (consume la lista) |
| `for x in &list` | — | **Borrow** (no consume) |
| `for x in ^&list` | — | **Mut borrow** (puedes mutar) |

### Stack (LIFO) vía list

```ky
pila: {i32} = {0}; pila.pop()
pila.push(10); pila.push(20); pila.push(30)
valor = pila.pop()  # → 30
```

### Queue (FIFO) vía list

```ky
cola: {i32} = {0}; cola.pop()
cola.push(10); cola.push(20)
valor = cola.pop_first()  # → 10
```

### Métodos completo

| Método | Firma | Descripción | Ownership |
|--------|-------|-------------|-----------|
| `push` | `fn(val: T)` | Agregar al final | `^&` + move |
| `pop` | `fn() T` | Sacar del final (LIFO) | `^&` |
| `pop_first` | `fn() T` | Sacar del inicio (FIFO) | `^&` |
| `len` | `fn() i64` | Cantidad de elementos | `&` |
| `get` | `fn(idx: i64) T` | Obtener por índice | `&` |
| `set` | `fn(idx: i64, val: T)` | Asignar por índice | `^&` |
| `contains` | `fn(val: T) bool` | `true` si el valor existe | `&` |
| `remove` | `fn(val: T) i32` | Eliminar por valor (1=encontrado) | `^&` |
| `remove_at` | `fn(idx: i64) T` | Eliminar por índice | `^&` |
| `insert` | `fn(idx: i64, val: T)` | Insertar en posición | `^&` |
| `clear` | `fn()` | Vaciar la lista | `^&` |
| `reserve` | `fn(capacity: i64)` | Pre-asignar capacidad | `^&` |
| `reverse` | `fn()` | Invertir orden | `^&` |

---

## Dict `{K: V}`

Diccionario hash en heap. Acceso por clave O(1) promedio.
Usa funciones del módulo `dict` (notación de namespace).

```ky
# Creación
d: {str: i32} = {}
prefs = {"tema": 1, "idioma": 2}  # inferencia

# Lectura/escritura
dict.set(d, "clave", 42)
val = dict.get(d, "clave")

# Verificar si una clave existe
if dict.contains(d, "clave"):
    println("existe")

# Eliminar por clave
dict.remove(d, "clave")

# Cantidad de pares
n = dict.len(d)

# Vaciar
dict.clear(d)
```

### Iteración

```ky
# Por borrow (inmutable) — solo lectura de claves
for key in &d:
    println(key)
    println(dict.get(d, key).to_str())
```

### Ownership

| Operación | Firma | Ownership |
|-----------|-------|-------------|
| `dict.set(d, key, val)` | `(^&{K: V}, K, V)` | Borrow mutable |
| `dict.get(d, key)` | `(&{K: V}, K) V` | Borrow inmutable |
| `dict.contains(d, key)` | `(&{K: V}, K) bool` | Borrow inmutable |
| `dict.remove(d, key)` | `(^&{K: V}, K)` | Borrow mutable |
| `dict.len(d)` | `(&{K: V}) i64` | Borrow inmutable |
| `dict.clear(d)` | `(^&{K: V})` | Borrow mutable |

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `dict.set` | `fn(d: ^&{K: V}, key: K, val: V)` | Asignar por clave |
| `dict.get` | `fn(d: &{K: V}, key: K) V` | Obtener por clave |
| `dict.contains` | `fn(d: &{K: V}, key: K) bool` | `true` si la clave existe |
| `dict.remove` | `fn(d: ^&{K: V}, key: K)` | Eliminar por clave |
| `dict.len` | `fn(d: &{K: V}) i64` | Cantidad de pares |
| `dict.clear` | `fn(d: ^&{K: V})` | Vaciar |

---

## Set `set<T>`

Hash set sin duplicados. Búsqueda O(1) promedio.

```ky
# Creación
s: set<i32> = set{1, 2, 3}
vacio: set<str> = set{}

# Agregar
s.add(4)
s.add(1)         # duplicado: ignorado

# Buscar
if s.contains(1):
    println("encontrado")   # true

# Eliminar
s.remove(1)

# Cantidad
n = s.len()      # → 3 (1 fue eliminado)
```

### Iteración

```ky
# Por borrow
for val in &s:
    println(val.to_str())
```

### Ownership

| Operación | Firma | Ownership |
|-----------|-------|-------------|
| `s.add(val)` | `(^&set<T>, T)` | Borrow mutable |
| `s.contains(val)` | `(&set<T>, T) bool` | Borrow inmutable |
| `s.remove(val)` | `(^&set<T>, T) bool` | Borrow mutable |
| `s.len()` | `(&set<T>) i64` | Borrow inmutable |
| `s.clear()` | `(^&set<T>)` | Borrow mutable |

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `add` | `fn(val: T)` | Agregar elemento |
| `contains` | `fn(val: T) bool` | `true` si existe |
| `remove` | `fn(val: T) bool` | Eliminar por valor |
| `len` | `fn() i64` | Cantidad |
| `clear` | `fn()` | Vaciar |

---

## Iterator

Iterador **lazy** sobre listas. No asigna hasta `collect()`.

```ky
nums: {i32} = {0}; nums.pop()
nums.push(1); nums.push(2); nums.push(3); nums.push(4); nums.push(5)

# Crear iterador
it = nums.iter()

# Map (transformar cada elemento)
dobles = it.map(fn(x: i32): x * 2).collect()  # asigna nueva lista

# Filter (filtrar)
pares = nums.iter().filter(fn(x: i32): x % 2 == 0).collect()

# Fold (reducir)
suma = nums.iter().fold(0, fn(acc: i64, x: i64): acc + x)

# Chain (encadenar operaciones)
resultado = nums.iter()
    .map(fn(x: i32): x * 2)
    .filter(fn(x: i32): x > 5)
    .collect()
```

### Métodos

| Método | Firma | Descripción |
|--------|-------|-------------|
| `next` | `fn() T?` | Siguiente elemento (None si termina) |
| `map` | `fn(fn(T) U) iter<U>` | Transformar cada elemento |
| `filter` | `fn(fn(T) bool) iter<T>` | Filtrar elementos |
| `fold` | `fn(init: U, fn(U, T) U) U` | Reducir a un valor |
| `collect` | `fn() {T}` | Recolectar en nueva lista |
| `sum` | `fn() i64` | Sumar todos (numérico) |
| `min` | `fn() T` | Mínimo valor |
| `max` | `fn() T` | Máximo valor |

---

## Comparativa de ownership

| Operación | Copy type (i32, f64, bool) | Move type (str, {T}, struct) |
|-----------|---------------------------|------------------------------|
| `for x in col` | Copia cada elemento | Move (consume) |
| `for x in &col` | Borrow inmutable | Borrow inmutable |
| `for x in ^&col` | Mut borrow | Mut borrow |
| `fn f(col: {T})` | N/A (no Copy) | Move (ownership transfer) |
| `fn f(col: &{T})` | N/A | Borrow |
| `fn f(col: ^&{T})` | N/A | Mut borrow |
| `fn f(arr: [N]T)` | Copia (stack)\(^*\) | N/A |
| `fn f(arr: &[N]T)` | Borrow inmutable | N/A |
| `fn f(arr: ^&[N]T)` | Mut borrow | N/A |
| `y = col` | Copia | **Move** (source inválido) |
| `y = col.clone()` | Copia | Copia explícita |

*\* Los arrays `[T, N]` son siempre Copy porque están en stack, independientemente de T.*

---

## Comparativa de rendimiento

| Operación | Array `[T,N]` | Lista `{T}` | Set `set<T>` | Dict `{K: V}` |
|-----------|:------------:|:-----------:|:------------:|:--------------:|
| Get por índice/clave | O(1) GEP | O(1) GEP | — | O(1) hash |
| Set por índice/clave | O(1) GEP | O(1) GEP | — | O(1) hash |
| Push al final | — | O(1)* | — | — |
| Pop del final | — | O(1) | — | — |
| Contains | O(n) for | O(n) for | **O(1)** hash | **O(1)** hash |
| Remove por valor/clave | — | O(n) find+shift | **O(1)** hash | **O(1)** hash |
| Insert medio | — | O(n) shift | — | — |
| Memoria | Stack (N × sizeof) | Heap + ptr | Heap + hash | Heap + hash |

*\* Amortizado, con crecimiento ocasional O(n).*

---

## See also

- `compound-types.md` — Sintaxis de tipos compuestos (arrays, listas, tuplas, dicts)
- `ownership.md` — Reglas de ownership y borrowing
- `primitive-types.md` — Tipos primitivos (`i32`, `str`, `bool`, etc.)
