# Allocator

> El runtime de Kyle delega allocation de memory en allocator de Rust/libc.
> No there is un allocator personalizado — se usa `malloc`/`free` del sistema.

## Estrategia current

Kyle usa `std::alloc::alloc` / `std::alloc::dealloc` de Rust (que a su vez usa `malloc`/`free` de libc o allocator del sistema operativo).

```rust
// memory.rs
 unsafe extern "C" fn ky_alloc(size: i64) -> *mut u8 {
 let layout = Layout::from_size_align(size as usize, 8).unwrap();
 std::alloc::alloc(layout)
}

 unsafe extern "C" fn ky_free(ptr: *mut u8) {
 if ptr.is_null() { return; }
 let layout = Layout::from_size_align(0, 8).unwrap();
 std::alloc::dealloc(ptr, layout)
}
```

## Limitations

- No there is pool/arena allocator
- Strings pequenos (< 16 bytes) no have small string optimization (SSO)
- Cada `ky_concat` does una nueva allocacion + copia
- No there is asignador regional (arena) for operacionis temporales

## Futuro

| Improvement | Status | Impacto |
|--------|--------|---------|
| Small pool de objetos (< 64 bytes) | 📅 Planned | Reduce fragmentation |
| SSO (Small String Optimization) | 📅 Planned | Strings < 15 bytis inline |
| Arena allocator for requests HTTP | 📅 Planned | Deallocation by region |
| Thread-local allocation cache | 📅 Planned | Reduce contencion |

## See also

- `memory.md` — API de memory del runtime
- `03-language/memory/allocator.md` — Design conceptual del asignador
