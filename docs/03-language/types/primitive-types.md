# Types

> **Leyenda:** `[x]` = implemented y funcional. `[x]` = All types implemented and tested.
> La syntax mostrada is syntax FINAL de Kyle. Lo que is `[ ]` aun no compila.

---

## Copy vs Move semantics [x]

| Semantics | Typis |
|-----------|-------|
| **Copy** (automatic en `y = x`) | `i8..u64`, `f32..f64`, `bool`, `char`, `ptr` |
| **Move** (ownership transfer en `y = x`) | `str`, `{T}`, `{K:V}`, `[T, N]`, classes, structs, enums |

```ky
x = 42; y = x # ambos vivos (Copy)
s = "hola"; t = s # s invalido (Move)
t = s.clone() # ambos vivos (copia explicita)
```

Ver `ownership.md` for detail completo.

---

## Primitive typis [x]

| Type | Copy? | Size | Description |
|------|-------|------|-------------|
| `i8` | ✅ | 1 | Signed 8-bit |
| `i16` | ✅ | 2 | Signed 16-bit |
| `i32` | ✅ | 4 | Signed 32-bit (default) |
| `i64` | ✅ | 8 | Signed 64-bit |
| `u8` | ✅ | 1 | Unsigned 8-bit |
| `u16` | ✅ | 2 | Unsigned 16-bit |
| `u32` | ✅ | 4 | Unsigned 32-bit |
| `u64` | ✅ | 8 | Unsigned 64-bit |
| `f32` | ✅ | 4 | 32-bit float |
| `f64` | ✅ | 8 | 64-bit float (default) |
| `bool` | ✅ | 1 | `true` or `false` |
| `char` | ✅ | 4 | Unicode code point |
| `str` | 🚫 | ptr | Heap-allocated string |
| `ptr` | ✅ | 8 | Raw pointer (FFI/unsafe) |
| `void` | — | 0 | No value (return only) |
| `never` | — | 0 | Diverging functions |

```ky
x = 42 # i32
x: i64 = 42 # i64 explicito
x = 3.14 # f64
b = true # bool
c = 'a' # char
s = "hello" # str
p = 0 as ptr # ptr
```

---

## Compound types

### Array: `[T, N]` [x]

Stack array, size fijo en compile-time. GEP directo, cero runtime calls.
Soporta multidimensional via anidamiento `[[T, N], M]`.

```ky
# Unidimensional
a = [1, 2, 3] # → [i32, 3]
b = [0; 100] # → [i32, 100], repetido
c = [1 as i64; 10000] # → [i64, 10000]
first = a[0] # GEP + load
a[0] = 99 # GEP + store

# Multidimensional: [[T, N], M]
matriz: [[i32, 4], 3] = [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]]
x = matriz[1][2] # → 7
```

### List: `{T}` [x]

Heap list dynamic.

```ky
v = {1, 2, 3} # → {i32}
v.push(4)
v.reserve(100) # pre-asigna capacidad
x = v[0] # list_get (runtime)
v[0] = 99 # list_set (runtime)
v.pop() # list_pop (runtime)
v.len() # list_len (runtime)
```

### Dict: `{K: V}` [x]

Heap dictionary (hash map).

```ky
d = {"name": "Kyle", "age": 30} # → {str: i32}
d["city"] = "NYC"
name = d["name"]
d.len()
```

### Tuple: `(T1, T2, ...)` [x]

Fixed-size heterogeneous.

```ky
t = (1, "hello", 3.14) # → (i32, str, f64)
x = t.0 # → 1
y = t.1 # → "hello"
```

### set: `set<T>` [x]

Hash set. Construction via `set{...}` o constructor.

```ky
s: set<i32> = set{1, 2, 3} # set literal
s.add(4)
s.contains(1) # → true
s.remove(1)
for val in s:
 println(val.to_str())
```

### Queue via list [x]

No there is type `Queue<T>` dedicado. Usar `{T}` with `.push()` / `.pop_first()`:

```ky
q = list_new()
q.push(10) # enqueue
q.push(20)
val = q.pop_first() # dequeue → 10 (FIFO)
q.len()
```

### Stack via list [x]

No there is type `Stack<T>` dedicado. Usar `{T}` with `.push()` / `.pop()` — ya funciona:

