# env — Environment variables

> Get, set, and manage environment variables. Load .env files.
> Installation: `ky add env`
> Import: `use env`

## Reading variables

```ky
use env

home: str = env.get("HOME")
println("home: " + home)
```

## Default values

```ky
port: str = env.get_or("PORT", "8080")
debug: str = env.get_or("DEBUG", "false")
```

## Typed access

```ky
port: i32 = env.get_int("PORT", 8080)
timeout: f64 = env.get_float("TIMEOUT", 30.0)
debug: bool = env.get_bool("DEBUG", false)
```

## Setting variables

```ky
env.set("MY_VAR", "my_value")
env.unset("TEMP_VAR")
```

## .env file loading

```ky
# Load .env from current directory
env.load_file!()   # loads .env

# Load specific file
env.load_file(".env.production")!

# After loading:
db_url: str = env.get("DATABASE_URL")
api_key: str = env.get("API_KEY")
```

## Listing all variables

```ky
keys: [str] = env.list()
for key in keys:
    println(key + "=" + env.get(key))
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `get(key)` | `fn(key: &str) str` | Read environment variable |
| `get_or(key, default)` | `fn(key: &str, default: &str) str` | Read with default |
| `get_int(key, default)` | `fn(key: &str, default: i32) i32` | Read as integer |
| `get_float(key, default)` | `fn(key: &str, default: f64) f64` | Read as float |
| `get_bool(key, default)` | `fn(key: &str, default: bool) bool` | Read as boolean (true/1/yes = true) |
| `set(key, value)` | `fn(key: &str, value: &str)` | Set environment variable |
| `unset(key)` | `fn(key: &str)` | Remove environment variable |
| `load_file(path)` | `fn(path: &str)!` | Load .env file |
| `load_file()` | `fn()!` | Load .env from current directory |
| `list()` | `fn() [str]` | List all environment variable names |

## .env file format

```
# Comments start with #
DATABASE_URL=postgres://localhost/mydb
API_KEY=secret123
PORT=8080
DEBUG=true
MAX_CONNECTIONS=100
```

## Example

```ky
use env

# Load configuration
env.load_file()!

# Read with defaults
host: str = env.get_or("HOST", "localhost")
port: i32 = env.get_int("PORT", 8080)
debug: bool = env.get_bool("DEBUG", false)

println("starting on " + host + ":" + port.to_str())
if debug:
    println("debug mode enabled")
```
