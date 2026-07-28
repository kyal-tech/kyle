use std::collections::VecDeque;

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_new() -> *mut std::ffi::c_void {
    let s = Box::new(Vec::<i64>::new());
    Box::into_raw(s) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_free(s: *mut std::ffi::c_void) {
    if s.is_null() { return; }
    unsafe { drop(Box::from_raw(s as *mut Vec<i64>)); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_push(s: *mut std::ffi::c_void, val: i64) {
    if s.is_null() { return; }
    let ss = unsafe { &mut *(s as *mut Vec<i64>) };
    ss.push(val);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_pop(s: *mut std::ffi::c_void) -> i64 {
    if s.is_null() { return 0; }
    let ss = unsafe { &mut *(s as *mut Vec<i64>) };
    ss.pop().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_peek(s: *const std::ffi::c_void) -> i64 {
    if s.is_null() { return 0; }
    let ss = unsafe { &*(s as *const Vec<i64>) };
    ss.last().copied().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_len(s: *const std::ffi::c_void) -> i64 {
    if s.is_null() { return 0; }
    let ss = unsafe { &*(s as *const Vec<i64>) };
    ss.len() as i64
}

/// Remove all elements from the stack.
#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_clear(stack: *mut std::ffi::c_void) {
    if stack.is_null() { return; }
    unsafe {
        let s = &mut *(stack as *mut Vec<i64>);
        s.clear();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_to_list(s: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    if s.is_null() { return std::ptr::null_mut(); }
    let ss = unsafe { &*(s as *const Vec<i64>) };
    let list = crate::list::ky_list_new();
    if list.is_null() { return std::ptr::null_mut(); }
    for &v in ss.iter() {
        crate::list::ky_list_push(list, v);
    }
    list as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_stack_to_queue(s: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    if s.is_null() { return std::ptr::null_mut(); }
    let ss = unsafe { &*(s as *const Vec<i64>) };
    let mut q = Box::new(VecDeque::<i64>::new());
    for &v in ss.iter() {
        q.push_back(v);
    }
    Box::into_raw(q) as *mut std::ffi::c_void
}
