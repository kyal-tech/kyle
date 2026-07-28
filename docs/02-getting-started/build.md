# Build

> Compilar proyectos Kyle with CLI `ky`.

## Compilar un file

```bash
ky build file.ky # compile (debug)
ky build --release file.ky # compile with optimizaciones
ky build file.ky -o output # especificar name de output
```

## Compilar y execute

```bash
ky run file.ky # compila y ejecuta
ky run --release file.ky # compila optimizado y ejecuta
```

## Type-check rapido

```bash
ky check file.ky # solo type checking, without code objeto
```

## Proyectos

```bash
ky build # compile proyecto current
ky build --release # compile en release
ky check # type-check del proyecto
```

## Flags

| Flag | Description |
|------|-------------|
| `--release` | Compilation optimizada (O3 + LTO) |
| `-o` | Nombre del binary de output |
| `--target` | Target triple (ej: `wasm32`) |

## See also

- `installation.md` — Instalar Kyle
- `first-program.md` — Primer program
- `06-compiler/pipeline.md` — Pipeline de compilation
