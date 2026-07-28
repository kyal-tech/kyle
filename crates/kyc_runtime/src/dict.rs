use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Type alias for the internal dict representation.
pub type KlDict = HashMap<String, i64>;

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_new() -> *mut std::ffi::c_void {
    let map = Box::new(HashMap::<String, i64>::new());
    Box::into_raw(map) as *mut std::ffi::c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_set(dict: *mut std::ffi::c_void, key: *const c_char, val: i64) {
    if dict.is_null() || key.is_null() {
        return;
    }
    let map = unsafe { &mut *(dict as *mut HashMap<String, i64>) };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("").to_string();
    map.insert(key_str, val);
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_get(dict: *mut std::ffi::c_void, key: *const c_char) -> i64 {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    map.get(key_str).copied().unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_len(dict: *mut std::ffi::c_void) -> i64 {
    if dict.is_null() {
        return 0;
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    map.len() as i64
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_contains(dict: *mut std::ffi::c_void, key: *const c_char) -> i32 {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    if map.contains_key(key_str) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_remove(dict: *mut std::ffi::c_void, key: *const c_char) -> i64 {
    if dict.is_null() || key.is_null() {
        return -1;
    }
    let map = unsafe { &mut *(dict as *mut HashMap<String, i64>) };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("").to_string();
    if map.remove(&key_str).is_some() { 0 } else { -1 }
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_free(dict: *mut std::ffi::c_void) {
    if dict.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(dict as *mut HashMap<String, i64>)); }
}

/// Create a dict and immediately populate it from a JSON string.
/// Returns a pointer to the dict, or null on parse error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_json_parse(json: *const u8) -> *mut std::ffi::c_void {
    if json.is_null() {
        return std::ptr::null_mut();
    }
    let raw = unsafe { std::ffi::CStr::from_ptr(json as *const std::os::raw::c_char) };
    let s = raw.to_str().unwrap_or("").trim().to_string();
    if s.is_empty() {
        return std::ptr::null_mut();
    }
    // Basic JSON object parser (handles {"key":123,"key2":456})
    let mut map = Box::new(HashMap::<String, i64>::new());
    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len()-1];
        for part in inner.split(',') {
            let part = part.trim();
            if part.is_empty() { continue; }
            if let Some(col_pos) = part.find(':') {
                let key_part = part[..col_pos].trim().trim_matches('"');
                let val_part = part[col_pos+1..].trim();
                if let Ok(val) = val_part.parse::<i64>() {
                    map.insert(key_part.to_string(), val);
                }
            }
        }
    }
    Box::into_raw(map) as *mut std::ffi::c_void
}

/// Serialize a dict to a JSON string.
/// Returns a heap-allocated C string (must be freed by caller with ky_free).
#[unsafe(no_mangle)]
pub extern "C" fn ky_json_stringify(dict: *mut std::ffi::c_void) -> *mut u8 {
    if dict.is_null() {
        return std::ptr::null_mut();
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let mut result = String::from('{');
    for (i, (key, val)) in map.iter().enumerate() {
        if i > 0 { result.push(','); }
        result.push_str(&format!("\"{}\":{}", key, val));
    }
    result.push('}');
    // Allocate on heap for RAII
    let c_str = CString::new(result).unwrap_or_default();
    let ptr = c_str.into_bytes_with_nul().leak().as_mut_ptr();
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn ky_clone_dict(dict: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if dict.is_null() {
        return std::ptr::null_mut();
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let cloned = Box::new(map.clone());
    Box::into_raw(cloned) as *mut std::ffi::c_void
}

/// Serialize a dict to JSON treating all values as string pointers.
/// Each value is a pointer to a null-terminated string that gets dereferenced.
#[unsafe(no_mangle)]
pub extern "C" fn ky_json_stringify_str(dict: *mut std::ffi::c_void) -> *mut u8 {
    if dict.is_null() {
        return std::ptr::null_mut();
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let mut result = String::from('{');
    for (i, (key, val)) in map.iter().enumerate() {
        if i > 0 { result.push(','); }
        // val is a pointer to a null-terminated string
        let v = *val;
        let s = if v != 0 {
            unsafe { std::ffi::CStr::from_ptr(v as *const std::ffi::c_char) }
                .to_str().unwrap_or("")
        } else { "" };
        result.push_str(&format!("\"{}\":\"{}\"", key, s));
    }
    result.push('}');
    let c_str = CString::new(result).unwrap_or_default();
    let ptr = c_str.into_bytes_with_nul().leak().as_mut_ptr();
    ptr
}

/// Serialize a `final class` struct to JSON.
/// `ptr` points to the struct in memory.
/// `descriptor` is a null-terminated C string like "name:str,age:i32,active:bool"
/// Returns heap-allocated JSON string. Caller must ky_free.
#[unsafe(no_mangle)]
pub extern "C" fn ky_struct_to_json(ptr: *const u8, descriptor: *const u8) -> *mut u8 {
    if ptr.is_null() || descriptor.is_null() {
        return std::ptr::null_mut();
    }
    let desc = unsafe { std::ffi::CStr::from_ptr(descriptor .cast()) };
    let desc_str = desc.to_str().unwrap_or("");
    if desc_str.is_empty() {
        return std::ptr::null_mut();
    }

    // Parse descriptor: "field1:type1,field2:type2,..."
    let mut result = String::from('{');
    let mut first = true;
    let mut offset: usize = 0;

    for field in desc_str.split(',') {
        let field = field.trim();
        if field.is_empty() { continue; }
        if let Some(col_pos) = field.find(':') {
            let name = field[..col_pos].trim();
            let ftype = field[col_pos + 1..].trim();

            if !first { result.push(','); }
            first = false;

            // Escape field name
            result.push_str(&format!("\"{}\":", name));

            match ftype {
                "str" => {
                    // Read string pointer (8 bytes at offset)
                    unsafe {
                        let str_ptr = *(ptr.add(offset) as *const *const u8);
                        if str_ptr.is_null() {
                            result.push_str("null");
                        } else {
                            let c_str = std::ffi::CStr::from_ptr(str_ptr .cast());
                            let s = c_str.to_str().unwrap_or("");
                            result.push_str(&format!("\"{}\"", s));
                        }
                    }
                    offset += 8;
                }
                "bool" => {
                    unsafe {
                        let val = *(ptr.add(offset) as *const u8);
                        result.push_str(if val != 0 { "true" } else { "false" });
                    }
                    offset += 1;
                }
                "i32" => {
                    unsafe {
                        let val = *(ptr.add(offset) as *const i32);
                        result.push_str(&val.to_string());
                    }
                    offset += 4;
                }
                "i64" => {
                    unsafe {
                        let val = *(ptr.add(offset) as *const i64);
                        result.push_str(&val.to_string());
                    }
                    offset += 8;
                }
                "f64" => {
                    unsafe {
                        let val = *(ptr.add(offset) as *const f64);
                        result.push_str(&val.to_string());
                    }
                    offset += 8;
                }
                _ => {
                    // Unknown type — skip (assume 8 bytes)
                    offset += 8;
                }
            }
        }
    }

    result.push('}');

    // Return heap-allocated C string
    let c_str = std::ffi::CString::new(result).unwrap_or_default();
    let bytes = c_str.into_bytes_with_nul();
    let len = bytes.len();
    let buf = crate::ky_alloc(len as i64);
    if buf.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
    }
    buf
}

/// Deserialize JSON to a struct buffer.
/// Writes field values directly into the struct memory.
/// Returns 0 on success, -1 on error.
#[unsafe(no_mangle)]
pub extern "C" fn ky_json_to_struct(json: *const u8, descriptor: *const u8, out: *mut u8) -> i32 {
    if json.is_null() || descriptor.is_null() || out.is_null() {
        return -1;
    }
    let json_cstr = unsafe { std::ffi::CStr::from_ptr(json .cast()) };
    let json_str = match json_cstr.to_str() {
        Ok(s) => {
            // Debug: print first 50 chars
            let _preview: String = s.chars().take(50).collect();
            // Actually, for stdout debugging use eprintln
            s.trim().to_string()
        }
        Err(_) => return -1,
    };
    let desc_cstr = unsafe { std::ffi::CStr::from_ptr(descriptor .cast()) };
    let desc = match desc_cstr.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };

    // Simple JSON object parser
    if !json_str.starts_with('{') || !json_str.ends_with('}') {
        return -1;
    }
    let inner = json_str[1..json_str.len()-1].trim().to_string();
    if inner.is_empty() {
        return 0;
    }

    // Parse JSON key:value pairs
    let mut json_pairs: Vec<(String, String)> = Vec::new();
    let mut pos: usize = 0;
    let chars: Vec<char> = inner.chars().collect();
    while pos < chars.len() {
        // Skip whitespace
        while pos < chars.len() && chars[pos].is_whitespace() { pos += 1; }
        if pos >= chars.len() { break; }

        // Find key (quoted)
        if chars[pos] != '"' { break; }
        pos += 1;
        let mut key = String::new();
        while pos < chars.len() && chars[pos] != '"' {
            key.push(chars[pos]);
            pos += 1;
        }
        if pos >= chars.len() { break; }
        pos += 1; // skip closing quote

        // Expect ':'
        while pos < chars.len() && chars[pos].is_whitespace() { pos += 1; }
        if pos >= chars.len() || chars[pos] != ':' { break; }
        pos += 1;
        while pos < chars.len() && chars[pos].is_whitespace() { pos += 1; }

        // Parse value
        if pos >= chars.len() { break; }
        let (val, new_pos) = if chars[pos] == '"' {
            pos += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                s.push(chars[pos]);
                pos += 1;
            }
            pos += 1; // closing quote
            (s, pos)
        } else if pos + 4 <= chars.len() && &inner[pos..pos+4] == "true" {
            ("true".to_string(), pos + 4)
        } else if pos + 5 <= chars.len() && &inner[pos..pos+5] == "false" {
            ("false".to_string(), pos + 5)
        } else if pos + 4 <= chars.len() && &inner[pos..pos+4] == "null" {
            ("null".to_string(), pos + 4)
        } else {
            // Number
            let start = pos;
            while pos < chars.len() && (chars[pos].is_digit(10) || chars[pos] == '.' || chars[pos] == '-') {
                pos += 1;
            }
            (inner[start..pos].to_string(), pos)
        };
        json_pairs.push((key, val));
        pos = new_pos;

        // Skip comma
        while pos < chars.len() && chars[pos].is_whitespace() { pos += 1; }
        if pos < chars.len() && chars[pos] == ',' { pos += 1; }
    }

    // Write fields to struct
    let mut offset: usize = 0;
    for field in desc.split(',') {
        let field = field.trim();
        if field.is_empty() { continue; }
        if let Some(col_pos) = field.find(':') {
            let name = field[..col_pos].trim();
            let ftype = field[col_pos + 1..].trim();

            // Find matching JSON key
            let json_val = json_pairs.iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str());

            match ftype {
                "str" => {
                    if let Some(val) = json_val {
                        // Remove quotes if present
                        let clean = val.trim_matches('"');
                        // Allocate string via ky_alloc
                        let bytes = clean.as_bytes();
                        let len = bytes.len();
                        let str_ptr = crate::ky_alloc((len + 1) as i64);
                        if !str_ptr.is_null() {
                            unsafe {
                                std::ptr::copy_nonoverlapping(bytes.as_ptr(), str_ptr, len);
                                *str_ptr.add(len) = 0;
                                *(out.add(offset) as *mut *mut u8) = str_ptr;
                            }
                        }
                    }
                    offset += 8;
                }
                "bool" => {
                    if let Some(val) = json_val {
                        unsafe {
                            let b = *val.as_bytes().first().unwrap_or(&0) == b't';
                            *(out.add(offset) as *mut u8) = if b { 1 } else { 0 };
                        }
                    }
                    offset += 1;
                }
                "i32" => {
                    if let Some(val) = json_val {
                        if let Ok(n) = val.parse::<i32>() {
                            unsafe { *(out.add(offset) as *mut i32) = n; }
                        }
                    }
                    offset += 4;
                }
                "i64" => {
                    if let Some(val) = json_val {
                        if let Ok(n) = val.parse::<i64>() {
                            unsafe { *(out.add(offset) as *mut i64) = n; }
                        }
                    }
                    offset += 8;
                }
                "f64" => {
                    if let Some(val) = json_val {
                        if let Ok(n) = val.parse::<f64>() {
                            unsafe { *(out.add(offset) as *mut f64) = n; }
                        }
                    }
                    offset += 8;
                }
                _ => {
                    offset += 8;
                }
            }
        }
    }

    0
}

/// Returns a list of string keys from a dict.
/// Each key is a heap-allocated string (caller must free via ky_free).
/// Returns pointer to KlList, or null if dict is null.
#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_keys(dict: *mut std::ffi::c_void) -> *mut crate::list::KlList {
    if dict.is_null() {
        return std::ptr::null_mut();
    }
    let map = unsafe { &*(dict as *const HashMap<String, i64>) };
    let list = crate::list::ky_list_new();
    if list.is_null() {
        return list;
    }
    for key in map.keys() {
        let bytes = key.as_bytes();
        let len = bytes.len();
        let buf = crate::memory::ky_alloc((len + 1) as i64);
        if !buf.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
                *buf.add(len) = 0;
            }
            let ptr_val = buf as i64;
            crate::list::ky_list_push(list, ptr_val);
        }
    }
    list
}

/// Return a list of all values in the dict.
#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_values(dict: *const std::ffi::c_void) -> *mut crate::list::KlList {
    if dict.is_null() { return crate::list::ky_list_new(); }
    let map = unsafe { &*(dict as *const KlDict) };
    let result = crate::list::ky_list_new();
    for v in map.values() {
        crate::list::ky_list_push(result, *v);
    }
    result
}

/// Return a list of all key-value pairs (tuples) in the dict.
/// Each element is a pointer to a {key: i64, val: i64} struct.
#[unsafe(no_mangle)]
pub extern "C" fn ky_dict_items(dict: *const std::ffi::c_void) -> *mut crate::list::KlList {
    if dict.is_null() { return crate::list::ky_list_new(); }
    let map = unsafe { &*(dict as *const KlDict) };
    let result = crate::list::ky_list_new();
    for (k, v) in map.iter() {
        let pair = crate::memory::ky_alloc(16) as *mut i64;
        unsafe {
            *pair = k.as_ptr() as i64;
            *pair.add(1) = *v;
        }
        crate::list::ky_list_push(result, pair as i64);
    }
    result
}
