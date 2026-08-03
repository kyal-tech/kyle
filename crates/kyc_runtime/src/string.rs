use core::ptr;
use std::alloc::{alloc, dealloc, realloc, Layout};

#[repr(C)]
pub struct strBuilder {
    data: *mut u8,
    len: i64,
    cap: i64,
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_builder_new(capacity: i64) -> *mut strBuilder {
    let cap = if capacity < 16 { 16i64 } else { capacity };
    let layout = Layout::from_size_align(
        cap as usize,
        16,
    ).unwrap_or_else(|_| Layout::from_size_align(16, 16).unwrap());
    let data = unsafe { alloc(layout) };
    if data.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { *data = 0; }
    let builder = Box::into_raw(Box::new(strBuilder {
        data,
        len: 0,
        cap,
    }));
    builder
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_builder_append(
    builder: *mut strBuilder,
    data: *const u8,
) {
    if builder.is_null() || data.is_null() {
        return;
    }
    let len = crate::ky_strlen(data) as i64;
    if len <= 0 {
        return;
    }
    let b = unsafe { &mut *builder };
    let new_len = b.len + len;
    if new_len > b.cap {
        let mut new_cap = b.cap * 2;
        while new_cap < new_len {
            new_cap *= 2;
        }
        let layout = Layout::from_size_align(b.cap as usize, 16)
            .unwrap_or_else(|_| Layout::from_size_align(16, 16).unwrap());
        let new_data = unsafe { realloc(b.data, layout, new_cap as usize) };
        if new_data.is_null() {
            return;
        }
        b.data = new_data;
        b.cap = new_cap;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(data, b.data.add(b.len as usize), len as usize);
        *b.data.add(new_len as usize) = 0;
    }
    b.len = new_len;
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_builder_to_str(builder: *const strBuilder) -> *mut u8 {
    if builder.is_null() {
        return std::ptr::null_mut();
    }
    let b = unsafe { &*builder };
    if b.len == 0 {
        let buf = crate::ky_alloc(1);
        if !buf.is_null() {
            unsafe { *buf = 0; }
        }
        return buf;
    }
    let buf = crate::ky_alloc(b.len + 1);
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(b.data, buf, b.len as usize);
        *buf.add(b.len as usize) = 0;
    }
    buf
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_builder_free(builder: *mut strBuilder) {
    if builder.is_null() {
        return;
    }
    let b = unsafe { Box::from_raw(builder) };
    if !b.data.is_null() {
        let layout = Layout::from_size_align(b.cap as usize, 16)
            .unwrap_or_else(|_| Layout::from_size_align(16, 16).unwrap());
        unsafe { dealloc(b.data, layout); }
    }
}

/// Convert f32 to string (via integer formatting — fptosi result).
#[unsafe(no_mangle)]
pub extern "C" fn ky_f32_to_str(val: i64) -> *const u8 {
    ky_i64_to_str(val)
}

/// Convert an i64 to its string representation.
/// Returns a heap-allocated null-terminated C string.
/// Caller must free with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_i64_to_str(val: i64) -> *const u8 {
    let mut tmp: [u8; 32] = [0u8; 32];
    let len = 32usize;
    let mut n = if val < 0 { -val } else { val };
    let mut i = len;
    loop {
        i -= 1;
        tmp[i] = (n % 10) as u8 + b'0';
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if val < 0 {
        i -= 1;
        tmp[i] = b'-';
    }
    let s = &tmp[i..];
    let alloc_len = s.len();
    let buf = crate::ky_alloc((alloc_len + 1) as i64);
    if buf.is_null() {
        return std::ptr::null();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.as_ptr(), buf, alloc_len);
        *buf.add(alloc_len) = 0;
    }
    buf .cast()
}

/// Convert an f64 to its string representation.
/// Returns a heap-allocated null-terminated C string.
/// Caller must free with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_f64_to_str(val: f64) -> *const u8 {
    // Use Rust's format to get the string, then copy to C string
    let s = if val == val.floor() && val.is_finite() {
        // No fractional part — use integer representation to avoid trailing .0
        // But always include at least one decimal to signal this is a float
        format!("{:.6}", val)
    } else {
        format!("{}", val)
    };
    let bytes = s.as_bytes();
    let len = bytes.len();
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() {
        return std::ptr::null();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *buf.add(len) = 0;
    }
    buf .cast()
}

/// Parse a null-terminated C string as a signed integer (i64).
///
/// Mirrors Rust's `str::parse::<i64>()` semantics: optional leading
/// `+`/`-`, decimal digits, stops at the first non-digit byte. Leading
/// whitespace is skipped. Returns 0 if the string is empty or contains no
/// digits. Overflow saturates to i64::MAX/MIN (no UB).
///
/// This is the inverse of `ky_i64_to_str`.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_to_i32(ptr: *const u8) -> i32 {
    ky_str_to_i64(ptr) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_to_i64(ptr: *const u8) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    let len = ky_strlen(ptr) as usize;
    if len == 0 {
        return 0;
    }
    let bytes: &[u8] = unsafe { core::slice::from_raw_parts(ptr, len) };
    // Skip leading ASCII whitespace.
    let mut start = 0;
    while start < bytes.len() && bytes[start] <= b' ' {
        start += 1;
    }
    if start >= bytes.len() {
        return 0;
    }
    let rest = &bytes[start..];
    // Determine sign.
    let (negative, digits) = match rest[0] {
        b'-' => (true, &rest[1..]),
        b'+' => (false, &rest[1..]),
        _ => (false, rest),
    };
    if digits.is_empty() {
        return 0;
    }
    // Accumulate with saturating overflow.
    let mut result: i64 = 0;
    for &b in digits {
        if !(b'0'..=b'9').contains(&b) {
            break;
        }
        let d = (b - b'0') as i64;
        result = match result.checked_mul(10).and_then(|v| {
            if negative { v.checked_sub(d) } else { v.checked_add(d) }
        }) {
            Some(v) => v,
            None => return if negative { i64::MIN } else { i64::MAX },
        };
    }
    result
}

