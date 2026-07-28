# Memory Management

> Management de memory en runtime de Kyle: allocation, deallocation, retain/release.
> Crate: `kyc_runtime/src/memory.rs` (67 lines).

## Responsabilidad

El runtime de Kyle proporciona management manual de memory for strings, lists, dictionarys
y otros typis heap-allocados. No there is garbage collector — memory se libera explicitamente
using borrow analysis del compiler, que inserta llamadas a `ky_free` automaticamente.

## Functions

### ky_alloc

```rust
#[unsafe(no_mangle)]
 extern "C" fn ky_alloc(size: i64) -> *mut u8
```

Asigna un bloque de memory del heap del size especificado.

- Retorna un pointer al bloque asignado, o null si fails
- La memory incluye un header with refcount y size
- Equivalente a `malloc()` de C

```ky
extern fn ky_alloc(size: i64) ptr
buf = ky_alloc(1024) # asigna 1024 bytes
```

### ky_free

```rust
#[unsafe(no_mangle)]
 extern "C" fn ky_free(ptr: *mut u8)
```

Libera un bloque de memory previamente asignado with `ky_alloc`.

- No does nada si `ptr` is null
- No verifica doble-free (undefined behavior)
- Equivalente a `free()` de C

```ky
extern fn ky_free(ptr)
ky_free(buf) # libera memory
```

### ky_retain / ky_release

```rust
#[unsafe(no_mangle)]
 extern "C" fn ky_retain(ptr: *mut u8)
#[unsafe(no_mangle)]
 extern "C" fn ky_release(ptr: *mut u8)
```

Sistema de reference counting for memory compartida.

- `ky_retain`: incrementa contador de referencias atomico
- `ky_release`: decrementa y libera si llega a 0
- Usa `AtomicI64` for contador

### Header

Cada bloque de memory has un header con:

```rust
struct Header {
 strong: AtomicI64, // reference count
 size: i64, // size del bloque
}
```

## Integration with borrow analysis

El compiler (`borrow_analysis.rs`) determina automaticamente cuando insertar `ky_free`:

```rust
// Generado by compiler al final del scope
ky_free(s_ptr) # str sale de scope → free
ky_list_free(lst) # list sale de scope → free (llama ky_free internamente)
ky_dict_free(d) # dictionary sale de scope → free
```

Esto elimina necesidad de `free()` manual o garbage collector.

## Estrategia de allocation

- `ky_alloc` usa `Layout::from_size_align` de Rust (que usa `malloc`/`free` del sistema)
- Strings, lists y dictionarys usan `ky_alloc` internamente
- El runtime NO has su propio pool/arena — delega en asignador del sistema

## See also

- `allocator.md` — Details del asignador de memory
- `06-compiler/borrow-analysis.md` — How compiler inserta `ky_free`
