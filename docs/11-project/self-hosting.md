# Kyle Self-Hosting: Rust → Kyle Migration

> Documento completo del proceso para que el compilador de Kyle esté escrito en Kyle (zero Rust).
> Creado: 2026-07-14 · Última sesión: 14-Jul-2026

---

## 1. Resumen Ejecutivo

**Kyle** es un lenguaje de programación compilado cuyo compilador (`ky`) y runtime (`libkyc_runtime.a`) están escritos en **Rust** (~50k líneas). El objetivo es portear ambos a **Kyle** para que Kyle sea un lenguaje self-hosting: el compilador se compila a sí mismo.

### Estado actual

| Componente | Rust | Kyle | Notas |
|------------|:----:|:----:|-------|
| Compilador (parser, typechecker, MIR, codegen) | ✅ | 🟡 | `ky2c.ky` genera C desde su propio source |
| Runtime (memoria, strings, I/O) | ✅ | ❌ | `alloc.ky` existe pero no linkea |
| CLI (`ky build`, `ky run`) | ✅ | ❌ | `driver.ky` es un stub |
| LLVM codegen | ✅ | 🟡 | `llvm.ky` tiene bindings, no linkea libLLVM |
| **Total** | | | ~1-2 semanas de trabajo para bootstrap |

---

## 2. Lo que HEMOS HECHO (9 Rust compiler fixes)

### 2.1 `str` es Copy type — no requiere `&`

**Archivo**: `crates/kyc_semantic/src/type_checker.rs:836`

**Problema**: Pasar `str` a una función requería `&` explícito: `foo(&msg)`.

**Solución**: Eliminar `"str"` de la lista de exclusión en `is_copy_type`:

```rust
// Antes:
let is_copy_type = type_name.map_or(false, |n| n != "str" && n != "void" && ...)
// Después:
let is_copy_type = type_name.map_or(false, |n| n != "void" && ...)
```

Ahora `str` se comporta como Copy — se pasa sin `&`.

### 2.2 Struct → extern fn ABI

**Archivo**: `crates/kyc_backend/src/codegen.rs` (SSA path + non-SSA path)

**Problema**: Pasar un struct de Kyle a una `extern fn` como `ky_list_push(list: ptr, value: i64)` fallaba porque LLVM esperaba `i64` pero recibía un `StructValue`.

**Solución**: Agregar `StructValue→IntType` cast en ambos paths de call argument preparation (SSA y non-SSA). Store a alloca temporal + `ptrtoint`.

### 2.3 Method aliases: `s.upper()`, `s.lower()`, `str.get(i)`

**Archivo**: `crates/kyc_mir/src/lower.rs`

| Alias | Línea |
|-------|-------|
| `s.upper()` aliased to `ky_str_to_upper` | 3997 |
| `s.lower()` aliased to `ky_str_to_lower` | 4006 |
| `str.get(i)` implemented via `ky_substr` | 4502 |
| `"upper"|"lower"` added to string_locals tracking | 5446, 7494 |

### 2.4 `!=` operator para strings

**Archivo**: `crates/kyc_mir/src/lower.rs:3414-3427`

**Problema**: `!=` usaba `UnaryOp::Not` que hace bitwise NOT. `Not(1)` = `~1` = `-2` (non-zero → TRUE). Pero `"a" != "a"` debería ser FALSE.

**Solución**: Reemplazar con `Xor(ky_eq_str(a,b), 1)` usando `BinaryOp::Xor`:

```rust
let one = ctx.alloc_local("_one", MirType::I32);
ctx.current_block.insts.push(MirInst::Store {
    dest: one,
    value: MirValue::Constant(MirConstant::I32(1)),
});
let neq = ctx.alloc_local("_bin", MirType::I32);
ctx.current_block.insts.push(MirInst::BinaryOp {
    dest: neq,
    op: MirBinaryOp::Xor,
    left: MirValue::Local(result),
    right: MirValue::Local(one),
});
```

**Importante**: `one` debe alocarse ANTES que `neq` para que `ctx.next_local - 1` apunte a `neq` (el resultado), no a `one`.

