use crate::memory::{ky_alloc, ky_free};

#[repr(C)]
pub struct KlList {
    data: *mut i64,
    len: i64,
    cap: i64,
}

const INITIAL_CAP: i64 = 4;

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_new() -> *mut KlList {
    let list = ky_alloc(std::mem::size_of::<KlList>() as i64) as *mut KlList;
    if list.is_null() {
        return std::ptr::null_mut();
    }
    let data = ky_alloc(INITIAL_CAP * std::mem::size_of::<i64>() as i64) as *mut i64;
    if data.is_null() {
        ky_free(list as *mut u8);
        return std::ptr::null_mut();
    }
    unsafe {
        (*list).data = data;
        (*list).len = 0;
        (*list).cap = INITIAL_CAP;
    }
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_free(list: *mut KlList) {
    if list.is_null() {
        return;
    }
    unsafe {
        if !(*list).data.is_null() {
            ky_free((*list).data as *mut u8);
        }
        ky_free(list as *mut u8);
    }
}

fn grow(list: *mut KlList) {
    unsafe {
        let new_cap = (*list).cap * 2;
        let new_data = ky_alloc(new_cap * std::mem::size_of::<i64>() as i64) as *mut i64;
        if new_data.is_null() {
            return;
        }
        for i in 0..(*list).len {
            std::ptr::write(new_data.add(i as usize), std::ptr::read((*list).data.add(i as usize)));
        }
        ky_free((*list).data as *mut u8);
        (*list).data = new_data;
        (*list).cap = new_cap;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_reserve(list: *mut KlList, capacity: i64) {
    if list.is_null() || capacity <= 0 {
        return;
    }
    unsafe {
        if capacity > (*list).cap {
            let new_cap = capacity;
            let new_data = ky_alloc(new_cap * std::mem::size_of::<i64>() as i64) as *mut i64;
            if new_data.is_null() {
                return;
            }
            std::ptr::copy_nonoverlapping((*list).data, new_data, (*list).len as usize);
            ky_free((*list).data as *mut u8);
            (*list).data = new_data;
            (*list).cap = new_cap;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_push(list: *mut KlList, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        if (*list).len >= (*list).cap {
            grow(list);
        }
        std::ptr::write((*list).data.add((*list).len as usize), val);
        (*list).len += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_get(list: *const KlList, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        if index < 0 || index >= (*list).len {
            return 0;
        }
        std::ptr::read((*list).data.add(index as usize))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_set(list: *mut KlList, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        if index < 0 || index >= (*list).len {
            return;
        }
        std::ptr::write((*list).data.add(index as usize), val);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_pop(list: *mut KlList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        if (*list).len <= 0 {
            return 0;
        }
        (*list).len -= 1;
        std::ptr::read((*list).data.add((*list).len as usize))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_len(list: *const KlList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe { (*list).len }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_range(count: i64) -> *mut KlList {
    let list = ky_list_new();
    if list.is_null() {
        return std::ptr::null_mut();
    }
    for i in 0..count {
        ky_list_push(list, i);
    }
    list
}

/// Create range [start, start+1, ..., end-1]
#[unsafe(no_mangle)]
pub extern "C" fn ky_range_two(start: i64, end: i64) -> *mut KlList {
    let list = ky_list_new();
    if list.is_null() {
        return std::ptr::null_mut();
    }
    for i in start..end {
        ky_list_push(list, i);
    }
    list
}

/// Convert C's argc/argv to a Kyle list<str>.
/// Skips argv[0] (program name).
#[unsafe(no_mangle)]
pub extern "C" fn ky_init_args(argc: i32, argv: *mut *mut u8) -> *mut KlList {
    let list = ky_list_new();
    if list.is_null() {
        return std::ptr::null_mut();
    }
    for i in 1..argc {
        let cstr = unsafe { *argv.add(i as usize) };
        if cstr.is_null() {
            continue;
        }
        let len = unsafe {
            let mut n: i32 = 0;
            while *cstr.add(n as usize) != 0 { n += 1; }
            n
        };
        // Allocate Kyle string (ptr + len layout: ptr to bytes, but we store null-terminated)
        let kstr = ky_alloc(len as i64 + 1) as *mut u8;
        if kstr.is_null() { continue; }
        for j in 0..len {
            unsafe { *kstr.add(j as usize) = *cstr.add(j as usize); }
        }
        unsafe { *kstr.add(len as usize) = 0; }
        ky_list_push(list, kstr as i64);
    }
    list
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_slice(list: *mut KlList, start: i64, end: i64) -> *mut KlList {
    if list.is_null() { return std::ptr::null_mut(); }
    let result = ky_list_new();
    if result.is_null() { return std::ptr::null_mut(); }
    unsafe {
        let len = (*list).len;
        let s = if start < 0 { 0 } else if start > len { len } else { start };
        let e = if end < 0 { len } else if end > len { len } else { end };
        let s = s.min(e);
        for i in s..e {
            let val = std::ptr::read((*list).data.add(i as usize));
            ky_list_push(result, val);
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_clone_list(list: *const KlList) -> *mut KlList {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    let result = ky_list_new();
    if result.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let len = (*list).len;
        for i in 0..len {
            let val = std::ptr::read((*list).data.add(i as usize));
            ky_list_push(result, val);
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_extend(dest: *mut KlList, src: *mut KlList) {
    if dest.is_null() || src.is_null() { return; }
    unsafe {
        let src_len = (*src).len;
        for i in 0..src_len {
            let val = std::ptr::read((*src).data.add(i as usize));
            ky_list_push(dest, val);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_pop_first(list: *mut KlList) -> i64 {
    if list.is_null() { return 0; }
    unsafe {
        if (*list).len <= 0 { return 0; }
        let val = std::ptr::read((*list).data);
        let len = (*list).len as usize;
        // Shift all elements left by one
        for i in 1..len {
            let src = (*list).data.add(i);
            let dst = (*list).data.add(i - 1);
            std::ptr::write(dst, std::ptr::read(src));
        }
        (*list).len -= 1;
        val
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_clear(list: *mut KlList) {
    if list.is_null() { return; }
    unsafe {
        (*list).len = 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_contains(list: *const KlList, val: i64) -> i32 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        for i in 0..len {
            let v = std::ptr::read((*list).data.add(i as usize));
            if v == val { return 1; }
        }
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_insert(list: *mut KlList, index: i64, val: i64) {
    if list.is_null() { return; }
    unsafe {
        let len = (*list).len;
        let idx = if index < 0 { 0 } else if index > len { len } else { index };
        // Ensure capacity
        if len >= (*list).cap {
            let new_cap = (*list).cap * 2;
            let new_data = ky_alloc((new_cap * std::mem::size_of::<i64>() as i64) as i64) as *mut i64;
            if new_data.is_null() { return; }
            std::ptr::copy_nonoverlapping((*list).data, new_data, len as usize);
            ky_free((*list).data as *mut u8);
            (*list).data = new_data;
            (*list).cap = new_cap;
        }
        // Shift elements right to make room
        for i in (idx as usize..len as usize).rev() {
            let src = (*list).data.add(i);
            let dst = (*list).data.add(i + 1);
            std::ptr::write(dst, std::ptr::read(src));
        }
        // Insert new value
        std::ptr::write((*list).data.add(idx as usize), val);
        (*list).len += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_remove_at(list: *mut KlList, index: i64) -> i64 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        if len <= 0 || index < 0 || index >= len { return 0; }
        let idx = index as usize;
        let val = std::ptr::read((*list).data.add(idx));
        // Shift elements left
        for i in (idx + 1)..len as usize {
            let src = (*list).data.add(i);
            let dst = (*list).data.add(i - 1);
            std::ptr::write(dst, std::ptr::read(src));
        }
        (*list).len -= 1;
        val
    }
}

/// Remove first occurrence of `val` from the list (by value, not index).
/// Returns 1 if found and removed, 0 if not found.
#[unsafe(no_mangle)]
pub extern "C" fn ky_list_remove_value(list: *mut KlList, val: i64) -> i32 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        for i in 0..len {
            let v = std::ptr::read((*list).data.add(i as usize));
            if v == val {
                // Found it — shift remaining elements left
                for j in (i as usize + 1)..len as usize {
                    let src = (*list).data.add(j);
                    let dst = (*list).data.add(j - 1);
                    std::ptr::write(dst, std::ptr::read(src));
                }
                (*list).len -= 1;
                return 1;
            }
        }
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_sum(list: *const KlList) -> i64 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        let mut total: i64 = 0;
        for i in 0..len {
            total += std::ptr::read((*list).data.add(i as usize));
        }
        total
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_product(list: *const KlList) -> i64 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        let mut total: i64 = 1;
        for i in 0..len {
            total *= std::ptr::read((*list).data.add(i as usize));
        }
        total
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_max(list: *const KlList) -> i64 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        if len <= 0 { return 0; }
        let mut max = std::ptr::read((*list).data);
        for i in 1..len {
            let v = std::ptr::read((*list).data.add(i as usize));
            if v > max { max = v; }
        }
        max
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_min(list: *const KlList) -> i64 {
    if list.is_null() { return 0; }
    unsafe {
        let len = (*list).len;
        if len <= 0 { return 0; }
        let mut min = std::ptr::read((*list).data);
        for i in 1..len {
            let v = std::ptr::read((*list).data.add(i as usize));
            if v < min { min = v; }
        }
        min
    }
}

/// Type of a function pointer used by map/filter/fold.
type FnI64 = unsafe extern "C" fn(i64) -> i64;
type FnI64I64 = unsafe extern "C" fn(i64, i64) -> i64;

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_map(list: *const KlList, fn_ptr: Option<FnI64>) -> *mut KlList {
    if list.is_null() || fn_ptr.is_none() { return std::ptr::null_mut(); }
    let f = fn_ptr.unwrap();
    let result = ky_list_new();
    if result.is_null() { return result; }
    unsafe {
        let len = (*list).len;
        for i in 0..len {
            let val = std::ptr::read((*list).data.add(i as usize));
            let mapped = f(val);
            ky_list_push(result, mapped);
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_filter(list: *const KlList, fn_ptr: Option<FnI64>) -> *mut KlList {
    if list.is_null() || fn_ptr.is_none() { return std::ptr::null_mut(); }
    let f = fn_ptr.unwrap();
    let result = ky_list_new();
    if result.is_null() { return result; }
    unsafe {
        let len = (*list).len;
        for i in 0..len {
            let val = std::ptr::read((*list).data.add(i as usize));
            let keep = f(val);
            if keep != 0 {
                ky_list_push(result, val);
            }
        }
    }
    result
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_fold(list: *const KlList, init: i64, fn_ptr: Option<FnI64I64>) -> i64 {
    if list.is_null() || fn_ptr.is_none() { return init; }
    let f = fn_ptr.unwrap();
    let mut acc = init;
    unsafe {
        let len = (*list).len;
        for i in 0..len {
            let val = std::ptr::read((*list).data.add(i as usize));
            acc = f(acc, val);
        }
    }
    acc
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_reduce(list: *const KlList, fn_ptr: Option<FnI64I64>) -> i64 {
    if list.is_null() || fn_ptr.is_none() { return 0; }
    let f = fn_ptr.unwrap();
    unsafe {
        let len = (*list).len;
        if len <= 0 { return 0; }
        let mut acc = std::ptr::read((*list).data);
        for i in 1..len {
            let val = std::ptr::read((*list).data.add(i as usize));
            acc = f(acc, val);
        }
        acc
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_reverse(list: *mut KlList) {
    if list.is_null() { return; }
    unsafe {
        let len = (*list).len;
        if len <= 1 { return; }
        let mut i = 0;
        let mut j = len - 1;
        while i < j {
            let tmp = std::ptr::read((*list).data.add(i as usize));
            std::ptr::write((*list).data.add(i as usize), std::ptr::read((*list).data.add(j as usize)));
            std::ptr::write((*list).data.add(j as usize), tmp);
            i += 1;
            j -= 1;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_index_of(list: *const KlList, val: i64) -> i64 {
    if list.is_null() { return -1; }
    unsafe {
        let len = (*list).len;
        let data = (*list).data;
        for i in 0..len {
            if *data.add(i as usize) == val {
                return i;
            }
        }
    }
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_sort(list: *mut KlList) {
    if list.is_null() { return; }
    unsafe {
        let len = (*list).len as usize;
        let data = (*list).data;
        if len <= 1 { return; }
        for i in 1..len {
            let key = *data.add(i);
            let mut j = i;
            while j > 0 && *data.add(j - 1) > key {
                *data.add(j) = *data.add(j - 1);
                j -= 1;
            }
            *data.add(j) = key;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_list_chunk(list: *const KlList, chunk_size: i64) -> *mut KlList {
    if list.is_null() || chunk_size <= 0 { return ky_list_new(); }
    let result = ky_list_new();
    unsafe {
        let len = (*list).len;
        let data = (*list).data;
        let mut i = 0;
        while i < len {
            let chunk = ky_list_new();
            let chunk_end = if i + chunk_size < len { i + chunk_size } else { len };
            for j in i..chunk_end {
                let val = *data.add(j as usize);
                ky_list_push(chunk, val);
            }
            ky_list_push(result, chunk as i64);
            i = chunk_end;
        }
    }
    result
}

// ============================================================
// Lazy Iterator API (Phase 10)
// ============================================================

/// Opaque iterator state stored as a heap-allocated i64 array:
///   [0] = source list ptr (i64)
///   [1] = current index
///   [2] = map fn ptr (0 = none)
///   [3] = filter fn ptr (0 = none)
#[repr(C)]
pub struct KlIter {
    pub source: i64,
    pub index: i64,
    pub map_fn: i64,
    pub filter_fn: i64,
}

/// Create a lazy iterator from a list.
/// Returns an opaque pointer to a KlIter (stored as i64 in Kyle).
#[unsafe(no_mangle)]
pub extern "C" fn ky_iter_new(list_ptr: i64) -> i64 {
    let boxed = Box::new(KlIter {
        source: list_ptr,
        index: 0,
        map_fn: 0,
        filter_fn: 0,
    });
    Box::into_raw(boxed) as i64
}

/// Get the next value from an iterator.
/// Returns the value, or i64::MIN when exhausted.
#[unsafe(no_mangle)]
pub extern "C" fn ky_iter_next(iter_ptr: i64) -> i64 {
    if iter_ptr == 0 { return i64::MIN; }
    let iter = unsafe { &mut *(iter_ptr as *mut KlIter) };
    let list_ptr = iter.source as *const KlList;
    if list_ptr.is_null() { return i64::MIN; }
    unsafe {
        let list = &*list_ptr;
        loop {
            if iter.index >= list.len as i64 {
                return i64::MIN;
            }
            let raw_val = std::ptr::read(list.data.add(iter.index as usize));
            iter.index += 1;
            let mut val = raw_val;

            // Apply map FIRST (transform the raw value)
            if iter.map_fn != 0 {
                let map: FnI64 = std::mem::transmute(iter.map_fn);
                val = map(val);
            }

            // Apply filter AFTER map (filter on transformed value)
            if iter.filter_fn != 0 {
                let filter: FnI64 = std::mem::transmute(iter.filter_fn);
                if filter(val) == 0 {
                    continue; // skip, index already advanced past this element
                }
            }

            return val;
        }
    }
}

/// Create a mapped iterator (lazy, no allocation).
/// Returns a new KlIter that wraps the source with a map function.
#[unsafe(no_mangle)]
pub extern "C" fn ky_iter_map(source_ptr: i64, fn_ptr: i64) -> i64 {
    if source_ptr == 0 { return 0; }
    let source = unsafe { &*(source_ptr as *const KlIter) };
    let boxed = Box::new(KlIter {
        source: source.source,
        index: source.index,
        map_fn: fn_ptr,
        filter_fn: source.filter_fn,
    });
    Box::into_raw(boxed) as i64
}

/// Create a filtered iterator (lazy, no allocation).
/// Returns a new KlIter that wraps the source with a filter function.
#[unsafe(no_mangle)]
pub extern "C" fn ky_iter_filter(source_ptr: i64, fn_ptr: i64) -> i64 {
    if source_ptr == 0 { return 0; }
    let source = unsafe { &*(source_ptr as *const KlIter) };
    let boxed = Box::new(KlIter {
        source: source.source,
        index: source.index,
        map_fn: source.map_fn,
        filter_fn: fn_ptr,
    });
    Box::into_raw(boxed) as i64
}

/// Collect an iterator into a new list.
#[unsafe(no_mangle)]
pub extern "C" fn ky_iter_collect(iter_ptr: i64) -> *mut KlList {
    if iter_ptr == 0 { return std::ptr::null_mut(); }
    let result = ky_list_new();
    if result.is_null() { return result; }
    loop {
        let val = ky_iter_next(iter_ptr);
        if val == i64::MIN { break; }
        ky_list_push(result, val);
    }
    result
}

/// Free an iterator.
#[unsafe(no_mangle)]
pub extern "C" fn ky_iter_free(iter_ptr: i64) {
    if iter_ptr == 0 { return; }
    unsafe {
        drop(Box::from_raw(iter_ptr as *mut KlIter));
    }
}
