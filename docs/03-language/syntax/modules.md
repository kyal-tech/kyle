# Modules

**Status:** [x] Documentación completa. [ ] Implementación en compilador.

## Module definition

A module is a `.ky` file. The module name is the filename without extension. El path del archivo **es** el namespace — no hay declaración `namespace` explícita.

## Imports

Solo `use`. No hay `from X import Y`.

```ky
use std.io                          # módulo completo
use std.io.{print, read}            # import selectivo
use std.io as io                    # alias
use ~utils.helpers                  # relativo (~ = desde el proyecto)
use std.io.print                    # símbolo directo (sin prefijo)

# en uso:
io.print("hello")                   # con prefijo de módulo
print("hello")                      # si importaste directo
```

## Visibilidad

| Prefix | Scope |
|--------|-------|
| `name` | Public |
| `_name` | Protected (same package / subclasses) |
| `__name` | Private (same module) |

## Packages

```
my-project/
├── ky.toml
└── src/
    ├── main.ky
    └── lib.ky
```

## Package manifest (ky.toml)

```toml
[project]
name = "my-project"
version = "0.1.0"
edition = "2024"

[dependencies]
ky-http = "1.0"
```

## Standard library

```ky
use std.io
use std.json
use std.math
use std.time
use std.collections.{queue, stack, deque}
```
