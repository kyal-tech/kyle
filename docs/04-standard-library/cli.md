# cli — Command-line argument parsing

> Parse command-line flags and positional arguments.
> Import: `use std.cli`

## Quick start

```ky
use std.cli

args: {str: str} = cli.parse()
name: str = args["name"] ?? "world"
verbose: bool = cli.has("verbose")
println("hello " + name)
```

## Running

```bash
ky run app.ky --name=Kyle --verbose
# hello Kyle
```

## Positional arguments

```ky
use std.cli

cli.parse()  # must be called first
first: str = cli.arg(0)
second: str = cli.arg(1)
println("first: " + first + ", second: " + second)
```

```bash
ky run app.ky input.txt output.txt
# first: input.txt, second: output.txt
```

## Default values

```ky
args: {str: str} = cli.parse()
port: str = args["port"] ?? "8080"
host: str = args["host"] ?? "localhost"
mode: str = args["mode"] ?? "release"
```

```bash
ky run app.ky --port=3000
# port=3000, host=localhost, mode=release
```

## Flag definitions with type hints

```ky
cli.define("port", "p", "server port", "8080")
cli.define("host", "h", "server host", "localhost")
cli.define("verbose", "v", "enable verbose output", "false")

args: {str: str} = cli.parse()

port: i32 = cli.get_int("port")
verbose: bool = cli.get_bool("verbose")
```

```bash
ky run app.ky -p 9000 -v
```

## Help text

If you define flags, `--help` is auto-generated:

```ky
cli.define("port", "p", "server port", "8080")
cli.define("host", "h", "server host", "localhost")
cli.parse()
```

```bash
ky run app.ky --help
# Usage: app.ky [options]
#   --port, -p   server port       (default: 8080)
#   --host, -h   server host       (default: localhost)
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse()` | `fn() {str: str}` | Parse CLI args, returns flag dict |
| `arg(n)` | `fn(n: i32) str` | Get positional argument at index |
| `has(name)` | `fn(name: &str) bool` | Check if flag was provided |
| `get_int(name)` | `fn(name: &str) i32` | Get flag as integer (with default) |
| `get_bool(name)` | `fn(name: &str) bool` | Get flag as boolean |
| `define(name, short, desc, default)` | `fn(name: &str, short: &str, desc: &str, default: &str)` | Define a flag with metadata |

## Example

```ky
use std.cli

cli.define("port", "p", "server port", "8080")
cli.define("db", "d", "database URL", "localhost")
cli.define("verbose", "v", "verbose output", "false")

args: {str: str} = cli.parse()

port: i32 = cli.get_int("port")
db: str = args["db"]
verbose: bool = cli.get_bool("verbose")

println("starting on port " + port.to_str())
if verbose:
    println("database: " + db)
```
