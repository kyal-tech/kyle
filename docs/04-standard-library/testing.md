# testing — Test assertions

> Assertions and testing framework.
> Import: `use std.testing`

## Assertions

```ky
use std.testing

#[test]
fn test_addition():
    assert.eq(2 + 2, 4)

#[test]
fn test_string():
    assert.eq("hello", "hello")
    assert.ne("hello", "world")

#[test]
fn test_conditions():
    assert.is_true(condition)
    assert.is_false(condition)
```

## Comparison

```ky
assert.eq(a, b)     # a == b
assert.ne(a, b)     # a != b
assert.gt(a, b)     # a > b
assert.lt(a, b)     # a < b
assert.gte(a, b)    # a >= b
assert.lte(a, b)    # a <= b
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `assert.is_true(cond)` | `fn(cond: bool)` | Assert condition is true |
| `assert.is_false(cond)` | `fn(cond: bool)` | Assert condition is false |
| `assert.eq(a, b)` | `fn(a: T, b: T)` | Assert a == b |
| `assert.ne(a, b)` | `fn(a: T, b: T)` | Assert a != b |
| `assert.gt(a, b)` | `fn(a: T, b: T)` | Assert a > b |
| `assert.lt(a, b)` | `fn(a: T, b: T)` | Assert a < b |
| `assert.gte(a, b)` | `fn(a: T, b: T)` | Assert a >= b |
| `assert.lte(a, b)` | `fn(a: T, b: T)` | Assert a <= b |
| `assert.near(a, b, epsilon)` | `fn(a: f64, b: f64, eps: f64)` | Assert float near |

## Running tests

```bash
ky test
```

## Example

```ky
use std.testing

fn sum_list(items: &[i32]) i32:
    result: ^i32 = 0
    for val in items:
        result = result + val
    result

#[test]
fn test_sum_list():
    assert.eq(sum_list([1, 2, 3]), 6)

#[test]
fn test_sum_empty():
    assert.eq(sum_list([]), 0)

#[test]
fn test_sum_single():
    assert.eq(sum_list([42]), 42)
```