/// Get the length of a null-terminated C string (strlen).
#[unsafe(no_mangle)]
pub extern "C" fn ky_strlen(ptr: *const u8) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    let mut len: i32 = 0;
    unsafe {
        while *ptr.add(len as usize) != 0 {
            len += 1;
        }
    }
    len
}

/// Concatenate two strings into a newly allocated buffer.
/// Returns a pointer to the concatenated null-terminated string.
/// The caller must free the result with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_concat(a: *const u8, a_len: i32, b: *const u8, b_len: i32) -> *mut u8 {
    if a.is_null() && b.is_null() {
        return core::ptr::null_mut();
    }
    let len = a_len + b_len;
    let total = (len + 1) as usize;
    let result = crate::ky_alloc(total as i64);
    if result.is_null() {
        return core::ptr::null_mut();
    }
    if !a.is_null() && a_len > 0 {
        unsafe {
            ptr::copy_nonoverlapping(a, result, a_len as usize);
        }
    }
    if !b.is_null() && b_len > 0 {
        unsafe {
            ptr::copy_nonoverlapping(b, result.add(a_len as usize), b_len as usize);
        }
    }
    unsafe {
        *result.add(len as usize) = 0;
    }
    result
}

/// Check if haystack contains needle.
/// Returns 1 if found, 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_contains(haystack: *const u8, needle: *const u8) -> i32 {
    if haystack.is_null() || needle.is_null() {
        return 0i32;
    }
    let hl = ky_strlen(haystack) as usize;
    let nl = ky_strlen(needle) as usize;
    if nl == 0 { return 1i32; }
    if nl > hl { return 0i32; }
    unsafe {
        for i in 0..=hl - nl {
            if core::ptr::read_unaligned(haystack.add(i) as *const [u8; 1]) == core::ptr::read_unaligned(needle as *const [u8; 1]) {
                let mut found = true;
                for j in 0..nl {
                    if *haystack.add(i + j) != *needle.add(j) {
                        found = false;
                        break;
                    }
                }
                if found { return 1i32; }
            }
        }
    }
    0i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_starts_with(s: *const u8, prefix: *const u8) -> i32 {
    if s.is_null() || prefix.is_null() { return 0i32; }
    let sl = ky_strlen(s) as usize;
    let pl = ky_strlen(prefix) as usize;
    if pl > sl { return 0i32; }
    unsafe {
        for i in 0..pl {
            if *s.add(i) != *prefix.add(i) { return 0i32; }
        }
    }
    1i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_ends_with(s: *const u8, suffix: *const u8) -> i32 {
    if s.is_null() || suffix.is_null() { return 0i32; }
    let sl = ky_strlen(s) as usize;
    let pl = ky_strlen(suffix) as usize;
    if pl > sl { return 0i32; }
    unsafe {
        for i in 0..pl {
            if *s.add(sl - pl + i) != *suffix.add(i) { return 0i32; }
        }
    }
    1i32
}

/// Convert string to uppercase. Returns heap-allocated string, caller must ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_to_upper(ptr: *const u8) -> *mut u8 {
    let len = ky_strlen(ptr) as usize;
    let result = crate::ky_alloc((len + 1) as i64);
    if result.is_null() { return result; }
    unsafe {
        for i in 0..len {
            let c = *ptr.add(i);
            *result.add(i) = if c >= b'a' && c <= b'z' { c - 32 } else { c };
        }
        *result.add(len) = 0;
    }
    result
}

