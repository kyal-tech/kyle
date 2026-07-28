# Syntax Evolution Roadmap

> Plan completo de implementación de la nueva sintaxis de Kyle.
> Desde la documentación → parser → type checker → MIR → codegen → tests → bootstrap.

---

## Leyenda

| Símbolo | Significado |
|---------|-------------|
| `[x]` | Ya implementado |
| `[~]` | Parcialmente implementado |
| `[ ]` | Pendiente |

---

## Fase 0: Documentación (COMPLETA)

Toda la documentación actualizada con la sintaxis definitiva.

| # | Archivo | Estado | Descripción |
|---|---------|--------|-------------|
| 0.1 | `docs/03-language/syntax/modules.md` | [x] | `use` system: módulo, selectivo `{}`, alias, relativo `~`, directo |
| 0.2 | `docs/03-language/syntax/variables.md` | [x] | `^` `&` `?` `!` modificadores, copy/move, scope, destructuring |
| 0.3 | `docs/03-language/syntax/collections.md` | [x] | **NUEVO**: [T], [T,N], set{T}, {K:V}, queue, stack, deque, linked_list |
| 0.4 | `docs/03-language/syntax/expressions.md` | [x] | Literales `[1,2,3]`, `set{1,2,3}`, `{"a":1}`, `^[]` |
| 0.5 | `docs/03-language/syntax/statements.md` | [x] | For loops con nuevas colecciones |
| 0.6 | `docs/03-language/syntax/README.md` | [x] | Índice con `collections.md` |
| 0.7 | `docs/09-specification/type-system.md` | [x] | Ortogonalidad `?` `!` `^` `&` en TODOS los tipos |
| 0.8 | `docs/15-kyle-syntax-reference.md` | [x] | Referencia completa actualizada |
| 0.9 | `docs/11-project/roadmap.md` | [x] | Roadmap con fases 5-7 |
| 0.10 | `docs/11-project/self-hosting.md` | [x] | Estado actualizado |
| 0.11 | `docs/11-project/syntax-roadmap.md` | [x] | **ESTE DOCUMENTO** |
| 0.12 | `AGENTS.md` | [x] | Contexto completo actualizado |

---

## Fase 1: Parser — Keywords y Tokens (semana 1)

### 1.1 Tokens nuevos

| # | Token | Descripción |
|---|-------|-------------|
| 1.1.1 | `Use` | Keyword `use` para imports |
| 1.1.2 | `As` | Keyword `as` para alias (ya existe) |
| 1.1.3 | `Tilde` | `~` para imports relativos (ya existe) |
| 1.1.4 | `LBrace`/`RBrace` | `{}` para import selectivo (ya existe) |
| 1.1.5 | `Set` | Keyword opcional `set` (o se infiere de `set{T}`) |

### 1.2 Nuevos parse rules

| # | Regla | Código ejemplo | Prioridad |
|---|-------|---------------|-----------|
| 1.2.1 | `use X.Y.Z` | `use std.io` | ALTA |
| 1.2.2 | `use X.Y.{A, B}` | `use std.io.{print, read}` | ALTA |
| 1.2.3 | `use X.Y as alias` | `use std.io as io` | ALTA |
| 1.2.4 | `use ~X.Y` | `use ~utils.helpers` | ALTA |
| 1.2.5 | `use X.Y.Z` (símbolo) | `use std.io.print` | ALTA |
| 1.2.6 | Deprecar `use X.Y` | Remover o mantener como error | MEDIA |

### 1.3 Parse de tipos de colección

| # | Regla | Sintaxis | Prioridad |
|---|-------|----------|-----------|
| 1.3.1 | `[T]` como lista | `items: [i32]` | ALTA |
| 1.3.2 | `[T, N]` como array (existe) | `arr: [i32, 5]` | ✅ YA |
| 1.3.3 | `set{T}` como set | `nums: set<i32>` | ALTA |
| 1.3.4 | `queue{T}` como queue | `q: queue<i32>` | MEDIA |
| 1.3.5 | `stack{T}` como stack | `s: stack<str>` | MEDIA |
| 1.3.6 | `deque{T}` como deque | `dq: deque<i32>` | BAJA |
| 1.3.7 | `linked_list{T}` | `ll: linked_list<str>` | BAJA |

### 1.4 Parse de literales de colección

| # | Literal | Tipo resultante | Prioridad |
|---|---------|-----------------|-----------|
| 1.4.1 | `[1, 2, 3]` | `[i32]` (lista) | ALTA |
| 1.4.2 | `^[1, 2]` | `^[i32]` (lista mutable) | ALTA |
| 1.4.3 | `[]` | `[T]` (lista vacía, T inferido) | ALTA |
| 1.4.4 | `^[]` | `^[T]` (lista mutable vacía) | ALTA |
| 1.4.5 | `set{1, 2, 3}` | `set<i32>` | ALTA |
| 1.4.6 | `{"a": 1}` | `{str: i32}` (existe) | ✅ YA |
| 1.4.7 | `queue{1, 2}` | `queue<i32>` | MEDIA |
| 1.4.8 | `stack{"a"}` | `stack<str>` | MEDIA |
| 1.4.9 | `deque{1}` | `deque<i32>` | BAJA |

