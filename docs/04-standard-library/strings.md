# strings — Utilidadis de String

> Module de string manipulation y type `str_builder`.
> Import: `use strings.{str, str_builder}`

## str: methods del type string

Los strings en Kyle are inmutables, heap-allocados y **Move semantics**.

```ky
use strings.str

s: str = " Hello World "
s2: str = s.trim()
s3: str = s.to_upper()
s4: str = s3.replace("HELLO", "HI")
```

### Methods de str

| Method | Firma | Description | Example |
|--------|-------|-------------|---------|
| `len` | `fn() i32` | Largo del string | `s.len()` |
| `contains` | `fn(sub: str) bool` | `true` si conhas substring | `s.contains("lo")` |
| `starts_with` | `fn(prefix: str) bool` | `true` si empieza with | `s.starts_with("He")` |
| `ends_with` | `fn(suffix: str) bool` | `true` si termina with | `s.ends_with("ld")` |
| `to_upper` | `fn() str` | Uppercase | `s.to_upper()` |
| `to_lower` | `fn() str` | Lowercase | `s.to_lower()` |
| `trim` | `fn() str` | Sin espacios extremos | `s.trim()` |
| `replace` | `fn(from: str, to: str) str` | Reemplazar substring | `s.replace("a", "b")` |
| `char_at` | `fn(idx: i32) i8` | Character en position | `s.char_at(0)` |
| `substr` | `fn(start: i32, count: i32) str` | Substring | `s.substr(0, 5)` |

### Functions standalone

```ky
n: i32 = len(s) # largo del string
es_digito: bool = str.is_digit('5')
es_alpha: bool = str.is_alpha('a')
es_alnum: bool = str.is_alnum('x')
es_space: bool = str.is_whitespace(' ')
es_mayus: bool = str.is_upper('A')
es_minus: bool = str.is_lower('a')
```

| Function | Firma | Description |
|---------|-------|-------------|
| `len(s)` | `fn(s: str) i32` | Largo del string |
| `str.is_digit(c)` | `fn(c: i8) bool` | `true` si is digito |
| `str.is_alpha(c)` | `fn(c: i8) bool` | `true` si is letra |
| `str.is_alnum(c)` | `fn(c: i8) bool` | `true` si is alfanumerico |
| `str.is_whitespace(c)` | `fn(c: i8) bool` | `true` si is espacio |
| `str.is_upper(c)` | `fn(c: i8) bool` | `true` si is mayuscula |
| `str.is_lower(c)` | `fn(c: i8) bool` | `true` si is minuscula |

### Chaining

```ky
result: str = " Hello World ".trim().to_upper().substr(0, 5)
println(result) # "HELLO"
```

## str_builder: construction eficiente de strings

`str_builder` is un buffer mutable for concatenacion eficiente.

```ky
sb = str_builder.new(100)
str_builder.append(sb, "hello ")
str_builder.append(sb, "world")
result = str_builder.to_str(sb)
println(result)   # "hello world"
str_builder.free(sb)
```

### API

| Function | Firma | Description |
|----------|-------|-------------|
| `str_builder.new` | `fn(capacity: i64) str` | Create new builder |
| `str_builder.append` | `fn(sb: str, s: str)` | Append string |
| `str_builder.to_str` | `fn(sb: str) str` | Build result string |
| `str_builder.free` | `fn(sb: str)` | Free memory |

### Performance

`append()` redimensiona with estrategia de duplicacion (2× capacidad). Comparado
with `s = s + "x"` (alloc + copy en cada concat), is **~380× more rapido**.
