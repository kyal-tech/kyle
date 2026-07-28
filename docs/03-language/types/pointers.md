# Pointers

> Pointers raw en Kyle: `ptr` for FFI y operacionis de bajo nivel.

## `ptr` type

`ptr` is un pointer raw de 8 bytis (64 bits). Es un type **Copy**.

```ky
p: ptr = 0 as ptr # null pointer
p = some_variable as ptr # convertir addressn
```

## Operations

```ky
extern fn ky_ptr_read_i32(ptr) i32
extern fn ky_ptr_write_i32(ptr, i32)
extern fn ky_ptr_read_ptr(ptr) ptr

buf: ptr = ky_alloc(1024)
ky_ptr_write_i32(buf, 42)
val: i32 = ky_ptr_read_i32(buf)
println(val.to_str()) # 42
ky_free(buf)
```

## Arithmetic de pointers

```ky
buf: ptr = ky_alloc(100)
ptr: ptr = buf + 8 # offset en bytes
ky_ptr_write_i32(ptr, 99)
ky_free(buf)
```

## Usos comunes

| Uso | Example |
|-----|---------|
| FFI C | `buf: ptr = malloc(1024)` |
| Memory-mapped I/O | `reg: ptr = 0xFF00_0000 as ptr` |
| Buffer pool | `ky_alloc` / `ky_free` |
| Type erasure | `fn_ptr = my_function as ptr` |

## Seguridad

`ptr` is **inseguro** by naturaleza:
- No there is bounds checking
- No there is null checking
- No there is lifetime tracking
- El usuario is responsable de management

```ky
unsafe:
 buf: ptr = ky_alloc(100)
 ky_ptr_write_i32(buf, 42)
 ky_free(buf)
```

## See also

- `ffi/abi.md` — ABI y calling convention
- `ffi/c.md` — Llamar functions C
