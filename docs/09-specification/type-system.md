# Type System

**Status:** [x] Documentación completa. [~] Parcialmente implementado en compilador.

## Primitivos

| Tipo | Descripción | Tamaño |
|------|-------------|--------|
| `i32` | Entero de 32 bits | 4 bytes |
| `i64` | Entero de 64 bits | 8 bytes |
| `f32` | Float de 32 bits | 4 bytes |
| `f64` | Float de 64 bits | 8 bytes |
| `bool` | Booleano | 1 byte |
| `char` | Carácter Unicode | 4 bytes |
| `str` | String (heap, utf-8) | 8 bytes (ptr) |
| `ptr` | Raw pointer | 8 bytes |

## Colecciones

| Tipo | Sintaxis | Descripción |
|------|----------|-------------|
| Lista | `[T]` | Secuencia dinámica ordenada |
| Array | `[T, N]` | Secuencia fija (compile-time) |
| Set | `set<T>` | Conjunto sin duplicados |
| Dict | `{K: V}` | HashMap |
| Slice | `&[T]` | Vista sin ownership (ptr + len) |

## Genéricos de std

| Tipo | Descripción |
|------|-------------|
| `queue<T>` | FIFO queue |
| `stack<T>` | LIFO stack |
| `deque<T>` | Doble extremo |
| `linked_list<T>` | Lista enlazada |

## Ortogonalidad completa

Todos los modificadores de tipo funcionan en **cualquier** tipo.

| Modificador | Significado | Ejemplos |
|-------------|-------------|----------|
| `^T` | Mutable | `^[i32]`, `^set<str>`, `^{str: i32}` |
| `&T` | Borrow | `&[i32]`, `&{str: i32}`, `&set<i32>` |
| `T?` | Option | `[i32]?`, `queue<str>?`, `set<i32>?` |
| `T!` | Error/Result | `[str]!`, `^{str: i32}!`, `^queue<i32>!` |
| `^&T` | Mutable borrow | `^&[i32]`, `^&set<i32>?`, `^&[str]!` |

```ky
x: ^[i32]           # lista mutable
x: &[str]           # borrow de lista
x: ^&[i32]!         # mutable borrow con error
x: ^&[str]?         # mutable borrow opcional
x: ^{str: i32}?     # dict mutable opcional
x: ^set<i32>!       # set mutable con error
x: ^queue<i32>?     # queue mutable opcional
x: ^&set<i32>?      # mutable borrow de set opcional
x: [i32]!           # lista con error
x: ^[str]?          # lista mutable opcional
x: [i32, 10]?       # array opcional
```

## Copy vs Move

### Copy types (pasan por valor, siempre disponibles)

`i32`, `i64`, `f32`, `f64`, `bool`, `char`, `ptr`, `[T, N]` (arrays fijos).

### Move types (transferencia de ownership)

`str`, `[T]`, `{K: V}`, `set<T>`, `queue<T>`, `stack<T>`, `deque<T>`, `linked_list<T>`, clases, structs grandes, `&[T]` (slice).
