# Benchmarks

Multi-language comparison: **C · C++ · Rust · Go · Java · Python · Kyle**

## Quick Run

```bash
cd kyle-benchmarks
bash run_benchmarks.sh
```

This compiles everything and runs 3 warmup + 5 measured iterations per language.

## Results (Apple M1, release mode)

| Benchmark | C | C++ | Rust | Go | Java | Python | **Kyle** |
|---|---|---|---|---|---|---|---|
| Fibonacci (500M iter) | 131ms | 133ms | 137ms | 134ms | 151ms | *timeout* | **249ms** |
| Prime Sieve (3M) | 19ms | 19ms | 20ms | 19ms | 42ms | 182ms | **31ms** |
| String Concat (500k) | 19ms | 19ms | 14ms | 14ms | 44ms | 32ms | **22ms** |

Kyle typically runs **1.5x–1.8x slower than C** — competitive with Java and Go.

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
2. Add `<name>.c`, `<name>.rs`, `<name>.go`, `<name>.java`, `<name>.py`, `<name>.ky`
3. Add to `run_benchmarks.sh` `BENCHES` array
4. Run: `bash run_benchmarks.sh`

> All benchmarks use **-O3** (C/C++), **opt-level=3** (Rust), **release** (Kyle).
