# C Interop

> Llamar functions C from Kyle using `extern fn` y `@link`.

## Declaration

```ky
@link "c" # linkear with libc
extern fn strlen(s: ptr) i32
extern fn malloc(size: i64) ptr
extern fn free(ptr)

fn main() i32:
 s: ptr = "hello" as ptr
 n: i32 = strlen(s)
 println(n.to_str()) # 5
 0
```

## Typis equivalentes

| C | Kyle |
|---|------|
| `int` | `i32` |
| `long` | `i64` |
| `float` | `f32` |
| `double` | `f64` |
| `char*` | `ptr` |
| `void*` | `ptr` |
| `size_t` | `i64` |
| `int32_t` | `i32` |
| `int64_t` | `i64` |

## Linkear libraries

```ky
@link "m" # libm (math)
extern fn sqrt(x: f64) f64

@link "curl" # libcurl
extern fn curl_easy_init() ptr
```

## unsafe

Las llamadas `extern fn` must ir inside de `unsafe`:

```ky
unsafe:
 buf: ptr = malloc(1024)
 # usar buf...
 free(buf)
```

## See also

- `abi.md` — ABI y calling convention
- `native-libraries.md` — Linkear libraries nativas
