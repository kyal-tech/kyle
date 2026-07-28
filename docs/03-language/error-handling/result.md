# Error Handling

**Status:** [x] `T!` fallible return type, `error(msg)`/`ok(val)` builtins, `ok(v)`/`error(e)` pattern match.

Kyle sigue filosofia de Rust: **no there is try/catch**. Los errors se manejan with typis `Result` y pattern matching.

## Fallible type: `T!`

`T!` is sugar for `Result<T, str>`. Functions que can failsr returnsn `T!`.

```ky
fn divide(a: i32, b: i32) i32!:
 if b == 0:
 return error("division by zero")
 a / b
```

## Handling errors with match

```ky
fn main() i32:
 r = divide(10, 2)
 mr = match r:
 ok(v): "ok: " + v.to_str()
 error(e): "err: " + e
 println(mr)
 0
```

## Custom errors

```ky
fn validate(age: i32) i32!:
 if age < 0:
 return error("age cannot be negative")
 if age > 150:
 return error("age seems unrealistic")
 age
```
