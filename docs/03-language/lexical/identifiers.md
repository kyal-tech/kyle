# Identifiers

> Reglas for nombris de identificadoris en Kyle.

## Reglas

- Deben empezar with letra o `_`
- Pueden contener letras, digitos y `_`
- Distinguen mayusculas/minusculas (`foo` ≠ `Foo`)
- Longitud ilimitada
- Palabras reservadas (keywords) no can be identificadores

## Validos

```ky
foo
mi_variable
_private
contador_123
MAX_SIZE
StringBuilder
```

## Invalidos

```ky
123foo # empieza with digito
foo-bar # guion no permitido
if # keyword reservada
```

## Convenciones

| Convention | Example | When usar |
|------------|---------|-------------|
| `snake_case` | `mi_variable`, `calcular_total` | Functions, variables, methods |
| `UPPER_SNAKE` | `MAX_SIZE`, `PI` | Constbefore (`:=`) |
| `_prefix` | `_internal`, `_cache` | Privado / interno |

## See also

- `keywords.md` — Palabras reservadas
- `comments.md` — Comentarios
