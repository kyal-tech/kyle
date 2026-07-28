# Atomics

> Operacionis atomicas lock-free for programacion concurrente.

## Status current

| Type | Status | Description |
|------|--------|-------------|
| `atomic_i64` | ✅ Implemented | Entero atomico de 64 bits |
| `atomic_bool` | ✅ Implemented | Booleano atomico |
| `atomic_ptr` | 📅 Planned | Puntero atomico |

## Design propuesto

```ky
counter: atomic_i64 = atomic_i64(0)

# Operacionis atomicas
counter.store(42)
val: i64 = counter.load()
counter.fetch_add(1)
counter.fetch_sub(1)
counter.compare_and_swap(10, 20) # CAS
```

### atomic_bool

```ky
flag: atomic_bool = atomic_bool(false)
flag.store(true)
val: bool = flag.load()
```

### Ordenamiento de memory

```ky
counter.store(42, "relaxed") # without barreras
counter.store(42, "acquire") # acquire semantics
counter.store(42, "release") # release semantics
counter.store(42, "acqrel") # acquire + release
counter.store(42, "seqcst") # sequentially consistent (default)
```

## Verification

Las operacionis atomicas are **lock-free** (no usan mutex internamente).
Son implementeds with instruccionis CPU as `LDXR`/`STXR` en ARM64.

## See also

- `synchronization.md` — Mutex, barriers
- `threads.md` — Threads del SO
