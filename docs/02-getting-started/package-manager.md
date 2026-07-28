# Package Manager

> Gestor de paquetis de Kyle: `ky add`, `ky remove`, `ky install`.

## Comandos

### Agregar dependency

```bash
ky add http # agrega http from registry
ky add http@1.0.0 # version especifica
```

### Instalar dependencies

```bash
ky install # instala todas dependencies de ky.lock
```

### Eliminar dependency

```bash
ky remove http # elimina http
```

### Publicar package

```bash
ky publish # publica en registry local
```

## Registry

Los paquetis se distribuyen via GitHub Pages:

```text
https://IT-KYNERA.github.io/KYLE/docs
```

## Paquetis oficiales

| Package | Description |
|---------|-------------|
| `http` | Cliente y servidor HTTP |
| `json` | Parseo y serializacion JSON |
| `sqlite` | Base de data SQLite |

## ky.lock

El file `ky.lock` se genera automaticamente al instalar dependencies.
No se must modificar manualmente.

## See also

- `build.md` — Compilation
- `project-layout.md` — Estructura de proyectos
- `08-ecosystem/` — Documentation del ecosistema