### 2.5 `print`/`println` con strings — Load antes de strlen

**Archivo**: `crates/kyc_mir/src/lower.rs:5240-5265`

**Problema**: `print(s)` hacía `ky_strlen(s)` sobre el MIR local crudo, que en SSA podía resolverse a un valor stale. El contenido se perdía.

**Solución**: Emitir `Load` PRIMERO, luego usar el valor cargado para `strlen` y `print`:

```rust
// ORDER: Load → strlen → print (antes era strlen → Load → print)
let print_arg = ctx.alloc_local("_parg", MirType::Str);
ctx.current_block.insts.push(MirInst::Load { dest: print_arg, src: *id });
let len_dest = ctx.alloc_local("_strlen", MirType::I32);
ctx.current_block.insts.push(MirInst::Call {
    dest: Some(len_dest), name: "ky_strlen".to_string(),
    args: vec![MirValue::Local(print_arg)],
});
// ... ky_print(print_arg, len_dest)
```

### 2.6 `continue` resetea loop state — PHI back-edge fix

**Archivo**: `crates/kyc_backend/src/codegen.rs:1243`

**Problema**: `continue` dentro de un while loop reseteaba TODAS las variables a su valor inicial.

**Causa raíz**: Al resolver los incomings de PHI nodes para el back-edge, solo se buscaba en `block_vals[pred_bi]` (el bloque predecesor). Cuando `continue` crea un bloque separado sin stores, ese bloque no tenía el valor actualizado.

**Solución**: Buscar en TODOS los `block_vals` anteriores al predecesor:

```rust
let mut val = block_vals[pred_bi].get(&val_id).copied()
    .or_else(|| {
        block_vals[..pred_bi].iter().rev()
            .filter_map(|bv| bv.get(&val_id).copied()).next()
    })
    .or_else(|| { func.const_values.get(&val_id).map(|c| self.constant_to_llvm(c)) })
```

### 2.7 Nested while con `str` param — `resolve_mir!` order fix

**Archivo**: `crates/kyc_backend/src/codegen.rs:586`

**Problema**: `while pos < slen and src[pos] != "\n":` dentro de un loop anidado siempre retornaba `""` (empty), causando loop infinito.

**Causa raíz**: `resolve_mir!` revisaba `block_local_map` PRIMERO. Para el bloque del loop anidado, `block_local_map` apuntaba a un `SsaValueId` que se resolvía al valor INCORRECTO (el de la entrada del loop, no el actualizado).

**Solución**: Para allocas no-promovibles (como `str`), cargar desde `alloca_map` DIRECTAMENTE, saltándose `block_local_map`:

```rust
// Para non-promotable allocas, cargar desde la alloca (memoria real)
if let Some(Some(ptr)) = self.alloca_map.get(*mir_id) {
    if let Some(pointee_type) = self.alloca_types.get(mir_id) {
        self.builder.build_load(*pointee_type, *ptr, "_ssaload")...
    }
} else if let Some(ssa_id) = func.block_local_map.get(bi)... {
    // Solo para promotable
}
```

### 2.8 `str` como move type en MIR (no SSA promotion)

**Archivo**: `crates/kyc_mir/src/mir.rs:181`, `crates/kyc_mir/src/ssa.rs:749`

**Problema**: Cambiamos `is_move_type(Str) = false` para arreglar borrows, pero eso causó que `str` allocas fueran promovibles a SSA, rompiendo el tracking de strings a través de while loops.

**Solución**: Revertir a `MirType::Str => true`. Así `str` allocas NO se promueven a SSA y permanecen como allocas de memoria real. El type checker sigue tratando `str` como Copy (no requiere `&`), pero el MIR lo trata como move type para prevent SSA promotion.

### 2.9 `print(s)` con strings acumulados — skip `block_local_map`

**Archivo**: `crates/kyc_backend/src/codegen.rs:586-602`

**Problema**: `print(s)` donde `s` se acumuló en un while loop imprimía vacío. `ky_strlen` retornaba 0 porque resolvía `s` a un valor stale de `block_local_map` (el del bloque de entrada, no el acumulado).

