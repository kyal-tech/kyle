# Comments

> Los comentarios en Kyle usan `#` hasta final de line.
> No there is comentarios multiline (`/* */`).

## Comentarios de line

```ky
# Esto is un comentario
x: i32 = 42 # comentario inline

fn suma(a: i32, b: i32) i32:
 # Los comentarios can be en bloquis indentados
 a + b
```

## No there is `/* */`

Kyle no supports comentarios multiline estilo C (`/* ... */`).
Usar `#` en cada line:

```ky
# Esta is una
# explanation
# multiline
```

## Doc comments

```ky
# Esta funcion suma dos numeros
# @param a: primer numero
# @param b: segundo numero
# @returns: suma
fn add(a: i32, b: i32) i32:
 a + b
```

## See also

- `literals.md` — Literalis del language
- `identifiers.md` — Reglas de identificadores
