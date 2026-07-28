# Threads

> Threads del sistema operativo for paralelismo real.
> Kyle expone threads del SO using `spawn_thread` / `join_thread`.

## Uso basico

```ky
fn worker(n: i64) i64:
 n * 2

fn main() i32:
 h: i64 = spawn_thread(worker as ptr, 21)
 r: i64 = join_thread(h)
 println(r.to_str()) # 42
 0
```

## Multiples threads

```ky
fn compute(n: i64) i64:
 result: ^i64 = 0
 i: ^i64 = 0
 while i < n:
 result = result + i
 i = i + 1
 result

fn main() i32:
 h1: i64 = spawn_thread(compute as ptr, 1000000)
 h2: i64 = spawn_thread(compute as ptr, 2000000)

 r1: i64 = join_thread(h1)
 r2: i64 = join_thread(h2)

 println("total: " + (r1 + r2).to_str())
 0
```

## Features

| Aspecto | Description |
|---------|-------------|
| Creation | `spawn_thread(fn_ptr, arg)` |
| Join | `join_thread(handle)` blocks hasta que termine |
| Argumento | Un solo `i64` by thread |
| Retorno | `i64` from cada thread |
| Stack | Stack dedicado by thread (1 MB by defecto) |

## See also

- `async-await.md` — Async about thread pool (more ligero)
- `synchronization.md` — Mutex, barriers
- `atomics.md` — Operacionis atomicas