```ky
st = list_new()
st.push(10) # push
st.push(20)
val = st.pop() # → 20 (LIFO)
st.len()
```

> **Decision de diseno:** Stack y Queue no have typis dedicados porque `{T}` ya supports
> todas operacionis necesarias with methods `.push()`, `.pop()`, `.pop_first()`.
> Esto sigue enfoque de Go y JavaScript, where arrays/lists cubren ambos roles.

### Slice: `&[T]` [x]

> Vista de secuencia contigua. Un fat pointer (ptr + len), Copy semantics.

```ky
arr: [i32, 5] = [10, 20, 30, 40, 50]
s: &[i32] = arr[1..4]    # slice: [20, 30, 40]
println(s.len().to_str()) # → 3
println(s[0].to_str())    # → 20
s2: &[i32] = arr[..]      # full slice: [10, 20, 30, 40, 50]
```

**Copy semantics:** `&[T]` es Copy → `s2 = s1` deja ambos vivos.

**Creación:** `&arr[range]`:
| Expresión | Resultado |
|-----------|-----------|
| `arr[0..n]` | Slice de arr[0] a arr[n-1] |
| `arr[..]` | Slice completo |
| `arr[i..]` | Desde i hasta el final |
| `arr[..j]` | Desde 0 hasta j-1 |

**Métodos:** `.len()`, indexación `[i]`.

---

## Optional / Fallible

### Optional: `T?` [x]

```ky
name: str? = None
if value = get_name(): # pattern matching
 println(value)
```

### Fallible: `T!` [x]

```ky
fn divide(a: i32, b: i32) i32!:
 if b == 0:
 return error("division by zero")
 a / b

result = divide(10, 2)
match result:
 ok(v): println(v)
 error(e): println(e)
```

---

## Ownership types

### Mutable: `^T` [x]

```ky
x: ^i32 = 0 # mutable
x = x + 1 # reallocation permitida
```

### Borrow: `&T` [x]

```ky
fn read(s: &str): # parameter: borrow
 println(s)

fn main():
 name = "Kyle"
 read(&name) # name prestado
 println(name) # ✅ name sigue vivo
```

### Mutable borrow: `^&T` [x]

```ky
fn fill(buf: ^&str):
 buf = "data"

fn main():
 buf: ^str = ""
 fill(^&buf)
 println(buf) # "data"
```

### box: `box<T>` [x]

Heap allocation explicita. Move semantics (ky_free al salir de scope).

```ky
b: box<i32> = box(42)
println(b.to_str()) # → addr del heap (ptr value)
```

### rc: `rc<T>` [x]

Reference counting (single-thread).

```ky
r: rc<str> = rc("hello")
r2 = r.clone() # incrementa refcount
println(*r) # deref
```

### arc: `arc<T>` [x]

Atomic reference counting (multi-thread).

```ky
a: arc<i64> = arc(0)
```

---

## Concurrency types

### async fn / await [x]

```ky
async fn fetch(url: &str) str:
 "response"

fn main():
 task = async: fetch("https://...")
 result = await task
```

### future: `future<T>` [ ]

```ky
task: future<str> = async:
 "result"
val = await task
```

### channel: `channel<T>` [x]

```ky
ch: channel<i32> = channel(16) # buffer 16
ch.send(42)
val = ch.recv()
ch.len()
ch.close()
```

### select [ ]

```ky
select:
 &msg -> ch1:
 println("got: " + msg)
 &msg -> ch2:
 println("got: " + msg)
 after 1s:
 println("timeout")
```

### mutex: `mutex<T>` [x]

```ky
m: mutex<i32> = mutex(0)
lock(m):
 *val += 1 # operation segura
```

### atomic: `atomic_i64` / `atomic_bool` [x]

```ky
counter: atomic_i64 = atomic_i64(0)
counter.fetch_add(1)
counter.load() # → 1

flag: atomic_bool = atomic_bool(false)
flag.store(true)
flag.load() # → true
```

### iterator [x]

```ky
iter = list.iter()
doubled = iter.map(fn(x): x * 2)
filtered = iter.filter(fn(x): x > 5)
result = doubled.collect() # → {i32}
```

---

## Specialized typis (NATIVOS)

> Todos estos are nativos de Kyle (no requieren `use X.Y`).
> Estan disponiblis globalmente as typis built-in del language.