**Solución**: Misma que 2.7 — para non-promotable allocas, cargar desde alloca directamente.

---

## 3. ky2c.ky — El compilador Kyle→C en Kyle

### 3.1 Ubicación

`runtimes/ky/ky2c.ky` — 180 líneas

### 3.2 Arquitectura

```
Source .ky ──▶ ky2c.ky (Kyle) ──▶ C code ──▶ clang ──▶ binary
```

ky2c.ky es un compilador de **traducción directa**: lee código Kyle línea por línea y emite C equivalente.

### 3.3 Cómo funciona

1. Lee stdin línea por línea (con manejo de blank lines: 3 consecutivas = EOF)
2. Acumula todo en `src`
3. Procesa `src` carácter por carácter en un solo while loop
4. Para cada línea:
   - Si es `# comentario` → emite línea en blanco
   - Si es `fn name():` → emite `void name() {`
   - Si es `return expr:` → emite `return expr;`
   - Si es `if cond:` → emite `if (cond) {`
   - Si es `elif cond:` → emite `} else if (cond) {`
   - Si es `else:` → emite `} else {`
   - Si es `while cond:` → emite `while (cond) {`
   - Si empieza con `println(` → emite `_pl(...)`
   - Si empieza con `print(` → emite `_p(...)`
   - Si no: emite la línea tal cual con `;`

### 3.4 Transforms implementados

| Kyle | C | Estado |
|------|---|--------|
| `println(x)` | `_pl(x)` | ✅ |
| `print(x)` | `_p(x)` | ✅ |
| `input()` | `ky_input()` | ✅ |
| `true` | `1` | ✅ |
| `false` | `0` | ✅ |
| `not x` | `!x` | ✅ |
| `x and y` | `x && y` | ✅ |
| `x or y` | `x \|\| y` | ✅ |
| `x + y` (string concat) | `_c(x, y)` | ❌ SIGSEGV en scanning loop |
| `x: str = "..."` | `char* x = "..."` | ❌ SIGSEGV en scanning loop |
| `x: i32 = 0` | `long long x = 0` | ❌ SIGSEGV en scanning loop |
| `x.len()` | `_len(x)` | ❌ No implementado |

### 3.5 Preamble generado

```c
extern char* ky_input();
extern char* ky_concat(char* a, int a_len, char* b, int b_len);
extern int ky_strlen(char* s);
extern int ky_eq_str(char* a, char* b);
extern void ky_print(char* s, int len);
extern void ky_println(char* s, int len);
extern char* ky_i64_to_str(long long v);

char* _s(long long v) { return ky_i64_to_str(v); }
char* _c(char* a, char* b) { return ky_concat(a, ky_strlen(a), b, ky_strlen(b)); }
int _eq(char* a, char* b) { return ky_eq_str(a, b); }
int _ne(char* a, char* b) { return !ky_eq_str(a, b); }
int _len(char* s) { return ky_strlen(s); }
void _p(char* s) { ky_print(s, ky_strlen(s)); }
void _pl(char* s) { ky_println(s, ky_strlen(s)); }
```

### 3.6 Output actual

```
cat ky2c.ky | ./target/debug/ky2c → 271 líneas de C
```

Incluye: preamble, `void main() {`, bodies con transforms básicos.

---

## 4. Lo que FALTA para zero Rust

### 4.1 Pipeline completo

```
FUENTE .ky
  │
  ▼
ky2c.ky (Kyle→C) ─── genera .c
  │
  ▼
clang + libkyc_runtime.a (Rust) ─── genera binary
  │
  ▼
BINARY .kyc (puede compilar .ky a .c)
```

Para **zero Rust**, necesitamos:

```
FUENTE .ky
  │
  ▼
ky2c.ky (Kyle) ─── genera .c
  │
  ▼
clang (sin Rust runtime) ─── genera binary
```

### 4.2 Tareas pendientes (orden priorizado)

#### PRIORIDAD 1: Arreglar SIGSEGV en scanning loops (~4 hrs)

Los 3 transforms que causan SIGSEGV:
- Type inference (detectar `=` y escanear RHS para `"` o `input`)
- Concat count (contar `+` en expresión)
- `.len()` detection

