# process — OS process management

> Execute commands and interact with the operating system.
> Import: `use std.process`
>
> For environment variables, use the `env` package (`ky add env`).

## Executing commands

```ky
use std.process

output: str = process.exec("ls -la")!
println(output)

# Safer: exec with explicit arguments (no shell injection)
output = process.exec_args("echo", ["hello", "world"])!
```

## Process control

```ky
pid: i64 = process.pid()
cwd: str = process.cwd()
process.chdir("/tmp")
process.exit(0)
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `exec(cmd)` | `fn(cmd: &str) str!` | Execute shell command (returns stdout) |
| `exec_args(cmd, args)` | `fn(cmd: &str, args: &[str]) str!` | Execute with arguments (no shell) |
| `exit(code)` | `fn(code: i32)` | Terminate process with exit code |
| `pid()` | `fn() i64` | Current process ID |
| `cwd()` | `fn() str` | Current working directory |
| `chdir(path)` | `fn(path: &str)!` | Change working directory |

## Example

```ky
use std.process

output: str = process.exec("python3 -c 'print(42)'")!
println("python says: " + output.trim())
```
