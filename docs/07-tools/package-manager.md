# Package Manager (Technical Reference)

> Documentation tecnica del gestor de packages.
> Para uso basico, ver `02-getting-started/package-manager.md`.

## Comandos

| Comando | Description |
|---------|-------------|
| `ky add <pkg>` | Agregar dependency |
| `ky remove <pkg>` | Eliminar dependency |
| `ky install` | Instalar dependencies del proyecto |
| `ky publish` | Publicar package en registry |

## Formato del package

```
package-name/
├── ky.toml # metadata del package
└── src/
 └── lib.ky # code source
```

## Registry URL

```
https://IT-KYNERA.github.io/KYLE/docs
```

## See also

- `02-getting-started/package-manager.md` — Guide de uso
- `08-ecosystem/registry.md` — Specification del registry
