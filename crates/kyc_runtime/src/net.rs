// ─── Platform-specific socket operations ────────────────────

#[cfg(unix)]
mod platform {
    use std::ffi::c_void;

    pub fn tcp_listen(port: i32) -> i32 {
        unsafe {
            let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
            if fd < 0 {
                return -1;
            }

            let optval: i32 = 1;
            libc::setsockopt(
                fd, libc::SOL_SOCKET, libc::SO_REUSEADDR,
                &optval as *const i32 as *const c_void,
                std::mem::size_of::<i32>() as u32,
            );

            let mut addr: libc::sockaddr_in = std::mem::zeroed();
            addr.sin_family = libc::AF_INET as libc::sa_family_t;
            addr.sin_port = (port as u16).to_be();
            addr.sin_addr.s_addr = 0;

            let bind_result = libc::bind(
                fd,
                &addr as *const libc::sockaddr_in as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_in>() as u32,
            );
            if bind_result < 0 {
                libc::close(fd);
                return -1;
            }

            let listen_result = libc::listen(fd, 128);
            if listen_result < 0 {
                libc::close(fd);
                return -1;
            }
            fd
        }
    }

    pub fn tcp_accept(fd: i32) -> i32 {
        unsafe {
            let mut addr: libc::sockaddr_in = std::mem::zeroed();
            let mut addrlen: libc::socklen_t = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
            libc::accept(
                fd,
                &mut addr as *mut libc::sockaddr_in as *mut libc::sockaddr,
                &mut addrlen,
            )
        }
    }

    pub fn tcp_read(fd: i32, buf: *mut u8, count: usize) -> isize {
        unsafe { libc::read(fd, buf as *mut c_void, count) }
    }

    pub fn tcp_write(fd: i32, buf: *const u8, len: usize) -> isize {
        unsafe { libc::write(fd, buf as *const c_void, len) }
    }

    pub fn tcp_close(fd: i32) -> i32 {
        unsafe { libc::close(fd) }
    }
}

#[cfg(windows)]
mod platform {
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Mutex;

    struct HandleEntry {
        listener: Option<TcpListener>,
        stream: Option<Mutex<TcpStream>>,
    }

