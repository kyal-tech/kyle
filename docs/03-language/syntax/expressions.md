# Expressions

**Status:** [x] Documentación completa. [~] Parcialmente implementado.

## Binary operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+` `-` `*` `/` `%` `**` |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical | `and` `or` `not` |
| Bitwise | `&` `\|` `^` `<<` `>>` |
| Range | `..` `..=` `..<` |
| Type test | `is` |
| Type cast | `as` |

## Primary expressions

```ky
42              # literal
"hello"         # string
name            # identifier
a + b           # binary
-a              # unary negate
not flag        # logical not
a.b             # property access
a.b()           # method call
a[b]            # index
a[b..c]         # slice
f(x, y)         # function call
```

## Collection literals

```ky
[1, 2, 3]           # lista [i32]
^[1, 2]             # lista mutable
[]                  # lista vacía
^[]                 # lista mutable vacía
set{1, 2, 3}        # set<i32>
{"a": 1, "b": 2}    # dict {str: i32}
queue{1, 2, 3}      # queue<i32>
stack{"a", "b"}     # stack<str>

# Arrays con tipo explícito:
arr: [i32, 5] = [1, 2, 3, 4, 5]
grid: [[i32, 3], 3] = [[1,0,0],[0,1,0],[0,0,1]]
```

## Assignment

```ky
x = value
x += 1
(x, y) = tuple
```

## Ternary

```ky
result = if x > 0: "positive" else: "negative"
```

## Match expression

```ky
name = match x:
    1: "one"
    2: "two"
    _: "other"
```

## Range

```ky
0..5     # exclusive: 0,1,2,3,4
0..=5    # inclusive: 0,1,2,3,4,5
0..<5    # exclusive alias
```

## Precedence

| Level | Operators | Assoc |
|-------|-----------|-------|
| 15 | `.` | Left |
| 14 | `()` | Left |
| 13 | `[]` | Left |
| 12 | `as` | Left |
| 11 | `**` | Right |
| 10 | `*` `/` `%` | Left |
| 9 | `+` `-` | Left |
| 8 | `<<` `>>` | Left |
| 7 | `&` | Left |
| 6 | `^` | Left |
| 5 | `\|` | Left |
| 4 | `<` `>` `<=` `>=` | Left |
| 3 | `==` `!=` `is` | Left |
| 2 | `..` `..=` `..<` | Left |
| 1 | `and` | Left |
| 0 | `or` | Left |
