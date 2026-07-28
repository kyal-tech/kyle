use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

// === Mutex<i64> ===

#[unsafe(no_mangle)]
pub extern "C" fn ky_mutex_new(val: i64) -> i64 {
    let m = Mutex::new(val);
    Box::into_raw(Box::new(m)) as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_mutex_lock(ptr: i64) -> i64 {
    if ptr == 0 { return 0; }
    let m = unsafe { &*(ptr as *const Mutex<i64>) };
    match m.lock() {
        Ok(val) => *val,
        Err(_) => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_mutex_store(ptr: i64, val: i64) {
    if ptr == 0 { return; }
    let m = unsafe { &*(ptr as *const Mutex<i64>) };
    // We need mutable access for store. We'll use a different approach.
    // Actually let's just unlock and re-lock with the pattern the docs suggest.
    // For now, this is a simple store via unsafe.
    unsafe {
        let m_mut = &mut *(ptr as *mut Mutex<i64>);
        match m_mut.lock() {
            Ok(mut v) => *v = val,
            Err(_) => {}
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_mutex_free(ptr: i64) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut Mutex<i64>)); }
    }
}

// === AtomicI64 ===

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_i64_new(val: i64) -> i64 {
    let a = AtomicI64::new(val);
    Box::into_raw(Box::new(a)) as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_i64_load(ptr: i64) -> i64 {
    if ptr == 0 { return 0; }
    let a = unsafe { &*(ptr as *const AtomicI64) };
    a.load(Ordering::SeqCst)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_i64_store(ptr: i64, val: i64) {
    if ptr == 0 { return; }
    let a = unsafe { &*(ptr as *const AtomicI64) };
    a.store(val, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_i64_add(ptr: i64, val: i64) -> i64 {
    if ptr == 0 { return 0; }
    let a = unsafe { &*(ptr as *const AtomicI64) };
    a.fetch_add(val, Ordering::SeqCst)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_i64_free(ptr: i64) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut AtomicI64)); }
    }
}

// === AtomicBool ===

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_bool_new(val: i32) -> i64 {
    let a = AtomicBool::new(val != 0);
    Box::into_raw(Box::new(a)) as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_bool_load(ptr: i64) -> i32 {
    if ptr == 0 { return 0; }
    let a = unsafe { &*(ptr as *const AtomicBool) };
    if a.load(Ordering::SeqCst) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_bool_store(ptr: i64, val: i32) {
    if ptr == 0 { return; }
    let a = unsafe { &*(ptr as *const AtomicBool) };
    a.store(val != 0, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_atomic_bool_free(ptr: i64) {
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut AtomicBool)); }
    }
}
