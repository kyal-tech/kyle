# Reflection

> Information de typis en tiempo de execution.
> Actualmente support minimum: `type()` returns information basica.

## type()

```ky
t: type_info = (42).type()
println(t.name) # "i32"
println(t.kind) # "primitive"
println(t.size) # 4 (bytes)
```

```ky
t = "hello".type()
println(t.name) # "str"
println(t.size) # 8 (bytis del pointer)
```

## type_info

```ky
class type_info:
 name: str
 kind: str # "primitive", "struct", "enum", "array", "list"
 size: i32 # bytes
```

## Usos

| Purpose | Example |
|-----------|---------|
| Debug | `println(val.type().name)` |
| Serialization | `json.serialize(val)` usa type internamente |
| Generics | El compiler infiere typis en tiempo de compilation |

## Limitations

- No there is `instanceof` o type checking dynamic
- No there is cast between typis no relacionados
- El type system is completamente estatico (excepto `type()`)

## See also

- `primitive-types.md` — Todos typis disponibles
- `generics.md` — Polimorfismo en tiempo de compilation
