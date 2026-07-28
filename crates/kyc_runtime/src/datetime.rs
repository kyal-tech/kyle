use chrono::{Utc, NaiveDateTime, NaiveDate, NaiveTime, Duration, Datelike, Timelike};

/// Get current UTC timestamp in milliseconds since epoch.
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_now() -> i64 {
    Utc::now().timestamp_millis()
}

/// Parse an ISO 8601 datetime string like "2026-07-04T12:00:00Z"
/// Returns milliseconds since epoch, or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_parse(ptr: *const u8) -> i64 {
    if ptr.is_null() { return -1; }
    let s = unsafe { std::ffi::CStr::from_ptr(ptr .cast()) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return -1,
    };
    // Try ISO 8601 with timezone
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return dt.timestamp_millis();
    }
    // Try without timezone (assume UTC)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return dt.and_utc().timestamp_millis();
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return dt.and_utc().timestamp_millis();
    }
    -1
}

/// Format a datetime (ms since epoch) to ISO 8601 string.
/// Returns a heap-allocated string (caller must free with ky_free).
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_format(ms: i64, fmt: *const u8) -> *mut u8 {
    if fmt.is_null() { return std::ptr::null_mut(); }
    let fmt_str = unsafe { std::ffi::CStr::from_ptr(fmt .cast()) };
    let fmt_str = match fmt_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return std::ptr::null_mut(); }
    let naive = match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt,
        None => {
            eprintln!("ky_dt_format: from_timestamp_opt failed for secs={}, nsecs={}", secs, nsecs);
            return std::ptr::null_mut();
        }
    };
    let formatted = naive.format(fmt_str).to_string();
    let bytes = formatted.as_bytes();
    let out_len = bytes.len();
    let ptr = crate::ky_alloc((out_len + 1) as i64);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, out_len);
        *ptr.add(out_len) = 0;
    }
    ptr
}

/// Get year from datetime (ms since epoch)
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_year(ms: i64) -> i32 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt.year(),
        None => -1,
    }
}

/// Get month from datetime (ms since epoch), 1-12
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_month(ms: i64) -> i32 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt.month() as i32,
        None => -1,
    }
}

/// Get day from datetime (ms since epoch), 1-31
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_day(ms: i64) -> i32 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt.day() as i32,
        None => -1,
    }
}

/// Get hour from datetime (ms since epoch), 0-23
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_hour(ms: i64) -> i32 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt.hour() as i32,
        None => -1,
    }
}

/// Get minute from datetime (ms since epoch), 0-59
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_minute(ms: i64) -> i32 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt.minute() as i32,
        None => -1,
    }
}

/// Get second from datetime (ms since epoch), 0-59
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_second(ms: i64) -> i32 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => dt.second() as i32,
        None => -1,
    }
}

/// Add days to datetime (ms since epoch). Returns new ms.
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_add_days(ms: i64, days: i32) -> i64 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => {
            let new_dt = dt + Duration::days(days as i64);
            new_dt.and_utc().timestamp_millis()
        }
        None => -1,
    }
}

/// Add hours to datetime (ms since epoch). Returns new ms.
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_add_hours(ms: i64, hours: i32) -> i64 {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    if nsecs > 1_000_000_000 { return -1; }
    match NaiveDateTime::from_timestamp_opt(secs, nsecs) {
        Some(dt) => {
            let new_dt = dt + Duration::hours(hours as i64);
            new_dt.and_utc().timestamp_millis()
        }
        None => -1,
    }
}

/// Difference between two datetimes in milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_diff(ms1: i64, ms2: i64) -> i64 {
    ms1 - ms2
}

/// Create a datetime from year, month, day, hour, minute, second (UTC).
/// Returns milliseconds since epoch, or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_datetime_from_ymdhms(year: i32, month: i32, day: i32, hour: i32, min: i32, sec: i32) -> i64 {
    if month < 1 || month > 12 || day < 1 || day > 31 || hour < 0 || hour > 23 || min < 0 || min > 59 || sec < 0 || sec > 59 {
        return -1;
    }
    match NaiveDate::from_ymd_opt(year, month as u32, day as u32) {
        Some(date) => {
            match NaiveTime::from_hms_opt(hour as u32, min as u32, sec as u32) {
                Some(time) => {
                    let dt = NaiveDateTime::new(date, time);
                    dt.and_utc().timestamp_millis()
                }
                None => -1,
            }
        }
        None => -1,
    }
}
