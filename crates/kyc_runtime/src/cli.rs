// CLI argument parsing runtime.
//
// Exposes ky_cli_* functions for the `std.cli` module. Parses the process
// command-line (argv via std::env::args) into positional args + flags,
// tracks flag definitions for typed access (get_int/get_bool) and help text.
//
// State is stored globally; `ky_cli_parse` must be called before the other
// accessors (mirrors the documented `cli.parse()` first-call requirement).

use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;

#[derive(Default)]
struct CliFlagDef {
    name: String,
    short: String,
    desc: String,
    default: String,
}

#[derive(Default)]
struct CliState {
    positional: Vec<String>,
    flags: HashMap<String, String>,
    defined: Vec<CliFlagDef>,
}

static STATE: Mutex<Option<CliState>> = Mutex::new(None);

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

fn parse_bool(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

/// Parse argv into positional args + flags. Returns a {str:str} dict handle
/// of the provided flags (values are heap strings; free the dict with
/// ky_dict_free). Must be called before arg/has/get/get_int/get_bool.
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_parse() -> *mut std::ffi::c_void {
    let mut state = CliState::default();
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Index of defined short flags -> (name, default)
    let shorts: HashMap<String, (String, String)> = STATE
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| {
            s.defined
                .iter()
                .filter(|d| !d.short.is_empty())
                .map(|d| (d.short.clone(), (d.name.clone(), d.default.clone())))
                .collect()
        })
        .unwrap_or_default();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(rest) = arg.strip_prefix("--") {
            if let Some((k, v)) = rest.split_once('=') {
                state.flags.insert(k.to_string(), v.to_string());
            } else {
                state.flags.insert(rest.to_string(), "true".to_string());
            }
        } else if let Some(rest) = arg.strip_prefix('-') {
            if let Some((k, v)) = rest.split_once('=') {
                state.flags.insert(k.to_string(), v.to_string());
            } else if let Some((name, default)) = shorts.get(rest).cloned() {
                // Defined short flag: consume next arg as value if it isn't a flag
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    state.flags.insert(name.clone(), args[i].clone());
                } else {
                    state.flags.insert(name, default);
                }
            } else {
                state.flags.insert(rest.to_string(), "true".to_string());
            }
        } else {
            state.positional.push(arg.clone());
        }
        i += 1;
    }

    // Build {str:str} dict of provided flags
    let dict = crate::ky_dict_new();
    for (k, v) in &state.flags {
        let vptr = alloc_str(v);
        crate::ky_dict_set(dict, k.as_ptr().cast(), vptr as i64);
    }

    *STATE.lock().unwrap() = Some(state);
    dict
}

/// Number of positional arguments.
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_argc() -> i32 {
    STATE.lock().unwrap().as_ref().map(|s| s.positional.len() as i32).unwrap_or(0)
}

/// Positional argument at index (null-terminated heap string), or null.
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_arg(n: i32) -> *mut u8 {
    if n < 0 {
        return std::ptr::null_mut();
    }
    STATE
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|s| s.positional.get(n as usize))
        .map(|v| alloc_str(v))
        .unwrap_or(std::ptr::null_mut())
}

/// 1 if a flag with this name (or short) was provided.
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_has(name: *const u8) -> i32 {
    let name = cstr(name);
    if name.is_empty() {
        return 0;
    }
    let s = STATE.lock().unwrap();
    let Some(s) = s.as_ref() else { return 0 };
    let long = s.defined.iter().find(|d| d.short == name).map(|d| d.name.clone()).unwrap_or(name.clone());
    i32::from(s.flags.contains_key(&long))
}

/// Value of a flag (null-terminated heap string), or null if not provided.
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_get(name: *const u8) -> *mut u8 {
    let name = cstr(name);
    if name.is_empty() {
        return std::ptr::null_mut();
    }
    let s = STATE.lock().unwrap();
    let Some(s) = s.as_ref() else { return std::ptr::null_mut() };
    let long = s.defined.iter().find(|d| d.short == name).map(|d| d.name.clone()).unwrap_or(name.clone());
    s.flags.get(&long).map(|v| alloc_str(v)).unwrap_or(std::ptr::null_mut())
}

/// Flag value as integer (uses the defined default when not provided).
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_get_int(name: *const u8) -> i64 {
    let name = cstr(name);
    if name.is_empty() {
        return 0;
    }
    let s = STATE.lock().unwrap();
    let Some(s) = s.as_ref() else { return 0 };
    let long = s.defined.iter().find(|d| d.short == name).map(|d| d.name.clone()).unwrap_or(name.clone());
    let value = s
        .flags
        .get(&long)
        .cloned()
        .or_else(|| s.defined.iter().find(|d| d.name == long).map(|d| d.default.clone()))
        .unwrap_or_default();
    value.trim().parse::<i64>().unwrap_or(0)
}

/// Flag value as boolean (uses the defined default when not provided).
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_get_bool(name: *const u8) -> i32 {
    let name = cstr(name);
    if name.is_empty() {
        return 0;
    }
    let s = STATE.lock().unwrap();
    let Some(s) = s.as_ref() else { return 0 };
    let long = s.defined.iter().find(|d| d.short == name).map(|d| d.name.clone()).unwrap_or(name.clone());
    let value = s
        .flags
        .get(&long)
        .cloned()
        .or_else(|| s.defined.iter().find(|d| d.name == long).map(|d| d.default.clone()))
        .unwrap_or_default();
    i32::from(parse_bool(&value))
}

/// Register a flag definition (name, short, description, default).
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_define(name: *const u8, short: *const u8, desc: *const u8, default: *const u8) -> i32 {
    let def = CliFlagDef {
        name: cstr(name),
        short: cstr(short),
        desc: cstr(desc),
        default: cstr(default),
    };
    if def.name.is_empty() {
        return -1;
    }
    let mut s = STATE.lock().unwrap();
    let state = s.get_or_insert_with(CliState::default);
    if !state.defined.iter().any(|d| d.name == def.name) {
        state.defined.push(def);
    }
    0
}

/// Build a --help usage string (null-terminated heap string).
#[unsafe(no_mangle)]
pub extern "C" fn ky_cli_help() -> *mut u8 {
    let s = STATE.lock().unwrap();
    let Some(s) = s.as_ref() else { return std::ptr::null_mut() };
    if s.defined.is_empty() {
        return alloc_str("Usage: <program> [options]\n  (no flags defined)");
    }
    let mut out = String::from("Usage: <program> [options]\n");
    for d in &s.defined {
        let mut line = format!("  --{}", d.name);
        if !d.short.is_empty() {
            line.push_str(&format!(", -{}", d.short));
        }
        if !d.desc.is_empty() {
            line.push_str(&format!("   {}", d.desc));
        }
        if !d.default.is_empty() {
            line.push_str(&format!("  (default: {})", d.default));
        }
        out.push_str(&line);
        out.push('\n');
    }
    alloc_str(&out)
}
