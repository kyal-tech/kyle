use chrono::{NaiveDate, NaiveTime, Datelike, Timelike};

/// Get current local date as (year, month, day) packed: year*10000 + month*100 + day
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_today() -> i32 {
    let now = chrono::Local::now().naive_local();
    now.year() * 10000 + now.month() as i32 * 100 + now.day() as i32
}

/// Create a date from year, month, day. Returns packed date or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_from_ymd(year: i32, month: i32, day: i32) -> i32 {
    match NaiveDate::from_ymd_opt(year, month as u32, day as u32) {
        Some(d) => d.year() * 10000 + d.month() as i32 * 100 + d.day() as i32,
        None => -1,
    }
}

/// Parse a date string (ISO: YYYY-MM-DD). Returns packed date or -1.
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_parse(s: *const u8) -> i32 {
    if s.is_null() { return -1; }
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return -1,
    };
    match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        Ok(d) => d.year() * 10000 + d.month() as i32 * 100 + d.day() as i32,
        Err(_) => -1,
    }
}

/// Extract year from packed date
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_year(packed: i32) -> i32 { packed / 10000 }

/// Extract month from packed date
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_month(packed: i32) -> i32 { (packed / 100) % 100 }

/// Extract day from packed date
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_day(packed: i32) -> i32 { packed % 100 }

/// Day of week: 0=Mon, 1=Tue, ..., 6=Sun
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_weekday(packed: i32) -> i32 {
    let y = ky_date_year(packed);
    let m = ky_date_month(packed);
    let d = ky_date_day(packed);
    match NaiveDate::from_ymd_opt(y, m as u32, d as u32) {
        Some(dt) => dt.format("%u").to_string().parse::<i32>().unwrap_or(0) - 1,
        None => -1,
    }
}

/// Add days to packed date. Returns new packed date.
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_add_days(packed: i32, days: i32) -> i32 {
    let y = ky_date_year(packed);
    let m = ky_date_month(packed);
    let d = ky_date_day(packed);
    match NaiveDate::from_ymd_opt(y, m as u32, d as u32) {
        Some(dt) => {
            let new = dt + chrono::Duration::days(days as i64);
            new.year() * 10000 + new.month() as i32 * 100 + new.day() as i32
        }
        None => -1,
    }
}

/// Format packed date with strftime format. Returns heap string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_date_format(packed: i32, fmt: *const u8) -> *mut u8 {
    if fmt.is_null() { return std::ptr::null_mut(); }
    let fmt_str = unsafe { std::ffi::CStr::from_ptr(fmt .cast()) };
    let fmt_str = match fmt_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let y = ky_date_year(packed);
    let m = ky_date_month(packed);
    let d = ky_date_day(packed);
    match NaiveDate::from_ymd_opt(y, m as u32, d as u32) {
        Some(dt) => {
            let formatted = dt.format(fmt_str).to_string();
            let bytes = formatted.as_bytes();
            let len = bytes.len();
            let ptr = crate::ky_alloc((len + 1) as i64);
            if ptr.is_null() { return std::ptr::null_mut(); }
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
                *ptr.add(len) = 0;
            }
            ptr
        }
        None => std::ptr::null_mut(),
    }
}

// ── time ──

/// Create a time from hour, minute, second. Returns packed time: hh*10000 + mm*100 + ss
#[unsafe(no_mangle)]
pub extern "C" fn ky_time_from_hms(hour: i32, min: i32, sec: i32) -> i32 {
    if hour < 0 || hour > 23 || min < 0 || min > 59 || sec < 0 || sec > 59 { return -1; }
    hour * 10000 + min * 100 + sec
}

/// Get current local time as packed time.
#[unsafe(no_mangle)]
pub extern "C" fn ky_time_now() -> i32 {
    let now = chrono::Local::now().naive_local();
    now.hour() as i32 * 10000 + now.minute() as i32 * 100 + now.second() as i32
}

/// Parse time string (HH:MM:SS). Returns packed time or -1.
#[unsafe(no_mangle)]
pub extern "C" fn ky_time_parse(s: *const u8) -> i32 {
    if s.is_null() { return -1; }
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return -1,
    };
    match NaiveTime::parse_from_str(s, "%H:%M:%S") {
        Ok(t) => t.hour() as i32 * 10000 + t.minute() as i32 * 100 + t.second() as i32,
        Err(_) => -1,
    }
}

/// Extract hour from packed time
#[unsafe(no_mangle)]
pub extern "C" fn ky_time_hour(packed: i32) -> i32 { packed / 10000 }

/// Extract minute from packed time
#[unsafe(no_mangle)]
pub extern "C" fn ky_time_minute(packed: i32) -> i32 { (packed / 100) % 100 }

/// Extract second from packed time
#[unsafe(no_mangle)]
pub extern "C" fn ky_time_second(packed: i32) -> i32 { packed % 100 }
