# Async Scheduler

> Scheduler de tasks asincronas: thread pool for async/await.
> Crate: `kyc_runtime/src/async_.rs` (152 lines), `kyc_runtime/src/task.rs` (41 lines).

## Responsabilidad

El scheduler gestiona execution de tasks `async fn` y bloquis `async:` using
un thread pool global. Cada task se ejecuta en un thread del pool y returns un
resultado i64 que se obhas via `await`.

## Thread Pool

```rust
type TaskFn = Box<dyn FnOnce() + Send>;

 struct Executor {
 running: Arc<AtomicBool>,
 task_sender: mpsc::Sender<TaskFn>,
 workers: Vec<thread::JoinHandle<()>>,
}
```

- Pool global inicializado with `available_parallelism()` workers
- Configurable via variable de entorno `KL_WORKERS`
- Communication using `std::sync::mpsc` channel

## Functions

### ky_spawn_task

```rust
#[unsafe(no_mangle)]
 extern "C" fn ky_spawn_task(
 func: Option<unsafe extern "C" fn(i64) -> i64>,
 arg: i64,
) -> i64
```

Spawnear una funcion en thread pool.

- `func`: pointer a funcion with C calling convention
- `arg`: argumento i64 unico
- Retorna: handle i64 opaco for `ky_await_task` (un `Arc` convertido a i64)

```ky
task = async: heavy_computation()
result = await task
```

### ky_await_task

```rust
#[unsafe(no_mangle)]
 extern "C" fn ky_await_task(handle: i64) -> i64
```

Esperar resultado de una task asincrona.

- `handle`: i64 devuelto by `ky_spawn_task`
- Bloquea thread current hasta que task completa
- Retorna: value i64 producido by task

### ky_yield

```rust
#[unsafe(no_mangle)]
 extern "C" fn ky_yield()
```

Cede paso al scheduler. Permite que otra task se ejecute.

## Task internals

```rust
 struct Task<T> {
 id: u64,
 future: Arc<Mutex<BoxedFuture<T>>>,
}
```

Donde `BoxedFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>`.

## Initialization lazy

El scheduler se inicializa bajo demanda. La primera llamada a `ky_spawn_task`
crea thread pool.

```rust
static EXECUTOR: OnceLock<Executor> = OnceLock::new();
```

## Configuration

```bash
export KL_WORKERS=8 # Limitar a 8 workers (default: available_parallelism)
```

## See also

- `03-language/concurrency/async-await.md` — Syntax async/await
- `thread.md` — Threads del sistema operativo
- `startup.md` — Initialization del runtime
