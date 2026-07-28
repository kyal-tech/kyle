#!/bin/bash
# benchmark.sh — Kyle vs Rust vs C
set -eu

KY_SRC="$(cd "$(dirname "$0")/.." && pwd)"
KY="$KY_SRC/target/release/ky"
RESULTS="$KY_SRC/BENCHMARK.md"
TMPDIR="/tmp/ky_bench_$$"
mkdir -p "$TMPDIR"
cd "$TMPDIR"

echo "# Kyle Benchmark Results" > "$RESULTS"
echo "" >> "$RESULTS"
echo "Date: $(date)" >> "$RESULTS"
echo "Machine: $(uname -a)" >> "$RESULTS"
echo "" >> "$RESULTS"

bench() {
    local binary=$1; shift
    if [ ! -x "$binary" ]; then echo "BIN_NOT_FOUND|"; return; fi
    local start_ns=$(python3 -c 'import time; print(time.time_ns())')
    local output=$($binary "$@" 2>/dev/null || echo "ERR")
    local end_ns=$(python3 -c 'import time; print(time.time_ns())')
    local elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
    echo "${elapsed_ms}ms|$output"
}

build_ky() {
    local src=$1; local dir=$(dirname "$src"); local base=$(basename "$src" .ky)
    (cd "$dir" && "$KY" build "$src" >/dev/null 2>&1) || true
    echo "$dir/target/debug/$base"
}

build_rs() {
    local src=$1; local base=$(basename "$src" .rs)
    rustc -O "$src" -o "$TMPDIR/${base}_rs" 2>/dev/null; echo "$TMPDIR/${base}_rs"
}

build_c() {
    local src=$1; local base=$(basename "$src" .c)
    clang -O3 "$src" -o "$TMPDIR/${base}_c" -lm 2>/dev/null; echo "$TMPDIR/${base}_c"
}

header() {
    echo "| Lang | Time | Result |" >> "$RESULTS"
    echo "|------|------|--------|" >> "$RESULTS"
}

run3() {
    local name=$1 ky=$2 rs=$3 c=$4 args=${5:-}
    echo "" >> "$RESULTS"
    echo "## $name" >> "$RESULTS"
    header
    echo "| Kyle | $(bench "$ky" $args) |" >> "$RESULTS"
    echo "| Rust | $(bench "$rs" $args) |" >> "$RESULTS"
    echo "| C    | $(bench "$c" $args) |" >> "$RESULTS"
}

# ═══════════════════════════════════════════════════
# 1. PRIMES
# ═══════════════════════════════════════════════════
cp "$KY_SRC/examples/bench/primes.ky" "$TMPDIR/."
cp "$KY_SRC/examples/bench/primes.rs" "$TMPDIR/."

cat > "$TMPDIR/primes.c" << 'CEOF'
#include <stdio.h>
int is_prime(int n) {
    if (n < 2) return 0;
    for (int i = 2; i * i <= n; i++)
        if (n % i == 0) return 0;
    return 1;
}
int main() {
    int count = 0;
    for (int n = 2; n < 3000000; n++)
        if (is_prime(n)) count++;
    printf("%d\n", count);
    return 0;
}
CEOF

echo "## 1. Primes (3M)" >> "$RESULTS"

KY_P=$(build_ky "$TMPDIR/primes.ky")
RS_P=$(build_rs "$TMPDIR/primes.rs")
C_P=$(build_c "$TMPDIR/primes.c")

echo "### Compilation" >> "$RESULTS"
echo "| Metric | Kyle | Rust | C |" >> "$RESULTS"
echo "|--------|------|------|---|" >> "$RESULTS"
echo "| Binary size | $(stat -f%z "$KY_P" 2>/dev/null) | $(stat -f%z "$RS_P" 2>/dev/null) | $(stat -f%z "$C_P" 2>/dev/null) |" >> "$RESULTS"
strip "$KY_P" 2>/dev/null || true; strip "$RS_P" 2>/dev/null || true; strip "$C_P" 2>/dev/null || true
echo "| Stripped | $(stat -f%z "$KY_P" 2>/dev/null) | $(stat -f%z "$RS_P" 2>/dev/null) | $(stat -f%z "$C_P" 2>/dev/null) |" >> "$RESULTS"

