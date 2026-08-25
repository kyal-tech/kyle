# Benchmarks

Multi-language comparison: **C · C++ · Rust · Kyle**

## Quick Run

```bash
cd kyle-benchmarks
bash run_bench_local.sh
```

This compiles everything and runs 3 warmup + 5 measured iterations per language.

## Benchmark History

> Snapshot log to track performance regressions/improvements over time.
> Latest snapshot is on top.

### 2026-08-25 (v0.8.8, Apple M5, release, median of 15 runs)

After inlining `ky_bytes_get`/`ky_bytes_set` (dense 1-byte buffer). Kyle/C ratio:

| Benchmark | C | C++ | Rust | **Kyle** | Kyle vs C |
|---|---|---|---|---|---|
| Fibonacci (500M iter) | 125ms | 121ms | 118ms | **128ms** | 1.02x |
| Prime Sieve (3M) | 7.9ms | 8.0ms | 8.3ms | **9.5ms** | 1.20x |
| String Concat (500k) | 8.3ms | 7.5ms | 1.9ms | **9.7ms** | 1.16x |
| MatMul (100x100x10) | 6.4ms | 6.4ms | 6.5ms | **6.1ms** | 0.95x |

> **Prime Sieve**: dropped from **3.2x → 1.20x** by switching to a dense 1-byte
> buffer (`ky_bytes_get`/`ky_bytes_set` inlined in codegen). The 8-byte `KlList`
> previously spilled 24MB out of L2 cache.

### 2026-08-25 (v0.8.8, Apple M5, release, median of 9 runs)

Baseline after the `str_builder` allocator fix. Kyle/C ratio per benchmark:

| Benchmark | C | C++ | Rust | **Kyle** | Kyle vs C |
|---|---|---|---|---|---|
| Fibonacci (500M iter) | 124ms | 126ms | 129ms | **131ms** | 1.06x |
| Prime Sieve (3M) | 8.5ms | 9.6ms | 9.0ms | **27ms** | 3.2x |
| String Concat (500k) | 8.3ms | 8.0ms | 2.0ms | **10.7ms** | 1.29x |
| MatMul (100x100x10) | 7.1ms | 6.7ms | 7.1ms | **6.7ms** | 0.94x |

## Results (Apple M5, release mode, median of 15 runs)

| Benchmark | C | C++ | Rust | **Kyle** | Kyle vs C |
|---|---|---|---|---|---|
| Fibonacci (500M iter) | 125ms | 121ms | 118ms | **128ms** | 1.02x |
| Prime Sieve (3M) | 7.9ms | 8.0ms | 8.3ms | **9.5ms** | 1.20x |
| String Concat (500k) | 8.3ms | 7.5ms | 1.9ms | **9.7ms** | 1.16x |
| MatMul (100x100x10) | 6.4ms | 6.4ms | 6.5ms | **6.1ms** | 0.95x |

**Key findings:**
- **MatMul: Kyle beats C/Rust (0.95x)** — nested loops + array access are native speed
- **Fibonacci: near-identical to C (1.02x)** — native `i64` counters compile to a tight loop
- **String Concat: 1.16x** — fixed the `str_builder` allocator mismatch (was 8.2x)
- **Prime Sieve: 1.20x** — dense 1-byte buffer inlined in codegen (was 3.2x)

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
