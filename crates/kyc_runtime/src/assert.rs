use std::process;

fn ky_failed(msg: &str) -> ! {
    eprintln!("KL ASSERT FAILED: {}", msg);
    process::abort();
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert(condition: i32) {
    if condition == 0 {
        ky_failed("assertion failed: condition was false");
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert_eq(a: i64, b: i64) {
    if a != b {
        ky_failed(&format!("assertion failed: {} != {}", a, b));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert_ne(a: i64, b: i64) {
    if a == b {
        ky_failed(&format!("assertion failed: {} == {}", a, b));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_assert_str_eq(a: *const u8, b: *const u8) {
    if a.is_null() || b.is_null() {
        ky_failed("assertion failed: string is null");
    }
    let a_str = unsafe { std::ffi::CStr::from_ptr(a as *const std::os::raw::c_char) };
    let b_str = unsafe { std::ffi::CStr::from_ptr(b as *const std::os::raw::c_char) };
    if a_str.to_bytes() != b_str.to_bytes() {
        ky_failed(&format!(
            "assertion failed: \"{}\" != \"{}\"",
            a_str.to_string_lossy(),
            b_str.to_string_lossy()
        ));
    }
}
