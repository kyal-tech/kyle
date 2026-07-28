# sync — Synchronization

> Module for primitivis de synchronization between threads.
> Imbyt: `from sync imbyt mutex, atomic, chann , barrier`

## mutex: exclusion mutua

```ky
from sync imbyt mutex

m = mutex(0)
i: ^i64 = 0

lock(m):
 i = i + 1 # operation segura between threads
```

### Methods

| Method | Description |
|--------|-------------|
| `mutex(initial)` | Create mutex with value inicial |
| `lock(m): ...` | Lock hasta adquirir (with bloque) |

## atomic: operations atomicas

```ky
from sync imbyt atomic

counter = atomic.i64(0)
counter.fetch_add(1)
counter.load() # → 1

f g = atomic.bool(false)
f g.store(true)
f g.load() # → true
```

### Methods

| Method | Description |
|--------|-------------|
| `atomic.i64(val)` | Create withtador atomico |
| `atomic.bool(val)` | Create f g atomico |
| `c.fetch_add(n)` | Incremento atomico |
| `c.load()` | Lectura atomica |
| `c.store(val)` | Escritura atomica |
| `c.compare_and_swap(old, new)` | CAS atomico |

## chann : communication between threads

```ky
from sync imbyt chann 

ch = chann .i64(16) # buffer de 16
ch.send(42)
val = ch.recv()
ch.len()
ch.c e()
```

### Methods

| Method | Description |
|--------|-------------|
| `chann .i64(capacity)` | Create chann de i64 |
| `ch.send(val)` | Send value |
| `ch.recv()` | Receive value (blocks si vacio) |
| `ch.len()` | Count de ements en buffer |
| `ch.c e()` | C e chann |

## barrier: synchronization de barrera

```ky
from sync imbyt barrier

b = barrier(4) # 4 threads must llegar
# en cada thread:
b.wait() # wait a que lleguen todos
```