**Síntoma**: `while pk < le: if src[pk] == "=" ... pk = pk + 1` crashea con SIGSEGV en `src[pk]`.

**Causa probable**: Algo en el compilador hace que `src[pk]` con `pk` variable dentro de un while loop acceda a memoria inválida. NO ocurre con `src[col]` (índice constante).

**Archivos**: `runtimes/ky/ky2c.ky`, `crates/kyc_backend/src/codegen.rs`

#### PRIORIDAD 2: Bootstrap completo (~1 día)

```
1. ky2c.ky genera C compilable
2. clang out.c -lkyc_runtime -o ky2c
3. cat ky2c.ky | ./ky2c > out2.c
4. diff out.c out2.c  # deben ser idénticos
```

#### PRIORIDAD 3: Portar runtime a Kyle (~1 semana)

Reemplazar cada archivo de `crates/kyc_runtime/src/` con su equivalente Kyle:

| Runtime Rust | Kyle | Archivo |
|-------------|------|---------|
| `memory.rs` | `ky_alloc`, `ky_free` | `runtimes/ky/alloc.ky` |
| `string.rs` | `ky_concat`, `ky_strlen`, etc | ❌ |
| `io.rs` | `ky_print`, `ky_input`, etc | ❌ |
| `list.rs` | `ky_list_new`, `ky_list_push` | ❌ |
| `dict.rs` | `ky_dict_new`, `ky_dict_get` | ❌ |

#### PRIORIDAD 4: CLI en Kyle (~2 días)

Reemplazar `kyc_cli/src/main.rs` con `cli.ky`:
- `ky build` → compilar .ky → .c → clang
- `ky run` → build + ejecutar
- `ky check` → parse + typecheck (usando parser.ky)

#### PRIORIDAD 5: LLVM codegen en Kyle (~2 semanas)

Usar `runtimes/ky/llvm.ky` (bindings a LLVM C API) + `runtimes/ky/codegen.ky` para generar LLVM IR directamente en vez de C.

---

## 5. Referencias

### 5.1 Archivos del compilador (Rust)

| Archivo | Propósito |
|---------|-----------|
| `crates/kyc_cli/src/main.rs` | Entry point CLI |
| `crates/kyc_driver/src/pipeline.rs` | Pipeline de compilación, prelude |
| `crates/kyc_frontend/src/lexer.rs` | Lexer (tokenizer) |
| `crates/kyc_frontend/src/parser.rs` | Parser (AST) |
| `crates/kyc_hir/src/lib.rs` | HIR desugaring |
| `crates/kyc_semantic/src/type_checker.rs` | Type checker |
| `crates/kyc_mir/src/lower.rs` | AST→MIR lowering |
| `crates/kyc_mir/src/ssa.rs` | SSA conversion |
| `crates/kyc_mir/src/borrow_analysis.rs` | Borrow checker |
| `crates/kyc_backend/src/codegen.rs` | LLVM codegen |
| `crates/kyc_backend/src/linker.rs` | Linker |
| `crates/kyc_runtime/src/` | Runtime (28 archivos) |

### 5.2 Archivos del runtime (Rust → eventualmente Kyle)

| Runtime file | Functions | Líneas |
|-------------|-----------|:------:|
| `memory.rs` | `ky_alloc`, `ky_free`, `ky_retain`, `ky_release` | 80 |
| `string.rs` | `ky_concat`, `ky_strlen`, `ky_substr`, `ky_eq_str`, etc | 200 |
| `io.rs` | `ky_print`, `ky_println`, `ky_input`, `ky_read_str` | 150 |
| `list.rs` | `ky_list_new/push/pop/get/set/len` | 250 |
| `dict.rs` | `ky_dict_new/get/set/len/contains` | 180 |
| `net.rs` | `ky_tcp_listen/accept/read/write/close` | 400 |
| `thread.rs` | `ky_spawn_thread/join_thread` | 80 |
| `channel.rs` | `ky_channel_new/send/recv/close` | 100 |
| `datetime.rs`, `uuid.rs`, `json.rs`, etc | Varios | 500 |
| **Total** | | **~2000** |