---

## Fase 2: AST — Nuevos nodos (semana 2)

### 2.1 Modificaciones en AST

| # | Cambio | Archivo | Prioridad |
|---|--------|---------|-----------|
| 2.1.1 | `Decl::Use` nuevo nodo | `ast/mod.rs` | ALTA |
| 2.1.2 | Remover o deprecar `Decl::FromImport` | `ast/mod.rs` | MEDIA |
| 2.1.3 | `AstType::List(inner)` tipo lista | `ast/mod.rs` | ALTA |
| 2.1.4 | `AstType::Set(inner)` tipo set | `ast/mod.rs` | ALTA |
| 2.1.5 | `AstType::Queue(inner)` tipo queue | `ast/mod.rs` | MEDIA |
| 2.1.6 | `AstType::Stack(inner)` tipo stack | `ast/mod.rs` | MEDIA |
| 2.1.7 | `AstType::Deque(inner)` tipo deque | `ast/mod.rs` | BAJA |
| 2.1.8 | `AstType::LinkedList(inner)` | `ast/mod.rs` | BAJA |
| 2.1.9 | Modificar `AstType::Dict` (ya existe) | `ast/mod.rs` | ✅ |
| 2.1.10 | Modificar `AstType::Array` (ya existe) | `ast/mod.rs` | ✅ |

### 2.2 Expr literales nuevos

| # | Expresión | AST Node |
|---|-----------|----------|
| 2.2.1 | `[1, 2, 3]` | `Expr::List(vec![...])` |
| 2.2.2 | `^[1, 2]` | `Expr::MutableList(vec![...])` |
| 2.2.3 | `set{1, 2}` | `Expr::SetLiteral(vec![...])` |
| 2.2.4 | `queue{1}` | `Expr::QueueLiteral(vec![...])` |
| 2.2.5 | `stack{1}` | `Expr::StackLiteral(vec![...])` |

---

## Fase 3: Type Checker (semana 3)

| # | Tarea | Prioridad |
|---|-------|-----------|
| 3.1 | Resolver `[T]` como tipo lista | ALTA |
| 3.2 | Resolver `set<T>` como tipo set (no confundir con `{K:V}`) | ALTA |
| 3.3 | Resolver `queue<T>`, `stack<T>` como genéricos de std | MEDIA |
| 3.4 | Inferir `[1,2,3]` → `[i32]` (lista) | ALTA |
| 3.5 | Inferir `^[1,2]` → `^[i32]` (lista mutable) | ALTA |
| 3.6 | Inferir `set{1,2,3}` → `set<i32>` | ALTA |
| 3.7 | Validar ortogonalidad: `^[i32]`, `&[str]`, `[i32]?`, `[str]!` | ALTA |
| 3.8 | Validar `^` `&` `?` `!` en sets, queues, etc. | ALTA |
| 3.9 | Inferir `[]` → lista vacía (tipo inferido del uso) | ALTA |
| 3.10 | Inferir `^[]` → lista mutable vacía | ALTA |
| 3.11 | Validar métodos `.push()`, `.pop()`, etc en tipos correctos | ALTA |
| 3.12 | Validar conversiones `.to_list()`, `.to_array()`, `.to_set()` | MEDIA |

---

## Fase 4: MIR Lowering (semanas 4-5)

### 4.1 Lowering de tipos

| # | Tipo MIR | Prioridad |
|---|----------|-----------|
| 4.1.1 | `MirType::List(T)` → `[T]` (reemplazar `{T}` actual) | ALTA |
| 4.1.2 | `MirType::Set(T)` → `set<T>` | ALTA |
| 4.1.3 | `MirType::Dict(K, V)` → `{K:V}` (existe) | ✅ |
| 4.1.4 | `MirType::Queue(T)` → `queue<T>` | MEDIA |
| 4.1.5 | `MirType::Stack(T)` → `stack<T>` | MEDIA |
| 4.1.6 | `MirType::Deque(T)` → `deque<T>` | BAJA |
| 4.1.7 | `MirType::LinkedList(T)` → `linked_list<T>` | BAJA |

### 4.2 Lowering de expresiones

