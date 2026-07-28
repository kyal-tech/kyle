/// Generate a UUID v4 string. Returns heap-allocated null-terminated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_uuid_v4() -> *mut u8 {
    let u = uuid::Uuid::new_v4();
    let s = u.to_string();
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

/// Parse a UUID string. Returns heap-allocated string or null on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_uuid_parse(s: *const u8) -> *mut u8 {
    if s.is_null() { return std::ptr::null_mut(); }
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return std::ptr::null_mut(),
    };
    match uuid::Uuid::parse_str(s) {
        Ok(u) => {
            let out = u.to_string();
            let bytes = out.as_bytes();
            let len = bytes.len();
            let ptr = crate::ky_alloc((len + 1) as i64);
            if ptr.is_null() { return std::ptr::null_mut(); }
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
                *ptr.add(len) = 0;
            }
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}