/// Convert string to lowercase. Returns heap-allocated string, caller must ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_to_lower(ptr: *const u8) -> *mut u8 {
    let len = ky_strlen(ptr) as usize;
    let result = crate::ky_alloc((len + 1) as i64);
    if result.is_null() { return result; }
    unsafe {
        for i in 0..len {
            let c = *ptr.add(i);
            *result.add(i) = if c >= b'A' && c <= b'Z' { c + 32 } else { c };
        }
        *result.add(len) = 0;
    }
    result
}

/// Trim whitespace from both ends. Returns heap-allocated string, caller must ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_trim(ptr: *const u8) -> *mut u8 {
    let len = ky_strlen(ptr) as usize;
    if len == 0 { return core::ptr::null_mut(); }
    unsafe {
        let mut start = 0;
        while start < len && *ptr.add(start) <= b' ' { start += 1; }
        let mut end = len;
        while end > start && *ptr.add(end - 1) <= b' ' { end -= 1; }
        let new_len = end - start;
        let result = crate::ky_alloc((new_len + 1) as i64);
        if result.is_null() { return result; }
        core::ptr::copy_nonoverlapping(ptr.add(start), result, new_len);
        *result.add(new_len) = 0;
        result
    }
}

/// Replace all occurrences of `from` with `to`. Returns heap-allocated string, caller must ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_replace(ptr: *const u8, from: *const u8, to: *const u8) -> *mut u8 {
    if ptr.is_null() { return core::ptr::null_mut(); }
    let slen = ky_strlen(ptr) as usize;
    let flen = ky_strlen(from) as usize;
    let tlen = ky_strlen(to) as usize;
    if flen == 0 || flen > slen {
        // no replacement possible, return copy
        let result = crate::ky_alloc((slen + 1) as i64);
        if result.is_null() { return result; }
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, result, slen);
            *result.add(slen) = 0;
        }
        return result;
    }
    unsafe {
        // count occurrences
        let mut count = 0i64;
        let mut i = 0;
        while i + flen <= slen {
            let mut match_ = true;
            for j in 0..flen {
                if *ptr.add(i + j) != *from.add(j) { match_ = false; break; }
            }
            if match_ { count += 1; i += flen; }
            else { i += 1; }
        }
        let new_len = slen + (tlen.saturating_sub(flen)) * (count as usize);
        let result = crate::ky_alloc((new_len + 1) as i64);
        if result.is_null() { return result; }
        let mut wp = result;
        let mut rp = ptr;
        let mut remaining = slen;
        while remaining >= flen {
            let mut match_ = true;
            for j in 0..flen {
                if *rp.add(j) != *from.add(j) { match_ = false; break; }
            }
            if match_ {
                core::ptr::copy_nonoverlapping(to, wp, tlen);
                wp = wp.add(tlen);
                rp = rp.add(flen);
                remaining -= flen;
            } else {
                *wp = *rp;
                wp = wp.add(1);
                rp = rp.add(1);
                remaining -= 1;
            }
        }
        while remaining > 0 {
            *wp = *rp;
            wp = wp.add(1);
            rp = rp.add(1);
            remaining -= 1;
        }
        *wp = 0;
        result
    }
}

