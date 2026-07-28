# Startup

> Secuencia de inicio del runtime de Kyle cuando un program se ejecuta.

## Secuencia de inicio

Cuando kernel/shell ejecuta un binary compilado with Kyle:

```
1. Cargador del SO carga binary en memory
2. C library initialization (__libc_init)
3. Rust runtime init (stack guard, thread local storage)
4. Kyle runtime init:
 a. Inicializar allocator
 b. Configurar panic handler
 c. Inicializar thread pool (lazy, primer uso)
5. Llamar a main() o kyle_main()
6. Ejecutar code del usuario
7. On exit:
 a. Destruir thread pool
 b. Free memory global
 c. Llamar a exit()
```

## Entry Points

### Con `fn main() i32:`

```llvm
defines i32 @main() { # Entry point real
 call i32 @kyle_main() # Delega al code Kyle
 ret i32 %0
}
```

### Con parameter args

```llvm
defines i32 @main(i32, ptr) { # Entry point real
 call i32 @kyle_main() # args se pasan as list
 ret i32 %0
}
```

Si code Kyle no has `fn main()`, compiler genera un main implicito
que ejecuta code del module as script.

## Initialization lazy

El runtime usa initialization lazy (via `OnceLock`) para:

- Thread pool (primer `ky_spawn_task`)
- Variablis de entorno (primer `ky_getenv`)
- Recursos globales

Esto minimiza overhead de programs que no usan featuris especificas.

## See also

- `scheduler.md` — Initialization del thread pool
- `memory.md` — Initialization del allocator
- `06-compiler/linker.md` — How se enlaza runtime