    static HANDLES: std::sync::OnceLock<Mutex<HashMap<i32, HandleEntry>>> = std::sync::OnceLock::new();
    static NEXT_HANDLE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);

    fn handles() -> &'static Mutex<HashMap<i32, HandleEntry>> {
        HANDLES.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn tcp_listen(port: i32) -> i32 {
        let addr = format!("0.0.0.0:{}", port);
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                let handle = NEXT_HANDLE
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let mut map = handles().lock().unwrap();
                map.insert(handle, HandleEntry { listener: Some(listener), stream: None });
                handle
            }
            Err(_) => -1,
        }
    }

    pub fn tcp_accept(fd: i32) -> i32 {
        // Remove listener from table to avoid blocking while holding the lock
        let listener = {
            let mut map = handles().lock().unwrap();
            map.remove(&fd).and_then(|entry| entry.listener)
        };

        match listener {
            Some(listener) => match listener.accept() {
                Ok((stream, _)) => {
                    let new_handle = NEXT_HANDLE
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let mut map = handles().lock().unwrap();
                    map.insert(fd, HandleEntry { listener: Some(listener), stream: None });
                    map.insert(new_handle, HandleEntry { listener: None, stream: Some(Mutex::new(stream)) });
                    new_handle
                }
                Err(_) => {
                    let mut map = handles().lock().unwrap();
                    map.insert(fd, HandleEntry { listener: Some(listener), stream: None });
                    -1
                }
            },
            None => -1,
        }
    }

    pub fn tcp_read(fd: i32, buf: *mut u8, count: usize) -> isize {
        let map = handles().lock().unwrap();
        match map.get(&fd) {
            Some(entry) => match &entry.stream {
                Some(stream_mutex) => {
                    let mut stream = stream_mutex.lock().unwrap();
                    let slice = unsafe { std::slice::from_raw_parts_mut(buf, count) };
                    stream.read(slice).unwrap_or(0) as isize
                }
                None => -1,
            },
            None => -1,
        }
    }

    pub fn tcp_write(fd: i32, buf: *const u8, len: usize) -> isize {
        let map = handles().lock().unwrap();
        match map.get(&fd) {
            Some(entry) => match &entry.stream {
                Some(stream_mutex) => {
                    let mut stream = stream_mutex.lock().unwrap();
                    let slice = unsafe { std::slice::from_raw_parts(buf, len) };
                    stream.write(slice).unwrap_or(0) as isize
                }
                None => -1,
            },
            None => -1,
        }
    }

    pub fn tcp_close(fd: i32) -> i32 {
        let mut map = handles().lock().unwrap();
        map.remove(&fd);
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_listen(port: i32) -> i32 {
    platform::tcp_listen(port)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_accept(fd: i32) -> i32 {
    platform::tcp_accept(fd)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_read(fd: i32, count: i32) -> *mut u8 {
    if fd < 0 || count <= 0 {
        return std::ptr::null_mut();
    }
    let buf = crate::ky_alloc(count as i64);
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    let n = platform::tcp_read(fd, buf, count as usize);
    if n <= 0 {
        crate::ky_free(buf);
        return std::ptr::null_mut();
    }
    unsafe { *buf.add(n as usize) = 0; }
    buf
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_write(fd: i32, buf: *const u8, len: i32) -> i32 {
    if fd < 0 || buf.is_null() || len <= 0 {
        return -1;
    }
    platform::tcp_write(fd, buf, len as usize) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_tcp_close(fd: i32) -> i32 {
    platform::tcp_close(fd)
}

// ─── Memory access (cross-platform) ─────────────────────────

/// Read an i32 value from a memory address.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_read_i32(ptr: *const u8) -> i32 {
    if ptr.is_null() { return 0; }
    unsafe { *(ptr as *const i32) }
}

/// Read a pointer value from a memory address.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_read_ptr(ptr: *const u8) -> *mut u8 {
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { *(ptr as *const *mut u8) }
}

/// Write i32 to memory.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_write_i32(ptr: *mut u8, val: i32) {
    if ptr.is_null() { return; }
    unsafe { *(ptr as *mut i32) = val; }
}

/// Write pointer to memory.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ptr_write_ptr(ptr: *mut u8, val: *mut u8) {
    if ptr.is_null() { return; }
    unsafe { *(ptr as *mut *mut u8) = val; }
}

// ─── Crypto & encoding (cross-platform) ─────────────────────

/// SHA-1 hash of a byte buffer.
/// `data` must be null-terminated. `out` must be 20 bytes.
/// Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn ky_sha1(data: *const u8) -> *mut u8 {
    if data.is_null() {
        return std::ptr::null_mut();
    }
    let len = crate::ky_strlen(data);
    let data_slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    let hash = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, data_slice);
    let hash_bytes = hash.as_ref();
    let out_len = hash_bytes.len();
    let buf = crate::ky_alloc((out_len + 1) as i64);
    if buf.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(hash_bytes.as_ptr(), buf, out_len);
        *buf.add(out_len) = 0;
    }
    buf
}

/// Base64 encode bytes. Returns heap-allocated string.
#[unsafe(no_mangle)]
pub extern "C" fn ky_base64_encode(data: *const u8) -> *mut u8 {
    if data.is_null() {
        return std::ptr::null_mut();
    }
    let data_len = crate::ky_strlen(data);
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len as usize) };
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data_slice);
    let bytes = encoded.as_bytes();
    let len = bytes.len();
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *buf.add(len) = 0;
    }
    buf
}

/// Perform WebSocket server-side upgrade handshake.
/// `key` is the Sec-WebSocket-Key header value (null-terminated).
/// Returns accept value as heap-allocated string, or null on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ws_accept(key: *const u8) -> *mut u8 {
    if key.is_null() { return std::ptr::null_mut(); }
    let key_str = unsafe { std::ffi::CStr::from_ptr(key .cast()) }
        .to_str().unwrap_or("");
    let concat = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key_str);
    let hash = ring::digest::digest(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY, concat.as_bytes());
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(hash.as_ref());
    let bytes = encoded.as_bytes();
    let len = bytes.len();
    let buf = crate::ky_alloc((len + 1) as i64);
    if buf.is_null() { return std::ptr::null_mut(); }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *buf.add(len) = 0;
    }
    buf
}

