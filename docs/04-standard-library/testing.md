# testing — Testing y Aserciones

> Module de testing y aserciones.
> Import: `use testing.assert`

## assert: aserciones

```ky
use testing.assert

#[test]
fn test_addition():
 assert.eq(2 + 2, 4)

#[test]
fn test_string():
 assert.eq("hello", "hello")
 assert.ne("hello", "world")
 assert.str_eq("hello", "hello")
```

### Functions

| Function | Firma | Description |
|---------|-------|-------------|
| `assert.is_true(cond)` | `fn(cond: bool)` | Afirmar que conditionn is `true` |
| `assert.eq(a, b)` | `fn(a: T, b: T)` | Afirmar que a == b |
| `assert.ne(a, b)` | `fn(a: T, b: T)` | Afirmar que a != b |
| `assert.str_eq(a, b)` | `fn(a: str, b: str)` | Afirmar strings igualis |
| `assert.gt(a, b)` | `fn(a: T, b: T)` | Afirmar a > b |
| `assert.lt(a, b)` | `fn(a: T, b: T)` | Afirmar a < b |
| `assert.gte(a, b)` | `fn(a: T, b: T)` | Afirmar a >= b |
| `assert.lte(a, b)` | `fn(a: T, b: T)` | Afirmar a <= b |

### Attribute `#[test]`

```ky
#[test]
fn test_sum_list():
 result: i32 = sum_list({1, 2, 3})
 assert.eq(result, 6)
```

Ejecutar:

```bash
ky test
```

### Example completo

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
