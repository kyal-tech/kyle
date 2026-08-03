# time — Date, time, and duration

> Types for working with dates, times, and durations.
> Import: `use std.time`

## datetime

Full date and time with second precision.

```ky
use std.time

now: datetime = datetime.now()
println(now.to_str())  # "2024-01-15T10:30:00"

dt: datetime = datetime.from_ymd_hms(2024, 1, 1, 12, 30, 0)
dt = datetime.parse("2024-01-01T12:30:00")

year: i32 = dt.year()
month: i32 = dt.month()
day: i32 = dt.day()
hour: i32 = dt.hour()
minute: i32 = dt.minute()
second: i32 = dt.second()
```

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `datetime.now()` | `datetime` | Current date and time |
| `datetime.from_ymd_hms(y, m, d, h, mi, s)` | `datetime` | Construct from components |
| `datetime.parse(s)` | `datetime` | Parse ISO 8601 string |
| `.year()` | `i32` | Year |
| `.month()` | `i32` | Month (1-12) |
| `.day()` | `i32` | Day (1-31) |
| `.hour()` | `i32` | Hour (0-23) |
| `.minute()` | `i32` | Minute (0-59) |
| `.second()` | `i32` | Second (0-59) |
| `.add_days(n)` | `datetime` | Add days |
| `.add_hours(n)` | `datetime` | Add hours |
| `.add_minutes(n)` | `datetime` | Add minutes |
| `.add_seconds(n)` | `datetime` | Add seconds |
| `.diff(other)` | `duration` | Difference between two datetimes |
| `.format(fmt)` | `str` | Format with pattern |
| `.to_str()` | `str` | ISO 8601 string |

```ky
start: datetime = datetime.now()
# ... some work ...
end: datetime = datetime.now()
elapsed: duration = start.diff(end)
println("took " + elapsed.total_milliseconds().to_str() + "ms")
```

## date

Calendar date without time.

```ky
today: date = date.today()
d: date = date.from_ymd(2024, 1, 1)
d = date.parse("2024-01-01")

year: i32 = d.year()
month: i32 = d.month()
day: i32 = d.day()
weekday: i32 = d.weekday()   # 0=sunday, 1=monday ...

next_week: date = d.add_days(7)
```

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `date.today()` | `date` | Current date |
| `date.from_ymd(y, m, d)` | `date` | Construct from components |
| `date.parse(s)` | `date` | Parse ISO date string |
| `.year()` | `i32` | Year |
| `.month()` | `i32` | Month |
| `.day()` | `i32` | Day |
| `.weekday()` | `i32` | Day of week |
| `.add_days(n)` | `date` | Add days |
| `.format(fmt)` | `str` | Format with pattern |
| `.to_str()` | `str` | ISO date string |

## time

Time of day without date.

```ky
now: time = time.now()
t: time = time.from_hms(12, 30, 0)
t = time.parse("12:30:00")

hour: i32 = t.hour()
minute: i32 = t.minute()
second: i32 = t.second()
```

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `time.now()` | `time` | Current time |
| `time.from_hms(h, m, s)` | `time` | Construct from components |
| `time.parse(s)` | `time` | Parse time string |
| `.hour()` | `i32` | Hour |
| `.minute()` | `i32` | Minute |
| `.second()` | `i32` | Second |
| `.to_str()` | `str` | HH:MM:SS string |

## duration

Time interval between two points in time.

```ky
d: duration = duration.from_seconds(60)
d = duration.from_milliseconds(1000)
d = duration.from_hours(1)
d = duration.from_days(7)

total_secs: i64 = d.total_seconds()
total_ms: i64 = d.total_milliseconds()
total_days: i64 = d.total_days()
text: str = d.to_str()   # "1h 0m 0s"
```

### Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `duration.from_seconds(n)` | `duration` | From seconds |
| `duration.from_milliseconds(n)` | `duration` | From milliseconds |
| `duration.from_hours(n)` | `duration` | From hours |
| `duration.from_days(n)` | `duration` | From days |
| `.total_seconds()` | `i64` | Total seconds |
| `.total_milliseconds()` | `i64` | Total milliseconds |
| `.total_days()` | `i64` | Total days |
| `.to_str()` | `str` | Human-readable string |

## sleep

Pause execution for a given number of milliseconds.

```ky
sleep(1000)  # pause for 1 second
```

## Example

```ky
use std.time

today: date = date.today()
birthday: date = date.from_ymd(2024, 12, 25)
days_left: i64 = today.diff(birthday).total_days()
println(days_left.to_str() + " days until birthday")

start: datetime = datetime.now()
sleep(500)
end: datetime = datetime.now()
elapsed: duration = start.diff(end)
println("took " + elapsed.total_milliseconds().to_str() + "ms")
```
