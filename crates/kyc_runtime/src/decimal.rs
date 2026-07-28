/// Simple decimal implementation using i64 with 2 decimal places (fixed precision).
/// Value = internal_i64 / 100.0

/// Create decimal from string like "99.99". Returns i64 * 100.
#[unsafe(no_mangle)]
pub extern "C" fn ky_decimal_from_str(s: *const u8) -> i64 {
    if s.is_null() { return 0; }
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return 0,
    };
    let parts: Vec<&str> = s.splitn(2, '.').collect();
    let int_part: i64 = parts.get(0).unwrap_or(&"0").parse().unwrap_or(0);
    let frac_part: i64 = if parts.len() > 1 {
        let f = parts[1];
        if f.len() == 0 { 0 }
        else if f.len() == 1 { f.parse::<i64>().unwrap_or(0) * 10 }
        else { f[..2].parse::<i64>().unwrap_or(0) }
    } else { 0 };
    int_part * 100 + frac_part
}

/// Convert decimal to string. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_decimal_to_str(val: i64) -> *mut u8 {
    let int_part = val / 100;
    let frac_part = (val % 100).abs();
    let s = if frac_part == 0 {
        format!("{}", int_part)
    } else {
        format!("{}.{:02}", int_part, frac_part)
    };
    let bytes = s.as_bytes();
    let len = bytes.len();
    let ptr = crate::ky_alloc((len + 1) as i64);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        *ptr.add(len) = 0;
    }
    ptr
}

/// Round decimal to given number of decimal places. Returns new decimal.
#[unsafe(no_mangle)]
pub extern "C" fn ky_decimal_round(val: i64, decimals: i32) -> i64 {
    let factor: i64 = 10i64.pow(decimals.max(0) as u32);
    let scaled = val as f64 / 100.0;
    let rounded = (scaled * factor as f64).round() / factor as f64;
    (rounded * 100.0) as i64
}

/// Truncate decimal to integer.
#[unsafe(no_mangle)]
pub extern "C" fn ky_decimal_truncate(val: i64) -> i64 {
    (val / 100) * 100
}
