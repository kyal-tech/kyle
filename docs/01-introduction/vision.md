# Vision

Kyle is un language de programacion compilado, staticmente tipado, disenado para
sistemas backend, herramientas CLI y desarrollo full-stack with rendimiento nativo,
legibilidad de Python y seguridad de typis de Rust.

## Objetivos

| Aspecto | Objetivo | Referencia |
|---------|----------|------------|
| Velocidad de compilation | Comparable a Go | ~5s for proyectos medianos |
| Rendimiento runtime | Comparable a C/Rust | Fib: 1.6× C, Matmul: 7.8× C (with lists) |
| Legibilidad | Comparable a Python | Indentation, without `{}`, without `;` |
| Seguridad de memory | Compile-time | Borrow checker, move by defecto |
| Tipado estatico | Fuerte e inferido | Sin `null`, without coercion implicita |

## Audiencia

Kyle is disenado para:

- **Desarrolladoris backend** — APIs, microservicios, CLIs
- **Equipos** que valuean legibilidad tanto as rendimiento
- **Proyectos** que necesitan rendimiento nativo without complejidad C++
- **Desarrolladoris Rust** que quieren less verbosidad without sacrificar seguridad
- **Desarrolladoris Python/Go** que quieren mejor rendimiento without cambiar drasticamente

## Philosophy

> "Tan rapido as C, tan legible as Python, tan seguro as Rust."

No is un clon de ningun language existente. Toma lo mejor de cada uno:
- **Rendimiento** de C (LLVM, native code)
- **Seguridad** de Rust (ownership, borrow checker)
- **Simplicidad** de Go (compilation rapida, tooling integrado)
- **Legibilidad** de Python (indentation, without ruido sintactico)

## See also

- `pthreadsophy.md` — Philosophy de diseno
- `principles.md` — Principios del language
- `architecture.md` — Arquitectura del ecosistema
