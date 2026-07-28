# Native Libraries

> Linkear y usar libraries nativas del sistema from Kyle.

## Linker flags with `@link`

```ky
@link "curl" # -lcurl
@link "pthread" # -lpthread
@link "m" # -lm (math)
@link "sqlite3" # -lsqlite3
@link "pq" # -lpq (PostgreSQL)
```

### Frameworks (macOS)

```ky
@link "-framework CoreFoundation"
@link "-framework Security"
@link "-framework SystemConfiguration"
```

### Library paths

```ky
@link "-L/usr/local/lib"
@link "-L/opt/homebrew/lib"
```

## Path de busqueda

El linker busca libraries en:

1. `-L` paths especificados en `@link`
2. Variablis de entorno (`LIBRARY_PATH`, `LD_LIBRARY_PATH`)
3. Paths del sistema (`/usr/lib`, `/usr/local/lib`, `/opt/homebrew/lib`)

## Compilation static vs dynamic

| Type | Ventaja | Desventaja |
|------|---------|-------------|
| Static (`.a`) | Binario portable | Size grande |
| Dynamic (`.so`/`.dylib`) | Size pequeno | Requiere runtime library |

Por defecto Kyle linkea dynamicmente. Para estatico, usa `@link "lib.a"`.

## See also

- `c.md` — Llamar functions C
- `abi.md` — ABI y calling convention
