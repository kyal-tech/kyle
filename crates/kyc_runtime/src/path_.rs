use std::ffi::CString;
use std::path::PathBuf;

fn to_str(ptr: *const u8) -> &'static str {
    if ptr.is_null() { return ""; }
    unsafe { std::ffi::CStr::from_ptr(ptr as *const std::os::raw::c_char) }
        .to_str().unwrap_or("")
}

fn alloc_str(s: &str) -> *mut u8 {
    let bytes = s.as_bytes();
    let out = crate::ky_alloc((bytes.len() + 1) as i64);
    if out.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len());
        *out.add(bytes.len()) = 0;
    }
    out
}

/// Create a PathBuf from a string. Returns heap-allocated pointer as i64.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_new(path: *const u8) -> i64 {
    let pb = PathBuf::from(to_str(path));
    Box::into_raw(Box::new(pb)) as i64
}

/// Get parent directory. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_dirname(ptr: i64) -> *mut u8 {
    if ptr == 0 { return std::ptr::null_mut(); }
    let pb = unsafe { &*(ptr as *const PathBuf) };
    let dir = pb.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    alloc_str(&dir)
}

/// Get file name (basename). Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_basename(ptr: i64) -> *mut u8 {
    if ptr == 0 { return std::ptr::null_mut(); }
    let pb = unsafe { &*(ptr as *const PathBuf) };
    let name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    alloc_str(&name)
}

/// Get file extension. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_extension(ptr: i64) -> *mut u8 {
    if ptr == 0 { return std::ptr::null_mut(); }
    let pb = unsafe { &*(ptr as *const PathBuf) };
    let ext = pb.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
    alloc_str(&ext)
}

/// Join path with another component. Returns new heap-allocated PathBuf ptr.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_join(ptr: i64, other: *const u8) -> i64 {
    if ptr == 0 { return 0; }
    let pb = unsafe { &*(ptr as *const PathBuf) };
    let joined = pb.join(to_str(other));
    Box::into_raw(Box::new(joined)) as i64
}

/// Convert PathBuf to string. Returns heap-allocated C string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_to_str(ptr: i64) -> *mut u8 {
    if ptr == 0 { return std::ptr::null_mut(); }
    let pb = unsafe { &*(ptr as *const PathBuf) };
    alloc_str(&pb.to_string_lossy())
}

/// Free a PathBuf.
#[unsafe(no_mangle)]
pub extern "C" fn ky_path_free(ptr: i64) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut PathBuf)); }
    }
}
