# thread — OS threads

> Spawn and manage operating system threads.
> Import: `use std.thread`

## Spawning and joining

```ky
use std.thread

fn worker(n: i64) i64:
    i: ^i64 = 0
    result: ^i64 = 0
    while i < n:
        result = result + i
        i = i + 1
    result

handle: thread = thread.spawn(worker, 1000000)
result: i64 = thread.join(handle)
println(result.to_str())
```

## Sleeping

```ky
thread.sleep(1000)   # milliseconds
```

## Current thread

```ky
id: i64 = thread.id()
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `spawn(fn_ptr, arg)` | `fn(fn(T) R, T) thread` | Create new OS thread |
| `join(handle)` | `fn(handle: thread) R` | Wait for thread to finish |
| `sleep(ms)` | `fn(ms: i64)` | Sleep current thread |
| `yield()` | `fn()` | Yield CPU to scheduler |
| `id()` | `fn() i64` | Current thread ID |

## Example

```ky
use std.thread

fn compute(n: i64) i64:
    i: ^i64 = 0
    result: ^i64 = 0
    while i < n:
        result = result + i
        i = i + 1
    result

h1: thread = thread.spawn(compute, 1000000)
h2: thread = thread.spawn(compute, 2000000)
r1: i64 = thread.join(h1)
r2: i64 = thread.join(h2)
println("total: " + (r1 + r2).to_str())
```
