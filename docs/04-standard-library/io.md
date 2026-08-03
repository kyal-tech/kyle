# io — Console input/output

> Print to and read from the terminal.
> Import: `use std.io`

## Printing

```ky
use std.io

print("hello")     # without newline
println("hello")   # with newline
```

## Reading input

```ky
line: str = io.input()       # read line
line = io.input("> ")        # read line with prompt
```

## Globals

`print()` and `println()` are available globally without import:

```ky
print("hello")
println("hello")
name: str = input("> ")
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `print(text)` | `fn(text: &str)` | Print text without newline |
| `println(text)` | `fn(text: &str)` | Print text with newline |
| `io.input(prompt)` | `fn(prompt: &str) str` | Read line with prompt |
| `io.clear()` | `fn()` | Clear terminal |

## Example

```ky
name: str = input("What is your name? ")
println("Hello, " + name + "!")
```