// ---------------------------------------------------------------------------
// Character operations
// ---------------------------------------------------------------------------

/// Convert a char (i8) to a single-character string.
/// Returns a heap-allocated null-terminated C string.
/// Caller must free with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_char_to_str(c: i32) -> *const u8 {
    let buf = crate::ky_alloc(2);
    if buf.is_null() {
        return std::ptr::null();
    }
    unsafe {
        *buf = c as u8;
        *buf.add(1) = 0;
    }
    buf
}

/// Return the byte at position `index` in string `ptr`, or 0 if out of bounds.
#[unsafe(no_mangle)]
pub extern "C" fn ky_char_at(ptr: *const u8, index: i32) -> i32 {
    if ptr.is_null() { return 0; }
    let len = ky_strlen(ptr);
    if index < 0 || index >= len { return 0; }
    unsafe { *ptr.add(index as usize) as i32 }
}

/// Returns 1 if the byte is an ASCII digit ('0'..'9').
#[unsafe(no_mangle)]
pub extern "C" fn ky_is_digit(c: i32) -> i32 {
    if c >= b'0' as i32 && c <= b'9' as i32 { 1 } else { 0 }
}

/// Returns 1 if the byte is an ASCII letter ('a'..'z' or 'A'..'Z').
#[unsafe(no_mangle)]
pub extern "C" fn ky_is_alpha(c: i32) -> i32 {
    if (c >= b'a' as i32 && c <= b'z' as i32) || (c >= b'A' as i32 && c <= b'Z' as i32) { 1 } else { 0 }
}

/// Returns 1 if the byte is an ASCII letter or digit.
#[unsafe(no_mangle)]
pub extern "C" fn ky_is_alnum(c: i32) -> i32 {
    ky_is_digit(c) | ky_is_alpha(c)
}

/// Returns 1 if the byte is whitespace (space, tab, newline, carriage return).
#[unsafe(no_mangle)]
pub extern "C" fn ky_is_whitespace(c: i32) -> i32 {
    let u = c as u8;
    if u == b' ' || u == b'\t' || u == b'\n' || u == b'\r' { 1 } else { 0 }
}

/// Returns 1 if the byte is an uppercase ASCII letter.
#[unsafe(no_mangle)]
pub extern "C" fn ky_is_upper(c: i32) -> i32 {
    if c >= b'A' as i32 && c <= b'Z' as i32 { 1 } else { 0 }
}

/// Returns 1 if the byte is a lowercase ASCII letter.
#[unsafe(no_mangle)]
pub extern "C" fn ky_is_lower(c: i32) -> i32 {
    if c >= b'a' as i32 && c <= b'z' as i32 { 1 } else { 0 }
}

/// Convert a char (i8) to its i32 representation.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ord(c: i32) -> i32 {
    c as i32
}

/// Extract a substring from a C string.
/// Returns a heap-allocated, null-terminated string.
/// start: byte offset, count: number of bytes to extract.
#[unsafe(no_mangle)]
pub extern "C" fn ky_substr(s: *const u8, start: i64, count: i64) -> *const u8 {
    if s.is_null() || count <= 0 {
        return std::ptr::null();
    }
    let len = count as usize;
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() {
        return std::ptr::null();
    }
    unsafe {
        for i in 0..len {
            *buf.add(i) = *s.add(start as usize + i);
        }
        *buf.add(len) = 0;
    }
    buf .cast()
}

/// Clone a substring given a pointer and length (no null termination needed).
/// Returns a heap-allocated null-terminated copy. Caller must free with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_clone_substr(s: *const u8, len: i64) -> *const u8 {
    if s.is_null() || len <= 0 {
        let buf = crate::ky_alloc(1);
        if buf.is_null() { return std::ptr::null(); }
        unsafe { *buf = 0; }
        return buf;
    }
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() {
        return std::ptr::null();
    }
    unsafe {
        for i in 0..len as usize {
            *buf.add(i) = *s.add(i);
        }
        *buf.add(len as usize) = 0;
    }
    buf.cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_clone_str(s: *const u8) -> *const u8 {
    if s.is_null() {
        // Return empty string instead of null to avoid crashes downstream
        let buf = crate::ky_alloc(1);
        if buf.is_null() { return std::ptr::null(); }
        unsafe { *buf = 0; }
        return buf;
    }
    let len = ky_strlen(s);
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() {
        return std::ptr::null();
    }
    unsafe {
        for i in 0..len as usize {
            *buf.add(i) = *s.add(i);
        }
        *buf.add(len as usize) = 0;
    }
    buf .cast()
}

