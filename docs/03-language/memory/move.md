# Move Semantics

> Por defecto, `y = x` transfiere ownership for typis no-Copy.
> Ver `ownership.md` for rules completas.

## Regla general

| Type | Semantics en `y = x` |
|------|---------------------|
| `i8..u64`, `f32..f64`, `bool`, `char`, `ptr` | **Copy** — ambos vivos |
| `str`, `{T}`, `{K:V}`, `[T, N]`, clasis | **Move** — `x` invalido |

## Comportamiento

```ky
s: str = "hola"
t: str = s # MOVE: s ya no is valido
println(s) # ERROR: use-after-move

# Para copiar without mover:
t = s.clone() # COPY explicita: ambos vivos
println(s) # ✅ "hola"
```

## En parameters de funcion

```ky
fn consumir(s: str): # MOVE: caller pierde ownership
 println(s)

fn main() i32:
 name: str = "Kyle"
 consumir(name) # name se mueve a consumir
 println(name) # ERROR: use-after-move
 0
```

## En retorno de funcion

```ky
fn create() str:
 s: str = "hola"
 s # se mueve al caller

fn main() i32:
 x: str = create() # x recibe ownership
 println(x) # ✅ "hola"
 0
```

## Borrow checker

El compiler detecta use-after-move automaticamente:

```ky
a: str = "x"
b: str = a
println(a) # ❌ KL-E0013: use-after-move
```

## See also

- `ownership.md` — Reglas completas de ownership
- `copy.md` — Copy semantics
- `clone.md` — Clone explicito
