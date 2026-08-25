# Benchmarks

Multi-language comparison: **C · C++ · Rust · Kyle**

## Quick Run

```bash
cd kyle-benchmarks
bash run_bench_local.sh
```

This compiles everything and runs 3 warmup + 5 measured iterations per language.

## Results (Apple M5, release mode)

| Benchmark | C | C++ | Rust | **Kyle** | Kyle vs C |
|---|---|---|---|---|---|
| Fibonacci (500M iter) | 113ms | 114ms | 114ms | **225ms** | 2.0x |
| Prime Sieve (3M) | 7.4ms | 7.4ms | 7.6ms | **18ms** | 2.4x |
| String Concat (500k) | 7.3ms | 7.8ms | 1.7ms | **8.9ms** | 1.2x |
| MatMul (100x100x10) | 6.1ms | 6.1ms | 6.2ms | **5.6ms** | 0.9x |

**Key findings:**
- **MatMul: Kyle matches/beats C/Rust** — nested loops + array access are native speed
- **String Concat: near-identical to C (1.2x)** — fixed the `str_builder` allocator mismatch
- **Fibonacci: 2x slower than C** — loop overhead from runtime type checking
- **Prime Sieve: 2.4x slower** — list operations have bounds-checking overhead

> **String Concat improvement**: was **8.2x slower** before the `ky_str_builder_new`
> allocator fix. The builder struct was allocated with Rust's `Box` (which places a
> Rust allocator header before the pointer), but the generic string free path
> (`ky_free`) expected a `ky_alloc` header — the resulting mismatched deallocate
> took ~48ms. Allocating the builder via `ky_alloc` fixed it, bringing concat from
> 58ms → 9ms.

## Individual Benchmarks

### Fibonacci (`kyle-benchmarks/fib/`)

Iterative Fibonacci, 500 million iterations. Tests loop + arithmetic performance.

```bash
ky run kyle-benchmarks/fib/fib.ky
# Expected: 1192085431 (500Mth Fib number)
```

### Prime Sieve (`kyle-benchmarks/primes/`)

Sieve of Eratosthenes up to 3 million. Tests array operations + memory.

```bash
ky run kyle-benchmarks/primes/primes.ky
# Expected: 216816 (count of primes up to 3M)
```

### String Concat (`kyle-benchmarks/concat/`)

String builder appending 500k "x" characters. Tests string/buffer operations.

```bash
ky run kyle-benchmarks/concat/concat.ky
# Expected: 500000 (final string length)
```

## Adding a New Benchmark

1. Create a directory `kyle-benchmarks/<name>/`
2. Add `<name>.c`, `<name>.cpp`, `<name>.rs`, `<name>.ky`
3. Add to `run_benchmarks.sh` `BENCHES` array
4. Run: `bash run_benchmarks.sh`

> All benchmarks use **-O3** (C/C++), **opt-level=3** (Rust), **LLVM release** (Kyle).