### 5.3 Archivos Kyle

| Archivo | Propósito | Estado |
|---------|-----------|--------|
| `runtimes/ky/ky2c.ky` | Compilador Kyle→C | ✅ 271 líneas de C generado |
| `runtimes/ky/alloc.ky` | `extern fn` malloc/free | ✅ `ky check` |
| `runtimes/ky/token.ky` | TokenKind enum | ✅ |
| `runtimes/ky/lexer.ky` | Lexer en Kyle | 🟡 `ky check`, runtime errors |
| `runtimes/ky/parser.ky` | Parser en Kyle (expresiones) | 🟡 solo expresiones |
| `runtimes/ky/checker.ky` | Type checker básico | 🟡 solo literales |
| `runtimes/ky/codegen.ky` | LLVM IR textual | 🟡 hardcoded add/fact |
| `runtimes/ky/llvm.ky` | Bindings LLVM C API | ✅ |
| `runtimes/ky/compiler.ky` | Pipeline demo | 🟡 hardcoded |

### 5.4 Documentación relevante

| Documento | Contenido |
|-----------|-----------|
| `STATUS.md` | Estado actual del self-hosting |
| `SELF-HOSTING.md` | **Este documento** — plan de migración |
| `docs/14-self-hosting.md` | Plan original de migración Rust→Kyle |
| `docs/15-kyle-syntax-reference.md` | Referencia completa de sintaxis Kyle |
| `docs/03-language/` | Especificación formal del lenguaje |
| `AGENTS.md` | Contexto del proyecto, comandos, estructura |
| `ROADMAP.md` | Roadmap de features con fases |

### 5.5 Commits relevantes

```
0b7071b FASE 2 COMPLETA: Bootstrap exitoso!
8570ccb FASE 2b: Pipeline source -> LLVM IR funcional
9aa4a56 FASE 2: LLVM IR codegen en Kyle (textual, sin libLLVM)
891d785 FASE 1e: CLI driver en Kyle
e47e18b FASE 1d: LLVM C API bindings en Kyle (150+ extern fn)
8f6304b FASE 1c: MIR textual en Kyle
6a3ec07 FASE 1b: Type checker basico en Kyle
367bdcc FASE 1a: Lexer + Parser (expresiones) en Kyle
```

---

## 6. Glosario

| Término | Significado |
|---------|-------------|
| Self-hosting | El compilador se compila a sí mismo |
| Bootstrap | Proceso de lograr self-hosting (Rust compila Kyle → Kyle compila Kyle) |
| ky2c.ky | Compilador Kyle→C escrito en Kyle |
| `ky` | Compilador actual (Rust) |
| `kyc_runtime.a` | Runtime actual (Rust) |
| PHI node | Instrucción SSA que selecciona valor según camino de control |
| SSA | Static Single Assignment — forma intermedia del compilador |
| Alloca | Stack allocation (memoria local de función) |
| Promotable | Alloca que puede convertirse a SSA (i32, bool, etc.) |
| Non-promotable | Alloca que queda como memoria real (str, list, struct) |
| Move type | Tipo con semántica de ownership transfer (str, list, dict) |
| Copy type | Tipo que se copia al asignarse (i32, bool) |

---

## 7. Comandos útiles

```bash
# Compilar y probar
cargo build --release --bin ky
./target/release/ky build file.ky
./target/debug/file
./target/release/ky run file.ky
./target/release/ky mir file.ky
./target/release/ky check file.ky
cargo test --workspace --exclude kyc_runtime_wasm

# Probar ky2c.ky
./target/release/ky build runtimes/ky/ky2c.ky
printf 'fn test():\n    msg = "hello"\n' | ./target/debug/ky2c

# Self-hosting test
cat runtimes/ky/ky2c.ky | ./target/debug/ky2c > /tmp/out.c
wc -l /tmp/out.c

# Ver estado de fixes
cd /Users/kynera/HCA/KYNERA/kl && git diff --stat

# Ver MIR de un programa
./target/release/ky mir /tmp/test.ky | sed -n '/^fn main/,/^$/p'
```
