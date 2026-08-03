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

/// Decode base64 string to byte buffer. Returns heap-allocated buffer.
/// out_size receives the buffer size. Caller must ky_bytes_free the result.
#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_base64(s: *const u8, out_size: *mut i32) -> *mut u8 {
    if s.is_null() { return std::ptr::null_mut(); }
    let s = unsafe { std::ffi::CStr::from_ptr(s as *const std::os::raw::c_char) };
    let s = match s.to_str() {
        Ok(s) => s.trim(),
        Err(_) => return std::ptr::null_mut(),
    };
    let result = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s);
    let bytes = match result {
        Ok(b) => b,
        Err(_) => return std::ptr::null_mut(),
    };
    let len = bytes.len();
    let layout = Layout::from_size_align(len, 1).unwrap();
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        *out_size = len as i32;
    }
    ptr
}

// ── Endian conversion ──

fn alloc_n(n: usize) -> *mut u8 {
    let layout = Layout::from_size_align(n, 1).unwrap();
    let ptr = unsafe { alloc(layout) };
    if !ptr.is_null() { unsafe { std::ptr::write_bytes(ptr, 0, n); } }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_to_be_i32(val: i32) -> *mut u8 {
    let ptr = alloc_n(4);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { std::ptr::write_unaligned(ptr as *mut i32, val.to_be()); }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_to_le_i32(val: i32) -> *mut u8 {
    let ptr = alloc_n(4);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { std::ptr::write_unaligned(ptr as *mut i32, val.to_le()); }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_to_be_i64(val: i64) -> *mut u8 {
    let ptr = alloc_n(8);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { std::ptr::write_unaligned(ptr as *mut i64, val.to_be()); }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_to_le_i64(val: i64) -> *mut u8 {
    let ptr = alloc_n(8);
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { std::ptr::write_unaligned(ptr as *mut i64, val.to_le()); }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_be_i32(ptr: *const u8) -> i32 {
    if ptr.is_null() { return 0; }
    unsafe { i32::from_be(std::ptr::read_unaligned(ptr as *const i32)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_le_i32(ptr: *const u8) -> i32 {
    if ptr.is_null() { return 0; }
    unsafe { i32::from_le(std::ptr::read_unaligned(ptr as *const i32)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_be_i64(ptr: *const u8) -> i64 {
    if ptr.is_null() { return 0; }
    unsafe { i64::from_be(std::ptr::read_unaligned(ptr as *const i64)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_le_i64(ptr: *const u8) -> i64 {
    if ptr.is_null() { return 0; }
    unsafe { i64::from_le(std::ptr::read_unaligned(ptr as *const i64)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_be_i32_at(ptr: *const u8, offset: i32) -> i32 {
    if ptr.is_null() || offset < 0 { return 0; }
    unsafe { i32::from_be(std::ptr::read_unaligned(ptr.add(offset as usize) as *const i32)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_bytes_from_be_i64_at(ptr: *const u8, offset: i32) -> i64 {
    if ptr.is_null() || offset < 0 { return 0; }
    unsafe { i64::from_be(std::ptr::read_unaligned(ptr.add(offset as usize) as *const i64)) }
}

// ── Buffer for building binary data ──

struct BytesBuffer {
    data: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_new(capacity: i32) -> i64 {
    let cap = if capacity < 64 { 64 } else { capacity as usize };
    let buf = Box::new(BytesBuffer { data: Vec::with_capacity(cap) });
    Box::into_raw(buf) as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_free(h: i64) {
    if h == 0 { return; }
    unsafe { drop(Box::from_raw(h as *mut BytesBuffer)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_len(h: i64) -> i32 {
    if h == 0 { return 0; }
    let buf = unsafe { &*(h as *mut BytesBuffer) };
    buf.data.len() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_write_byte(h: i64, b: i32) {
    if h == 0 { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    buf.data.push(b as u8);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_write_be_i32(h: i64, val: i32) {
    if h == 0 { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    buf.data.extend_from_slice(&val.to_be_bytes());
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_write_le_i32(h: i64, val: i32) {
    if h == 0 { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    buf.data.extend_from_slice(&val.to_le_bytes());
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_write_be_i64(h: i64, val: i64) {
    if h == 0 { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    buf.data.extend_from_slice(&val.to_be_bytes());
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_write_le_i64(h: i64, val: i64) {
    if h == 0 { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    buf.data.extend_from_slice(&val.to_le_bytes());
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_write_str(h: i64, s: *const u8) {
    if h == 0 || s.is_null() { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    let cstr = unsafe { std::ffi::CStr::from_ptr(s as *const std::os::raw::c_char) };
    let s = cstr.to_bytes();
    buf.data.extend_from_slice(s);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_to_bytes(h: i64, out_size: *mut i32) -> *mut u8 {
    if h == 0 { return std::ptr::null_mut(); }
    let buf = unsafe { &*(h as *mut BytesBuffer) };
    let len = buf.data.len();
    if len == 0 { unsafe { *out_size = 0; } return std::ptr::null_mut(); }
    let layout = Layout::from_size_align(len, 1).unwrap();
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(buf.data.as_ptr(), ptr, len);
        *out_size = len as i32;
    }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_buffer_clear(h: i64) {
    if h == 0 { return; }
    let buf = unsafe { &mut *(h as *mut BytesBuffer) };
    buf.data.clear();
}
