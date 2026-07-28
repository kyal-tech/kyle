use num_bigint::BigInt;
use num_traits::Num;

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

/// Create BigInt from decimal string. Returns heap-allocated pointer as i64.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_from_str(s: *const u8) -> i64 {
    if s.is_null() { return 0; }
    let s = unsafe { std::ffi::CStr::from_ptr(s as *const std::os::raw::c_char) }
        .to_str().unwrap_or("0");
    match BigInt::from_str_radix(s, 10) {
        Ok(n) => Box::into_raw(Box::new(n)) as i64,
        Err(_) => 0,
    }
}

/// Create BigInt from i64.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_from_i64(val: i64) -> i64 {
    let n = BigInt::from(val);
    Box::into_raw(Box::new(n)) as i64
}

/// Add two BigInts. Returns new BigInt ptr.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_add(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 { return 0; }
    let lhs = unsafe { &*(a as *const BigInt) };
    let rhs = unsafe { &*(b as *const BigInt) };
    Box::into_raw(Box::new(lhs + rhs)) as i64
}

/// Subtract two BigInts.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_sub(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 { return 0; }
    let lhs = unsafe { &*(a as *const BigInt) };
    let rhs = unsafe { &*(b as *const BigInt) };
    Box::into_raw(Box::new(lhs - rhs)) as i64
}

/// Multiply two BigInts.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_mul(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 { return 0; }
    let lhs = unsafe { &*(a as *const BigInt) };
    let rhs = unsafe { &*(b as *const BigInt) };
    Box::into_raw(Box::new(lhs * rhs)) as i64
}

/// Convert BigInt to string. Returns heap-allocated C string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_to_str(ptr: i64) -> *mut u8 {
    if ptr == 0 { return std::ptr::null_mut(); }
    let n = unsafe { &*(ptr as *const BigInt) };
    alloc_str(&n.to_str_radix(10))
}

/// Free a BigInt.
#[unsafe(no_mangle)]
pub extern "C" fn ky_big_int_free(ptr: i64) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut BigInt)); }
    }
}