### date_time [x]

```ky
dt = date_time.now()
dt = date_time.parse("2024-01-01T00:00:00")
dt = date_time.from_ymdhms(2024, 1, 1, 0, 0, 0)
year = dt.year()
month = dt.month()
day = dt.day()
hour = dt.hour()
minute = dt.minute()
second = dt.second()
dt2 = dt.add_days(7)
dt3 = dt.add_hours(3)
diff = dt.diff(dt2)
```

### duration [x]

```ky
d = duration.from_secs(60)
d = duration.from_millis(1000)
d = duration.from_hours(1)
d = duration.from_days(7)
d.to_str() # → "1h 0m 0s"
```

### date [x]

```ky
d = date.today()
d = date.from_ymd(2024, 1, 1)
d = date.parse("2024-01-01")
year = d.year()
month = d.month()
weekday = d.weekday()
d2 = d.add_days(7)
```

### time [x]

```ky
t = time.now()
t = time.from_hms(12, 30, 0)
t = time.parse("12:30:00")
hour = t.hour()
minute = t.minute()
second = t.second()
```

### bytis [x]

```ky
b = bytes.new(1024)
b = bytes.from_hex("deadbeef")
b = bytes.from_base64("SGVsbG8=")
b.len()
val = b.get(0) # → byte
b.set(0, 255)
hex = b.hex()
b64 = b.to_base64()
```

### decimal [x]

```ky
d = decimal.from_str("3.14")
d = decimal.from_i64(314, 2) # 3.14
d.round(1) # → 3.1
d.truncate() # → 3
d.to_str() # → "3.14"
```

### uuid [x]

```ky
id = uuid.v4()
id = uuid.parse("550e8400-e29b-41d4-a716-446655440000")
id.to_str()
```

### url [x]

```ky
u = url.parse("https://user:pass@host:8080/path?q=1#frag")
u.scheme() # → "https"
u.host() # → "host"
u.port() # → 8080
u.path() # → "/path"
u.query() # → "q=1"
```

### regex [x]

```ky
re = regex("[0-9]+")
re.is_match("abc123") # → true
m = re.find("abc123") # → "123"
result = re.replace("abc123", "X") # → "abcX"
```

### env [x]

```ky
val = env("PATH")
env("MY_VAR", "value") # set
```

### jare [x]

```ky
j = json.parse('{"name": "Kyle", "age": 30}')
name = j["name"] # access field
j["city"] = "NYC" # set field
str = j.stringify() # serialize
str = j.pretty() # pretty-print
```

### file [x]

```ky
f = file.open("/tmp/test.txt", "w")
f.write("hello world")
f.close()

f = file.open("/tmp/test.txt", "r")
content = f.read() # → str
f.close()
f.exists() # → true
```

### socket [x]

```ky
server = socket.tcp_listen(8080)
client = server.accept()
data = client.read(1024)
client.write("HTTP/1.1 200 OK\r\n\r\n")
client.close()
server.close()

# client mode
conn = socket.tcp_connect("example.com", 80)
conn.write("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
resp = conn.read(4096)
conn.close()
```

### path [x]

```ky
p = path("/home/user/file.txt")
p.dirname() # → "/home/user"
p.basename() # → "file.txt"
p.extension() # → ".txt"
p.exists() # → true
p.is_file() # → true
p.is_dir() # → false
p.join("subdir") # → "/home/user/file.txt/subdir"
```

### big_int [x]

```ky
n = big_int("123456789012345678901234567890")
n2 = big_int.from_i64(42)
n3 = n + n2
n4 = n * n2
n.to_str()
```

### str_builder [x]

```ky
sb = str_builder(50000)
sb.append("x")
sb.append("hello")
result = sb.to_str()
```

---

## Status summary

| Category | [x] Completo | [ ] Designed | ❌ No planned |
|-----------|:-----------:|:-----------:|:-------------:|
| Primitivis | 14 | 1 (never) | 0 |
| Compounds | 7 (tuples, sets, queue, stack) | 1 | 0 |
| Ownership | 6 (rc, arc) | 1 (weak) | 0 |
| Concurrency | 5 (async/await, channel, mutex, atomic, iterator) | 2 (future, select) | 0 |
| Specialized | 14 | 0 | 0 |
| **Total** | **46** | **4** | **0** |
