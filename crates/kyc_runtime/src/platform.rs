//! C-extern wrappers for kyc_platform functions exposed as namespaced APIs.

use std::ffi::CStr;
use std::os::raw::c_char;

fn to_str(ptr: *const u8) -> &'static str {
    if ptr.is_null() { return ""; }
    unsafe { CStr::from_ptr(ptr as *const c_char) }
        .to_str().unwrap_or("")
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_exists(path: *const u8) -> i32 {
    if kyc_platform::fs::exists(to_str(path)) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_is_dir(path: *const u8) -> i32 {
    if kyc_platform::fs::is_dir(to_str(path)) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_is_file(path: *const u8) -> i32 {
    if kyc_platform::fs::is_file(to_str(path)) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_size(path: *const u8) -> i64 {
    kyc_platform::fs::size(to_str(path))
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_copy(src: *const u8, dst: *const u8) -> i32 {
    match kyc_platform::fs::copy(to_str(src), to_str(dst)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_remove(path: *const u8) -> i32 {
    match kyc_platform::fs::remove(to_str(path)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_create_dir(path: *const u8) -> i32 {
    match kyc_platform::fs::create_dir(to_str(path)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_remove_dir(path: *const u8) -> i32 {
    match kyc_platform::fs::remove_dir(to_str(path)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_rename(src: *const u8, dst: *const u8) -> i32 {
    match kyc_platform::fs::rename(to_str(src), to_str(dst)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_read_to_string(path: *const u8) -> *mut u8 {
    match kyc_platform::fs::read_to_string(to_str(path)) {
        Ok(s) => {
            let bytes = s.as_bytes();
            let len = bytes.len();
            let buf = crate::ky_alloc((len + 1) as i64);
            if buf.is_null() { return std::ptr::null_mut(); }
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
                *buf.add(len) = 0;
            }
            buf
        }
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_write_string(path: *const u8, data: *const u8) -> i32 {
    match kyc_platform::fs::write_string(to_str(path), to_str(data)) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_fs_list_dir(path: *const u8) -> i64 {
    if path.is_null() { return -1; }
    let s = unsafe { CStr::from_ptr(path as *const c_char) }
        .to_str().unwrap_or("");
    match std::fs::read_dir(s) {
        Ok(rd) => rd.count() as i64,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_time_now_ms() -> i64 {
    kyc_platform::time::now_ms()
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_time_now_us() -> i64 {
    kyc_platform::time::now_us()
}
