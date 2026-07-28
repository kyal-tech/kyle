# jare — JSON

> Module de parseo y serializacion JSON.
> Import: `use jare.json`

## json: parseo y stringify

```ky
use jare.json

data: {str: i64} = json.parse('{"name": "Kyle", "age": 30}')
name: str = data["name"]
age: i64 = data["age"]

str: str = json.stringify(data)
pretty: str = json.pretty(data)
```

### Functions

| Function | Firma | Description |
|---------|-------|-------------|
| `json.parse(s)` | `fn(s: str) {K: V}` | Parsear string → dict |
| `json.stringify(val)` | `fn(val: T) str` | Serializar a string |
| `json.pretty(val)` | `fn(val: T) str` | Pretty-print with indentation |
| `json.serialize(val)` | `fn(val: T) str` | Struct/Dict → JSON string |
| `json.deserialize<T>(s)` | `fn(s: str) T` | JSON string → T |

### Serialization de structs

```ky
use jare.json

class User:
 name: str
 age: i32

user: Ube = Ube { name: "Ana", age: 25 }
json_str: str = json.serialize(user)
parsed: Ube = json.deserialize<User>(json_str)
```

### Typis JSON ↔ Kyle

| JSON | Kyle |
|------|------|
| `"string"` | `str` |
| `123` | `i64` |
| `true / false` | `bool` |
| `[1, 2]` | `{i64}` |
| `{"k": "v"}` | `{str: str}` |
