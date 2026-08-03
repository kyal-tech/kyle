# config — Configuration file loader

> Load YAML and TOML config files into typed classes.
> Merge defaults, file values, and environment variables.
> Installation: `ky add config`
> Import: `use config`

## Load TOML

```ky
use config

class ServerConfig:
    host: str
    port: i32
    debug: bool

class AppConfig:
    server: ServerConfig
    name: str

cfg: AppConfig = config.load_toml<AppConfig>("app.toml")!
println("server: " + cfg.server.host + ":" + cfg.server.port.to_str())
```

`app.toml`:
```toml
name = "myapp"

[server]
host = "localhost"
port = 8080
debug = true
```

## Load YAML

```ky
cfg: AppConfig = config.load_yaml<AppConfig>("app.yaml")!
```

`app.yaml`:
```yaml
name: myapp
server:
  host: localhost
  port: 8080
  debug: true
```

## Defaults with merge

```ky
class DbConfig:
    host: str
    port: i32
    database: str
    user: str

# Defaults
defaults: DbConfig = DbConfig {
    host: "localhost",
    port: 5432,
    database: "mydb",
    user: "admin",
}

# File overrides defaults
cfg: DbConfig = config.load_yaml_with_defaults<DbConfig>("db.yaml", defaults)!
```

## Environment variable overrides

```ky
# Environment variables override file values
# DATABASE_HOST=prod.example.com overrides db.yaml's host

cfg: DbConfig = config.load_yaml<DbConfig>("db.yaml", {
    "host": "DATABASE_HOST",
    "port": "DATABASE_PORT",
    "user": "DATABASE_USER",
})!
```

## Config from string

```ky
toml_str: str = """
[server]
host = "localhost"
port = 8080
"""

cfg: ServerConfig = config.from_toml_str<ServerConfig>(toml_str)!
println(cfg.host)
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `load_toml<T>(path)` | `fn(path: &str) T!` | Load TOML file to class |
| `load_yaml<T>(path)` | `fn(path: &str) T!` | Load YAML file to class |
| `load_toml<T>(path, env_map)` | `fn(path: &str, env_map: &{str: str}) T!` | Load with env var overrides |
| `load_yaml<T>(path, env_map)` | `fn(path: &str, env_map: &{str: str}) T!` | Load with env var overrides |
| `load_yaml_with_defaults<T>(path, defaults)` | `fn(path: &str, defaults: T) T!` | Load with defaults |
| `from_toml_str<T>(s)` | `fn(s: &str) T!` | Parse TOML string to class |
| `from_yaml_str<T>(s)` | `fn(s: &str) T!` | Parse YAML string to class |

## Example: 12-factor app config

```ky
use config

class Config:
    host: str
    port: i32
    db_url: str
    redis_url: str
    log_level: str

defaults: Config = Config {
    host: "0.0.0.0",
    port: 8080,
    db_url: "postgres://localhost/mydb",
    redis_url: "redis://localhost",
    log_level: "info",
}

cfg: Config = config.load_yaml_with_defaults<Config>("config.yaml", defaults)!
println("starting on " + cfg.host + ":" + cfg.port.to_str())
```
