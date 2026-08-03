# str — String utilities

> String manipulation methods and utilities.
> Import: `use std.str`

## Basic methods

These methods are available on all `str` values directly.

```ky
s: str = " Hello World "

trimmed: str = s.trim()           # "Hello World"
upper: str = s.to_upper()         # " HELLO WORLD "
lower: str = s.to_lower()         # " hello world "
length: i32 = s.len()             # 14
```

## Searching

```ky
s: str = "hello world"

s.contains("world")    # true
s.starts_with("hel")   # true
s.ends_with("rld")     # true
index: i32 = s.find("world")  # 6
```

## Slicing

```ky
s: str = "hello world"

first: str = s.substr(0, 5)     # "hello"
chars: [i8] = s.chars()         # ['h', 'e', 'l', 'l', 'o', ' ', 'w', ...]
c: i8 = s.char_at(0)            # 'h' (as i8 byte value)
```

## Transformation

```ky
s: str = "hello world"

replaced: str = s.replace("world", "kyle")  # "hello kyle"
parts: [str] = s.split(" ")                  # ["hello", "world"]
joined: str = parts.join(", ")               # "hello, world"
```

## String builder

For efficient concatenation in loops, use `str_builder`.

```ky
sb: str = str_builder.new(100)
str_builder.append(sb, "hello ")
str_builder.append(sb, "world")
result: str = str_builder.to_str(sb)
str_builder.free(sb)
println(result)   # "hello world"
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `.len()` | `fn() i32` | String length in bytes |
| `.contains(sub)` | `fn(sub: &str) bool` | True if contains substring |
| `.starts_with(prefix)` | `fn(prefix: &str) bool` | True if starts with prefix |
| `.ends_with(suffix)` | `fn(suffix: &str) bool` | True if ends with suffix |
| `.find(sub)` | `fn(sub: &str) i32` | Index of first occurrence, -1 if not found |
| `.to_upper()` | `fn() str` | Uppercase |
| `.to_lower()` | `fn() str` | Lowercase |
| `.trim()` | `fn() str` | Strip leading and trailing whitespace |
| `.trim_start()` | `fn() str` | Strip leading whitespace |
| `.trim_end()` | `fn() str` | Strip trailing whitespace |
| `.replace(from, to)` | `fn(from: &str, to: &str) str` | Replace all occurrences |
| `.split(sep)` | `fn(sep: &str) [str]` | Split into list |
| `.join(parts)` | `fn(parts: &[str]) str` | Join list with separator |
| `.char_at(idx)` | `fn(idx: i32) i8` | Character at index as byte |
| `.substr(start, count)` | `fn(start: i32, count: i32) str` | Extract substring |
| `.chars()` | `fn() [i8]` | All characters as byte list |
| `.trim()` | `fn() str` | Strip whitespace |

## str_builder

| Function | Signature | Description |
|----------|-----------|-------------|
| `str_builder.new(capacity)` | `fn(capacity: i64) str` | Create new builder |
| `str_builder.append(sb, s)` | `fn(sb: &str, s: &str)` | Append string |
| `str_builder.to_str(sb)` | `fn(sb: &str) str` | Build result string |
| `str_builder.free(sb)` | `fn(sb: &str)` | Free memory |

## Performance

`append()` grows with doubling strategy (2x capacity). Compared to `s = s + "x"`
which allocates and copies on every concatenation, `str_builder` is ~380x faster
for large strings.

## Example

```ky
use std.str

csv_line: str = "  hello, world, kyle  "
parts: [str] = csv_line.trim().split(",")
for part in parts:
    println(part.trim())

# Build a string efficiently
sb: str = str_builder.new(50)
for i in 0..100:
    str_builder.append(sb, i.to_str())
    str_builder.append(sb, ", ")
result: str = str_builder.to_str(sb)
str_builder.free(sb)
```