| # | Expresión | Prioridad |
|---|-----------|-----------|
| 4.2.1 | `[1, 2, 3]` → `ky_list_new() + ky_list_push` | ALTA |
| 4.2.2 | `^[1, 2]` → igual pero variable mutable | ALTA |
| 4.2.3 | `[]` → `ky_list_new()` | ALTA |
| 4.2.4 | `set{1, 2}` → `ky_set_new() + ky_set_add` | ALTA |
| 4.2.5 | `queue{1}` → `ky_queue_new() + ky_queue_push` | MEDIA |

### 4.3 Lowering de métodos

| # | Método | Runtime fn | Prioridad |
|---|--------|------------|-----------|
| 4.3.1 | `.push()` en lista | `ky_list_push` | ALTA |
| 4.3.2 | `.pop()` en lista | `ky_list_pop` | ALTA |
| 4.3.3 | `.map()` en lista | `ky_list_map` | ALTA |
| 4.3.4 | `.filter()` en lista | `ky_list_filter` | ALTA |
| 4.3.5 | `.fold()` en lista | `ky_list_fold` | ALTA |
| 4.3.6 | `.contains()` en lista | `ky_list_contains` | ALTA |
| 4.3.7 | `.add()` en set | `ky_set_add` | ALTA |
| 4.3.8 | `.contains()` en set | `ky_set_contains` | ALTA |
| 4.3.9 | `.union()`, `.intersection()` en set | `ky_set_union`, etc | MEDIA |
| 4.3.10 | `.to_list()` en todas | `ky_X_to_list` | ALTA |
| 4.3.11 | `.to_array()` en lista/set | `ky_list_to_array` | MEDIA |
| 4.3.12 | `.to_set()` en lista | `ky_list_to_set` | MEDIA |
| 4.3.13 | `.push()` en queue/stack | `ky_queue_push`, `ky_stack_push` | MEDIA |
| 4.3.14 | `.pop()` en queue/stack | `ky_queue_pop`, `ky_stack_pop` | MEDIA |

### 4.4 Runtime functions nuevas

| # | Función | Descripción | Prioridad |
|---|---------|-------------|-----------|
| 4.4.1 | `ky_set_new()` | Crear set vacío | ALTA |
| 4.4.2 | `ky_set_add(s, val)` | Agregar a set | ALTA |
| 4.4.3 | `ky_set_contains(s, val)` | Check set | ALTA |
| 4.4.4 | `ky_set_remove(s, val)` | Remover de set | ALTA |
| 4.4.5 | `ky_set_len(s)` | Tamaño del set | ALTA |
| 4.4.6 | `ky_set_to_list(s)` | Set → lista | ALTA |
| 4.4.7 | `ky_set_union(a, b)` | Unión de sets | MEDIA |
| 4.4.8 | `ky_set_intersection(a, b)` | Intersección | MEDIA |
| 4.4.9 | `ky_queue_new()` | Crear queue | MEDIA |
| 4.4.10 | `ky_queue_push(q, val)` | Enqueue | MEDIA |
| 4.4.11 | `ky_queue_pop(q)` | Dequeue | MEDIA |
| 4.4.12 | `ky_stack_new()` | Crear stack | MEDIA |
| 4.4.13 | `ky_stack_push(s, val)` | Push stack | MEDIA |
| 4.4.14 | `ky_stack_pop(s)` | Pop stack | MEDIA |

---

## Fase 5: Codegen (semana 5)

| # | Tarea | Prioridad |
|---|-------|-----------|
| 5.1 | `MirType::List` → LLVM `%list = type { i8*, i64, i64 }` | ALTA |
| 5.2 | `MirType::Set` → LLVM `%set = type { i8*, i64, i64 }` | ALTA |
| 5.3 | `MirType::Queue` → LLVM como struct opaco | MEDIA |
| 5.4 | `MirType::Stack` → LLVM como struct opaco | MEDIA |
| 5.5 | Llamadas a runtime functions nuevas | ALTA |

---

## Fase 6: Tests (semana 6)

| # | Test | Descripción | Prioridad |
|---|------|-------------|-----------|
| 6.1 | Actualizar `06_collections.ky` | Nueva sintaxis `[T]`, `set{T}`, métodos | ALTA |
| 6.2 | Actualizar `11_operators.ky` | Si hay cambios | BAJA |
| 6.3 | **NUEVO** `14_imports.ky` | Sistema `use` completo | ALTA |
| 6.4 | **NUEVO** `15_conversions.ky` | `to_list()`, `to_array()`, `to_set()` | MEDIA |
| 6.5 | **NUEVO** `16_orthogonality.ky` | `^` `&` `?` `!` en todos los tipos | ALTA |
| 6.6 | **NUEVO** `17_queue_stack.ky` | Queue, Stack, Deque básico | MEDIA |
| 6.7 | Workspace tests | `cargo test --workspace` pasa | ALTA |

---

## Fase 7: Bugs Bloqueantes Self-Hosting (semanas 6-7)

