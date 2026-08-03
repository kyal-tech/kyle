// CSV parsing runtime.
//
// Exposes ky_csv_* functions for the `std.csv` module. Parsing and
// serialization happen in Rust; Kyle code works with an opaque handle.
// Row 0 is the header row; data rows are 1..N. `ky_csv_get_col` resolves
// column names against the header for the documented `row.get(name)` use case.

use std::ffi::CStr;

#[derive(Default)]
pub struct CsvData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

fn cstr(s: *const u8) -> String {
    if s.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(s.cast()) }
        .to_str()
        .unwrap_or("")
        .to_string()
}

fn alloc_str(s: &str) -> *mut u8 {
    if s.is_empty() {
        return std::ptr::null_mut();
    }
    let out = crate::ky_alloc(s.len() as i64 + 1);
    if out.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(s.as_ptr(), out, s.len());
        *out.add(s.len()) = 0;
    }
    out
}

/// Parse CSV data into a handle. delim defaults to ",". Returns null on empty input.
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_parse(data: *const u8, delim: *const u8) -> *mut std::ffi::c_void {
    let s = cstr(data);
    let d = if delim.is_null() { String::from(",") } else { cstr(delim) };
    if d.is_empty() {
        return std::ptr::null_mut();
    }
    let mut csv = CsvData::default();
    let mut header_set = false;
    for line in s.split('\n') {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split(&d).map(|f| f.to_string()).collect();
        if !header_set {
            csv.header = fields;
            header_set = true;
        } else {
            csv.rows.push(fields);
        }
    }
    if !header_set {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(csv)) as *mut std::ffi::c_void
}

/// Release a CSV handle.
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_free(h: *mut std::ffi::c_void) {
    if h.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(h.cast::<CsvData>())) };
}

/// Number of data rows (excluding the header).
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_row_count(h: *mut std::ffi::c_void) -> i32 {
    if h.is_null() {
        return 0;
    }
    let csv = unsafe { &*(h.cast::<CsvData>()) };
    csv.rows.len() as i32
}

/// Number of columns (header length).
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_col_count(h: *mut std::ffi::c_void) -> i32 {
    if h.is_null() {
        return 0;
    }
    let csv = unsafe { &*(h.cast::<CsvData>()) };
    csv.header.len() as i32
}

/// Cell value at (row, col). Row 0 is the header. Returns null-terminated heap
/// string, or null when out of bounds.
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_get(h: *mut std::ffi::c_void, r: i32, c: i32) -> *mut u8 {
    if h.is_null() || r < 0 || c < 0 {
        return std::ptr::null_mut();
    }
    let csv = unsafe { &*(h.cast::<CsvData>()) };
    let val = if r == 0 {
        csv.header.get(c as usize)
    } else {
        csv.rows.get(r as usize - 1).and_then(|row| row.get(c as usize))
    };
    val.map(|v| alloc_str(v)).unwrap_or(std::ptr::null_mut())
}

/// Value at column `col` for data row `r` (0-based). Resolves the column index
/// against the header. Returns null when the column or row doesn't exist.
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_get_col(h: *mut std::ffi::c_void, col: *const u8, r: i32) -> *mut u8 {
    if h.is_null() || r < 0 || col.is_null() {
        return std::ptr::null_mut();
    }
    let col = cstr(col);
    let csv = unsafe { &*(h.cast::<CsvData>()) };
    let Some(idx) = csv.header.iter().position(|h| *h == col) else {
        return std::ptr::null_mut();
    };
    let val = csv.rows.get(r as usize).and_then(|row| row.get(idx));
    val.map(|v| alloc_str(v)).unwrap_or(std::ptr::null_mut())
}

/// Serialize a CSV handle back to CSV text.
#[unsafe(no_mangle)]
pub extern "C" fn ky_csv_to_str(h: *mut std::ffi::c_void) -> *mut u8 {
    if h.is_null() {
        return std::ptr::null_mut();
    }
    let csv = unsafe { &*(h.cast::<CsvData>()) };
    let mut out = String::new();
    let push_row = |fields: &[String], out: &mut String| {
        for (i, f) in fields.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(f);
        }
        out.push('\n');
    };
    push_row(&csv.header, &mut out);
    for row in &csv.rows {
        push_row(row, &mut out);
    }
    alloc_str(&out)
}
