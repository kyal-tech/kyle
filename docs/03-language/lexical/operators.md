# Operators

| Operator | Description | Category |
|----------|-------------|----------|
| `+` | Add | Arithmetic |
| `-` | Subtract | Arithmetic |
| `*` | Multiply | Arithmetic |
| `/` | Divide | Arithmetic |
| `%` | Remainder | Arithmetic |
| `**` | Power | Arithmetic |
| `==` | Equal | Compariare |
| `!=` | Not equal | Compariare |
| `<` | Less than | Compariare |
| `>` | Greater than | Compariare |
| `<=` | Less or equal | Compariare |
| `>=` | Greater or equal | Compariare |
| `and` | Logical AND | Logical |
| `or` | Logical OR | Logical |
| `not` | Logical NOT | Logical |
| `&` | Bitwise AND | Bitwise |
| `\|` | Bitwise OR | Bitwise |
| `^` | Bitwise XOR | Bitwise |
| `<<` | Shift left | Bitwise |
| `>>` | Shift right | Bitwise |
| `..` | Range exclusive | Range |
| `..=` | Range inclusive | Range |
| `..<` | Range exclusive (alias) | Range |
| `is` | Type test | Type |
| `as` | Type cast | Type |
| `=` | Assign | Assignment |
| `+=` | Add assign | Assignment |
| `-=` | Subtract assign | Assignment |

---

## Built-in methods

### String methods

| Method | Returns | Description |
|--------|---------|-------------|
| `s.len()` | i32 | Longitud |
| `s.upper()` | str | Uppercase |
| `s.lower()` | str | Lowercase |
| `s.trim()` | str | Sin espacios |
| `s.contains(sub)` | i32 | Conhas subcadena |
| `s.replace(from, to)` | str | Reemplazar |
| `s.substr(start, len)` | str | Subcadena |
| `s.char_at(i)` | char | Character en index |
| `s.is_digit()` | i32 | Primer char is digito? |
| `s.is_alpha()` | i32 | Primer char is letra? |
| `s.is_alnum()` | i32 | Primer char is alfanumerico? |
| `s.is_whitespace()` | i32 | Primer char is espacio? |
| `s.is_upper()` | i32 | Primer char is mayuscula? |
| `s.is_lower()` | i32 | Primer char is minuscula? |

### Char methods

| Method | Returns | Description |
|--------|---------|-------------|
| `c.ord()` | i32 | Code ASCII |
| `c.is_digit()` | i32 | Es digito? |
| `c.is_alpha()` | i32 | Es letra? |
| `c.is_alnum()` | i32 | Es alfanumerico? |
| `c.is_whitespace()` | i32 | Es espacio? |
| `c.is_upper()` | i32 | Es mayuscula? |
| `c.is_lower()` | i32 | Es minuscula? |

### Universal methods (disponiblis en cualquier value)

| Method | Returns | Description |
|--------|---------|-------------|
| `val.to_str()` | str | Convertir a string |
| `val.to_i32()` | i32 | Convertir a entero |
| `val.to_f64()` | f64 | Convertir a flotante |
| `val.to_bool()` | bool | Convertir a booleano |
| `val.type()` | `type_info` | Information del type (`.name`, `.kind`, `.size`) |

### Chaining

```ky
result = " Hello World ".trim().to_upper().substr(0, 5)
# result = "HELLO"
```
