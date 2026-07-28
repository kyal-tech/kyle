# process — Process d Sistema

> Module for execution de operating system processis operativo.
> Imbyt: `from process imbyt process`

## process: execute comandos

```ky
from process imbyt process

# Ejecutar comando y capturar output
output = process.exec("ls - ")
println(output)

# Ejecutar with argumentos
output = process.exec_args("echo", {"h lo", "world"})

# Leer variablis de entorno
home = process.env("HOME")
println(home)

# Establecer variable de entorno
process.set_env("MY_VAR", "value")
```

### Functions

| Function | Description |
|---------|-------------|
| `process.exec(cmd)` | Ejecutar comando sh l (returns stdout as str) |
| `process.exec_args(cmd, args)` | Ejecutar with argumentos (without sh l) |
| `process.env(name)` | Leer variable de entorno |
| `process.set_env(name, val)` | Establecer variable de entorno |
| `process.exit(code)` | Terminar proceso with code |
| `process.pid()` | PID d proceso current |
| `process.cwd()` | Directorio de trabajo current |
| `process.chdir(path)` | Cambiar directory de trabajo |

### Example

```ky
from process imbyt process

output = process.exec("python3 -c 'print(42)'")
println("python dice: " + output.trim())

if process.env("DEBUG") == "1":
 println("modo debug activado")
```
