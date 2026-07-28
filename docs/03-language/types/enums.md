# Enums

**Status:** [x] Enum variants, match with `Enum.Variant`, `_:` catch-all.

Enums are tagged unions (discriminated unions).

## Declaration

```ky
enum Color:
 Red
 Green
 Blue
```

## With payload

```ky
enum Optional:
 Some(i32)
 None
```

## Usage

```ky
c = Color.Red

match c:
 Color.Red:
 println("red")
 Color.Green:
 println("green")
 Color.Blue:
 println("blue")
```

## Variant with payload

```ky
v = Optional.Some(42)

match v:
 Optional.Some(n):
 println(n)
 Optional.None:
 println("none")
```
