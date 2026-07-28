use url::Url;

/// Helper: parse URL string, return Url or None
fn parse_url(s: *const u8) -> Option<Url> {
    if s.is_null() { return None; }
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = s.to_str().ok()?.trim();
    Url::parse(s).ok()
}

/// Helper: alloc a string, return ptr (caller must free)
fn alloc_str(s: &str) -> *mut u8 {
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

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_scheme(url: *const u8) -> *mut u8 {
    parse_url(url).map(|u| alloc_str(u.scheme())).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_host(url: *const u8) -> *mut u8 {
    parse_url(url).and_then(|u| u.host_str().map(|h| alloc_str(h))).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_port(url: *const u8) -> i32 {
    parse_url(url).and_then(|u| u.port()).map(|p| p as i32).unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_path(url: *const u8) -> *mut u8 {
    parse_url(url).map(|u| alloc_str(u.path())).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_query(url: *const u8) -> *mut u8 {
    parse_url(url).map(|u| alloc_str(u.query().unwrap_or(""))).unwrap_or(std::ptr::null_mut())
}
