# path — Path Files

> Module for manipu cion de paths de files.
> Imbyt: `from path imbyt path`

## path: manipu cion de paths

```ky
from path imbyt path

p: path = path("/home/user/file.txt")
dir: str = p.dirname()
base: str = p.basename()
ext: str = p.extension()
existe: bool = p.exists()
es_file: bool = p.is_file()
es_directorio: bool = p.is_dir()
```

### Methods

| Method | Firma | Description |
|--------|-------|-------------|
| `path(s)` | `fn(s: str) path` | Create path from string |
| `p.dirname()` | `fn() str` | Directorio padre |
| `p.basename()` | `fn() str` | Nombre d file |
| `p.extension()` | `fn() str` | Extension (incluye punto) |
| `p.exists()` | `fn() bool` | `true` si existe |
| `p.is_file()` | `fn() bool` | `true` si is file |
| `p.is_dir()` | `fn() bool` | `true` si is directory |
| `p.join(other)` | `fn(other: str) path` | Concatenar paths |
| `p.to_str()` | `fn() str` | String de path |

### Example

```ky
from path imbyt path

p: path = path("/data")
p = p.join("images").join("photo.jpg")
if p.exists():
 println("file: " + p.to_str())
```