echo "" >> "$RESULTS"
echo "### Execution" >> "$RESULTS"
header
echo "| Kyle | $(bench "$KY_P" 3000000) |" >> "$RESULTS"
echo "| Rust | $(bench "$RS_P" 3000000) |" >> "$RESULTS"
echo "| C    | $(bench "$C_P" 3000000) |" >> "$RESULTS"

echo "" >> "$RESULTS"
echo "### Compilation Memory" >> "$RESULTS"
echo "| Lang | Peak Memory |" >> "$RESULTS"
echo "|------|-------------|" >> "$RESULTS"
echo "| Kyle | $(/usr/bin/time -l "$KY" build "$TMPDIR/primes.ky" 2>&1 | grep 'peak memory' | awk '{print $1}') |" >> "$RESULTS"
echo "| Rust | $(/usr/bin/time -l rustc -O "$TMPDIR/primes.rs" -o /dev/null 2>&1 | grep 'peak memory' | awk '{print $1}') |" >> "$RESULTS"
echo "| C    | $(/usr/bin/time -l clang -O3 "$TMPDIR/primes.c" -o /dev/null 2>&1 | grep 'peak memory' | awk '{print $1}') |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
# 2. FIBONACCI
# ═══════════════════════════════════════════════════
cat > "$TMPDIR/fib.ky" << 'KYEOF'
fn fib(n: i32) i32:
    if n <= 1:
        n
    else:
        fib(n - 1) + fib(n - 2)

fn main() i32:
    r = fib(40)
    print(r)
    print("\n")
    0
KYEOF
cat > "$TMPDIR/fib.rs" << 'RSEOF'
fn fib(n: i32) -> i32 {
    if n <= 1 { n } else { fib(n-1) + fib(n-2) }
}
fn main() { println!("{}", fib(40)); }
RSEOF
cat > "$TMPDIR/fib.c" << 'CEOF'
#include <stdio.h>
int fib(int n) { return n <= 1 ? n : fib(n-1) + fib(n-2); }
int main() { printf("%d\n", fib(40)); return 0; }
CEOF

run3 "## 2. Fibonacci 40" "$(build_ky "$TMPDIR/fib.ky")" "$(build_rs "$TMPDIR/fib.rs")" "$(build_c "$TMPDIR/fib.c")"

# ═══════════════════════════════════════════════════
# 3. STRING CONCAT
# ═══════════════════════════════════════════════════
cat > "$TMPDIR/strcat.ky" << 'KYEOF'
fn main() i32:
    s = ""
    i: &i32 = 0
    while i < 100000:
        s = s + "x"
        i = i + 1
    print(len(s))
    print("\n")
    0
KYEOF
cat > "$TMPDIR/strcat.rs" << 'RSEOF'
fn main() {
    let mut s = String::new();
    for _ in 0..100000 { s.push('x'); }
    println!("{}", s.len());
}
RSEOF
cat > "$TMPDIR/strcat.c" << 'CEOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main() {
    char *s = malloc(100001);
    if (!s) return 1;
    for (int i = 0; i < 100000; i++) s[i] = 'x';
    s[100000] = 0;
    printf("%zu\n", strlen(s));
    free(s);
    return 0;
}
CEOF

run3 "## 3. String concat (100k)" "$(build_ky "$TMPDIR/strcat.ky")" "$(build_rs "$TMPDIR/strcat.rs")" "$(build_c "$TMPDIR/strcat.c")"

# ═══════════════════════════════════════════════════
# 4. LIST/VECTOR PUSH
# ═══════════════════════════════════════════════════
cat > "$TMPDIR/list.ky" << 'KYEOF'
fn main() i32:
    lst = []
    i: &i32 = 0
    while i < 100000:
        push(lst, i)
        i = i + 1
    print(list_len(lst))
    print("\n")
    0
