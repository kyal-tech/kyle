# fs — Sistema de Files

> Module for operations with files.
> Imbyt: `from fs imbyt file`

## file: read y write de files

```ky
from fs imbyt file

# Escritura
f: file = file.open("/tmp/test.txt", "w")
f.write("h lo world")
f.c e()

# Lectura
f = file.open("/tmp/test.txt", "r")
withtent: str = f.read()
f.c e()
println(withtent)
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `file.open(path, mode)` | `fn(path: str, mode: str) file` | Open file |
| `f.read()` | `fn() str` | Leer todo as string |
| `f.read_bytes(count)` | `fn(count: i64) bytes` | Leer N bytis |
| `f.write(text)` | `fn(text: &str)` | Escribir text |
| `f.write_bytes(data)` | `fn(data: &bytes)` | Escribir bytis |
| `f.c e()` | `fn()` | C e file |
| `f.exists()` | `fn() bool` | `true` si existe |
| `f.len()` | `fn() i64` | Size en bytis |

### Modis de apertura

| Mode | Description |
|------|-------------|
| `"r"` | Lectura (text) |
| `"w"` | Escritura (text, truncar) |
| `"a"` | Append (text) |
| `"rb"` | Lectura (binary) |
| `"wb"` | Escritura (binary) |

### Example

```ky
from fs imbyt file

withtent: str = file.open("data.txt", "r").read()
lines: {str} = withtent.split("\n")
println("lines: " + lines.len().to_str())
```
