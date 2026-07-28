# Classes

**Status:** [x] final class, struct literal, methods, constructors, `Class :: Parent` inheritance, properties (get/set).

## final class

Lightweight struct with fields. No inheritance.

```ky
final class Vec2:
 x: i32
 y: i32
```

### Construction

Fields are assigned by name using struct literal syntax:

```ky
v = Vec2 { x: 10, y: 20 }
```

### Explicit constructor

Use `class_name(...)` to defines a constructor.

```ky
class Config:
 name: str
 port: i32

 Config(name: str, port: i32 = 8080):
 this.name = name
 this.port = port
```

## class

Full class with single inheritance.

```ky
class Animal:
 name: str

 fn speak():
 println("...")
```

### Inheritance

```ky
class Dog :: Animal:
 fn speak():
 println("woof")
```

### Abstract class

Cannot be instantiated. Servis as a base for subclasses.

```ky
abstract class Shape:
 fn area() f64
```

## Mutable fields

```ky
class Config:
 name: str
 port: ^i32 # mutable field

config = Config { name: "server", port: 8080 }
config.port = 9090 # allowed
```

## Properties

Properties with getter/setter blocks:

```ky
class Person:
    _name: str

    name: str:
        get:
            this._name
        set:
            this._name = value

person = Person { _name: "Alice" }
print(person.name)    # "Alice" — llama al getter
person.name = "Bob"   # llama al setter
```

The `prop` keyword syntax (`prop name:`) no está implementada. Usar
`name: type:` (field declaration seguido de get/set blocks indentados).
