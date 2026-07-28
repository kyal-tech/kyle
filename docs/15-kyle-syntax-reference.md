# Kyle Syntax Reference — Para portar el compilador de Rust a Kyle

> **Propósito:** Referencia rápida y completa de la sintaxis de Kyle para usar cuando se porte el compilador (parser, type checker, MIR, codegen, runtime, CLI) de Rust a Kyle.
>
> **Cada ejemplo es código Kyle válido.** Si un ejemplo no compila, es un bug.

---

## 1. Variables

No hay `let`, `var`, `const`, `mut`. Se declaran con `nombre = valor`.

```kyle
name = "KYOS"              # Inmutable por defecto
count: ^i32 = 0            # ^T = mutable
count += 1                 # ++ no existe, usar += 1

name = "otro"              # ERROR: name es inmutable

msg: ^str = "hola"
msg = "chau"               # OK: msg es mutable

x: i32? = none              # T? = optional (none = null)
if x:
    print(x)                # safe unwrap automático en if

y: i32! = ok(42)            # T! = fallible (Result)
z = y!                      # ! propaga el error o unwrapea
```

### Tipos básicos

```kyle
# Primitivos
b: bool = true
i: i32 = 42
j: i64 = 9999999999
u: u8 = 255
f: f64 = 3.14
s: str = "hola"
byte: bytes = [0x48, 0x65, 0x6C, 0x6C, 0x6F]

# Casting
x = 42 as f64
y = 3.14 as i32  # trunca
```

### Strings

```kyle
s1 = "hola"
s2 = "mundo"
s3 = s1 + " " + s2        # Concatenación: "hola mundo"

# Interpolación
name = "Kyle"
msg = "Hola, {name}!"      # "Hola, Kyle!"
calc = "2+2 = {2 + 2}"     # "2+2 = 4"
nested = "_{"{name}"}"     # "_{"Kyle"}"

# Multi-line
text = """
linea 1
linea 2
"""

# Operaciones
len = s1.len()              # 4
upper = s1.upper()          # "HOLA"
lower = s1.lower()          # "hola"
chars = s1.chars()          # ['h', 'o', 'l', 'a']
contains = s1.contains("ol") # true
```

### Propiedad y préstamos

```kyle
# Move por defecto para tipos no-Copy
s1 = "hola"
s2 = s1          # s1 se mueve a s2
# print(s1)     # ERROR: s1 ya no es válido

# Copy para primitivos
a = 42
b = a            # a sigue siendo válido (i32 es Copy)
print(a)         # OK

# Borrow (&T) — prestamo inmutable
fn len(s: &str) i32:
    return s.len()

s = "hola"
print(len(&s))   # pasar prestamo
print(s)         # s sigue siendo válido

# Mutable borrow (^&T)
fn append(s: ^&str, suffix: str):
    s = s + suffix

msg: ^str = "hola"
append(&msg, " mundo")
print(msg)       # "hola mundo"
```

---

## 2. Funciones

```kyle
# Básica
fn greet(name: str):
    print("Hola, " + name)

# Con retorno
fn add(a: i32, b: i32) i32:
    return a + b

# Con múltiples retornos (tupla)
fn divide(a: i32, b: i32) (i32, i32):
    return (a / b, a % b)

q, r = divide(10, 3)     # destructuring

# Con valor por defecto
fn connect(host: str, port: i32 = 8080):
    print("Conectando a " + host + ":" + port.to_str())

connect("localhost")        # port=8080
connect("localhost", 3000)  # port=3000

# async
async fn fetch(url: str) str:
    return http.get(url)

# Parámetros: move, borrow, mutable borrow
fn take(s: str):           # toma ownership
    print(s)

fn view(s: &str):          # prestamo inmutable
    print(s)

fn update(s: ^&str):       # prestamo mutable
    s = s + "!"

# Function pointers
fn execute(callback: fn(i32) i32, val: i32) i32:
    return callback(val)

result = execute(fn(x): x * 2, 5)  # 10
```

### extern fn (FFI a C)

```kyle
@link "c"
extern fn malloc(size: i64) ptr
extern fn free(ptr)
extern fn write(fd: i32, buf: ptr, count: i64) i64
extern fn read(fd: i32, buf: ptr, count: i64) i64
extern fn open(path: &str, flags: i32, mode: i32) i32
extern fn close(fd: i32) i32

@link "-lcurl"
extern fn curl_easy_init() ptr
extern fn curl_easy_setopt(handle: ptr, option: i32, value: ...) i32

@link "-framework CoreFoundation"
extern fn CFStringCreateWithCString(alloc: ptr, cStr: &str, encoding: i32) ptr
```

### @link directives

```kyle
@link "c"                   # linkea libc
@link "-lm"                 # linkea libm
@link "-lpthread"           # linkea pthreads
@link "-lcurl"              # linkea libcurl
@link "-framework Cocoa"    # macOS framework
@link "-L/opt/homebrew/lib" # search path
```

---

## 3. Control Flow

