use std::ffi::CStr;
use std::os::raw::c_char;
use std::io::Write;
use std::sync::Mutex;

static LOG_LEVEL: Mutex<i32> = Mutex::new(0);
static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

const LEVEL_DEBUG: i32 = 0;
const LEVEL_INFO: i32 = 1;
const LEVEL_WARN: i32 = 2;
const LEVEL_ERROR: i32 = 3;

fn level_prefix(level: i32) -> &'static str {
    match level {
        0 => "DEBUG",
        1 => "INFO",
        2 => "WARN",
        3 => "ERROR",
        _ => "UNKNOWN",
    }
}

fn to_str(ptr: *const u8) -> &'static str {
    if ptr.is_null() { return ""; }
    unsafe { CStr::from_ptr(ptr as *const c_char) }
        .to_str().unwrap_or("")
}

fn write_output(buf: &[u8]) {
    let mut file = LOG_FILE.lock().unwrap();
    if let Some(ref mut f) = *file {
        let _ = f.write_all(buf);
        let _ = f.flush();
    } else {
        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(buf);
        let _ = stderr.flush();
    }
}

fn log_msg(level: i32, msg: *const u8) {
    let current_level = LOG_LEVEL.lock().unwrap();
    if level < *current_level { return; }
    let s = to_str(msg);
    let prefix = level_prefix(level);
    let output = format!("[{}] {}\n", prefix, s);
    write_output(output.as_bytes());
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_debug(msg: *const u8) {
    log_msg(LEVEL_DEBUG, msg);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_info(msg: *const u8) {
    log_msg(LEVEL_INFO, msg);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_warn(msg: *const u8) {
    log_msg(LEVEL_WARN, msg);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_error(msg: *const u8) {
    log_msg(LEVEL_ERROR, msg);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_set_level(level: i32) {
    let mut l = LOG_LEVEL.lock().unwrap();
    *l = level;
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_set_output(path: *const u8) -> i32 {
    let s = to_str(path);
    if s.is_empty() { return -1; }
    match std::fs::OpenOptions::new().create(true).append(true).open(s) {
        Ok(f) => {
            let mut file = LOG_FILE.lock().unwrap();
            *file = Some(f);
            0
        }
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_log_set_console() {
    let mut file = LOG_FILE.lock().unwrap();
    *file = None;
}