/// Compare two C strings for equality.
#[unsafe(no_mangle)]
pub extern "C" fn ky_eq_str(a: *const u8, b: *const u8) -> i32 {
    if a.is_null() || b.is_null() {
        return 0;
    }
    unsafe {
        let mut i: isize = 0;
        loop {
            let ca = *a.offset(i);
            let cb = *b.offset(i);
            if ca != cb {
                return 0;
            }
            if ca == 0 {
                return 1;
            }
            i += 1;
        }
    }
}

/// Compare two C strings lexicographically.
/// Returns -1 if a < b, 0 if a == b, 1 if a > b.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_cmp(a: *const u8, b: *const u8) -> i32 {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }
    unsafe {
        let mut i: isize = 0;
        loop {
            let ca = *a.offset(i);
            let cb = *b.offset(i);
            if ca < cb {
                return -1;
            }
            if ca > cb {
                return 1;
            }
            if ca == 0 {
                return 0;
            }
            i += 1;
        }
    }
}

/// Convert a null-terminated C string pointer to a heap-allocated string.
/// Returns null on null input.
#[unsafe(no_mangle)]
pub extern "C" fn ky_from_cstr(s: *const u8) -> *mut u8 {
    if s.is_null() {
        return core::ptr::null_mut();
    }
    let len = ky_strlen(s);
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        core::ptr::copy_nonoverlapping(s, buf, len as usize);
        *buf.add(len as usize) = 0;
    }
    buf
}

/// Set an environment variable. Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn ky_setenv(name: *const u8, value: *const u8, overwrite: i32) -> i32 {
    if name.is_null() || value.is_null() {
        return -1;
    }
    let name_str = unsafe { std::ffi::CStr::from_ptr(name as *const std::os::raw::c_char) }
        .to_str().unwrap_or("");
    let val_str = unsafe { std::ffi::CStr::from_ptr(value as *const std::os::raw::c_char) }
        .to_str().unwrap_or("");
    if overwrite != 0 || std::env::var(name_str).is_err() {
        // SAFETY: single-threaded context (Kyle FFI call)
        unsafe { std::env::set_var(name_str, val_str); }
    }
    0
}

/// Read environment variable by name. Returns heap-allocated string or null.
#[unsafe(no_mangle)]
pub extern "C" fn ky_getenv(name: *const u8) -> *mut u8 {
    if name.is_null() {
        return core::ptr::null_mut();
    }
    let c_name = unsafe { core::ffi::CStr::from_ptr(name .cast()) };
    match std::env::var(c_name.to_str().unwrap_or("")) {
        Ok(val) => {
            let bytes = val.as_bytes();
            let len = bytes.len();
            let buf = crate::ky_alloc((len + 1) as i64);
            if buf.is_null() {
                return core::ptr::null_mut();
            }
            unsafe {
                core::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
                *buf.add(len) = 0;
            }
            buf
        }
        Err(_) => core::ptr::null_mut(),
    }
}

/// Bootstrap helper: str_len(s) returns string length (matches Kyle str→C mapping)
#[unsafe(no_mangle)]
pub extern "C" fn str_len(s: *const u8) -> i32 {
    ky_strlen(s)
}

/// Bootstrap helper: str_cat(a, b) concatenates two strings (simple ptr+ptr, no len args)
#[unsafe(no_mangle)]
pub extern "C" fn str_cat(a: *const u8, b: *const u8) -> *const u8 {
    ky_concat(a, ky_strlen(a), b, ky_strlen(b))
}

