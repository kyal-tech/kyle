use std::collections::HashSet;

#[repr(C)]
pub struct KlSet {
    set: HashSet<i64>,
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_new() -> *mut std::ffi::c_void {
    let set = Box::new(KlSet { set: HashSet::new() });
    Box::into_raw(set) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_free(set: *mut std::ffi::c_void) {
    if set.is_null() { return; }
    unsafe { let _ = Box::from_raw(set as *mut KlSet); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_add(set: *mut std::ffi::c_void, val: i64) {
    if set.is_null() { return; }
    unsafe { (*set.cast::<KlSet>()).set.insert(val); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_contains(set: *const std::ffi::c_void, val: i64) -> i32 {
    if set.is_null() { return 0; }
    unsafe { (*set.cast::<KlSet>()).set.contains(&val) as i32 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_remove(set: *mut std::ffi::c_void, val: i64) {
    if set.is_null() { return; }
    unsafe { (*set.cast::<KlSet>()).set.remove(&val); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_len(set: *const std::ffi::c_void) -> i64 {
    if set.is_null() { return 0; }
    unsafe { (*set.cast::<KlSet>()).set.len() as i64 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_clear(set: *mut std::ffi::c_void) {
    if set.is_null() { return; }
    unsafe { (*set.cast::<KlSet>()).set.clear(); }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_union(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let result = ky_set_new();
    if !a.is_null() {
        unsafe {
            for &v in &(*a.cast::<KlSet>()).set {
                ky_set_add(result, v);
            }
        }
    }
    if !b.is_null() {
        unsafe {
            for &v in &(*b.cast::<KlSet>()).set {
                ky_set_add(result, v);
            }
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_intersection(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let result = ky_set_new();
    if a.is_null() || b.is_null() { return result; }
    unsafe {
        let set_a = &(*a.cast::<KlSet>()).set;
        let set_b = &(*b.cast::<KlSet>()).set;
        for &v in set_a {
            if set_b.contains(&v) {
                ky_set_add(result, v);
            }
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_difference(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let result = ky_set_new();
    if a.is_null() || b.is_null() { return result; }
    unsafe {
        let set_a = &(*a.cast::<KlSet>()).set;
        let set_b = &(*b.cast::<KlSet>()).set;
        for &v in set_a {
            if !set_b.contains(&v) {
                ky_set_add(result, v);
            }
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_symmetric_difference(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    let result = ky_set_new();
    if a.is_null() || b.is_null() { return result; }
    unsafe {
        let set_a = &(*a.cast::<KlSet>()).set;
        let set_b = &(*b.cast::<KlSet>()).set;
        for &v in set_a {
            if !set_b.contains(&v) {
                ky_set_add(result, v);
            }
        }
        for &v in set_b {
            if !set_a.contains(&v) {
                ky_set_add(result, v);
            }
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_is_subset(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> i32 {
    if a.is_null() || b.is_null() { return 0; }
    unsafe {
        let set_a = &(*a.cast::<KlSet>()).set;
        let set_b = &(*b.cast::<KlSet>()).set;
        set_a.is_subset(set_b) as i32
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_set_to_list(a: *const std::ffi::c_void) -> *mut std::ffi::c_void {
    if a.is_null() { return std::ptr::null_mut(); }
    unsafe {
        let set_a = &(*a.cast::<KlSet>()).set;
        let list = crate::list::ky_list_new();
        if list.is_null() { return std::ptr::null_mut(); }
        for &v in set_a {
            crate::list::ky_list_push(list, v);
        }
        list as *mut std::ffi::c_void
    }
}