| # | Bug | Síntoma | Causa | Prioridad |
|---|-----|---------|-------|-----------|
| 7.1 | #7: `elif` + strings → `ky_free` crash | Cualquier `elif name == "x":` crashea | SSA promotion de string clones con `ky_free` en distinto block | 🔴 |
| 7.2 | #1: while-loop strings pierden contenido | `s = s + "x"` dentro de `while` — `s.len()` ok, datos vacíos | SSA/non-promotable en restauración dominador | 🔴 |
| 7.3 | struct str field → lectura vacía | Structs con campo `str` producen string vacío | ABI struct layout en LLVM codegen | 🔴 |
| 7.4 | `^{str}` listas con garbage | `list<str>` con strings literales produce valores basura | str→i64 cast inverso en `ky_list_get` | 🟡 |
| 7.5 | `^str.get/set` → `_call` linker error | `pos.set(pos.get()+1)` falla | Method dispatch para tipos mutables | 🟡 |
| 7.6 | `extern fn` retorna `str` → `_call` error | `extern fn foo() str:` no linkea | ABI mismatch runtime ptr vs str | 🟢 |

---

## Fase 8: Bootstrap Self-Hosting (semanas 7-9)

| # | Tarea | Depende de | Prioridad |
|---|-------|-----------|-----------|
| 8.1 | ky2c.ky genera C correcto | Bugs 7.1-7.3 | 🔴 |
| 8.2 | ky2c.ky compila ky2c.ky → output idéntico | 8.1 | 🔴 |
| 8.3 | lexer.ky runtime funcional | Bug 7.3 (struct str field) | 🔴 |
| 8.4 | token.ky + lexer.ky integrados | 8.3 | 🔴 |
| 8.5 | parser.ky type-check + runtime | 8.4 | 🔴 |
| 8.6 | checker.ky funcional | 8.5 | 🔴 |
| 8.7 | llvm.ky bindings funcionales | — | 🟡 |
| 8.8 | codegen.ky con llvm.ky | 8.7 | 🟡 |

---

## Fase 9: Zero Rust (semanas 9-12)

| # | Tarea | Descripción | Prioridad |
|---|-------|-------------|-----------|
| 9.1 | Runtime en Kyle | Portar `ky_alloc`, `ky_free`, strings, listas, dicts a Kyle | 🔴 |
| 9.2 | CLI en Kyle | `ky build`, `ky run`, `ky check` en Kyle | 🔴 |
| 9.3 | LSP en Kyle | Language server en Kyle | 🟡 |
| 9.4 | Package manager en Kyle | `ky add`, `ky install`, etc | 🟡 |
| 9.5 | Formateador en Kyle | `ky fmt` en Kyle | 🟢 |

---

## Resumen de sintaxis (cheat sheet final)

```ky
# ===== IMPORTS =====
use std.io                          # io.print()
use std.io.{print, read}            # print() directo
use std.io as io                    # alias
use ~utils.helpers                  # relativo
use std.io.print                    # símbolo directo

# ===== LISTAS [T] =====
items = [1, 2, 3]                   # [i32] inferido
items: ^[str] = ["a", "b"]          # mutable
items = ^[]                         # mutable vacío
items: &[i32] = &[1, 2]            # borrow
items: ^&[i32]                      # mutable borrow

items.push(4)
items.pop()                         # Option<T>
items[0]
items.get(0)                        # Option<T>
items.map(fn(x) x * 2)
items.filter(fn(x) x > 0)
items.fold(0, fn(acc, x) acc + x)
items.to_array()
items.to_set()

# ===== ARRAYS [T, N] =====
arr: [i32, 5] = [1, 2, 3, 4, 5]
arr.len()                           # 5
arr[0] = 99
arr.to_list()

# ===== SETS set{T} =====
nums = set{1, 2, 3}
nums: ^set<i32> = set()
nums.add(4)
nums.contains(2)
nums.remove(2)
nums.to_list()

# ===== DICTS {K: V} =====
d = {"a": 1}
d: ^{str: i32} = {}
d["key"]
d.get("key")                        # Option<V>
d.has("key")
d.keys()
d.values()

# ===== QUEUE queue{T} =====
q = queue{1, 2, 3}
q: ^queue<i32> = queue()
q.push(4)
val = q.pop()                       # Option<T>

# ===== STACK stack{T} =====
s = stack{"a", "b"}
s: ^stack<str> = stack()
s.push("c")
top = s.pop()

# ===== ORTOGONALIDAD =====
x: ^[i32]           # lista mutable
x: &[str]           # borrow de lista
x: ^&[i32]!         # mutable borrow con error
x: ^&[str]?         # mutable borrow opcional
x: ^{str: i32}?     # dict mutable opcional
x: ^set<i32>!       # set mutable con error
x: ^queue<i32>?     # queue mutable opcional
```
