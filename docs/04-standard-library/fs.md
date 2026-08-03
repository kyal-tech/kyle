# fs — File system operations

> Read, write, and manage files and directories.
> Import: `use std.fs`

## Reading files

```ky
use std.fs

content: str = fs.read("data.txt")!
lines: [str] = content.split("\n")
println("lines: " + lines.len().to_str())
```

## Writing files

```ky
fs.write("output.txt", "hello world")!
fs.append("log.txt", "new log entry")!
```

## File information

```ky
if fs.exists("data.txt"):
    size: i64 = fs.size("data.txt")
    println("size: " + size.to_str() + " bytes")
```

## Directory operations

```ky
fs.create_dir("data")!
fs.create_dir_all("data/images/icons")!  # creates parent dirs too

items: [str] = fs.list_dir("data")!
for item in items:
    println(item)

fs.remove_dir("data/temp")!
fs.remove_dir_all("data/temp")!   # recursive
```

## Copy, move, remove

```ky
fs.copy("source.txt", "backup.txt")!
fs.rename("old.txt", "new.txt")!
fs.remove("temp.txt")!
```

## Binary files

```ky
data: bytes = fs.read_bytes("image.png")!
fs.write_bytes("copy.png", data)!
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `read(path)` | `fn(path: &str) str!` | Read entire file as string |
| `read_bytes(path)` | `fn(path: &str) bytes!` | Read entire file as bytes |
| `write(path, content)` | `fn(path: &str, content: &str)!` | Write string to file |
| `write_bytes(path, content)` | `fn(path: &str, content: &bytes)!` | Write bytes to file |
| `append(path, content)` | `fn(path: &str, content: &str)!` | Append string to file |
| `exists(path)` | `fn(path: &str) bool` | Check if path exists |
| `size(path)` | `fn(path: &str) i64!` | File size in bytes |
| `copy(from, to)` | `fn(from: &str, to: &str)!` | Copy file |
| `rename(from, to)` | `fn(from: &str, to: &str)!` | Rename or move file |
| `remove(path)` | `fn(path: &str)!` | Delete file |
| `create_dir(path)` | `fn(path: &str)!` | Create single directory |
| `create_dir_all(path)` | `fn(path: &str)!` | Create directories recursively |
| `remove_dir(path)` | `fn(path: &str)!` | Remove empty directory |
| `remove_dir_all(path)` | `fn(path: &str)!` | Remove directory and contents |
| `list_dir(path)` | `fn(path: &str) [str]!` | List directory contents |

## Example

```ky
use std.fs

fn backup_config():
    if not fs.exists("config.json"):
        println("config.json not found")
        return
    content: str = fs.read("config.json")!
    fs.write("config.json.bak", content)!
    println("backup created")
```
