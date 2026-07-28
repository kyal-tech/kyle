# Variables

**Status:** [x] Documentación completa. [~] Parcialmente implementado.

## Declaration

```ky
x: i32 = 42         # type explícito + value
y = 10              # type inferido (i32)
name: str = "Ana"
```

No hay `let`, `var` ni `const`.

## Inmutabilidad por defecto

```ky
x: i32 = 10
x = 20   # ERROR
```

### Mutable con `^`

```ky
x: ^i32 = 10
x = x + 1          # ✅

items: ^[str] = [] # lista mutable
items.push("a")
```

### Borrow con `&`

```ky
x: &str = &"hello"
items: &[i32] = &[1, 2, 3]
```

### Mutable borrow `^&`

```ky
x: ^&str
items: ^&[i32]
```

## Option `?` y Error `!`

Ortogonales: funcionan en CUALQUIER tipo.

```ky
x: i32?          # Option<i32>
x: [str]?        # lista opcional
x: ^[i32]!       # lista mutable con error
x: ^&{str: i32}? # dict mutable borrow opcional
x: queue<i32>!   # queue con error
```

## Copy vs Move

### Copy (i32, f64, bool, char, ptr, [T, N])

```ky
x = 42
y = x   # COPY: ambos vivos
```

### Move (str, [T], {K:V}, set<T>, queue<T>, clases)

```ky
s = "hola"
t = s   # MOVE: s inválido
```

## Globals

```ky
print("hello")
name = input("name? ")
```

## Casting

```ky
x = 42 as f64
```

## Scope

```ky
x = 1
if true:
    y = 2
    x = x + y
```

## Destructuring

```ky
punto: (i32, str) = (10, "hello")
(x, y) = punto
```