/// Find index of first occurrence of a substring. Returns -1 if not found.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_index_of(s: *const u8, substr: *const u8) -> i32 {
    if s.is_null() || substr.is_null() { return -1; }
    let s_str = unsafe { std::ffi::CStr::from_ptr(s.cast::<std::ffi::c_char>()) }.to_str().unwrap_or("");
    let sub_str = unsafe { std::ffi::CStr::from_ptr(substr.cast::<std::ffi::c_char>()) }.to_str().unwrap_or("");
    s_str.find(sub_str).map(|i| i as i32).unwrap_or(-1)
}

/// Split a string by a delimiter. Returns a list of substrings.
#[unsafe(no_mangle)]
pub extern "C" fn ky_str_split(s: *const u8, delimiter: *const u8) -> *mut std::ffi::c_void {
    use crate::list::{ky_list_new, ky_list_push};
    use crate::string::ky_clone_str;
    let result = ky_list_new();
    if s.is_null() || delimiter.is_null() { return result as *mut std::ffi::c_void; }
    let s_str = unsafe { std::ffi::CStr::from_ptr(s.cast::<std::ffi::c_char>()) }.to_str().unwrap_or("");
    let delim_str = unsafe { std::ffi::CStr::from_ptr(delimiter.cast::<std::ffi::c_char>()) }.to_str().unwrap_or("");
    if delim_str.is_empty() {
        // Split into characters
        for c in s_str.chars() {
            let mut buf = [0u8; 8];
            let cs = c.encode_utf8(&mut buf);
            let cstr = ky_clone_str(cs.as_ptr());
            ky_list_push(result, cstr as i64);
        }
    } else {
        for part in s_str.split(delim_str) {
            let cstr = ky_clone_substr(part.as_ptr(), part.len() as i64);
            ky_list_push(result, cstr as i64);
        }
    }
    result as *mut std::ffi::c_void
}

// Bootstrap helpers for ky2c.ky StringBuilder optimization
use crate::list::{ky_list_new, ky_list_push, ky_list_len, ky_list_get};

/// Bootstrap helper: str_builder_new() creates a new string builder.
/// Returns a pointer to a list (used internally as a char buffer).
#[unsafe(no_mangle)]
pub extern "C" fn str_builder_new() -> *mut std::ffi::c_void {
    ky_list_new() as *mut std::ffi::c_void
}

/// Bootstrap helper: str_builder_append(sb, s) appends a string to the builder.
#[unsafe(no_mangle)]
pub extern "C" fn str_builder_append(sb: *mut std::ffi::c_void, s: *const u8) {
    if sb.is_null() || s.is_null() { return; }
    let len = ky_strlen(s);
    let list = sb as *mut u8 as _;
    for i in 0..len {
        unsafe {
            let c = *s.add(i as usize);
            ky_list_push(list, c as i64);
        }
    }
}

/// Bootstrap helper: str_builder_to_str(sb) converts builder to a string.
#[unsafe(no_mangle)]
pub extern "C" fn str_builder_to_str(sb: *mut std::ffi::c_void) -> *mut u8 {
    if sb.is_null() { return std::ptr::null_mut(); }
    let list = sb as *mut u8 as _;
    let len = ky_list_len(list) as usize;
    let result = crate::memory::ky_alloc((len + 1) as i64) as *mut u8;
    for i in 0..len {
        unsafe {
            let val = ky_list_get(list, i as i64);
            *result.add(i) = val as u8;
        }
    }
    unsafe { *result.add(len) = 0; } // null-terminate
    result
}

/// Bootstrap helper: str_builder_free(sb) — no-op.
/// Runtime automatically frees str-type variables at scope exit via ky_release.
/// Explicit free would cause double-free since both this and the runtime free the same pointer.
#[unsafe(no_mangle)]
pub extern "C" fn str_builder_free(_sb: *mut std::ffi::c_void) {
    // No-op — runtime manages cleanup automatically
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_str_to_list(s: *const u8) -> *mut std::ffi::c_void {
    use crate::list::{ky_list_new, ky_list_push};
    let result = ky_list_new();
    if result.is_null() { return std::ptr::null_mut(); }
    if s.is_null() { return result as *mut std::ffi::c_void; }
    let len = ky_strlen(s);
    for i in 0..len {
        unsafe {
            let c = *s.add(i as usize);
            ky_list_push(result, c as i64);
        }
    }
    result as *mut std::ffi::c_void
}
