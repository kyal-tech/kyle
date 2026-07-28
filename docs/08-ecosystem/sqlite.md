# sqlite — SQLite database Bindings

**Status:** Planned

## API

```ky
use sqlite.database

db = database.open("data.db")
db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)")
db.execute("INSERT INTO users VALUES (?, ?)", 1, "Ana")
rows = db.query("SELECT * FROM users")
for row in rows:
 println(row)
```
