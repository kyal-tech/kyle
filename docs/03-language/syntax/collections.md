# Collections

**Status:** [x] Documentación completa de sintaxis. [ ] Implementación en compilador.

## Principios de diseño

- **snake_case** para todos los tipos (`list`, `set`, `dict`, `queue`, `stack`)
- **Sin `new`**: `Type{valores}` para constructor literal, `Type()` para vacío
- **Sin `;`**: Kyle no usa punto y coma
- **Ortogonalidad total**: `^`, `&`, `?`, `!` funcionan en todas las colecciones
- **Consistencia**: métodos como `.push()`, `.pop()`, `.len()` iguales en todas

---

## Lista `[T]`

Colección ordenada y mutable (si es `^[T]`).

```ky
items = [1, 2, 3]                   # [i32] inferido
items: [i32] = [1, 2, 3]           # explícito, inmutable
items: ^[str] = ["a", "b"]          # mutable
items: ^[i32] = []                  # mutable vacío
items = ^[]                         # mutable vacío inferido
items = ^[1, 2]                     # mutable con valores inferido
items: &[i32] = &[1, 2]            # borrow
items: ^&[i32]                      # mutable borrow
items: ^&[i32]?                     # mutable borrow opcional
items: ^&[i32]!                     # mutable borrow con error

# Métodos
items.len()
items.push(4)
items.pop()                         # Option<T>
items[0]                            # indexado
items.get(0)                        # Option<T> (seguro)
items.first()                       # Option<T>
items.last()                        # Option<T>
items.insert(1, 99)
items.remove(0)
items.contains(2)
items.index_of(2)                   # Option<i32>
items.map(fn(x) x * 2)
items.filter(fn(x) x > 0)
items.fold(0, fn(acc, x) acc + x)
items.reverse()
items.sort()
items.chunk(2)                      # [[1,2], [3,4], ...]
items.clear()

# Conversión
items.to_array()
items.to_set()
items.to_queue()
items.to_stack()
items.to_deque()
```

---

## Array `[T, N]`

Tamaño fijo conocido en compile-time.

```ky
arr: [i32, 5] = [1, 2, 3, 4, 5]    # array fijo
arr: [i32, 100] = []                 # array de 100 ceros
arr: ^[str, 10] = ["a", "b"]        # array mutable (raro)
arr: [i32, 5]?                      # array opcional

# Matrices
grid: [[i32, 3], 3] = [
    [1, 0, 0],
    [0, 1, 0],
    [0, 0, 1],
]
cube: [[[i32, 5], 5], 5] = []

# Métodos
arr.len()                           # constante (compile-time)
arr[0] = 99                         # asignación directa
arr.to_list()                       # → [T]
slice = arr[1..3]                   # &[i32] slice
```

**Lista vs Array:**
- `[T]` = lista (tamaño dinámico)
- `[T, N]` = array (tamaño fijo)
- `[1, 2, 3]` sin tipo = lista (porque no especifica tamaño)

---

## Set `set<T>`

Colección no ordenada sin duplicados.

```ky
nums = set{1, 2, 3}                 # set<i32> inferido
nums: set<i32> = set{1, 2, 3}
nums: ^set<str> = set{"a", "b"}
nums: ^set<i32> = set()             # vacío
nums = set()                        # vacío inferido
nums: ^set<i32>?

# Métodos
nums.len()
nums.contains(2)
nums.add(4)
nums.remove(2)
nums.clear()

# Operaciones de conjunto
a = set{1, 2, 3}
b = set{3, 4, 5}
a.union(b)
a.intersection(b)
a.difference(b)
a.symmetric_difference(b)
a.is_subset(b)

# Conversión
nums.to_list()
nums.to_array()
```

---

## Dict `{K: V}`

```ky
d = {"a": 1, "b": 2}                # {str: i32} inferido
d: {str: i32} = {"a": 1}
d: ^{str: i32} = {}                 # mutable vacío
d = {}                              # vacío inferido
d: ^{str: i32}?

# Métodos
d.len()
d["key"]                            # crash si no existe
d.get("key")                        # Option<V>
d.has("key")
d.keys()                            # [K]
d.values()                          # [V]
d.items()                           # [(K, V)]
d["key"] = value
d.remove("key")                     # Option<V>
d.clear()

# Iteración
for key, val in d:
    print(key)
    print(val)
```

---

## Queue `queue<T>`

FIFO — primero en entrar, primero en salir.

```ky
q = queue{1, 2, 3}                  # queue<i32> inferido
q: ^queue<i32> = queue()
q: ^queue<i32> = queue{1, 2, 3}
q: ^queue<i32>?

q.push(4)                           # encolar
val = q.pop()                       # desencolar, Option<T>
front = q.peek()                    # Option<T> (sin sacar)
q.len()
q.clear()

q.to_list()
q.to_stack()
q.to_deque()
```

---

## Stack `stack<T>`

LIFO — último en entrar, primero en salir.

```ky
s = stack{1, 2, 3}
s: ^stack<i32> = stack()
s: ^stack<str> = stack{"a", "b"}

s.push(4)                           # apilar
val = s.pop()                       # desapilar, Option<T>
top = s.peek()                      # Option<T>
s.len()
s.clear()

s.to_list()
s.to_queue()
```

---

## Deque `deque<T>`

Doble extremo — push/pop en ambos lados.

```ky
dq = deque{1, 2, 3}
dq: ^deque<i32> = deque()

dq.push_front(0)
dq.push_back(4)
val = dq.pop_front()                # Option<T>
val = dq.pop_back()                 # Option<T>
front = dq.peek_front()
back = dq.peek_back()
dq.len()
dq.clear()

dq.to_list()
dq.to_queue()
dq.to_stack()
```

---

## Linked List `linked_list<T>`

```ky
ll = linked_list{1, 2, 3}
ll: ^linked_list<i32> = linked_list()

ll.push_front(0)
ll.push_back(4)
val = ll.pop_front()
val = ll.pop_back()
ll.insert(2, 99)
ll.remove(1)
ll.first()
ll.last()
ll.get(2)                           # Option<T>
ll.len()

ll.to_list()
```

---

## String `str`

```ky
s = "hello world"

s.len()
s[0]                                # 'h' como char
s.char_at(0)                        # Option<char>
s.contains("world")
s.starts_with("hello")
s.ends_with("world")
s.index_of("world")                  # Option<i32>

s += "!"                            # solo ^str
s.to_upper()
s.to_lower()
s.trim()
s.replace("world", "kyle")
s.substr(0, 5)                      # "hello"
s.split(" ")                        # ["hello", "world"]
s.to_list()                         # ['h','e','l',...]
```

---

## Tabla de ortogonalidad

| Tipo | `?` | `!` | `^` | `&` | `^&` |
|------|-----|-----|-----|-----|------|
| `i32` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `str` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `[T]` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `[T,N]` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `set<T>` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `{K:V}` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `queue<T>` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `stack<T>` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `deque<T>` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `linked_list<T>` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `(T, U)` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `&[T]` (slice) | ✅ | ✅ | ❌ | ❌ | ❌ |
