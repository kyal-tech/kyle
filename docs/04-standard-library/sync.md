# sync — Synchronization primitives

> Mutex, atomic operations, and channels for thread synchronization.
> Import: `use std.sync`

## mutex

Mutual exclusion for shared state.

```ky
use std.sync

m: mutex<i64> = mutex(0)

# Lock with block (auto-release)
lock(m):
    m.value = m.value + 1

# Manual lock/unlock
m.lock()
m.value = m.value + 1
m.unlock()
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `mutex(initial)` | `fn(initial: T) mutex<T>` | Create mutex with initial value |
| `.lock()` | `fn()` | Acquire lock (blocks) |
| `.unlock()` | `fn()` | Release lock |
| `lock(m): ...` | block | Acquire lock for block duration |

## atomic

Lock-free atomic operations for simple values.

```ky
counter: atomic_i64 = sync.atomic_i64(0)
counter.fetch_add(1)
val: i64 = counter.load()   # 1

flag: atomic_bool = sync.atomic_bool(false)
flag.store(true)
val: bool = flag.load()     # true
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `atomic_i64(val)` | `fn(val: i64) atomic_i64` | Create atomic counter |
| `atomic_bool(val)` | `fn(val: bool) atomic_bool` | Create atomic flag |
| `.fetch_add(n)` | `fn(n: i64) i64` | Atomic increment (returns old value) |
| `.load()` | `fn() T` | Atomic read |
| `.store(val)` | `fn(val: T)` | Atomic write |
| `.compare_and_swap(old, new)` | `fn(old: T, new: T) bool` | CAS operation |

## channel

Communication between threads via typed channels.

```ky
ch: chan<i64> = sync.channel(16)   # buffered channel (16 items)
ch.send(42)
val: i64 = ch.recv()
ch.close()
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `channel(capacity)` | `fn(capacity: i32) chan<T>` | Create buffered channel |
| `.send(val)` | `fn(val: T)` | Send value (blocks if full) |
| `.recv()` | `fn() T` | Receive value (blocks if empty) |
| `.len()` | `fn() i32` | Number of items in buffer |
| `.close()` | `fn()` | Close channel |

## Example: producer-consumer

```ky
use std.sync
use std.thread

ch: chan<i64> = sync.channel(16)

fn producer(ch: ^&chan<i64>):
    for i in 0..10:
        ch.send(i)
    ch.close()

fn consumer(ch: ^&chan<i64>):
    while true:
        val: i64 = ch.recv()
        println("got: " + val.to_str())

producer_h: thread = thread.spawn(producer, ch)
consumer_h: thread = thread.spawn(consumer, ch)
thread.join(producer_h)
thread.join(consumer_h)
```
