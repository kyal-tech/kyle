use std::collections::VecDeque;

/// Double-ended queue implementation using Rust's VecDeque.
#[repr(C)]
pub struct KlDeque {
    deque: VecDeque<i64>,
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_new() -> *mut std::ffi::c_void {
    let d = Box::new(KlDeque { deque: VecDeque::new() });
    Box::into_raw(d) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_free(deque: *mut std::ffi::c_void) {
    if deque.is_null() { return; }
    unsafe { let _ = Box::from_raw(deque as *mut KlDeque); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_push_back(deque: *mut std::ffi::c_void, val: i64) {
    if deque.is_null() { return; }
    unsafe { (*deque.cast::<KlDeque>()).deque.push_back(val); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_push_front(deque: *mut std::ffi::c_void, val: i64) {
    if deque.is_null() { return; }
    unsafe { (*deque.cast::<KlDeque>()).deque.push_front(val); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_pop_back(deque: *mut std::ffi::c_void) -> i64 {
    if deque.is_null() { return 0; }
    unsafe { (*deque.cast::<KlDeque>()).deque.pop_back().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_pop_front(deque: *mut std::ffi::c_void) -> i64 {
    if deque.is_null() { return 0; }
    unsafe { (*deque.cast::<KlDeque>()).deque.pop_front().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_peek_back(deque: *const std::ffi::c_void) -> i64 {
    if deque.is_null() { return 0; }
    unsafe { (*deque.cast::<KlDeque>()).deque.back().copied().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_peek_front(deque: *const std::ffi::c_void) -> i64 {
    if deque.is_null() { return 0; }
    unsafe { (*deque.cast::<KlDeque>()).deque.front().copied().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_len(deque: *const std::ffi::c_void) -> i64 {
    if deque.is_null() { return 0; }
    unsafe { (*deque.cast::<KlDeque>()).deque.len() as i64 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_deque_clear(deque: *mut std::ffi::c_void) {
    if deque.is_null() { return; }
    unsafe { (*deque.cast::<KlDeque>()).deque.clear(); }
}
