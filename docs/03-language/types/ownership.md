# Ownership

**Status:** [x] `^` = mutable, `&` = borrow, `^&` = mutable borrow, move by defecto. Borrow checker implemented.

## Reglas

1. **Move by defecto**: `y = x` transfiere ownership de `x` a `y` (for typis no-Copy).
2. **Borrow with `&`**: `f(&x)` presta `x` without transferir ownership.
3. **Mutable with `^`**: `x: ^str` declara variable mutable.
4. **Mutable borrow with `^&`**: `f(^&x)` presta `x` with permiso de modificar.
5. **Copy automatic**: Typis numericos, `bool`, `char`, `ptr` se copian en `y = x`.
6. **Clone explicito**: `y = x.clone()` for copiar typis Move.
7. **One mutable XOR many immutable** en un mismo scope.
8. **No dangling pointers**: referencias no can outlive al value original.
9. **No `&` return**: prohibido devolver referencias (evita problemas de lifetime).

## Copy vs Move

| Copia (automatic) | Mueve (by defecto) |
|--------------------|---------------------|
| `i8`, `i16`, `i32`, `i64` | `str` |
| `u8`, `u16`, `u32`, `u64` | `{T}` (list) |
| `f32`, `f64` | `{K:V}` (dict) |
| `bool`, `char` | `[T, N]` (array) |
| `ptr` | classes, structs, enums |

## Variables

```ky
x = 42 # inmutable (default), COPY (i32)
s = "hola" # inmutable (default), OWNED (str)
x: ^i32 = 0 # mutable, COPY
buf: ^str = "" # mutable, OWNED
```

## Parameters

```ky
fn f(s: str) # MOVE: caller pierde ownership
fn f(s: &str) # BORROW: caller presta
fn f(s: ^&str) # MUT BORROW: caller presta mutable
```

## Expressions

```ky
# Move (default)
a = "hola"
b = a # MOVE: a invalido after
println(a) # ERROR: a fue movido

# Borrow
a = "hola"
println(&a) # BORROW: a sigue vivo

# Mutable borrow
buf: ^str = ""
fill(^&buf) # MUT BORROW: buf mutable prestado
println(buf) # ✅ buf sigue vivo, modificado

# Clone (copia explicita)
a = "hola"
b = a.clone() # COPY: ambos vivos
println(a) # ✅ "hola"
println(b) # ✅ "hola"

# Copy typis (automatic)
x = 42
y = x # COPY: ambos vivos
println(x) # ✅ 42
```

## Clasis (without `this` obligatorio)

```ky
class Point:
 x: ^i32 = 0
 y: ^i32 = 0

 fn move_to(nx: &i32, ny: &i32):
 x = nx # campo directo, without this.x
 y = ny

 fn clone():
 Point(x, y)

 fn register():
 poll_events(&this) # autoreferencia with this
```

## Borrow checker (implemented)

El borrow checker valida:

1. ✅ **Use-after-move**: `y = x; println(x)` → error.
2. ✅ **Aliasing mutability**: `r1 = ^&x; r2 = &x` → error (uno mutable + otro inmutable).
3. 🔶 **Dangling references**: `x = &y; drop(y); println(x)` — pendiente.

## Comparison with Rust

| Concepto | Rust | Kyle |
|----------|------|------|
| Variable inmutable | `let x` | `x = v` |
| Variable mutable | `let mut x` | `x: ^T = v` |
| Move | `y = x` (default) | `y = x` (default) |
| Borrow | `&x` | `&x` |
| Mutable borrow | `&mut x` | `^&x` |
| Clone | `.clone()` | `.clone()` |
| Copy typis | `#[derive(Copy)]` | Built-in for numericos |
| Lifetime params | `'a` | **No existen** (prohibido return `&`) |
| `self` | `self.foo` | `foo` (directo) |
