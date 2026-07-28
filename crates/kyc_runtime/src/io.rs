fn write_stdout(buf: &[u8]) {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(buf);
    let _ = stdout.flush();
}

#[allow(dead_code)]
fn write_int(val: i64) {
    let mut buf = [0u8; 20];
    let mut n = if val < 0 {
        -val
    } else {
        val
    };
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if val < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    write_stdout(&buf[i..]);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_print(ptr: *const u8, len: i32) {
    if ptr.is_null() || len <= 0 {
        return;
    }
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len as usize);
        write_stdout(slice);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_println(ptr: *const u8, len: i32) {
    ky_print(ptr, len);
    write_stdout(b"\n");
}



/// Read a line from stdin, return heap-allocated null-terminated string.
/// Caller must free with ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_input() -> *mut u8 {
    ky_input_with_prompt(std::ptr::null(), 0)
}

/// Read a line from stdin with an optional prompt.
/// If prompt is non-null and prompt_len > 0, the prompt is printed first.
/// Returns heap-allocated null-terminated string (caller must ky_free).
#[unsafe(no_mangle)]
pub extern "C" fn ky_input_with_prompt(prompt: *const u8, prompt_len: i32) -> *mut u8 {
    if !prompt.is_null() && prompt_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(prompt, prompt_len as usize) };
        write_stdout(slice);
    }
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {
            let trimmed = line.trim_end_matches('\n');
            let len = trimmed.len();
            let ptr = crate::ky_alloc((len + 1) as i64);
            if !ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(trimmed.as_ptr(), ptr, len);
                    *ptr.add(len) = 0;
                }
            }
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

// Platform-specific constants for open flags
#[cfg(unix)]
mod platform {
    pub const O_RDONLY: i32 = 0;
    pub const O_WRONLY: i32 = 1;
    pub const O_RDWR: i32 = 2;
    pub const O_CREAT: i32 = 64;
    pub const O_TRUNC: i32 = 512;
    pub const O_APPEND: i32 = 1024;

    pub fn open_file(path: *const u8, flags: i32, mode: i32) -> i32 {
        unsafe {
            libc::open(path as *const libc::c_char, flags, mode as libc::c_int)
        }
    }

    pub fn read_fd(fd: i32, buf: *mut u8, count: i64) -> i64 {
        unsafe {
            libc::read(fd, buf as *mut libc::c_void, count as usize) as i64
        }
    }

    pub fn write_fd(fd: i64, buf: *const u8, count: i64) -> i64 {
        unsafe {
            libc::write(fd as i32, buf as *const libc::c_void, count as usize) as i64
        }
    }

    pub fn close_fd(fd: i32) -> i32 {
        unsafe { libc::close(fd) }
    }
}

#[cfg(windows)]
mod platform {
    // Placeholder constants — exact values don't matter since fns return -1
    pub const O_RDONLY: i32 = 0;
    pub const O_WRONLY: i32 = 1;
    pub const O_RDWR: i32 = 2;
    pub const O_CREAT: i32 = 64;
    pub const O_TRUNC: i32 = 512;
    pub const O_APPEND: i32 = 1024;

    // Windows implementation — placeholder for Phase 7 Windows port
    pub fn open_file(_path: *const u8, _flags: i32, _mode: i32) -> i32 { -1 }
    pub fn read_fd(_fd: i32, _buf: *mut u8, _count: i64) -> i64 { -1 }
    pub fn write_fd(_fd: i64, _buf: *const u8, _count: i64) -> i64 { -1 }
    pub fn close_fd(_fd: i32) -> i32 { -1 }
}

fn mode_from_string(mode: *const u8) -> i32 {
    if mode.is_null() { return platform::O_RDONLY; }
    let c = unsafe { *mode };
    match c {
        b'r' => {
            let c2 = unsafe { *mode.add(1) };
            if c2 == b'+' { platform::O_RDWR } else { platform::O_RDONLY }
        }
        b'w' => {
            let c2 = unsafe { *mode.add(1) };
            if c2 == b'+' { platform::O_RDWR | platform::O_CREAT | platform::O_TRUNC } else { platform::O_WRONLY | platform::O_CREAT | platform::O_TRUNC }
        }
        b'a' => {
            let c2 = unsafe { *mode.add(1) };
            if c2 == b'+' { platform::O_RDWR | platform::O_CREAT | platform::O_APPEND } else { platform::O_WRONLY | platform::O_CREAT | platform::O_APPEND }
        }
        _ => platform::O_RDONLY,
    }
}

/// Open a file. Returns fd (>=0) or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_open(path: *const u8, mode: *const u8) -> i32 {
    if path.is_null() { return -1; }
    let flags = mode_from_string(mode);
    platform::open_file(path, flags, 0o644)
}

/// Read from a file descriptor into a heap-allocated null-terminated string.
/// Returns pointer to the string. Caller must free with ky_free.
/// On error, returns null pointer.
#[unsafe(no_mangle)]
pub extern "C" fn ky_read_str(fd: i32, count: i32) -> *mut u8 {
    if count <= 0 { return std::ptr::null_mut(); }
    let ptr = crate::ky_alloc(count as i64);
    if ptr.is_null() { return std::ptr::null_mut(); }
    let n = platform::read_fd(fd, ptr, count as i64);
    if n < 0 {
        crate::ky_free(ptr);
        return std::ptr::null_mut();
    }
    unsafe { *ptr.add(n as usize) = 0; }
    ptr
}

/// Write a null-terminated string to a file descriptor.
/// Uses C strlen to determine length. Returns bytes written or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_write_str(fd: i32, buf: *const u8) -> i32 {
    if buf.is_null() { return -1; }
    let mut len: i64 = 0;
    while unsafe { *buf.add(len as usize) } != 0 { len += 1; }
    let result = platform::write_fd(fd as i64, buf, len);
    result as i32
}

/// Close a file descriptor. Returns 0 on success, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_close(fd: i32) -> i32 {
    platform::close_fd(fd)
}

/// Sleep for a given number of milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn ky_sleep(ms: i32) {
    if ms <= 0 { return; }
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

/// Get current time in milliseconds since epoch.
#[unsafe(no_mangle)]
pub extern "C" fn ky_now() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(_) => -1,
    }
}
