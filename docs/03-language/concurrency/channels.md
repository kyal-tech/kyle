# Channels

> Communication between threads using canales. Actualmente solo `i64`.
> Los canalis have buffer y supportsn `send`/`recv` blockings.

## Uso basico

```ky
ch: i64 = channel_new(16) # canal with buffer de 16
channel_send(ch, 42)
val: i64 = channel_recv(ch)
println(val.to_str()) # 42
channel_free(ch)
```

## Productor-Consumidor

```ky
fn productor(ch: i64):
 i: ^i64 = 0
 while i < 10:
 channel_send(ch, i)
 i = i + 1
 channel_close(ch)

fn consumidor(ch: i64):
 loop:
 val: i64 = channel_recv(ch)
 if val == -1: # senal de cierre
 break
 println(val.to_str())

fn main() i32:
 ch: i64 = channel_new(16)
 h1: i64 = spawn_thread(productor as ptr, ch)
 h2: i64 = spawn_thread(consumidor as ptr, ch)
 join_thread(h1)
 join_thread(h2)
 channel_free(ch)
 0
```

## Functions

| Function | Description |
|---------|-------------|
| `channel_new(capacity)` | Crear canal with buffer |
| `channel_send(ch, val)` | Enviar (blocks si lleno) |
| `channel_recv(ch)` | Recibir (blocks si vacio) |
| `channel_close(ch)` | Cerrar canal |
| `channel_len(ch)` | Elements en buffer |
| `channel_free(ch)` | Free canal |

## Limitacionis currentes

- Solo `i64` as type de value
- Sin `select` for multiplexar canales
- Canalis without tipado generico (`channel<T>`)
- Futuro: `channel<T>`, `select`

## See also

- `threads.md` — Threads del SO
- `synchronization.md` — Mutex, barriers
