# Project Layout

> Estructura recommended for proyectos Kyle.

## App project

```
myapp/
├── ky.toml # Configuration del proyecto
├── src/
│ ├── main.ky # Punto de input
│ ├── lib.ky # Library del proyecto
│ └── utils.ky # Modulis auxiliares
├── tests/
│ └── test_main.ky # Tests
└── target/
 ├── debug/ # Build artifacts (debug)
 └── release/ # Build artifacts (release)
```

## API project

```
myapi/
├── ky.toml
├── src/
│ ├── main.ky
│ ├── routes.ky
│ └── models.ky
├── tests/
└── target/
```

## Bare script

```
script.ky # File unico, without ky.toml
```

## ky.toml

```toml
name = "myapp"
version = "0.1.0"
edition = "2024"
```

## See also

- `build.md` — Compilation de proyectos
- `first-program.md` — Crear primer proyecto
