use std::alloc::{alloc, dealloc, Layout};

/// Create a new byte buffer of given size. Returns pointer to the buffer.
/// The buffer is heap-allocated and must be freed with ky_bytes_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_new(size: i32) -> *mut u8 {
    if size <= 0 { return std::ptr::null_mut(); }
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { std::ptr::write_bytes(ptr, 0, size as usize); }
    ptr
}

/// Free a byte buffer created by ky_bytes_new.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_free(ptr: *mut u8, size: i32) {
    if ptr.is_null() || size <= 0 { return; }
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { dealloc(ptr, layout); }
}

/// Read a byte from a buffer at index.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_get(ptr: *const u8, index: i32) -> i32 {
    if ptr.is_null() || index < 0 { return -1; }
    unsafe { *ptr.add(index as usize) as i32 }
}

/// Write a byte to a buffer at index.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_set(ptr: *mut u8, index: i32, val: i32) {
    if ptr.is_null() || index < 0 { return; }
    unsafe { *ptr.add(index as usize) = val as u8; }
}

/// Convert byte buffer to hex string. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_to_hex(ptr: *const u8, size: i32) -> *mut u8 {
    if ptr.is_null() || size <= 0 { return std::ptr::null_mut(); }
    let slice = unsafe { std::slice::from_raw_parts(ptr, size as usize) };
    let hex: String = slice.iter().map(|b| format!("{:02x}", b)).collect();
    let bytes = hex.as_bytes();
    let out_len = bytes.len();
    let out = crate::ky_alloc((out_len + 1) as i64);
    if out.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, out_len);
        *out.add(out_len) = 0;
    }
    out
}

/// Convert hex string to byte buffer. Returns heap-allocated buffer, caller must ky_bytes_free.
/// Returns null on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_hex(s: *const u8, out_size: *mut i32) -> *mut u8 {
    if s.is_null() { return std::ptr::null_mut(); }
    let s = unsafe { std::ffi::CStr::from_ptr(s .cast()) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return std::ptr::null_mut(),
    };
    let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit() || *c == ' ' || *c == '-').collect();
    let clean: String = clean.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() % 2 != 0 { return std::ptr::null_mut(); }
    let out_len = clean.len() / 2;
    let layout = Layout::from_size_align(out_len, 1).unwrap();
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() { return std::ptr::null_mut(); }
    for i in 0..out_len {
        let byte_str = &clean[i*2..i*2+2];
        let byte = u8::from_str_radix(byte_str, 16).unwrap_or(0);
        unsafe { *ptr.add(i) = byte; }
    }
    unsafe { *out_size = out_len as i32; }
    ptr
}

/// Convert byte buffer to base64 string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_to_base64(ptr: *const u8, size: i32) -> *mut u8 {
    if ptr.is_null() || size <= 0 { return std::ptr::null_mut(); }
    let slice = unsafe { std::slice::from_raw_parts(ptr, size as usize) };
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, slice);
    let bytes = b64.as_bytes();
    let out_len = bytes.len();
    let out = crate::ky_alloc((out_len + 1) as i64);
    if out.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, out_len);
        *out.add(out_len) = 0;
    }
    out
}
