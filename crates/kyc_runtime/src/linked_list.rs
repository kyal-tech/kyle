use std::collections::LinkedList;

/// Linked list implementation using Rust's LinkedList.
#[repr(C)]
pub struct KlLinkedList {
    list: LinkedList<i64>,
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_new() -> *mut std::ffi::c_void {
    let l = Box::new(KlLinkedList { list: LinkedList::new() });
    Box::into_raw(l) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_free(list: *mut std::ffi::c_void) {
    if list.is_null() { return; }
    unsafe { let _ = Box::from_raw(list as *mut KlLinkedList); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_push_back(list: *mut std::ffi::c_void, val: i64) {
    if list.is_null() { return; }
    unsafe { (*list.cast::<KlLinkedList>()).list.push_back(val); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_push_front(list: *mut std::ffi::c_void, val: i64) {
    if list.is_null() { return; }
    unsafe { (*list.cast::<KlLinkedList>()).list.push_front(val); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_pop_back(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() { return 0; }
    unsafe { (*list.cast::<KlLinkedList>()).list.pop_back().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_pop_front(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() { return 0; }
    unsafe { (*list.cast::<KlLinkedList>()).list.pop_front().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_peek_back(list: *const std::ffi::c_void) -> i64 {
    if list.is_null() { return 0; }
    unsafe { (*list.cast::<KlLinkedList>()).list.back().copied().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_peek_front(list: *const std::ffi::c_void) -> i64 {
    if list.is_null() { return 0; }
    unsafe { (*list.cast::<KlLinkedList>()).list.front().copied().unwrap_or(0) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_len(list: *const std::ffi::c_void) -> i64 {
    if list.is_null() { return 0; }
    unsafe { (*list.cast::<KlLinkedList>()).list.len() as i64 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_linked_list_clear(list: *mut std::ffi::c_void) {
    if list.is_null() { return; }
    unsafe { (*list.cast::<KlLinkedList>()).list.clear(); }
}
