use std::thread;

/// Spawn a thread running `func(arg)`. Returns an opaque handle (i64 pointer).
/// `func` must be an `extern "C" fn(i64) -> i64` — guaranteed by codegen.
#[unsafe(no_mangle)]
pub extern "C" fn ky_spawn_thread(func: Option<unsafe extern "C" fn(i64) -> i64>, arg: i64) -> i64 {
    let handle = thread::spawn(move || {
        let f = func.expect("ky_spawn_thread: null function pointer");
        unsafe { f(arg) }
    });
    Box::into_raw(Box::new(handle)) as i64
}

/// Join a thread spawned by `ky_spawn_thread`. Returns the i64 result.
#[unsafe(no_mangle)]
pub extern "C" fn ky_join_thread(handle_ptr: i64) -> i64 {
    let handle = unsafe { Box::from_raw(handle_ptr as *mut thread::JoinHandle<i64>) };
    handle.join().unwrap_or(0)
}
