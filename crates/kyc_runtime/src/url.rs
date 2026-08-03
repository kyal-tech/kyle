use std::ffi::CStr;
use std::os::raw::c_char;
use url::Url;
use percent_encoding::{utf8_percent_encode, percent_decode_str, NON_ALPHANUMERIC};

/// Helper: parse URL string, return Url or None
fn parse_url(s: *const u8) -> Option<Url> {
    if s.is_null() { return None; }
    let s = unsafe { CStr::from_ptr(s as *const c_char) };
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

/// Helper: get a handle (i64) from an Option<Box<Url>>
fn handle_from_url(u: Option<Box<Url>>) -> i64 {
    match u {
        Some(b) => Box::into_raw(b) as i64,
        None => 0,
    }
}

/// Helper: get &mut Url from handle
fn url_from_handle(h: i64) -> &'static mut Url {
    unsafe { &mut *(h as *mut Url) }
}

// ── Existing string-based URL functions (used by prelude) ──

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

// ── Handle-based URL functions (used by std.url module) ──

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_new() -> i64 {
    let u = Box::new(Url::parse("https://localhost").unwrap());
    handle_from_url(Some(u))
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_parse(s: *const u8) -> i64 {
    let u = parse_url(s).map(Box::new);
    handle_from_url(u)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_free(h: i64) {
    if h == 0 { return; }
    unsafe { drop(Box::from_raw(h as *mut Url)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_to_str(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    alloc_str(u.as_str())
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_scheme(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    alloc_str(u.scheme())
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_host(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    match u.host_str() {
        Some(s) => alloc_str(s),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_port(h: i64) -> i32 {
    if h == 0 { return -1; }
    let u = url_from_handle(h);
    u.port().map(|p| p as i32).unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_path(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    alloc_str(u.path())
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_query(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    alloc_str(u.query().unwrap_or(""))
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_fragment(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    alloc_str(u.fragment().unwrap_or(""))
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_userinfo(h: i64) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    let user = u.username();
    let pass = u.password().unwrap_or("");
    if pass.is_empty() {
        alloc_str(user)
    } else {
        alloc_str(&format!("{}:{}", user, pass))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_get_query_value(h: i64, key: *const u8) -> *mut u8 {
    if h == 0 || key.is_null() { return std::ptr::null_mut(); }
    let u = url_from_handle(h);
    let k = unsafe { CStr::from_ptr(key as *const c_char) };
    let k = k.to_str().unwrap_or("");
    for (name, value) in u.query_pairs() {
        if name == k {
            return alloc_str(&value);
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_set_scheme(h: i64, s: *const u8) -> i32 {
    if h == 0 || s.is_null() { return -1; }
    let u = url_from_handle(h);
    let val = unsafe { CStr::from_ptr(s as *const c_char) };
    let val = val.to_str().unwrap_or("");
    match u.set_scheme(val) {
        Ok(()) => 0,
        Err(()) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_set_host(h: i64, s: *const u8) -> i32 {
    if h == 0 || s.is_null() { return -1; }
    let u = url_from_handle(h);
    let val = unsafe { CStr::from_ptr(s as *const c_char) };
    let val = val.to_str().unwrap_or("");
    match u.set_host(Some(val)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_set_port(h: i64, p: i32) -> i32 {
    if h == 0 { return -1; }
    let u = url_from_handle(h);
    match u.set_port(Some(p as u16)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_set_path(h: i64, s: *const u8) -> i32 {
    if h == 0 || s.is_null() { return -1; }
    let u = url_from_handle(h);
    let val = unsafe { CStr::from_ptr(s as *const c_char) };
    let val = val.to_str().unwrap_or("");
    u.set_path(val);
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_set_query(h: i64, s: *const u8) -> i32 {
    if h == 0 || s.is_null() { return -1; }
    let u = url_from_handle(h);
    let val = unsafe { CStr::from_ptr(s as *const c_char) };
    let val = val.to_str().unwrap_or("");
    u.set_query(Some(val));
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_encode(s: *const u8) -> *mut u8 {
    if s.is_null() { return std::ptr::null_mut(); }
    let val = unsafe { CStr::from_ptr(s as *const c_char) };
    let val = val.to_str().unwrap_or("");
    let encoded: String = utf8_percent_encode(val, NON_ALPHANUMERIC).collect();
    alloc_str(&encoded)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_url_decode(s: *const u8) -> *mut u8 {
    if s.is_null() { return std::ptr::null_mut(); }
    let val = unsafe { CStr::from_ptr(s as *const c_char) };
    let val = val.to_str().unwrap_or("");
    let decoded = percent_decode_str(val).decode_utf8().unwrap_or_default();
    alloc_str(&decoded)
}
