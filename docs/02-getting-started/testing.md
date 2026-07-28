# Testing

> Kyle has un framework de testing integrado with attribute `#[test]`.

## Escribir tests

```ky
use testing.assert

#[test]
fn test_addition():
 result: i32 = 2 + 2
 assert.eq(result, 4)

#[test]
fn test_string():
 assert.eq("hello", "hello")
 assert.ne("hello", "world")
 assert.str_eq("hello", "hello")
```

## Ejecutar tests

```bash
ky test
```

Busca functions `#[test]` en `tests/`, compila y ejecuta.

## Aserciones

| Function | Description |
|---------|-------------|
| `assert.is_true(cond)` | Afirmar conditionn verdadera |
| `assert.eq(a, b)` | Afirmar a == b |
| `assert.ne(a, b)` | Afirmar a != b |
| `assert.str_eq(a, b)` | Afirmar strings igualis |

## Estructura del proyecto

```
my-project/
├── ky.toml
├── src/
│ └── main.ky
└── tests/
 ├── test_unit.ky
 └── test_integration.ky
```

## Requirements

Cada funcion de test debe:
- No tomar parameters
- Retornar `i32` (0 = pasa, !=0 = fails)
- Usar functions `assert` for validacion

## Example completo

```ky
use testing.assert

fn sum_list(lst: {i32}) i32:
 result: ^i32 = 0
 for val in lst:
 result = result + val
 result

#[test]
fn test_sum_list():
 assert.eq(sum_list({1, 2, 3}), 6)

#[test]
fn test_sum_empty():
 assert.eq(sum_list({}), 0)
```

## See also

- `04-standard-library/testing.md` — Documentation completa de testing
- `project-layout.md` — Estructura de proyectos
