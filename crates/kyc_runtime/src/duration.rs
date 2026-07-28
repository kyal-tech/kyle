use std::time::Duration;

/// Create a duration from seconds.
#[unsafe(no_mangle)]
pub extern "C" fn ky_duration_from_secs(secs: i64) -> i64 {
    let d = Duration::from_secs(if secs < 0 { 0 } else { secs as u64 });
    Box::into_raw(Box::new(d)) as i64
}

/// Create a duration from milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn ky_duration_from_millis(ms: i64) -> i64 {
    let d = Duration::from_millis(if ms < 0 { 0 } else { ms as u64 });
    Box::into_raw(Box::new(d)) as i64
}

/// Create a duration from hours.
#[unsafe(no_mangle)]
pub extern "C" fn ky_duration_from_hours(hours: i64) -> i64 {
    let secs = if hours < 0 { 0 } else { hours as u64 * 3600 };
    let d = Duration::from_secs(secs);
    Box::into_raw(Box::new(d)) as i64
}

/// Create a duration from days.
#[unsafe(no_mangle)]
pub extern "C" fn ky_duration_from_days(days: i64) -> i64 {
    let secs = if days < 0 { 0 } else { days as u64 * 86400 };
    let d = Duration::from_secs(secs);
    Box::into_raw(Box::new(d)) as i64
}

/// Format duration as human-readable string.
/// Returns heap-allocated C string, caller must free with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_duration_to_str(ptr: i64) -> *mut u8 {
    if ptr == 0 { return std::ptr::null_mut(); }
    let d = unsafe { &*(ptr as *const Duration) };
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let s = if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    };
    let bytes = s.as_bytes();
    let out = crate::ky_alloc((bytes.len() + 1) as i64);
    if out.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
        *out.add(bytes.len()) = 0;
    }
    out
}

/// Free a duration object.
#[unsafe(no_mangle)]
pub extern "C" fn ky_duration_free(ptr: i64) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut Duration)); }
    }
}
