# net — TCP networking

> Low-level TCP client and server.
> Import: `use std.net`

## TCP server

```ky
use std.net

listener: tcp_listener = net.tcp_listen(8080)!
println("listening on port 8080")

while true:
    client: tcp_stream = listener.accept()!
    data: str = client.read(1024)
    client.write("HTTP/1.1 200 OK\r\n\r\nHello World")
    client.close()
```

## TCP client

```ky
conn: tcp_stream = net.tcp_connect("example.com", 80)!
conn.write("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
response: str = conn.read(4096)
conn.close()
```

## tcp_listener

| Method | Signature | Description |
|--------|-----------|-------------|
| `net.tcp_listen(port)` | `fn(port: i32) tcp_listener!` | Start listening |
| `.accept()` | `fn() tcp_stream!` | Accept connection (blocks) |
| `.close()` | `fn()` | Stop listening |

## tcp_stream

| Method | Signature | Description |
|--------|-----------|-------------|
| `net.tcp_connect(host, port)` | `fn(host: &str, port: i32) tcp_stream!` | Connect to host |
| `.read(count)` | `fn(count: i32) str` | Read up to N bytes |
| `.read_all()` | `fn() str` | Read until connection closes |
| `.write(data)` | `fn(data: &str)` | Send data |
| `.close()` | `fn()` | Close connection |

## Example: simple HTTP server

```ky
use std.net

fn handle_request(client: tcp_stream):
    request: str = client.read(4096)
    client.write("HTTP/1.1 200 OK\r\n")
    client.write("Content-Type: text/plain\r\n")
    client.write("Content-Length: 12\r\n")
    client.write("\r\n")
    client.write("Hello World")
    client.close()

listener: tcp_listener = net.tcp_listen(8080)!
while true:
    client: tcp_stream = listener.accept()!
    handle_request(client)
```
