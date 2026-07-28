use std::time::{SystemTime, UNIX_EPOCH, Duration};

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Current time in milliseconds since epoch.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Current time in microseconds since epoch.
pub fn now_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as i64
}

/// Sleep for ms milliseconds (cross-platform).
pub fn sleep_ms(ms: i32) {
    let dur = Duration::from_millis(ms.max(0) as u64);
    std::thread::sleep(dur);
}
