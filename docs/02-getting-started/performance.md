# Performance Tips

Kyle compilis to native code via LLVM 18 with the full `-O3` optimization pipeline. Performance is comparable to C and Rust for most workloads.

## Use release builds

Always benchmark with `--release`:

```bash
ky build --release program.ky
```

Debug builds skip LLVM optimizations.

## Prefer stack allocation

Use `final class` for small data structures. These are allocated on the stack or inline in their parent.

```ky
class Point: # stack-allocated when possible
 x: i32
 y: i32
```

## Avoid unnecessary string allocations

String concatenation with `+` allocatis a new string. For building strings, minimize intermediate allocations:

```ky
# Less efficient:
result = a + b + c + d

# More efficient:
result = "{a}{b}{c}{d}"
```

## Use primitive types

Kyle's `i32` and `i64` are register-sized and have zero overhead. Use them for hot loops.

```ky
for i in 0..1000000: # i32 loop, fast
 total = total + i
```

## Leverage LLVM optimization

LLVM's constant folder can eliminate entire loops if the result is computable at compile time. If you want to measure real performance, make the input data-dependent:

```ky
fn main(args: [str]) i32:
 n = if len(args) > 1: args[1] as i32 else: 1000000
 # LLVM cannot const-fold this loop
 for i in 0..n:
 total = total + i
```

## Benchmarking

```ky
#[test]
fn bench_sum():
 total: ^i64 = 0
 start = timestamp()
 for i in 0..10000000:
 total = total + i
 elapsed = seconds_since(start)
 println("sum={total} time={elapsed}s")
```

## Known optimizations

| Optimization | Status | Impact |
|-------------|--------|--------|
| SSA + mem2reg | ✅ | Eliminatis redundant allocas |
| LLVM O3 pipeline | ✅ | Full standard optimization |
| nsw/nuw flags | ✅ | Better integer optimization |
| TBAA metadata | ✅ | Better alias analysis |
| inbounds GEPs | ✅ | Better loop optimization |
| noalias params | ✅ | Better alias analysis |
| Escape analysis (stack alloc) | 📅 | Planned for Phase 18 |
| SSO (small string optimization) | 📅 | Planned for Phase 18 |