```kyle
# if/elif/else
if x > 0:
    print("positivo")
elif x < 0:
    print("negativo")
else:
    print("cero")

# if como expresión
status = if x > 0: "ok" else: "error"

# Binding if (pattern binding)
if name = optional_value:
    print(name)  # name existe dentro del bloque

# while
while i < 10:
    i += 1

# Binding while (pattern binding)
while line = read_line():
    print(line)

# for sobre lista
for item in items:
    print(item)

# for con índice
for i, item in items:
    print(i.to_str() + ": " + item)

# for sobre rango
for i in 0..10:
    print(i)

# for-else (se ejecuta si NO hubo break)
for item in items:
    if item == target:
        print("encontrado")
        break
else:
    print("no encontrado")

# break/continue
for i in 0..100:
    if i % 2 == 0: continue
    if i > 50: break
    print(i)

# defer (se ejecuta al salir del scope)
fn read_file(path: &str) str:
    f = open(path)
    defer close(f)
    return read_all(f)   # close() se llama al salir

# guard (returns early si es None)
fn get_name(id: i32) str:
    user = find_user(id)
    guard user else: return "desconocido"
    return user.name
```

---

## 4. Pattern Matching (match)

```kyle
# match sobre entero
match x:
    0: print("cero")
    1: print("uno")
    _: print("otro")

# match como expresión
name = match x:
    0: "cero"
    1: "uno"
    _: "otro"

# match con enum
enum Color:
    Red(g: i32)
    Green
    Blue(b: i32, a: i32)

c = Color.Red(255)
match c:
    Color.Red(g): print("red: " + g.to_str())
    Color.Green: print("green")
    Color.Blue(b, a): print("blue")

# or-pattern
match x:
    0 | 1: print("cero o uno")
    2..5: print("entre 2 y 5")  # range pattern
    _: print("otro")

# match con guard
match x:
    n if n > 10: print("grande")
    n: print("chico")

# match sobre tipo
match value:
    is str: print("es string: " + value)
    is i32: print("es entero: " + value.to_str())
    _: print("otro tipo")

# match sobre optional
result = might_return_none()
match result:
    val: print(val)       # Some case
    none: print("nada")   # None case
```

---

## 5. Clases y Tipos

```kyle
# Clase final (no heredable)
final class Point:
    x: i32
    y: i32

p = Point(x: 10, y: 20)
p.x = 30   # ERROR: campos inmutables por defecto

# Clase con campos mutables
final class Counter:
    count: ^i32 = 0
    name: str

    fn increment(self):
        self.count += 1

    fn get_count(self) i32:
        return self.count

c = Counter(name: "test")
c.increment()
print(c.get_count())  # 1

# Clase heredable
class Animal:
    name: str

    fn speak(self) str:
        return "..."

# Herencia
class Dog :: Animal:
    fn speak(self) str:
        return "guau!"

# Clase abstracta
abstract class Shape:
    fn area(self) f64

class Circle :: Shape:
    radius: f64

    fn area(self) f64:
        return 3.14159 * radius * radius

# Enum con payload
enum Result:
    Ok(value: str)
    Error(code: i32, msg: str)

r = Result.Ok("exito")
match r:
    Result.Ok(v): print(v)
    Result.Error(c, m): print(c.to_str() + ": " + m)
```

### Genéricos

```kyle
# Clase genérica
final class Box<T>:
    value: T

    fn get(self) T:
        return self.value

b = Box(value: 42)
v = b.get()   # v es i32

# Función genérica
fn first<T>(items: &[T]) T:
    return items[0]

# Genérico con constraint
fn max<T: copy>(a: T, b: T) T:
    if a > b: return a
    return b

# Múltiples parámetros
final class Pair<A, B>:
    first: A
    second: B
```

---

## 6. Colecciones

```kyle
# Lista {T}
items: ^{i32} = {1, 2, 3}
items.push(4)
items.pop()
first = items[0]
items[0] = 99
len = items.len()
contains = items.contains(2)

# Dict {K: V}
scores: ^{str: i32} = {"ana": 100, "bob": 85}
scores.set("ana", 95)     # actualizar
if scores.has("carlos"):
    print(scores.get("carlos"))
scores.remove("bob")

# Set set<T>
ids: ^set<i32> = set()
ids.add(1)
ids.add(2)
if ids.contains(1):
    ids.remove(1)

# Array fijo [T, N]
arr: [i32, 3] = [10, 20, 30]
first = arr[0]

# Tupla
pair = (1, "hola")
a, b = pair   # destructuring

# Slice (vista prestada)
fn sum_slice(nums: &[i32]) i32:
    total = 0
    for n in nums:
        total += n
    return total

# Iterators
squares = items.map(fn(x): x * x).collect()
evens = items.filter(fn(x): x % 2 == 0).collect()
sum = items.fold(0, fn(acc, x): acc + x)
```

---

## 7. Punteros

