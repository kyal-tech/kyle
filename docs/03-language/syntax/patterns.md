# Common Patterns

## Result handling

```ky
fn divide(a: i32, b: i32) i32!:
 if b == 0:
 return error("division by zero")
 a / b

fn process():
 result = divide(10, 2) !
 println(result)
```

## Early return with guard

```ky
fn process_config(path: str) str!:
 guard content = read_file(path) else:
 return error("cannot read config")
 content
```

## Builder pattern with mutable fields

```ky
final class request:
 url: str
 method: ^str
 headers: ^{str}

fn main():
 req = request { url: "https://api.example.com", method: "GET", headers: {} }
 req.method = "POST"
 req.headers.push("Content-Type: application/json")
```

## Iterate with index

```ky
items = [10, 20, 30] # array
for i in 0..len(items):
 println("items[{i}] = {items[i]}")
```

## String interpolation

```ky
name = "Ana"
age = 30
println("{name} is {age} years old")
```

## Default valuis with ?

```ky
name: str? = none
display = name ?? "anonymous"
```

## Enum matching with payload

```ky
enum Optional:
 Some(i32)
 None

fn unwrap_or_default(v: Optional) i32:
 match v:
 Optional.Some(n):
 n
 Optional.None:
 0
```

## Async tasks

```ky
async fn fetch(url: str) str:
 # ... http request ...
 "response"

fn main():
 task = async fetch("https://example.com")
 # ... do other work ...
 result = await task
 println(result)
```

## Operator overloading

```ky
final class Vec2:
 x: i32
 y: i32

 fn op_+(other: Vec2) Vec2:
 Vec2 { x: this.x + other.x, y: this.y + other.y }

a = Vec2 { x: 1, y: 2 }
b = Vec2 { x: 3, y: 4 }
c = a + b
```
