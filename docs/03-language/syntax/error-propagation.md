# Error Propagation (`!`)

**Status:** Implemented
**Related:** [result.md](../error-handling/result.md), [functions.md](functions.md)

---

## 1. El operador `!`

El operador `!` (postfijo) propaga errores automáticamente. Es equivalente
a Rust `?` o Python `raise` implícito. Si la expresión es `T!` (fallible) y
contiene un error, la función retorna ese error inmediatamente.

```kyle
fn divide(a: i32, b: i32) i32!:
    if b == 0:
        error("division by zero")
    a / b

fn calculate() i32!:
    result = divide(10, 2) !   # Si divide() falla, calculate() retorna el error
    result * 2

fn main():
    match calculate():
        ok(val):
            print("Result: " + val.to_str())
        error(e):
            print("Error: " + e)
```

### 1.1 Sin `!` (manual)

Sin el operador `!`, hay que manejar el error manualmente:

```kyle
fn calculate() i32!:
    match divide(10, 2):
        ok(val):
            val * 2
        error(e):
            error(e)  # propagación manual
```

### 1.2 Con `!` (automático)

```kyle
fn calculate() i32!:
    divide(10, 2) ! * 2   # equivalente al de arriba
```

---

## 2. Reglas

| Regla | Descripción |
|-------|-------------|
| **Solo en funciones fallible** | La función debe retornar `T!` para poder usar `!` |
| **Propaga cualquier error** | El error se propaga sin modificar (sin wrapping) |
| **No se puede usar en `fn main`** | `main` no es fallible — el panic es la única salida |
| **Encadenamiento** | `a() !.method() !.another() !` |

---

## 3. Encadenamiento

```kyle
fn process_file(path: str) str!:
    content = read_file(path) !
    parsed = parse_json(content) !
    parsed.get("name") !.to_str()
```

Cada `!` propaga el error si ocurre, o continúa con el valor.

---

## 4. Con `T?` (Optional)

```kyle
fn find_user(id: i32) User?:
    # ...

fn get_name(id: i32) str?:
    user = find_user(id) !
    user.name
```

Si `find_user` retorna `None`, el `!` propaga `None`.

---

## 5. Diferencia con `not`

| Símbolo | Significado | Uso |
|---------|-------------|-----|
| `!` (postfijo) | Propagación de error | `expr !` |
| `not` (prefijo) | Negación lógica | `not flag` |

```kyle
result = risky_call() !    # propaga error
visible = not hidden        # negación lógica
```

---

## 6. Referencias

- [result.md](../error-handling/result.md) — Tipo `T!`
- [option.md](../error-handling/option.md) — Tipo `T?`
- [functions.md](functions.md) — Return types