```kyle
# ptr — raw pointer (sin ownership, sin borrow checking)
extern fn ky_ptr_read_i32(p: ptr) i32
extern fn ky_ptr_write_i32(p: ptr, val: i32)

fn read_word(addr: ptr) i32:
    return ky_ptr_read_i32(addr)

# box<T> — heap allocation con ownership único
final class Node:
    value: i32
    next: box<Node>?   # None si es el último

n = Node(value: 1, next: none)

# unsafe para operaciones con punteros
fn copy_memory(dest: ptr, src: ptr, len: i64):
    unsafe:
        for i in 0..len:
            byte = ky_ptr_read_i32(src + i)
            ky_ptr_write_i32(dest + i, byte)
```

---

## 8. Manejo de Errores

```kyle
# T! — función que puede fallar
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        return error("division by zero")
    return ok(a / b)

# ! — operador de propagación
fn calculate(a: i32, b: i32) i32!:
    result = divide(a, b)!   # si falla, propaga el error
    return ok(result * 2)

# T? — valor opcional
fn find_user(id: i32) User?:
    if id < 0: return none
    return User(id: id)

# Uso de optional
user = find_user(42)
match user:
    u: print(u.name)
    none: print("not found")

# ? — null-coalescing
name = user.name ?? "anonymous"
```

---

## 9. Módulos e Imports

```kyle
# Importar símbolo específico
use http.server.Router

# Importar múltiples
use json.{parse, stringify}

# Importar todo
use os

# Importar con alias
use views.home as home_page

# Import relativo
use ~utils.helper
use ~~models.User

# Import de package
use sqlite.connection.Connection

# Visibilidad
name = "KYOS"              # público (default)
_internal = "hidden"       # _ = protected (visible en módulo)
__private = "secret"       # __ = privado (solo este archivo)
```

---

## 10. Operator Overloading

```kyle
final class Vec2:
    x: f32
    y: f32

    fn op_add(this, other: Vec2) Vec2:
        return Vec2(x: this.x + other.x, y: this.y + other.y)

    fn op_sub(this, other: Vec2) Vec2:
        return Vec2(x: this.x - other.x, y: this.y - other.y)

    fn op_eq(this, other: Vec2) bool:
        return this.x == other.x and this.y == other.y

a = Vec2(x: 1, y: 2)
b = Vec2(x: 3, y: 4)
c = a + b           # llama a op_add
print(a == b)       # llama a op_eq
```

### Operadores sobrecargables

| Categoría | Operador | Método |
|-----------|----------|--------|
| Aritmético | `+` | `op_add` |
| | `-` | `op_sub` |
| | `*` | `op_mul` |
| | `/` | `op_div` |
| | `%` | `op_mod` |
| Comparación | `==` | `op_eq` |
| | `!=` | `op_not` (negación de op_eq) |
| | `<` `>` `<=` `>=` | `op_cmp` |
| Unario | `-` | `op_neg` |
| | `!` | `op_not` |
| Bitwise | `&` | `op_and` |
| | `\|` | `op_or` |
| | `^` | `op_xor` |
| | `<<` | `op_shl` |
| | `>>` | `op_shr` |
| Index | `[]` | `op_index` / `op_index_set` |

---

## 11. Compund Assignment

```kyle
count += 1      # count = count + 1
count -= 1      # count = count - 1
count *= 2      # count = count * 2
count /= 2      # count = count / 2
count %= 2      # count = count % 2
```

---

## 12. Async

```kyle
async fn fetch_data(url: &str) str:
    return http.get(url)

async fn main():
    result = await fetch_data("https://api.example.com")
    print(result)

# Channels
ch: chan<i32> = channel()

async fn producer(ch: chan<i32>):
    for i in 0..10:
        ch.send(i)

async fn consumer(ch: chan<i32>):
    while true:
        val = ch.recv()
        print(val)
```

---

## 13. Atajos de teclado del compilador

```bash
ky build file.ky              # Compilar a binario
ky build freestanding file.ky # Compilar con entry _start (sin wrapper)
ky build --release file.ky    # Compilar con optimizaciones
ky run file.ky                # Compilar y ejecutar
ky check file.ky              # Solo type-check (rápido)
ky fmt file.ky                # Formatear código
ky new project                # Crear nuevo proyecto
```

---

## 14. Errores comunes al portar

```kyle
# ERROR 1: Usar variable inmutable como mutable
name = "KYOS"
name = "otro"  # ERROR: name no es ^str

# ERROR 2: No marcar ^ en parámetros mutables
fn append(s: str):
    s = s + "!"  # ERROR: s no es mutable

# ERROR 3: No usar & para prestamo
fn view(s: &str):
    print(s)

s = "hola"
view(s)   # ERROR: debe ser view(&s)

# ERROR 4: No hay ++
count++   # ERROR
count += 1 # OK

# ERROR 5: No hay pass
if true:
    pass    # ERROR
    none    # OK (expresión nula)
    true    # OK (expresión que no hace nada)

# ERROR 6: No hay let/var/const/mut
let x = 5  # ERROR
x: ^i32 = 5 # OK

# ERROR 7: No hay goto
loop:
    goto loop   # ERROR
while true:
    none        # OK
```
