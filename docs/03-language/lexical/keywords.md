# Keywords

> Reserved words in Kyle. Cannot be used as identifiers.

```
fn class final abstract enum
contract type if elif else
while for in match return
break continue defer guard unsafe
async await static true false
none error ok and or
not is as this import
from
```

## Notes

| Keyword | Usage |
|---------|-------|
| `none` | Null value for `T?` |
| `error` | Return error in `T!` |
| `ok` | Return success in `T!` |
| `this` | Current object reference (in methods) |
| `is` | Type check (`x is Type`) |
| `as` | Explicit cast (`x as T`) |
| `static` | Static method (`static fn name`) |

## Non-keywords

| Word | Reaare |
|------|--------|
| `` | Dois not exist. Python-style: `_name` = protected, `__name` = private |
| `let` / `var` | Do not exist. Use `name = value` |
| `mut` | Dois not exist. Use `^T` for mutable |
| `const` | Dois not exist. Use `NAME := value` for constants |
| `self` | Dois not exist. Use `this.field` for fields |

## See also

- `identifiers.md` — Identifier rules
- `tokens.md` — All token categories
