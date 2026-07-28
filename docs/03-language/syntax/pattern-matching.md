# Pattern Matching

**Status:** [x] Basic match with `_:` and literal patterns. [x] `1|2` or-pattern. [x] `..=` range pattern.

## Match statement

```ky
match value:
 pattern1:
 body1
 pattern2:
 body2
 _:
 default
```

## Match expression

```ky
result = match x:
 1: "one"
 2: "two"
 _: "other"
```

## Patterns

### Literal
```ky
match x:
 0: println("zero")
```

### Identifier
```ky
match x:
 n: println(n) # binds value to n
```

### Wildcard
```ky
match x:
 _: println("anything")
```

### Or-pattern
```ky
match x:
 1 | 2: println("one or two")
```

### Guard
```ky
match x:
 n if n > 10: println("big")
 n: println(n)
```

### Enum variant
```ky
enum Optional:
 Some(i32)
 None

match v:
 Optional.Some(n):
 println(n)
 Optional.None:
 println("none")
```

### IsType
```ky
match x:
 is str: println("string")
 is i32: println("integer")
```

## Binding if

```ky
if name = optional_value:
 println(name)
```

## Binding while

```ky
while line = read_line():
 process(line)
```