KYEOF
cat > "$TMPDIR/list.rs" << 'RSEOF'
fn main() {
    let mut v = Vec::new();
    for i in 0..100000 { v.push(i); }
    println!("{}", v.len());
}
RSEOF
cat > "$TMPDIR/list.c" << 'CEOF'
#include <stdio.h>
#include <stdlib.h>
int main() {
    int cap = 4, len = 0;
    int *v = malloc(cap * sizeof(int));
    if (!v) return 1;
    for (int i = 0; i < 100000; i++) {
        if (len >= cap) { cap *= 2; v = realloc(v, cap * sizeof(int)); }
        v[len++] = i;
    }
    printf("%d\n", len);
    free(v);
    return 0;
}
CEOF

run3 "## 4. List/Vector push (100k)" "$(build_ky "$TMPDIR/list.ky")" "$(build_rs "$TMPDIR/list.rs")" "$(build_c "$TMPDIR/list.c")"

# ═══════════════════════════════════════════════════
# 5. MANDELBROT
# ═══════════════════════════════════════════════════
cp "$KY_SRC/examples/bench/mandelbrot.ky" "$TMPDIR/."
cp "$KY_SRC/examples/bench/mandelbrot.rs" "$TMPDIR/."
cat > "$TMPDIR/mandelbrot.c" << 'CEOF'
#include <stdio.h>
#include <complex.h>
#include <math.h>
int main() {
    int count = 0;
    for (int y = -90; y < 90; y++) {
        for (int x = -90; x < 90; x++) {
            double zx = 0, zy = 0, cx = x / 90.0, cy = y / 90.0;
            int iter = 0;
            while (iter < 1000 && zx*zx + zy*zy < 4.0) {
                double tmp = zx*zx - zy*zy + cx;
                zy = 2.0*zx*zy + cy;
                zx = tmp;
                iter++;
            }
            if (iter < 1000) count++;
        }
    }
    printf("%d\n", count);
    return 0;
}
CEOF

run3 "## 5. Mandelbrot (float)" "$(build_ky "$TMPDIR/mandelbrot.ky")" "$(build_rs "$TMPDIR/mandelbrot.rs")" "$(build_c "$TMPDIR/mandelbrot.c")"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "---" >> "$RESULTS"
echo "## Features NOT benchmarked" >> "$RESULTS"
echo "| Feature | Why | ETA |" >> "$RESULTS"
echo "|---------|-----|-----|" >> "$RESULTS"
echo "| SIMD (AVX, NEON) | Kyle no expone intrinsics | 📅 Fase 19 |" >> "$RESULTS"
echo "| Concurrency (threads, mutex) | No implementado en Kyle | 📅 Fase D |" >> "$RESULTS"
echo "| Async/Await | Runtime existe, no expuesto | 📅 Fase D |" >> "$RESULTS"
echo "| TCP/UDP/HTTP Server | Packages en desarrollo | 📅 Fase 4-5 |" >> "$RESULTS"
echo "| HashMap avanzado | Dict → i64 limitado | 📅 Fase C |" >> "$RESULTS"
echo "| Regex/Crypto/Compression | No existen packages | 📅 Futuro |" >> "$RESULTS"
echo "| PGO | No implementado en toolchain | 📅 Futuro |" >> "$RESULTS"
echo "| Cache Miss, IPC, Branch Miss | Requiere perf (Linux) | 📅 Futuro |" >> "$RESULTS"
echo "| WebSocket/SSE | No implementado | 📅 Fase 5 |" >> "$RESULTS"
echo "| LLVM Vectorization control | Kyle no expone atributos | 📅 Futuro |" >> "$RESULTS"
echo "| Arena/Pool allocators | No implementados | 📅 Futuro |" >> "$RESULTS"
echo "" >> "$RESULTS"

rm -rf "$TMPDIR"
echo "Done. Results: $RESULTS"
