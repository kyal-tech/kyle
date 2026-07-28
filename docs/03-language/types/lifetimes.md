# Lifetimes

> Kyle **no has lifetimis explicitos** as Rust. Los borrows se validan
> using borrow checker en tiempo de compilation, pero without anotaciones
> de lifetime en code source.

## How funciona

En Rust, referencias llevan anotacionis de lifetime:

```rust
fn get_first<'a>(s: &'a str) -> &'a str { &s[0..1] }
```

En Kyle, no there is `'a`:

```ky
fn get_first(s: &str) &str: # ❌ PROHIBIDO
 s.substr(0, 1)
```

Kyle **prohibe** devolver referencias. Solo se can devolver valueis owned:

```ky
fn get_first(s: &str) str: # ✅ OWNED, correcto
 s.substr(0, 1)
```

## Borrows as parameters

Los borrows solo can be parameters entrantes, nunca valueis de retorno:

```ky
fn read(s: &str): # ✅ borrow as parameter
 println(s)

fn read_and_return(s: &str) &str: # ❌ prohibido
 s
```

## Limitation

Esta limitation simplifica language pero impide ciertos patronis como
iterators que returnsn referencias a data internos. Para esos cases,
usa `.clone()` o redisena API.

## See also

- `ownership.md` — Reglas de ownership
- `borrow-analysis.md` — Implementation del borrow checker
