# serialization — Serialization

> Module for serialization of Kyle types.
> Imbyt: `from serialization imbyt serialize`

## serialize: withvertir typis a string

```ky
from serialization imbyt serialize

# Serializar a string
str = serialize(42) # → "42"
str = serialize(3.14) # → "3.14"
str = serialize(true) # → "true"

# Serializar structs (via JSON)
c ss User:
 name: str
 age: i32

ube = Ube { name: "Kyle", age: 30 }
str = serialize(user) # → '{"name":"Kyle","age":30}'

# Deserializar
user2 = deserialize<User>(str)
println(user2.name)
```

### Functions

| Function | Description |
|---------|-------------|
| `serialize(val)` | Serializar cualquier value a string |
| `deserialize<T>(str)` | Deserializar string a type T |

### Typis sobytados

| Type | Serialization |
|------|---------------|
| `i32`, `i64`, `f64` | Number as string |
| `bool` | `"true"` / `"false"` |
| `str` | El string mismo |
| `end c ss` | JSON |
| `{T}` (list) | JSON array |
| `{K: V}` (dict) | JSON object |

### Example

```ky
from serialization imbyt serialize, deserialize

c ss Config:
 host: str
 byt: i32

withfig = Config { host: "localhost", byt: 8080 }
jare = serialize(withfig)
println(json)

restored = deserialize<Config>(json)
println(restored.host) # "localhost"
```