/// Read a WebSocket frame from a socket.
/// Returns a heap-allocated WsFrame struct (8 bytes: opcode + payload_len + payload_ptr),
/// or null on error.
/// Caller must ky_free the returned struct.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ws_read_frame(fd: i32) -> *mut u8 {
    fn read_all(fd: i32, buf: &mut [u8]) -> bool {
        platform::tcp_read(fd, buf.as_mut_ptr(), buf.len()) == buf.len() as isize
    }

    // Read first 2 bytes
    let mut header = [0u8; 2];
    if !read_all(fd, &mut header) { return std::ptr::null_mut(); }

    let fin = (header[0] & 0x80) != 0;
    let opcode = header[0] & 0x0F;
    let masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as u64;

    // Extended payload length
    if payload_len == 126 {
        let mut ext = [0u8; 2];
        if !read_all(fd, &mut ext) { return std::ptr::null_mut(); }
        payload_len = u16::from_be_bytes(ext) as u64;
    } else if payload_len == 127 {
        let mut ext = [0u8; 8];
        if !read_all(fd, &mut ext) { return std::ptr::null_mut(); }
        payload_len = u64::from_be_bytes(ext);
    }

    // Read mask key if present
    let mask_key: u32;
    if masked {
        let mut mask_bytes = [0u8; 4];
        if !read_all(fd, &mut mask_bytes) { return std::ptr::null_mut(); }
        mask_key = u32::from_be_bytes(mask_bytes);
    } else {
        mask_key = 0;
    }

    // Read payload
    let payload_size = payload_len as usize;
    let payload_buf = crate::ky_alloc((payload_size + 1) as i64);
    if payload_buf.is_null() { return std::ptr::null_mut(); }
    if payload_size > 0 {
        let n = platform::tcp_read(fd, payload_buf, payload_size);
        if n as usize != payload_size {
            crate::ky_free(payload_buf);
            return std::ptr::null_mut();
        }
        // Unmask if needed
        if masked {
            for i in 0..payload_size {
                let key_byte = ((mask_key >> (24 - (i % 4) * 8)) & 0xFF) as u8;
                unsafe { *payload_buf.add(i) ^= key_byte; }
            }
        }
    }
    unsafe { *payload_buf.add(payload_size) = 0; }

    // Allocate WsFrame struct: [opcode: i32, fin: i32, payload_len: i32, payload: ptr]
    let frame = crate::ky_alloc(32) as *mut i32;
    if frame.is_null() { crate::ky_free(payload_buf); return std::ptr::null_mut(); }
    unsafe {
        *frame = opcode as i32;
        *frame.add(1) = if fin { 1 } else { 0 };
        *frame.add(2) = payload_size as i32;
        *(frame.add(3) as *mut *mut u8) = payload_buf;
    }
    frame as *mut u8
}

/// Send a WebSocket frame.
/// Returns bytes written or -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_ws_send_frame(fd: i32, opcode: i32, payload: *const u8, payload_len: i32) -> i32 {
    if fd < 0 { return -1; }
    let plen = payload_len as usize;
    let mut header_buf: Vec<u8> = Vec::new();
    header_buf.push(0x80 | (opcode as u8));
    if plen < 126 {
        header_buf.push(plen as u8);
    } else if plen < 65536 {
        header_buf.push(126);
        header_buf.extend_from_slice(&(plen as u16).to_be_bytes());
    } else {
        header_buf.push(127);
        header_buf.extend_from_slice(&(plen as u64).to_be_bytes());
    }

    // Write header + payload
    let n = platform::tcp_write(fd, header_buf.as_ptr(), header_buf.len());
    if n < 0 { return -1; }
    let mut written = n as i32;
    if plen > 0 && !payload.is_null() {
        let n2 = platform::tcp_write(fd, payload, plen);
        if n2 < 0 { return -1; }
        written += n2 as i32;
    }
    written
}
