# regex — Expresionis Regulares

> Module de expresionis regulares.
> Import: `use regex`

## regex: search and replace

```ky
use regex

re = regex("[0-9]+")
println(re.is_match("abc123")) # true
println(re.find("abc123")) # "123"
println(re.replace("abc123", "X")) # "abcX"

# Con grupos
re2 = regex("(\\w+)@(\\w+)")
match = re2.find("user@host")
println(match) # "user@host"
```

### Methods

| Method | Description |
|--------|-------------|
| `regex(pattern)` | Compile regular expression |
| `re.is_match(s)` | `true` si string matchea |
| `re.find(s)` | Primera ocurrencia |
| `re.find_all(s)` | Todas ocurrencias (list) |
| `re.replace(s, replacement)` | Reemplazar ocurrencias |
| `re.split(s)` | Split string by pattern |

### Example

```ky
use regex

# Validar email
email_re = regex("^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$")
if email_re.is_match("user@example.com"):
 println("valid email")

# Extract numbers
num_re = regex("[0-9]+")
text = "precio: 42 unidades: 7"
nums = num_re.find_all(text)
println(nums) # {"42", "7"}
```
