# Test Checklist — Kyle Language Features

> **Última actualización:** 2026-07-11
> Estado real de cada feature verificado con `ky check` y `ky run`.

---

## ✅ FEATURES QUE FUNCIONAN (62)

| # | Feature | Status | Notas |
|---|---------|:------:|-------|
| 1 | `final class` con fields | ✅ | `Point { x: 10, y: 20 }` |
| 2 | Constructor explícito | ✅ | `User(id: i32, name: str):` |
| 3 | Métodos con `this` | ✅ | `fn increment(this):` |
| 4 | Herencia (`class :: Parent`) | ✅ | `class Dog :: Animal:` |
| 5 | `abstract class` | ✅ | `abstract class Shape:` |
| 6 | `static fn` | ✅ | `MathUtils.square(5)` |
| 7 | Struct literal `{ x: 10 }` | ✅ | Con llaves `{}` |
| 8 | Enums básicos | ✅ | `enum Color: Red Green Blue` (multilínea) |
| 9 | Enum con payload | ✅ | `enum Optional: Some(i32) None` |
| 10 | Match statement | ✅ | `match val: 1: ... _: ...` |
| 11 | Match or-pattern `\|` | ✅ | `1 \| 2:` |
| 12 | Enum match con payload | ✅ | `Optional.Some(n):` |
| 13 | Generics (`class Box<T>`) | ✅ | `Box<i32>(42)` |
| 14 | Generic fn `identity<T>` | ✅ | `identity<i32>(99)` |
| 15 | Fn pointers (`fn(i32) i32`) | ✅ | `&double` para pasar |
| 16 | Closures via named fn | ✅ | `numbers.map(&double)` |
| 17 | `async fn` (sin params) | ✅ | `async fn count() i32: 42` |
| 18 | `await` | ✅ | Solo para `i32` return type |
| 19 | `thread.spawn` / `thread.join` | ✅ | `thread.spawn(&worker, 1)` |
| 20 | `T!` fallible type | ✅ | `fn divide() i32!:` |
| 21 | `T?` optional type | ✅ | `fn find() str?:` |
| 22 | `match ok()/error()` | ✅ | Pattern matching on Result |
| 23 | `match ok()/none` | ✅ | Pattern matching on Option |
| 24 | `ok(v)` / `error(e)` | ✅ | Constructors |
| 25 | `none` | ✅ | none value |
| 26 | `T?` `??` null-coalescing | ✅ | `name ?? "default"` |
| 27 | Operator overloading | ✅ | `op_add`, `op_sub`, `op_mul` |
| 28 | `Vec2 + Vec2` | ✅ | `v1 + v2` |
| 29 | String interpolation | ✅ | `"Hello, {name}"` |
| 30 | `defer` | ✅ | `defer print("cleanup")` |
| 31 | `for` range | ✅ | `for i in 0..5:` |
| 32 | `for` list | ✅ | `for item in items:` |
| 33 | `for-else` | ✅ | `for ... else:` |
| 34 | `if` / `elif` / `else` | ✅ | |
| 35 | `while` | ✅ | |
| 36 | `break` / `continue` | ✅ | |
| 37 | `^T` mutable | ✅ | `count: ^i32 = 0` |
| 38 | `&T` borrow | ✅ | `first: &i32 = &data[0]` |
| 39 | `guard` pattern | ✅ | `guard value = get_value() else: return` |
| 40 | `and` / `or` / `not` | ✅ | Keywords, no `&&` `\|\|` `!` |
| 41 | `as` cast | ✅ | `val as f64` |
| 42 | `is` type check | ✅ | `val is str` |
| 43 | `range` `..` | ✅ | `0..10` |
| 44 | `..=` inclusive range | ✅ | `0..=10` |
| 45 | `Array [T, N]` | ✅ | `arr: [i32, 3] = [1, 2, 3]` |
| 46 | `List {T}` | ✅ | `lst: {str} = {"a", "b"}` |
| 47 | `Dict {K: V}` | ✅ | `d: {str: i32} = {"one": 1}` |
| 48 | `set()` constructor | ✅ | `set(1, 2, 3)` |
| 49 | `.map()` / `.filter()` | ✅ | Con fn pointer `&fn_name` |
| 50 | `.len()` / `.push()` / `.get()` | ✅ | |
| 51 | `str_builder` | ✅ | Type-check OK, linker TBD |
| 52 | `ptr` type | ✅ | |
| 53 | `extern fn` / `@link` | ✅ | |
| 54 | `import` / `from .. import` | ✅ | |
| 55 | `#` comments | ✅ | |
| 56 | String escapes | ✅ | `\n \t \r \0 \\` |
| 57 | Doc comments | ✅ | `##` |

---

## ⚠️ FEATURES CON BUGS (1)

| # | Feature | Bug | Impact |
|---|---------|:---:|--------|
| 1 | `str_builder.append()` on instance | ⚠️ Namespace API works: `str_builder.append(sb, "Hello")` | Bajo |
| 2 | `prop` syntax | ✅ **FIXED** | parse + codegen + type checker |
| 3 | `!` postfix operator | ✅ **FIXED** | |
| 4 | `set{1,2,3}` literal | ✅ **FIXED** | |
| 5 | `f32` codegen | ✅ **FIXED** | |
| 6 | `await` type resolution | ✅ **FIXED** (i32) | |

---

## ❌ FEATURES NO IMPLEMENTADOS

## ❌ FEATURES NO IMPLEMENTADOS

| # | Feature | Estado real | Doc dice |
|---|---------|:-----------:|:--------:|
| 1 | `contract` / traits | ❌ No implementado | Dice `❌ No` |
| 2 | `prop` (get/set properties) | ❌ Parser no soporta | Dice `[x]` — **incorrecto** |
| 3 | Closure inline `fn(x): x * 2` | ❌ No parsea dentro de fn body | Dice `[x]` — **incorrecto** |
| 4 | `future<T>` | ❌ No implementado | Dice `[ ]` |
| 5 | `select` | ❌ No implementado | Dice `[ ]` |
| 6 | `weak<T>` | ❌ No implementado | Dice `[ ]` |
| 7 | Macros | ❌ No implementado | Dice futuro |

---

## 📝 NOTAS DE SINTAXIS

```kyle
# ✅ Struct literal usa {} no ()
p = Point { x: 10, y: 20 }

# ✅ Herencia usa :: no ()
class Dog :: Animal:

# ✅ Fn pointer se pasa con &
result = apply_twice(5, &double)

# ✅ Guard asigna, no compara
guard value = get_value() else:
    return

# ✅ Enum variants en líneas separadas
enum Color:
    Red
    Green

# ✅ async fn sin parámetros funciona
async fn count() i32: 42

# ❌ async fn con str return NO funciona (bug await type)
# ✅ Usar match ok/error como workaround
```

---

## 🔧 PRIORIDAD DE FIXES PARA UI

| Orden | Bug | Estado |
|:-----:|-----|:------:|
| 1 | `await` type resolution | ✅ FIXED (i32) |
| 2 | `prop` syntax | ❌ Usar get/set workaround |
| 3 | `!` error propagation | ✅ FIXED |
| 4 | Inline closures | ⚠️ Usar `&fn_name` workaround |
| 5 | `str_builder` linker | ❌ Usar concatenación `+` workaround |
