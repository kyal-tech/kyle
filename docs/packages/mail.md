# mail — Email sending

> Send emails via SMTP with TLS, attachments, and HTML body.
> Installation: `ky add mail`
> Import: `use mail`

## Quick start

```ky
use mail

msg: mail.message = mail.message(
    "from@example.com",
    "to@example.com",
    "Hello from Kyle",
    "This is the email body"
)

mail.send("smtp.example.com", 587, "user", "pass", msg)!
```

## HTML email

```ky
msg: mail.message = mail.message(
    "from@example.com",
    "to@example.com",
    "Welcome!",
    """
    <h1>Welcome to Kyle</h1>
    <p>Thanks for joining.</p>
    """
)
msg.set_html(true)

mail.send("smtp.example.com", 587, "user", "pass", msg)!
```

## Multiple recipients

```ky
msg: mail.message = mail.message(
    "from@example.com",
    ["alice@example.com", "bob@example.com"],
    "Team announcement",
    "Meeting at 3pm"
)
```

## With attachment

```ky
msg: mail.message = mail.message(
    "from@example.com",
    "to@example.com",
    "Report",
    "Please find the report attached."
)

msg.attach("report.pdf")
msg.attach_bytes("data.csv", csv_data, "text/csv")

mail.send("smtp.example.com", 587, "user", "pass", msg)!
```

## CC and BCC

```ky
msg: mail.message = mail.message(
    "from@example.com",
    "to@example.com",
    "Hello",
    "Body text"
)
msg.cc("manager@example.com")
msg.bcc("archive@example.com")
```

## Connection configuration

```ky
# With explicit TLS (port 465)
mail.send_tls("smtp.example.com", 465, "user", "pass", msg)!

# With STARTTLS (port 587)
mail.send("smtp.example.com", 587, "user", "pass", msg)!

# Without auth (internal relay)
mail.send_plain("mail.internal:25", msg)!
```

## mail.message

| Method | Signature | Description |
|--------|-----------|-------------|
| `message(from, to, subject, body)` | `fn(from, to, subject, body) message` | Create email |
| `message(from, to_list, subject, body)` | `fn(from, to_list: &[str], subject, body) message` | Multiple recipients |
| `.set_html(is_html)` | `fn(is_html: bool)` | Set HTML content type |
| `.cc(address)` | `fn(address: &str)` | Add CC recipient |
| `.bcc(address)` | `fn(address: &str)` | Add BCC recipient |
| `.attach(path)` | `fn(path: &str)!` | Attach file |
| `.attach_bytes(name, data, mime)` | `fn(name: &str, data: &bytes, mime: &str)` | Attach bytes |
| `.subject()` | `fn() str` | Get subject |
| `.to_str()` | `fn() str` | Serialize to SMTP format |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `send(host, port, user, pass, msg)` | `fn(host: &str, port: i32, user: &str, pass: &str, msg: &message)!` | Send via STARTTLS |
| `send_tls(host, port, user, pass, msg)` | `fn(host: &str, port: i32, user: &str, pass: &str, msg: &message)!` | Send via TLS |
| `send_plain(host, msg)` | `fn(host: &str, msg: &message)!` | Send without auth |

## Example

```ky
use mail
use std.fs

# Build email
msg: mail.message = mail.message(
    "noreply@myapp.com",
    "user@gmail.com",
    "Your report is ready",
    """
    <h1>Monthly Report</h1>
    <p>Download your report attached.</p>
    """
)
msg.set_html(true)
msg.attach("report.pdf")!

# Send
mail.send("smtp.gmail.com", 587, "myapp@gmail.com", "app-password", msg)!
println("email sent")
```
