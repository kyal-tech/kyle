# Traits / Contracts

> Interfacis en Kyle using `contract`. Definen comportamiento que clases
> can implementar.

## Declaration

```ky
contract Drawable:
 fn draw()
 fn get_size() (i32, i32)

contract Serializable:
 fn serialize() str
 fn deserialize(data: str)
```

## Implementation

```ky
class Circle :: Drawable, Serializable:
 radius: i32

 fn draw():
 println("circle r=" + radius.to_str())

 fn get_size() (i32, i32):
 (radius * 2, radius * 2)

 fn serialize() str:
 "Circle(" + radius.to_str() + ")"

 fn deserialize(data: str):
 radius = 10 # simplified
```

## Herencia simple

```ky
class Animal:
 name: str

class Dog :: Animal:
 fn speak():
 println(name + " says woof")
```

## Contracts vs class inheritance

| Concepto | `contract` | `class :: Parent` |
|----------|-----------|------------------|
| Purpose | Interface | Herencia de implementation |
| Multiples | ✅ Si | ❌ No (simple) |
| Status | ❌ No | ✅ Si (fields) |
| Methods default | ❌ No | ✅ Si |

## See also

- `generics.md` — Generics with constraints
- `structs.md` — Clasis y structs
