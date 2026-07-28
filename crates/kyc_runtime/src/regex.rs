use regex::Regex;

/// Compile a regex pattern. Returns a pointer to the compiled Regex (opaque), or null on error.
/// The returned pointer must be freed with ky_regex_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_regex_new(pattern: *const u8) -> *mut std::ffi::c_void {
    if pattern.is_null() { return std::ptr::null_mut(); }
    let s = unsafe { std::ffi::CStr::from_ptr(pattern .cast()) };
    let s = match s.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match Regex::new(s) {
        Ok(re) => Box::into_raw(Box::new(re)) as *mut std::ffi::c_void,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a compiled regex.
#[unsafe(no_mangle)]
pub extern "C" fn ky_regex_free(re: *mut std::ffi::c_void) {
    if re.is_null() { return; }
    unsafe { drop(Box::from_raw(re as *mut Regex)); }
}

/// Check if string matches regex. Returns 1 if match, 0 if not, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_regex_is_match(re: *mut std::ffi::c_void, s: *const u8) -> i32 {
    if re.is_null() || s.is_null() { return -1; }
    let re = unsafe { &*(re as *const Regex) };
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    if re.is_match(s) { 1 } else { 0 }
}

/// Find first match. Returns heap-allocated string or null.
#[unsafe(no_mangle)]
pub extern "C" fn ky_regex_find(re: *mut std::ffi::c_void, s: *const u8) -> *mut u8 {
    if re.is_null() || s.is_null() { return std::ptr::null_mut(); }
    let re = unsafe { &*(re as *const Regex) };
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match re.find(s) {
        Some(m) => {
            let matched = m.as_str();
            let bytes = matched.as_bytes();
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

/// Replace all matches with replacement string. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_regex_replace(re: *mut std::ffi::c_void, s: *const u8, with: *const u8) -> *mut u8 {
    if re.is_null() || s.is_null() || with.is_null() { return std::ptr::null_mut(); }
    let re = unsafe { &*(re as *const Regex) };
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let with = unsafe { std::ffi::CStr::from_ptr(with .cast()) };
    let with = match with.to_str() {
        Ok(w) => w,
        Err(_) => return std::ptr::null_mut(),
    };
    let result = re.replace_all(s, with).to_string();
    let bytes = result.as_bytes();
    let len = bytes.len();
    let ptr = crate::ky_alloc((len + 1) as i64);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        *ptr.add(len) = 0;
    }
    ptr
}
