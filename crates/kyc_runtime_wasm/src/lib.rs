#![no_std]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

extern crate alloc;
use alloc::alloc::{alloc, dealloc, Layout};
use core::panic::PanicInfo;
use core::ptr;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static ALLOCATOR: WasmAllocator = WasmAllocator;

struct WasmAllocator;

unsafe impl alloc::alloc::GlobalAlloc for WasmAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}

/// Allocate memory (replacement for ky_alloc)
#[unsafe(no_mangle)]
pub extern "C" fn ky_alloc(size: i64) -> i64 {
    if size <= 0 { return 0; }
    let layout = Layout::from_size_align(size as usize, 16).unwrap_or(Layout::from_size_align(16, 16).unwrap());
    let ptr = unsafe { alloc(layout) };
    ptr as i64
}

/// Free memory (replacement for ky_free)
#[unsafe(no_mangle)]
pub extern "C" fn ky_free(ptr: i64) {
    if ptr == 0 { return; }
    let layout = Layout::from_size_align(16, 16).unwrap(); // approximate
    unsafe { dealloc(ptr as *mut u8, layout); }
}

/// Print a string to console (delegates to JS via WASM import)
#[unsafe(no_mangle)]
pub extern "C" fn ky_print(_s: i64) {
    // Will be implemented via JS import
}

/// Println — same as print for WASM
#[unsafe(no_mangle)]
pub extern "C" fn ky_println(s: i64) {
    ky_print(s);
}

/// String length
#[unsafe(no_mangle)]
pub extern "C" fn ky_strlen(s: i64) -> i64 {
    if s == 0 { return 0; }
    let ptr = s as *const u8;
    let mut len: isize = 0;
    unsafe {
        while *ptr.offset(len) != 0 {
            len += 1;
        }
    }
    len as i64
}

/// i64 to string
#[unsafe(no_mangle)]
pub extern "C" fn ky_i64_to_str(val: i64) -> i64 {
    // Allocate a small buffer (21 bytes for i64 min + sign)
    let layout = Layout::from_size_align(24, 16).unwrap();
    let buf = unsafe { alloc(layout) };
    if buf.is_null() { return 0; }
    let mut v = if val < 0 { -val } else { val } as u64;
    let mut i = 22usize;
    unsafe {
        *buf.add(23) = 0;
        loop {
            i -= 1;
            *buf.add(i) = b'0' + (v % 10) as u8;
            v /= 10;
            if v == 0 { break; }
        }
        if val < 0 {
            i -= 1;
            *buf.add(i) = b'-';
        }
    }
    (buf as i64) + i as i64
}

/// i32 to string
#[unsafe(no_mangle)]
pub extern "C" fn ky_i32_to_str(val: i32) -> i64 {
    ky_i64_to_str(val as i64)
}

/// String concatenation
#[unsafe(no_mangle)]
pub extern "C" fn ky_concat(a: i64, b: i64) -> i64 {
    if a == 0 && b == 0 { return 0; }
    let a_ptr = a as *const u8;
    let b_ptr = b as *const u8;
    let a_len = if a == 0 { 0 } else { ky_strlen(a) as usize };
    let b_len = if b == 0 { 0 } else { ky_strlen(b) as usize };
    let total = a_len + b_len + 1;
    let layout = Layout::from_size_align(total, 16).unwrap();
    let result = unsafe { alloc(layout) };
    if result.is_null() { return 0; }
    unsafe {
        if a_len > 0 { ptr::copy_nonoverlapping(a_ptr, result, a_len); }
        if b_len > 0 { ptr::copy_nonoverlapping(b_ptr, result.add(a_len), b_len); }
        *result.add(a_len + b_len) = 0;
    }
    result as i64
}

/// List new
#[unsafe(no_mangle)]
pub extern "C" fn ky_list_new() -> i64 {
    // Simple growable array: store capacity, length, then elements
    let layout = Layout::from_size_align(16, 16).unwrap(); // cap(8) + len(8)
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() { return 0; }
    unsafe {
        *(ptr as *mut i64) = 16;   // capacity
        *(ptr.add(8) as *mut i64) = 0; // length
    }
    ptr as i64
}

/// List push
#[unsafe(no_mangle)]
pub extern "C" fn ky_list_push(list: i64, val: i64) {
    if list == 0 { return; }
    let ptr = list as *mut u8;
    unsafe {
        let cap = *(ptr as *mut i64);
        let len = *(ptr.add(8) as *mut i64);
        if len >= cap {
            let new_cap = cap * 2;
            let new_layout = Layout::from_size_align(16 + new_cap as usize * 8, 16).unwrap();
            let old_layout = Layout::from_size_align(16 + cap as usize * 8, 16).unwrap();
            let new_ptr = alloc(new_layout);
            if !new_ptr.is_null() {
                ptr::copy_nonoverlapping(ptr, new_ptr, (16 + cap as usize * 8) as usize);
                dealloc(ptr, old_layout);
                *(new_ptr as *mut i64) = new_cap;
                let data_ptr = new_ptr.add(16) as *mut i64;
                *data_ptr.add(len as usize) = val;
                *(new_ptr.add(8) as *mut i64) = len + 1;
            }
        } else {
            let data_ptr = ptr.add(16) as *mut i64;
            *data_ptr.add(len as usize) = val;
            *(ptr.add(8) as *mut i64) = len + 1;
        }
    }
}

/// List get
#[unsafe(no_mangle)]
pub extern "C" fn ky_list_get(list: i64, idx: i64) -> i64 {
    if list == 0 { return 0; }
    let ptr = list as *mut u8;
    unsafe {
        let len = *(ptr.add(8) as *mut i64);
        if idx < 0 || idx >= len { return 0; }
        let data_ptr = ptr.add(16) as *mut i64;
        *data_ptr.offset(idx as isize)
    }
}

/// List len
#[unsafe(no_mangle)]
pub extern "C" fn ky_list_len(list: i64) -> i64 {
    if list == 0 { return 0; }
    unsafe { *((list as *mut u8).add(8) as *mut i64) }
}

/// List free
#[unsafe(no_mangle)]
pub extern "C" fn ky_list_free(list: i64) {
    if list == 0 { return; }
    let ptr = list as *mut u8;
    unsafe {
        let cap = *(ptr as *mut i64);
        let layout = Layout::from_size_align(16 + cap as usize * 8, 16).unwrap();
        dealloc(ptr, layout);
    }
}
