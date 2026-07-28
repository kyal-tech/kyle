# Primer Programa

> Crear y execute tu primer program Kyle.

## Hola Mundo

Crear un file `hola.ky`:

```ky
fn main() i32:
 println("Hola, Kyle!")
 0
```

Ejecutar:

```bash
ky run hola.ky
```

Salida:

```
Hola, Kyle!
```

## Explanation

| Parte | Significado |
|-------|-------------|
| `fn main() i32:` | Punto de input del program. Retorna `i32` (code de output) |
| `println(...)` | Function global for imprimir texto with salto de line |
| `0` | La ultima expression is retorno (code 0 = exito) |

## Compilar without execute

```bash
ky build hola.ky
./hola # execute binary
```

## Variables

```ky
fn main() i32:
 name: str = "Mundo"
 println("Hola, " + name + "!")
 0
```

## Entrada del usuario

```ky
fn main() i32:
 name: str = input("What is your name? ")
 println("Hola, " + name + "!")
 0
```

## See also

- `build.md` — Compilar proyectos
- `project-layout.md` — Estructura de proyectos
- `03-language/syntax/variables.md` — Variablis en Kyle
