use std::fs;
use std::path::Path;

/// Check if a path exists.
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if path is a directory.
pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

/// Check if path is a regular file.
pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

/// Get file size in bytes. Returns -1 on error.
pub fn size(path: &str) -> i64 {
    fs::metadata(path).map(|m| m.len() as i64).unwrap_or(-1)
}

/// Copy file from src to dst.
pub fn copy(src: &str, dst: &str) -> Result<(), String> {
    fs::copy(src, dst).map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove a file.
pub fn remove(path: &str) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| e.to_string())
}

/// Create a directory.
pub fn create_dir(path: &str) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| e.to_string())
}

/// Remove a directory.
pub fn remove_dir(path: &str) -> Result<(), String> {
    fs::remove_dir(path).map_err(|e| e.to_string())
}

/// Rename/move a file.
pub fn rename(src: &str, dst: &str) -> Result<(), String> {
    fs::rename(src, dst).map_err(|e| e.to_string())
}

/// Read entire file into a String.
pub fn read_to_string(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Write a string to a file (overwrite).
pub fn write_string(path: &str, data: &str) -> Result<(), String> {
    fs::write(path, data).map_err(|e| e.to_string())
}

/// List directory entries. Returns entry names (without full path).
pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let rd = fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for entry in rd {
        let entry = entry.map_err(|e| e.to_string())?;
        if let Some(name) = entry.file_name().to_str() {
            entries.push(name.to_string());
        }
    }
    Ok(entries)
}

// --- Low-level fd operations (Unix only) ---

#[cfg(unix)]
mod unix_fd {
    use libc::size_t;

    pub fn open(path: &str) -> Result<i32, String> {
        let cpath = std::ffi::CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY) };
        if fd < 0 {
            Err(format!("cannot open '{}'", path))
        } else {
            Ok(fd)
        }
    }

    pub fn create(path: &str) -> Result<i32, String> {
        let cpath = std::ffi::CString::new(path).map_err(|e| format!("invalid path: {}", e))?;
        let mode = (libc::S_IRUSR | libc::S_IWUSR | libc::S_IRGRP | libc::S_IROTH) as libc::c_uint;
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, mode) };
        if fd < 0 {
            Err(format!("cannot create '{}'", path))
        } else {
            Ok(fd)
        }
    }

    pub fn read(fd: i32, buf: &mut [u8]) -> Result<usize, String> {
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len() as size_t) };
        if n < 0 {
            Err("read error".to_string())
        } else {
            Ok(n as usize)
        }
    }

    pub fn write(fd: i32, data: &[u8]) -> Result<usize, String> {
        let n = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len() as size_t) };
        if n < 0 {
            Err("write error".to_string())
        } else {
            Ok(n as usize)
        }
    }

    pub fn close(fd: i32) {
        unsafe { libc::close(fd); }
    }
}

#[cfg(not(unix))]
mod unix_fd {
    // Stub: fd operations not supported on this platform
    pub fn open(_path: &str) -> Result<i32, String> {
        Err("file descriptor operations not supported on this platform".to_string())
    }

    pub fn create(_path: &str) -> Result<i32, String> {
        Err("file descriptor operations not supported on this platform".to_string())
    }

    pub fn read(_fd: i32, _buf: &mut [u8]) -> Result<usize, String> {
        Err("file descriptor operations not supported on this platform".to_string())
    }

    pub fn write(_fd: i32, _data: &[u8]) -> Result<usize, String> {
        Err("file descriptor operations not supported on this platform".to_string())
    }

    pub fn close(_fd: i32) {}
}

pub use unix_fd::{open, create, read, write, close};
