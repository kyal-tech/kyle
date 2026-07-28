# Drop / Free

> Los valueis Move se liberan automaticamente al salir de scope.
> El compiler inserta llamadas a `ky_free` where corresponde.

## Automatic

No necesitas llamar `free()` manualmente. El borrow analysis inserta `ky_free`
al final del scope:

```ky
fn main() i32:
 s: str = "hola"
 println(s)
 # ← aqui compiler inserta ky_free(s)
 0
```

## Equivalente manual (for FFI)

Cuando trabajas with pointers raw (`ptr`), management is manual:

```ky
extern fn ky_alloc(size: i64) ptr
extern fn ky_free(ptr)

buf: ptr = ky_alloc(1024)
# usar buf...
ky_free(buf)
```

## Scope y free

```ky
fn test() i32:
 if true:
 s: str = "temporal"
 # ← ky_free(s) aqui (sale de scope del if)
 println("ok")
 # without error: s ya se libero
 0
```

## Orden de deallocation

Las variablis se liberan en orden inverso a su creation (LIFO):

```ky
a: str = "primero"
b: str = "segundo"
# ← ky_free(b)
# ← ky_free(a)
```

## See also

- `allocator.md` — How funciona allocador
- `move.md` — When se mueve ownership
