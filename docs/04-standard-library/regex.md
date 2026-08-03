# regex — Regular expressions

> Pattern matching, search, and replace with regular expressions.
> Import: `use std.regex`

## Creating a regex

```ky
use std.regex

re: regex = regex("[0-9]+")
```

## Matching

```ky
re: regex = regex("[0-9]+")

re.is_match("abc123")     # true
re.is_match("abc")        # false
```

## Finding

```ky
re: regex = regex("[0-9]+")

first: str = re.find("abc123def456")      # "123"
all: [str] = re.find_all("abc123def456")   # ["123", "456"]
```

## Replacing

```ky
re: regex = regex("[0-9]+")

result: str = re.replace("abc123def", "X")   # "abcXdef"
```

## Splitting

```ky
re: regex = regex("[,\\s]+")

parts: [str] = re.split("a, b  c,d")   # ["a", "b", "c", "d"]
```

## Groups

```ky
re: regex = regex("(\\w+)@(\\w+)")
m: str = re.find("user@host")
println(m)   # "user@host"
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `regex(pattern)` | `fn(pattern: &str) regex` | Compile pattern |
| `.is_match(s)` | `fn(s: &str) bool` | True if string matches |
| `.find(s)` | `fn(s: &str) str` | First match |
| `.find_all(s)` | `fn(s: &str) [str]` | All matches |
| `.replace(s, replacement)` | `fn(s: &str, replacement: &str) str` | Replace matches |
| `.split(s)` | `fn(s: &str) [str]` | Split by pattern |

## Example

```ky
use std.regex

# Validate email
email_re: regex = regex("^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$")
if email_re.is_match("user@example.com"):
    println("valid email")

# Extract numbers
num_re: regex = regex("[0-9]+")
text: str = "precio: 42, unidades: 7"
nums: [str] = num_re.find_all(text)
println(nums)   # ["42", "7"]
```
