# Kyle Self-Hosting Status

## 3 Rust Compiler Fixes (this session)

| Fix | Archivo | Líneas cambiadas |
|-----|---------|:----------------:|
| `str` es Copy — no requiere `&` al pasar a funciones | `kyc_semantic/type_checker.rs:836` | 1 |
| Struct → `i64` para `extern fn` (SSA + non-SSA) | `kyc_backend/codegen.rs:820,3635` | 26 |
| `s.upper()`, `s.lower()`, `str.get(i)` aliases | `kyc_mir/lower.rs:3997,4006,4333,5446,7494` | 28 |

**Todos los 181 tests Rust pasan.**

---

## Kyle Components (`runtimes/ky/`)

| Componente | `ky check` | `ky run` | Estado real |
|-----------|:----------:|:--------:|-------------|
| `token.ky` | ✅ | ✅ | Tipos de token (enum + class) |
| `alloc.ky` | ✅ | ✅ | `extern fn` wrappers (malloc/free) |
| `codegen.ky` | ✅ | ✅ | Genera LLVM IR para add/max/fact |
| `compiler.ky` | ✅ | ✅ | Pipeline demo hardcodeado (add/fact) |
| `bootstrap.ky` | ✅ | ✅ | Demo 2+3=5 (NO es self-hosting real) |
| `ky2c.ky` | ✅ | ⚠️ | **Parcial**: genera C pero bugs de runtime lo bloquean |
| `lexer.ky` | ✅ | ❌ | `class` fields → `_call` linker error |
| `llvm.ky` | ✅ | ❌ | Necesita `-lLLVM` al linkear |
| `mir.ky` | ✅ | ❌ | Runtime funcs sin link |
| `keywords.ky` | ❌ | ❌ | No imports entre archivos Kyle |
| `parser.ky` | ❌ | ❌ | Solo expresiones, sin statements |
| `checker.ky` | ❌ | ❌ | Antes: borrow; ahora: `and` parser issue |
| `driver.ky` | ❌ | ❌ | Antes: borrow |
| `chars.ky` | ❌ | ❌ | `^str.get/set` → `_call` |
| `lexer_test.ky` | ✅ | ❌ | Struct ABI (igual que lexer) |

---

## Bugs Restantes en el Compilador Rust (bloquean ky2c.ky)

| # | Bug | Síntoma | Dónde arreglar |
|---|-----|---------|----------------|
| 1 | **Strings en while-loop pierden contenido** | `s = s + x` dentro de `while` produce `s.len()` correcto pero `s` vacío | `kyc_mir/ssa.rs` o `kyc_backend/codegen.rs` |
| 2 | **`!=` en strings compara punteros** | `s[i] != prefix[i]` siempre true aunque chars iguales | `kyc_mir/lower.rs` (no hay overload de `!=` para str) |
| 3 | **`^str.get/set`** genera `_call` | `pos.set(pos.get() + 1)` → `_call` linker error | `kyc_mir/lower.rs` (mutable method dispatch) |
| 4 | **Structs con campo `str`** no se pueden leer | `tok.text` produce output vacío | `kyc_backend/codegen.rs` (struct field str ABI) |
| 5 | **`^{str}` lista** da valores basura | `items.push("hello"); items.get(0)` → garbage | `kyc_backend/codegen.rs` (str→i64 cast) |
| 6 | **`extern fn` retorna `str`** produce `_call` | `ky_read_str()` espera `ptr`, Kyle espera `str` | ABI mismatch entre runtime y type system |

**El bug #1 es el bloqueador principal de ky2c.ky.**

---

## Pipeline Real para Zero Rust

```
HOY: .ky ─▶ Rust parser ─▶ Rust types ─▶ Rust MIR ─▶ Rust LLVM ─▶ binary
              (kyc_frontend) (kyc_semantic) (kyc_mir) (kyc_backend)
              + Rust runtime (kyc_runtime)

FALTAN 6 BUGS DEL COMPILADOR RUST
↓

PASO 1: Arreglar bug #1 (while-loop strings)
  → ky2c.ky puede correr sin acumular en loops
  → genera C correcto para input de una línea

PASO 2: Arreglar bugs #2-#6 
  → ky2c.ky puede compilar archivos completos
  → genera C correcto para cualquier input

PASO 3: ky2c.ky se compila a sí mismo
  → cat ky2c.ky | ky run ky2c.ky > ky2c.c
  → clang ky2c.c -o ky2c
  → ./ky2c < ky2c.ky > ky2c2.c
  → cmp ky2c.c ky2c2.c  # IDÉNTICO → KENO! Punto fijo alcanzado

PASO 4: Portar runtime a Kyle
  → Reemplazar kyc_runtime/ (Rust) con runtime.ky (Kyle)
  → Usar @link "c" + extern fn para syscalls

PASO 5: Cero Rust
  → ky (Rust) compila ky2c.ky → ky2c binario
  → ky2c compila ky_compiler.ky → ky binario
  → ky (Kyle) compila su propio código fuente
```

## Próximo Paso Inmediato

**Arreglar bug #1**: strings en while-loop pierden contenido.

Es un bug en `kyc_mir/ssa.rs` o `kyc_backend/codegen.rs`. La string se crea correctamente (length correcto) pero el pointer a datos se pierde. Probablemente el SSA mem2reg o la promoción de alloca está matando el buffer de la string.

Crear repro mínimo y arreglarlo. Eso destraba ky2c.ky.
