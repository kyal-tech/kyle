# Synchronization

> Primitivas de sincronizacion between threads.

## Status current

| Primitiva | Status | Description |
|-----------|--------|-------------|
| `mutex<T>` | ✅ Implemented | Exclusion mutua with data protegido |
| `rwlock<T>` | 📅 Planned | Readers-writer lock |
| `barrier` | 📅 Planned | Barrera de sincronizacion |
| `condvar` | 📅 Planned | Condition variable |
| `once` | 📅 Planned | Initialization unica |

## Design propuesto

### mutex

```ky
m: mutex<i32> = mutex(0)

lock(m):
 *val += 1 # operation segura between threads
```

### barrier

```ky
b: barrier = barrier(4) # wait a 4 threads

fn worker(b: &barrier):
 # ... trabajar ...
 b.wait() # wait que todos lleguen
```

### Uso with threads

```ky
m: mutex<i32> = mutex(0)
n: ^i64 = 0

fn worker(arg: i64) i64:
 lock(m):
 *val += 1
 *val

# spawn multiplis workers compartiendo mutex
```

## Alternativa: channels

Para muchos cases de sincronizacion, canalis are more seguros y simples:

```ky
ch: i64 = ky_channel_new(16)
ky_channel_send(ch, 42)
val: i64 = ky_channel_recv(ch)
```

## See also

- `channels.md` — Communication by canales
- `atomics.md` — Operacionis atomicas lock-free
- `threads.md` — Threads del SO
