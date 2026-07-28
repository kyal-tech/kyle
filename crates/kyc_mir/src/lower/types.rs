use std::cell::RefCell;
use kyc_core::ast::*;
use crate::mir::*;
use std::collections::HashMap;

use super::{LowerCtx, TYPE_ALIAS_CACHE};

pub(crate) fn build_descriptor(fields: &[(String, MirType)]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (fname, ftype) in fields {
        let type_name = match ftype {
            MirType::Str => "str",
            MirType::I32 => "i32",
            MirType::I64 => "i64",
            MirType::Bool => "bool",
            MirType::F64 => "f64",
            _ => "i32",
        };
        parts.push(format!("{}:{}", fname, type_name));
    }
    parts.join(",")
}

/// Check if a call name refers to a builtin that returns a string.
#[allow(dead_code)]
pub(crate) fn is_string_builtin_name(name: &str) -> bool {
    matches!(name, "ky_strlen" | "ky_i64_to_str" | "ky_input" | "ky_concat"
        | "ky_str_to_upper" | "ky_str_to_lower" | "ky_str_trim" | "ky_str_replace"
        | "ky_read_str"
        | "to_upper" | "to_lower" | "trim" | "replace" | "input" | "input_with_prompt" | "read_str")
}

/// Return the MIR type for known builtin functions, or None for generic functions.
pub(crate) fn builtin_return_type(name: &str) -> Option<MirType> {
    match name {
        "print" | "println" => Some(MirType::Void),
        "contains" => Some(MirType::I32),
        "to_upper" | "to_lower" | "upper" | "lower" | "trim" | "replace" | "input" | "input_with_prompt" => Some(MirType::Str),
        "open" | "close" | "write_str" => Some(MirType::I32),
        "read_str" => Some(MirType::Str),
        "char_at" => Some(MirType::Char),
        "ord" => Some(MirType::I32),
        "is_digit" | "is_alpha" | "is_alnum" | "is_whitespace" | "is_upper" | "is_lower" => Some(MirType::I32),
        "now" => Some(MirType::I64),
        "sleep" | "list_push" | "list_set" | "assert" | "assert_eq" | "assert_ne" | "assert_str" => Some(MirType::Void),
        "list_new" => Some(MirType::List(Box::new(MirType::I32))),
        "list_get" => Some(MirType::I64),
        "list_len" => Some(MirType::I64),
        "substr" => Some(MirType::Str),
        "eq_str" => Some(MirType::I32),
        "json_parse" => Some(MirType::Dict(Box::new(MirType::Str), Box::new(MirType::I64))),
        "json_stringify" | "json_stringify_str" => Some(MirType::Str),
        "serialize" => Some(MirType::Str),
        "ky_struct_to_json" => Some(MirType::Str),
        "type" => Some(MirType::Struct("TypeInfo".to_string(), vec![
            ("name".into(), MirType::Str),
            ("kind".into(), MirType::Str),
            ("size".into(), MirType::I32),
        ])),
        "ceil" | "floor" | "round" => Some(MirType::F64),
        "ky_getenv" | "ky_setenv" | "ky_base64_encode" | "ky_sha1" => Some(MirType::Str),
        "ky_spawn_thread" | "ky_join_thread" | "ky_parallel_for" => Some(MirType::I64),
        "ky_channel_new" => Some(MirType::I64),
        "ky_channel_send" | "ky_channel_recv" | "ky_channel_len" | "ky_channel_free" => Some(MirType::I64),
        "ky_channel_close" => Some(MirType::Void),
        "ok" | "error" => Some(MirType::Struct("Result".to_string(), vec![
            ("disc".to_string(), MirType::I32),
            ("payload".to_string(), MirType::I64),
        ])),
        "box" => None, // handled specially
        "ky_list_remove_value" => Some(MirType::I32),
        "ky_dict_contains" => Some(MirType::I32),
        "ky_dict_remove" => Some(MirType::I64),
        "ky_dict_keys" => Some(MirType::List(Box::new(MirType::I64))),
        "ky_str_builder_new" | "str_builder_new" => Some(MirType::Str),
        "str_builder_append" | "str_builder_free" => Some(MirType::Void),
        "str_builder_to_str" => Some(MirType::Str),
        "ky_str_builder_append" | "ky_str_builder_free" => Some(MirType::Void),
        "ky_str_builder_to_str" => Some(MirType::Str),
        "ky_fs_exists" | "ky_fs_is_dir" | "ky_fs_is_file"
            | "ky_fs_copy" | "ky_fs_remove" | "ky_fs_create_dir"
            | "ky_fs_remove_dir" | "ky_fs_rename" | "ky_fs_write_string" => Some(MirType::I32),
        "ky_fs_size" | "ky_time_now_ms" | "ky_time_now_us" | "ky_fs_list_dir" | "ky_set_len" => Some(MirType::I64),
        "ky_set_new" => Some(MirType::Set(Box::new(MirType::I32))),
        "ky_set_add" | "ky_set_free" => Some(MirType::Void),
        "ky_set_contains" | "ky_set_remove" => Some(MirType::I32),
        "ky_set_difference" | "ky_set_symmetric_difference" | "ky_set_union" | "ky_set_intersection" => Some(MirType::Set(Box::new(MirType::I64))),
        "ky_set_is_subset" => Some(MirType::I32),
        "ky_str_to_list" => Some(MirType::List(Box::new(MirType::Str))),
        "ky_set_to_list" | "ky_queue_to_list" | "ky_stack_to_list" => Some(MirType::List(Box::new(MirType::I64))),
        "ky_queue_to_stack" => Some(MirType::Stack(Box::new(MirType::I64))),
        "ky_stack_to_queue" => Some(MirType::Queue(Box::new(MirType::I64))),
        "ky_open" => Some(MirType::I32),
        "ky_read_str" => Some(MirType::Str),
        "ky_write_str" | "ky_close" => Some(MirType::I32),
        "ky_fs_read_to_string" => Some(MirType::Str),
        "ky_bytes_new" => Some(MirType::Bytes),
        "ky_bytes_get" | "ky_bytes_set" => Some(MirType::I32),
        "ky_bytes_to_hex" | "ky_bytes_to_base64" => Some(MirType::Str),
        "ky_bytes_from_hex" => Some(MirType::Bytes),
        _ => None,
    }
}

/// Convert an optional AstType annotation to a MirType.
pub(crate) fn param_type_from_annotation(ann: &Option<AstType>) -> Option<MirType> {
    match ann {
        Some(AstType::Primitive { name, .. } | AstType::User { name, .. }) => match name.as_str() {
            "str" => Some(MirType::Str),
            "i32" => Some(MirType::I32),
            "i64" => Some(MirType::I64),
            "f64" => Some(MirType::F64),
            "bool" => Some(MirType::I32),
            _ => None,
        },
        _ => None,
    }
}

/// Infer a closure parameter's MIR type by analyzing how it's used in the body.
/// Checks if the param participates in string concatenation (Str) or arithmetic (I32).
pub(crate) fn infer_closure_param_type(param: &str, body: &Expr) -> MirType {
    if let Some(t) = infer_expr_param_type(param, body) {
        return t;
    }
    MirType::Ptr(Box::new(MirType::I8))
}

pub(crate) fn infer_expr_param_type(param: &str, expr: &Expr) -> Option<MirType> {
    match expr {
        Expr::Binary { left, right, operator, .. } => {
            if matches!(operator, BinaryOp::Add) {
                if contains_param(param, left) && is_str_expr(right) {
                    return Some(MirType::Str);
                }
                if contains_param(param, right) && is_str_expr(left) {
                    return Some(MirType::Str);
                }
            }
            if let Some(t) = infer_expr_param_type(param, left) { return Some(t); }
            if let Some(t) = infer_expr_param_type(param, right) { return Some(t); }
        }
        Expr::Unary { operand, .. } => {
            if let Some(t) = infer_expr_param_type(param, operand) { return Some(t); }
        }
        Expr::Ternary { then_expr, else_expr, .. } => {
            if contains_param(param, then_expr) && !is_str_expr(then_expr) {
                if let Some(t) = infer_expr_param_type(param, then_expr) { return Some(t); }
            }
            if let Some(t) = infer_expr_param_type(param, else_expr) { return Some(t); }
        }
        Expr::FunctionCall { target, arguments, .. } => {
            // Check if param is the receiver of a method call: param.method(args)
            if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                if let Expr::Identifier { name, .. } = object.as_ref() {
                    if name == param {
                        // Infer type from known method patterns
                        match property.as_str() {
                            "json" | "text" | "status" | "send" | "redirect" => {
                                return Some(MirType::Ptr(Box::new(MirType::Struct("Res".into(), vec![]))));
                            }
                            "param" | "header" | "body" | "query" => {
                                return Some(MirType::Ptr(Box::new(MirType::Struct("Request".into(), vec![]))));
                            }
                            _ => {}
                        }
                    }
                }
            }
            for arg in arguments {
                if let Some(t) = infer_expr_param_type(param, arg) { return Some(t); }
            }
        }
        Expr::PropertyAccess { object, property, .. } => {
            if let Expr::Identifier { name, .. } = object.as_ref() {
                if name == param {
                    // Known struct field access patterns
                    match property.as_str() {
                        "fd" | "sent" | "status" | "json" | "text" => {
                            return Some(MirType::Ptr(Box::new(MirType::Struct("Res".into(), vec![]))));
                        }
                        "path" | "method" | "body" | "_pattern" => {
                            return Some(MirType::Ptr(Box::new(MirType::Struct("Request".into(), vec![]))));
                        }
                        _ => {
                            return Some(MirType::Ptr(Box::new(MirType::I8)));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    None
}

pub(crate) fn contains_param(param: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Identifier { name, .. } => name == param,
        Expr::Binary { left, right, .. } => contains_param(param, left) || contains_param(param, right),
        Expr::Unary { operand, .. } => contains_param(param, operand),
        Expr::Ternary { then_expr, else_expr, .. } => {
            contains_param(param, then_expr) || contains_param(param, else_expr)
        }
        Expr::FunctionCall { target, arguments, .. } => {
            // Check target for method calls: param.method(args)
            if let Expr::PropertyAccess { object, .. } = target.as_ref() {
                if contains_param(param, object) { return true; }
            }
            arguments.iter().any(|a| contains_param(param, a))
        }
        Expr::PropertyAccess { object, .. } => contains_param(param, object),
        _ => false,
    }
}

pub(crate) fn is_str_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Literal { value, .. } => matches!(value, Literal::String(_)),
        Expr::Binary { left, right, operator, .. } if matches!(operator, BinaryOp::Add) => {
            is_str_expr(left) || is_str_expr(right)
        }
        Expr::Ternary { then_expr, else_expr, .. } => is_str_expr(then_expr) || is_str_expr(else_expr),
        _ => false,
    }
}

/// Convert an AST type to an MIR type.
/// Check if an AstType references a specific type param name (e.g., `T` in `first: T`).
pub(crate) fn is_type_ref(ast: &AstType, tp_name: &str) -> bool {
    match ast {
        AstType::User { name, .. } | AstType::Primitive { name, .. } => name == tp_name,
        AstType::Generic { name, args, .. } => {
            name == tp_name || args.iter().any(|a| is_type_ref(a, tp_name))
        }
        _ => false,
    }
}

/// Build a TypeInfo struct literal for a given MirType.
/// Allocates locals, stores field values, and returns the final struct local.
pub(crate) fn build_typeinfo_struct(mir_type: &MirType, ctx: &mut LowerCtx) {
    let type_name = mir_type_to_type_name(mir_type);
    let type_kind = mir_type_to_kind(mir_type);
    let type_size = mir_type_to_size(mir_type);

    let struct_type = MirType::Struct("TypeInfo".into(), vec![
        ("name".into(), MirType::Str),
        ("kind".into(), MirType::Str),
        ("size".into(), MirType::I32),
    ]);

    let struct_local = ctx.alloc_local("_ti", struct_type.clone());
    let tin = ctx.alloc_local("_tin", MirType::Str);
    let tik = ctx.alloc_local("_tik", MirType::Str);
    ctx.string_locals.push(tin);
    ctx.string_locals.push(tik);

    // Field 0: name (str)
    let name_const = ctx.alloc_local("_tiname", MirType::Str);
    ctx.current_block.insts.push(MirInst::Store {
        dest: name_const,
        value: MirValue::Constant(MirConstant::String(type_name)),
    });
    let name_str = ctx.alloc_local("_tistr", MirType::Str);
    ctx.string_locals.push(name_str);
    ctx.current_block.insts.push(MirInst::Call {
        dest: Some(name_str),
        name: "ky_clone_str".to_string(),
        args: vec![MirValue::Local(name_const)],
    });
    let f0 = ctx.alloc_local("_tif0", MirType::I64);
    ctx.current_block.insts.push(MirInst::FieldPtr {
        dest: f0, ptr: struct_local, field_index: 0,
        struct_type: Box::new(struct_type.clone()),
    });
    ctx.current_block.insts.push(MirInst::Store {
        dest: f0,
        value: MirValue::Local(name_str),
    });

    // Field 1: kind (str)
    let kind_const = ctx.alloc_local("_tikind", MirType::Str);
    ctx.current_block.insts.push(MirInst::Store {
        dest: kind_const,
        value: MirValue::Constant(MirConstant::String(type_kind)),
    });
    let kind_str = ctx.alloc_local("_tikstr", MirType::Str);
    ctx.string_locals.push(kind_str);
    ctx.current_block.insts.push(MirInst::Call {
        dest: Some(kind_str),
        name: "ky_clone_str".to_string(),
        args: vec![MirValue::Local(kind_const)],
    });
    let f1 = ctx.alloc_local("_tif1", MirType::I64);
    ctx.current_block.insts.push(MirInst::FieldPtr {
        dest: f1, ptr: struct_local, field_index: 1,
        struct_type: Box::new(struct_type.clone()),
    });
    ctx.current_block.insts.push(MirInst::Store {
        dest: f1,
        value: MirValue::Local(kind_str),
    });

    // Field 2: size (i32)
    let f2 = ctx.alloc_local("_tif2", MirType::I64);
    ctx.current_block.insts.push(MirInst::FieldPtr {
        dest: f2, ptr: struct_local, field_index: 2,
        struct_type: Box::new(struct_type.clone()),
    });
    ctx.current_block.insts.push(MirInst::Store {
        dest: f2,
        value: MirValue::Constant(MirConstant::I32(type_size)),
    });

    // Load struct as result
    let load = ctx.alloc_local("_tires", struct_type);
    ctx.current_block.insts.push(MirInst::Load {
        dest: load,
        src: struct_local,
    });
}

/// Get the string name of a MirType (for TypeInfo.name)
pub(crate) fn mir_type_to_type_name(t: &MirType) -> String {
    match t {
        MirType::I1 => "i1".into(),
        MirType::I8 => "i8".into(),
        MirType::I16 => "i16".into(),
        MirType::I32 => "i32".into(),
        MirType::I64 => "i64".into(),
        MirType::U8 => "u8".into(),
        MirType::U16 => "u16".into(),
        MirType::U32 => "u32".into(),
        MirType::U64 => "u64".into(),
        MirType::F32 => "f32".into(),
        MirType::F64 => "f64".into(),
        MirType::Bool => "bool".into(),
        MirType::Char => "char".into(),
        MirType::Str => "str".into(),
        MirType::Void => "void".into(),
        MirType::Ptr(_) => "ptr".into(),
        MirType::List(inner) => format!("list<{}>", mir_type_to_type_name(inner)),
        MirType::Struct(name, _) => name.clone(),
        MirType::Dict(k, v) => format!("dict<{},{}>", mir_type_to_type_name(k), mir_type_to_type_name(v)),
        MirType::Set(inner) => format!("set<{}>", mir_type_to_type_name(inner)),
        MirType::Queue(inner) => format!("queue<{}>", mir_type_to_type_name(inner)),
        MirType::Stack(inner) => format!("stack<{}>", mir_type_to_type_name(inner)),
        MirType::Deque(inner) => format!("deque<{}>", mir_type_to_type_name(inner)),
        MirType::LinkedList(inner) => format!("linked_list<{}>", mir_type_to_type_name(inner)),
        MirType::Array(inner, _size) => format!("[{}]", mir_type_to_type_name(inner)),
        MirType::Slice(inner) => format!("&[{}]", mir_type_to_type_name(inner)),
        MirType::Box(inner) => format!("box<{}>", mir_type_to_type_name(inner)),
        MirType::Chan(inner) => format!("chan<{}>", mir_type_to_type_name(inner)),
        MirType::Bytes => "bytes".into(),
    }
}

/// Get the kind string of a MirType (for TypeInfo.kind)
pub(crate) fn mir_type_to_kind(t: &MirType) -> String {
    match t {
        MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64
        | MirType::U8 | MirType::U16 | MirType::U32 | MirType::U64
        | MirType::F32 | MirType::F64 | MirType::Bool | MirType::Char
        | MirType::Str | MirType::Void => "primitive".into(),
        MirType::Ptr(_) => "ptr".into(),
        MirType::List(_) => "list".into(),
        MirType::Struct(_, _) => "struct".into(),
        MirType::Dict(_, _) => "dict".into(),
        MirType::Set(_) => "set".into(),
        MirType::Queue(_) => "queue".into(),
        MirType::Stack(_) => "stack".into(),
        MirType::Deque(_) => "deque".into(),
        MirType::LinkedList(_) => "linked_list".into(),
        MirType::Array(_, _) => "array".into(),
        MirType::Slice(_) => "slice".into(),
        MirType::Box(_) => "box".into(),
        MirType::Chan(_) => "chan".into(),
        MirType::Bytes => "bytes".into(),
    }
}

/// Get the byte size of a MirType (for TypeInfo.size)
pub(crate) fn mir_type_to_size(t: &MirType) -> i32 {
    match t {
        MirType::I1 | MirType::Bool => 1,
        MirType::I8 | MirType::U8 | MirType::Char => 1,
        MirType::I16 | MirType::U16 => 2,
        MirType::I32 | MirType::U32 => 4,
        MirType::I64 | MirType::U64 => 8,
        MirType::F32 => 4,
        MirType::F64 => 8,
        MirType::Str | MirType::Ptr(_) | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Deque(_) | MirType::LinkedList(_) | MirType::Chan(_) | MirType::Bytes => 8,
        MirType::Void => 0,
        MirType::Struct(_, fields) => {
            // Estimate size: sum of field sizes (approximately, without padding)
            fields.iter().map(|(_, ft)| mir_type_to_size(ft) as i32).sum()
        }
        MirType::Array(inner, _) => mir_type_to_size(inner) * 8, // rough estimate
        MirType::Slice(_) => 16, // ptr (8) + len (8)
        MirType::Box(_) => 8, // ptr
        MirType::Chan(_) | MirType::Bytes => 8, // ptr
    }
}

/// Serialize a MirType to a string for concrete struct name mangling.
pub(crate) fn mir_type_to_string(t: &MirType) -> String {
    match t {
        MirType::I8 => "i8".into(),
        MirType::I16 => "i16".into(),
        MirType::I32 => "i32".into(),
        MirType::I64 => "i64".into(),
        MirType::U8 => "u8".into(),
        MirType::U16 => "u16".into(),
        MirType::U32 => "u32".into(),
        MirType::U64 => "u64".into(),
        MirType::F32 => "f32".into(),
        MirType::F64 => "f64".into(),
        MirType::Bool => "bool".into(),
        MirType::Char => "char".into(),
        MirType::Str => "str".into(),
        MirType::Void => "void".into(),
        MirType::Ptr(_) => "ptr".into(),
        MirType::List(inner) => format!("list_{}", mir_type_to_string(inner)),
        MirType::Struct(n, _) => n.clone(),
        MirType::I1 => "i1".into(),
        MirType::Array(inner, _) => format!("arr_{}", mir_type_to_string(inner)),
        MirType::Dict(key, val) => format!("dict_{}_{}", mir_type_to_string(key), mir_type_to_string(val)),
        MirType::Set(inner) => format!("set_{}", mir_type_to_string(inner)),
        MirType::Queue(inner) => format!("queue_{}", mir_type_to_string(inner)),
        MirType::Stack(inner) => format!("stack_{}", mir_type_to_string(inner)),
        MirType::Deque(inner) => format!("deque_{}", mir_type_to_string(inner)),
        MirType::LinkedList(inner) => format!("llist_{}", mir_type_to_string(inner)),
        MirType::Slice(inner) => format!("slice_{}", mir_type_to_string(inner)),
        MirType::Box(inner) => format!("box_{}", mir_type_to_string(inner)),
        MirType::Chan(inner) => format!("chan_{}", mir_type_to_string(inner)),
        MirType::Bytes => "bytes".into(),
    }
}

/// Create a mangled concrete struct name from a generic name and concrete type args.
pub(crate) fn make_concrete_name(name: &str, type_args: &[MirType]) -> String {
    if type_args.is_empty() {
        return name.to_string();
    }
    let args_str: Vec<String> = type_args.iter().map(mir_type_to_string).collect();
    format!("{}__{}", name, args_str.join("_"))
}

/// Extract the inner MirType from an Option__T struct name.
pub(crate) fn extract_struct_inner(name: &str) -> MirType {
    extract_inner_type(name)
}

/// Extract the inner type T from an Option__T struct name.
/// For example, "Option__i32" → MirType::I32, "Option__str" → MirType::Str.
pub(crate) fn extract_inner_type(struct_name: &str) -> MirType {
    if let Some(inner) = struct_name.strip_prefix("Option__") {
        match inner {
            "i8" => MirType::I8,
            "u8" => MirType::U8,
            "i16" => MirType::I16,
            "u16" => MirType::U16,
            "i32" | "int" => MirType::I32,
            "u32" => MirType::U32,
            "i64" => MirType::I64,
            "u64" => MirType::U64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            "void" => MirType::Void,
            _ => MirType::I32, // fallback
        }
    } else {
        MirType::I32
    }
}

/// Register Option__T struct fields in struct_defs for a given AstType
/// that may contain Option types (e.g. `i32?`, `Option<i32>`).
pub(crate) fn register_option_type(ast: &AstType, struct_defs: &mut std::collections::HashMap<String, Vec<(String, MirType)>>) {
    match ast {
        AstType::Optional { inner, .. } => {
            let inner_mir = ast_type_to_mir(inner, Some(struct_defs));
            let concrete_name = make_concrete_name("Option", &[inner_mir]);
            if !struct_defs.contains_key(&concrete_name) {
                struct_defs.insert(concrete_name, vec![
                    ("disc".to_string(), MirType::I32),
                    ("payload".to_string(), MirType::I64),
                ]);
            }
        }
        AstType::Generic { name, args, .. } if name == "Option" && args.len() == 1 => {
            register_option_type(&args[0], struct_defs);
            let inner_mir = ast_type_to_mir(&args[0], Some(struct_defs));
            let concrete_name = make_concrete_name("Option", &[inner_mir]);
            if !struct_defs.contains_key(&concrete_name) {
                struct_defs.insert(concrete_name, vec![
                    ("disc".to_string(), MirType::I32),
                    ("payload".to_string(), MirType::I64),
                ]);
            }
        }
        _ => {}
    }
}

/// Extract generic type bindings by matching an AstType (parameter type with type params)
/// against a concrete MirType (argument type).
/// Scan an AstType for generic struct references and pre-register concrete versions.
/// This must run before lower_function so that the function signature can resolve
/// return types like `Pair<i32, str>` as `MirType::Struct("Pair__i32_str", fields)`.
pub(crate) fn pre_register_generic_type(
    ast: &AstType,
    type_subst: &std::collections::HashMap<String, MirType>,
    generic_struct_templates: &std::collections::HashMap<String, StructDecl>,
    struct_defs: &mut std::collections::HashMap<String, Vec<(String, MirType)>>,
) -> Option<MirType> {
    match ast {
        AstType::Generic { name, args, .. } if name != "list" && !args.is_empty() => {
            if let Some(tpl) = generic_struct_templates.get(name) {
                let concrete_args: Vec<MirType> = args.iter()
                    .map(|a| {
                        let mir = ast_type_to_mir_with_subst(a, Some(struct_defs), type_subst);
                        // Recurse for nested generic types
                        if let AstType::Generic { .. } = a {
                            if let Some(m) = pre_register_generic_type(a, type_subst, generic_struct_templates, struct_defs) {
                                return m;
                            }
                        }
                        mir
                    })
                    .collect();
                let concrete_name = make_concrete_name(name, &concrete_args);
                if !struct_defs.contains_key(&concrete_name) {
                    let concrete_fields: Vec<(String, MirType)> = tpl.fields.iter()
                        .map(|f| (f.name.clone(), ast_type_to_mir_with_subst(&f.type_, Some(struct_defs), type_subst)))
                        .collect();
                    struct_defs.insert(concrete_name.clone(), concrete_fields.clone());
                }
                return struct_defs.get(&concrete_name).map(|f| MirType::Struct(concrete_name, f.clone()));
            }
        }
        _ => {}
    }
    None
}

pub(crate) fn extract_generic_bindings(
    param_type: &AstType,
    arg_type: &MirType,
    type_params: &[TypeParam],
    subst: &mut std::collections::HashMap<String, MirType>,
) {
    match (param_type, arg_type) {
        (AstType::User { name, .. } | AstType::Primitive { name, .. }, _) => {
            if type_params.iter().any(|tp| tp.name == *name) && !subst.contains_key(name) {
                subst.insert(name.clone(), arg_type.clone());
            }
        }
        (AstType::Generic { name, args, .. }, MirType::List(inner)) if name == "list" => {
            if let Some(elem_type) = args.first() {
                extract_generic_bindings(elem_type, inner, type_params, subst);
            }
        }
        (AstType::Optional { inner, .. }, MirType::Ptr(inner_type)) => {
            extract_generic_bindings(inner, inner_type, type_params, subst);
        }
        (AstType::Optional { inner, .. }, MirType::Struct(name, _)) if name.starts_with("Option__") => {
            let inner_type = extract_struct_inner(name);
            extract_generic_bindings(inner, &inner_type, type_params, subst);
        }
        _ => {}
    }
}

/// Infer type param bindings for a generic function call from the concrete argument types.
pub(crate) fn infer_function_type_params(
    template: &FunctionDecl,
    arg_types: &[MirType],
) -> std::collections::HashMap<String, MirType> {
    let mut subst: std::collections::HashMap<String, MirType> = std::collections::HashMap::new();
    for (param, arg_type) in template.params.iter().zip(arg_types) {
        extract_generic_bindings(&param.type_, arg_type, &template.type_params, &mut subst);
    }
    subst
}

/// Convert a MirType to an AstType for substitution into function AST.
pub(crate) fn mir_type_to_ast_type(t: &MirType, _span: kyc_core::span::Span) -> AstType {
    match t {
        MirType::I8 => AstType::Primitive { name: "i8".into(), span: _span },
        MirType::I16 => AstType::Primitive { name: "i16".into(), span: _span },
        MirType::I32 => AstType::Primitive { name: "i32".into(), span: _span },
        MirType::I64 => AstType::Primitive { name: "i64".into(), span: _span },
        MirType::U8 => AstType::Primitive { name: "u8".into(), span: _span },
        MirType::U16 => AstType::Primitive { name: "u16".into(), span: _span },
        MirType::U32 => AstType::Primitive { name: "u32".into(), span: _span },
        MirType::U64 => AstType::Primitive { name: "u64".into(), span: _span },
        MirType::F32 => AstType::Primitive { name: "f32".into(), span: _span },
        MirType::F64 => AstType::Primitive { name: "f64".into(), span: _span },
        MirType::Bool => AstType::Primitive { name: "bool".into(), span: _span },
        MirType::Char => AstType::Primitive { name: "char".into(), span: _span },
        MirType::Str => AstType::Primitive { name: "str".into(), span: _span },
        MirType::Void => AstType::Primitive { name: "void".into(), span: _span },
        MirType::List(inner) => AstType::Generic {
            name: "list".into(),
            args: vec![mir_type_to_ast_type(inner, _span)],
            span: _span,
        },
        MirType::Struct(name, _) => AstType::User { name: name.clone(), span: _span },
        MirType::Ptr(_) => AstType::User { name: "ptr".into(), span: _span },
        MirType::Dict(key, value) => AstType::Dict {
            key: Box::new(mir_type_to_ast_type(key, _span)),
            value: Box::new(mir_type_to_ast_type(value, _span)),
            span: _span,
        },
        MirType::Set(inner) => AstType::Set {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::Queue(inner) => AstType::Queue {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::Stack(inner) => AstType::Stack {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::Deque(inner) => AstType::Deque {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::LinkedList(inner) => AstType::LinkedList {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::I1 => AstType::Primitive { name: "bool".into(), span: _span },
        MirType::Array(inner, size) => AstType::Array {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            size: *size,
            span: _span,
        },
        MirType::Slice(inner) => AstType::Slice {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::Box(inner) => AstType::Generic {
            name: "box".into(),
            args: vec![mir_type_to_ast_type(inner, _span)],
            span: _span,
        },
        MirType::Chan(inner) => AstType::Chan {
            inner: Box::new(mir_type_to_ast_type(inner, _span)),
            span: _span,
        },
        MirType::Bytes => AstType::Bytes { span: _span },
    }
}

/// Substitute type params in an AstType with concrete AstTypes.
pub(crate) fn substitute_ast_type(ast: &AstType, subst: &std::collections::HashMap<String, AstType>) -> AstType {
    match ast {
        AstType::User { name, .. } => {
            if let Some(replacement) = subst.get(name) {
                replacement.clone()
            } else {
                ast.clone()
            }
        }
        AstType::Primitive { name, .. } => {
            if let Some(replacement) = subst.get(name) {
                replacement.clone()
            } else {
                ast.clone()
            }
        }
        AstType::Generic { name, args, span } => {
            AstType::Generic {
                name: name.clone(),
                args: args.iter().map(|a| substitute_ast_type(a, subst)).collect(),
                span: *span,
            }
        }
        AstType::Optional { inner, span } => {
            AstType::Optional {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Error { inner, span } => {
            AstType::Error {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Dict { key, value, span } => {
            AstType::Dict {
                key: Box::new(substitute_ast_type(key, subst)),
                value: Box::new(substitute_ast_type(value, subst)),
                span: *span,
            }
        }
        AstType::FnPtr { params, return_, span } => {
            AstType::FnPtr {
                params: params.iter().map(|p| substitute_ast_type(p, subst)).collect(),
                return_: Box::new(substitute_ast_type(return_, subst)),
                span: *span,
            }
        }
        AstType::Mutable { inner, span } => {
            AstType::Mutable {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Borrow { inner, span } => {
            AstType::Borrow {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Borrow { inner, span } => {
            AstType::Borrow {
                inner: Box::new(substitute_ast_type(inner, subst)),
                span: *span,
            }
        }
        AstType::Ptr { span } => AstType::Ptr { span: *span },
        AstType::Set { inner, span, .. } => AstType::Set {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::Queue { inner, span, .. } => AstType::Queue {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::Stack { inner, span, .. } => AstType::Stack {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::Deque { inner, span, .. } => AstType::Deque {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::LinkedList { inner, span, .. } => AstType::LinkedList {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::List { inner, span, .. } => AstType::List {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::Array { inner, size, span, .. } => AstType::Array {
            inner: Box::new(substitute_ast_type(inner, subst)),
            size: *size,
            span: *span,
        },
        AstType::Slice { inner, span } => AstType::Slice {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::Chan { inner, span } => AstType::Chan {
            inner: Box::new(substitute_ast_type(inner, subst)),
            span: *span,
        },
        AstType::Bytes { span } => AstType::Bytes { span: *span },
    }
}

/// Clone a FunctionDecl, substituting its type params with concrete type args.
pub(crate) fn clone_and_specialize_function(
    template: &FunctionDecl,
    type_subst: &std::collections::HashMap<String, MirType>,
) -> FunctionDecl {
    let mut f = template.clone();
    // Build AstType substitution map
    let ast_subst: std::collections::HashMap<String, AstType> = type_subst.iter()
        .map(|(name, mir_type)| (name.clone(), mir_type_to_ast_type(mir_type, f.span)))
        .collect();
    // Substitute param types
    for p in &mut f.params {
        p.type_ = substitute_ast_type(&p.type_, &ast_subst);
    }
    // Substitute return type
    if let Some(rt) = &mut f.return_type {
        *rt = substitute_ast_type(rt, &ast_subst);
    }
    // Substitute types in block statements (variable declarations)
    if let Some(body) = &mut f.body {
        for stmt in &mut body.statements {
            substitute_stmt_types(stmt, &ast_subst);
        }
    }
    f
}

/// Substitute type params in variable declarations inside a statement.
pub(crate) fn substitute_stmt_types(stmt: &mut Stmt, subst: &std::collections::HashMap<String, AstType>) {
    match stmt {
        Stmt::Variable(v) | Stmt::TypedVariable(v) => {
            if let Some(t) = &mut v.type_ {
                *t = substitute_ast_type(t, subst);
            }
        }
        Stmt::If(s) => {
            for s_ in &mut s.body.statements { substitute_stmt_types(s_, subst); }
            for el in &mut s.elif_branches {
                for s_ in &mut el.body.statements { substitute_stmt_types(s_, subst); }
            }
            if let Some(b) = &mut s.else_branch {
                for s_ in &mut b.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::While(w) => {
            for s_ in &mut w.body.statements { substitute_stmt_types(s_, subst); }
            if let Some(b) = &mut w.else_branch {
                for s_ in &mut b.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::For(f) => {
            for s_ in &mut f.body.statements { substitute_stmt_types(s_, subst); }
            if let Some(b) = &mut f.else_branch {
                for s_ in &mut b.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::Match(m) => {
            for arm in &mut m.arms {
                for s_ in &mut arm.body.statements { substitute_stmt_types(s_, subst); }
            }
        }
        Stmt::Unsafe(u) => {
            for s_ in &mut u.body.statements { substitute_stmt_types(s_, subst); }
        }
        Stmt::Guard(g) => {
            for s_ in &mut g.body.statements { substitute_stmt_types(s_, subst); }
        }
        Stmt::BindingIf(b) => {
            for s_ in &mut b.body.statements { substitute_stmt_types(s_, subst); }
            if let Some(el) = &mut b.else_branch {
                for s_ in &mut el.statements { substitute_stmt_types(s_, subst); }
            }
        }
        _ => {}
    }
}

/// Convert an AstType to MirType with type param substitution.
pub(crate) fn ast_type_to_mir_with_subst(
    ast: &AstType,
    struct_defs: Option<&std::collections::HashMap<String, Vec<(String, MirType)>>>,
    subst: &std::collections::HashMap<String, MirType>,
) -> MirType {
    match ast {
        AstType::Primitive { name, .. } => {
            if let Some(t) = subst.get(name) { return t.clone(); }
            match name.as_str() {
                "i8" => MirType::I8,
                "u8" => MirType::U8,
                "i16" => MirType::I16,
                "u16" => MirType::U16,
                "i32" | "int" => MirType::I32,
                "u32" => MirType::U32,
                "i64" => MirType::I64,
                "u64" => MirType::U64,
                "f32" => MirType::F32,
                "f64" => MirType::F64,
                "bool" => MirType::Bool,
                "char" => MirType::Char,
                "str" => MirType::Str,
                "void" => MirType::Void,
                _ => MirType::I32,
            }
        }
        AstType::User { name, .. } => {
            if let Some(t) = subst.get(name) { return t.clone(); }
            match name.as_str() {
                "i8" => MirType::I8,
                "u8" => MirType::U8,
                "i16" => MirType::I16,
                "u16" => MirType::U16,
                "i32" | "int" => MirType::I32,
                "u32" => MirType::U32,
                "i64" => MirType::I64,
                "u64" => MirType::U64,
                "f32" => MirType::F32,
                "f64" => MirType::F64,
                "bool" => MirType::Bool,
                "char" => MirType::Char,
                "str" => MirType::Str,
                name => {
                    if let Some(defs) = struct_defs {
                        if let Some(fields) = defs.get(name) {
                            return MirType::Struct(name.to_string(), fields.clone());
                        }
                    }
                    MirType::Struct(name.to_string(), vec![])
                }
            }
        }
        AstType::Generic { name, args, .. } => {
            if let Some(t) = subst.get(name) { return t.clone(); }
            if name == "set" {
                if args.is_empty() { MirType::Set(Box::new(MirType::I32)) }
                else { MirType::Set(Box::new(ast_type_to_mir_with_subst(&args[0], struct_defs, subst))) }
            } else if name == "list" {
                if args.is_empty() { MirType::List(Box::new(MirType::I32)) }
                else { MirType::List(Box::new(ast_type_to_mir_with_subst(&args[0], struct_defs, subst))) }
            } else if name == "tuple" {
                // (T, U, ...) → anonymous struct with _0, _1, ... fields
                let field_types: Vec<(String, MirType)> = args.iter().enumerate()
                    .map(|(i, el)| (format!("_{}", i), ast_type_to_mir_with_subst(el, struct_defs, subst)))
                    .collect();
                // Create unique name based on field types
                let type_suffix: String = field_types.iter()
                    .map(|(_, t)| match t {
                        MirType::I32 => "i32",
                        MirType::I64 => "i64",
                        MirType::Str => "str",
                        MirType::Bool => "bool",
                        MirType::F64 => "f64",
                        MirType::F32 => "f32",
                        MirType::I16 => "i16",
                        MirType::I8 => "i8",
                        MirType::Char => "char",
                        _ => "x",
                    })
                    .collect();
                let struct_name = format!("_tuple_{}_{}", field_types.len(), type_suffix);
                MirType::Struct(struct_name, field_types)
            } else if args.is_empty() {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                MirType::Struct(name.clone(), vec![])
            } else {
                // Check if the base name is already registered in struct_defs with known
                // fields (e.g. enums). Enums and non-generic structs use the base name
                // directly; generic structs/classes are registered with concrete names.
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        if !fields.is_empty() {
                            return MirType::Struct(name.to_string(), fields.clone());
                        }
                    }
                }
                // Handle Option<T> — always create struct with disc + payload fields
                if name == "Option" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir_with_subst(&args[0], struct_defs, subst);
                    let concrete_name = make_concrete_name("Option", &[inner_mir]);
                    return MirType::Struct(concrete_name, vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]);
                }
                // Handle box<T> — built-in heap pointer type
                if name == "box" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir_with_subst(&args[0], struct_defs, subst);
                    return MirType::Box(Box::new(inner_mir));
                }
                // User-defined generic with concrete args — create concrete version
                let concrete_args: Vec<MirType> = args.iter()
                    .map(|a| ast_type_to_mir_with_subst(a, struct_defs, subst))
                    .collect();
                let concrete_name = make_concrete_name(name, &concrete_args);
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(&concrete_name) {
                        return MirType::Struct(concrete_name, fields.clone());
                    }
                }
                MirType::Struct(concrete_name, vec![])
            }
        }
        AstType::Optional { inner, .. } => {
            let inner_mir = ast_type_to_mir_with_subst(inner, struct_defs, subst);
            let concrete_name = make_concrete_name("Option", &[inner_mir.clone()]);
            MirType::Struct(concrete_name, vec![
                ("disc".to_string(), MirType::I32),
                ("payload".to_string(), MirType::I64),
            ])
        }
        AstType::Dict { key, value, .. } => MirType::Dict(
            Box::new(ast_type_to_mir_with_subst(key, struct_defs, subst)),
            Box::new(ast_type_to_mir_with_subst(value, struct_defs, subst)),
        ),
        AstType::Error { inner: _, .. } => MirType::Struct("Result".to_string(), vec![
            ("disc".to_string(), MirType::I32),
            ("payload".to_string(), MirType::I64),
        ]),
        AstType::FnPtr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => ast_type_to_mir_with_subst(inner, struct_defs, subst),
        AstType::Borrow { inner, .. } => ast_type_to_mir_with_subst(inner, struct_defs, subst),
        AstType::Ptr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Set { inner, .. } => MirType::Set(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::Queue { inner, .. } => MirType::Queue(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::Stack { inner, .. } => MirType::Stack(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::Deque { inner, .. } => MirType::Deque(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::LinkedList { inner, .. } => MirType::LinkedList(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::List { inner, .. } => MirType::List(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::Array { inner, size, .. } => MirType::Array(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst)), *size),
        AstType::Slice { inner, .. } => MirType::Slice(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::Chan { inner, .. } => MirType::Chan(Box::new(ast_type_to_mir_with_subst(inner, struct_defs, subst))),
        AstType::Bytes { .. } => MirType::Bytes,
    }
}

pub(crate) fn ast_type_to_mir(ast: &AstType, struct_defs: Option<&std::collections::HashMap<String, Vec<(String, MirType)>>>) -> MirType {
    match ast {
        AstType::Primitive { name, .. } => match name.as_str() {
            "i8" => MirType::I8,
            "u8" => MirType::U8,
            "i16" => MirType::I16,
            "u16" => MirType::U16,
            "i32" | "int" => MirType::I32,
            "u32" => MirType::U32,
            "i64" => MirType::I64,
            "u64" => MirType::U64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            "void" => MirType::Void,
            _ => MirType::I32,
        },
        AstType::User { name, .. } => match name.as_str() {
            "i8" => MirType::I8,
            "u8" => MirType::U8,
            "i16" => MirType::I16,
            "u16" => MirType::U16,
            "i32" | "int" => MirType::I32,
            "u32" => MirType::U32,
            "i64" => MirType::I64,
            "u64" => MirType::U64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            "bool" => MirType::Bool,
            "char" => MirType::Char,
            "str" => MirType::Str,
            name => {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                // Resolve type alias if defined
                let alias_mir = TYPE_ALIAS_CACHE.with(|cache| {
                    let defs = cache.borrow();
                    defs.get(name).and_then(|ast| {
                        // Recursively resolve alias chain
                        let mut seen = std::collections::HashSet::new();
                        let mut current = ast;
                        let mut current_name = name;
                        loop {
                            if !seen.insert(current_name.to_string()) {
                                return None; // cycle
                            }
                            match current {
                                AstType::Primitive { name, .. } | AstType::User { name, .. } => {
                                    let n = name.as_str();
                                    let mir = match n {
                                        "i8" => Some(MirType::I8), "i16" => Some(MirType::I16),
                                        "i32" | "int" => Some(MirType::I32), "i64" => Some(MirType::I64),
                                        "f32" => Some(MirType::F32), "f64" => Some(MirType::F64),
                                        "bool" => Some(MirType::Bool), "char" => Some(MirType::Char),
                                        "str" => Some(MirType::Str),
                                        other => {
                                            if let Some(defs) = struct_defs {
                                                if let Some(_) = defs.get(other) {
                                                    // Let ast_type_to_mir handle structs
                                                    return None;
                                                }
                                            }
                                            // Try dereferencing another alias
                                            if let Some(next) = defs.get(other) {
                                                current = next;
                                                current_name = other;
                                                continue;
                                            }
                                            None
                                        }
                                    };
                                    return mir;
                                }
                                _ => return None,
                            }
                        }
                    })
                });
                if let Some(mir) = alias_mir {
                    return mir;
                }
                MirType::Struct(name.to_string(), vec![])
            }
        },
        AstType::Generic { name, args, .. } => {
            if name == "set" {
                if args.is_empty() { MirType::Set(Box::new(MirType::I32)) }
                else { MirType::Set(Box::new(ast_type_to_mir(&args[0], struct_defs))) }
            } else if name == "list" {
                if args.is_empty() { MirType::List(Box::new(MirType::I32)) }
                else { MirType::List(Box::new(ast_type_to_mir(&args[0], struct_defs))) }
            } else if name == "tuple" {
                // (T, U, ...) → anonymous struct
                let field_types: Vec<(String, MirType)> = args.iter().enumerate()
                    .map(|(i, el)| (format!("_{}", i), ast_type_to_mir(el, struct_defs)))
                    .collect();
                let type_suffix: String = field_types.iter()
                    .map(|(_, t)| match t {
                        MirType::I32 => "i32",
                        MirType::I64 => "i64",
                        MirType::Str => "str",
                        MirType::Bool => "bool",
                        MirType::F64 => "f64",
                        MirType::F32 => "f32",
                        MirType::I16 => "i16",
                        MirType::I8 => "i8",
                        MirType::Char => "char",
                        _ => "x",
                    })
                    .collect();
                let struct_name = format!("_tuple_{}_{}", field_types.len(), type_suffix);
                MirType::Struct(struct_name, field_types)
            } else if args.is_empty() {
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        return MirType::Struct(name.to_string(), fields.clone());
                    }
                }
                MirType::Struct(name.clone(), vec![])
            } else {
                // Check if the base name is already registered in struct_defs with known
                // fields (e.g. enums). Enums and non-generic structs use the base name
                // directly; generic structs/classes are registered with concrete names.
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(name) {
                        if !fields.is_empty() {
                            return MirType::Struct(name.to_string(), fields.clone());
                        }
                    }
                }
                // Handle Option<T> — always create struct with disc + payload fields
                if name == "Option" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir(&args[0], struct_defs);
                    let concrete_name = make_concrete_name("Option", &[inner_mir]);
                    return MirType::Struct(concrete_name, vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]);
                }
                // Handle box<T> — built-in heap pointer type
                if name == "box" && args.len() == 1 {
                    let inner_mir = ast_type_to_mir(&args[0], struct_defs);
                    return MirType::Box(Box::new(inner_mir));
                }
                // User-defined generic with concrete type args — create concrete name
                let concrete_args: Vec<MirType> = args.iter()
                    .map(|a| ast_type_to_mir(a, struct_defs))
                    .collect();
                let concrete_name = make_concrete_name(name, &concrete_args);
                if let Some(defs) = struct_defs {
                    if let Some(fields) = defs.get(&concrete_name) {
                        return MirType::Struct(concrete_name, fields.clone());
                    }
                }
                MirType::Struct(concrete_name, vec![])
            }
        }
        AstType::Optional { inner, .. } => {
            let inner_mir = ast_type_to_mir(inner, struct_defs);
            let concrete_name = make_concrete_name("Option", &[inner_mir.clone()]);
            MirType::Struct(concrete_name, vec![
                ("disc".to_string(), MirType::I32),
                ("payload".to_string(), MirType::I64),
            ])
        }
        AstType::Dict { key, value, .. } => MirType::Dict(
            Box::new(ast_type_to_mir(key, struct_defs)),
            Box::new(ast_type_to_mir(value, struct_defs)),
        ),
        AstType::Error { inner: _, .. } => MirType::Struct("Result".to_string(), vec![
            ("disc".to_string(), MirType::I32),
            ("payload".to_string(), MirType::I64),
        ]),
        AstType::FnPtr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => ast_type_to_mir(inner, struct_defs),
        AstType::Ptr { .. } => MirType::Ptr(Box::new(MirType::Void)),
        AstType::Set { inner, .. } => MirType::Set(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Queue { inner, .. } => MirType::Queue(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Stack { inner, .. } => MirType::Stack(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Deque { inner, .. } => MirType::Deque(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::LinkedList { inner, .. } => MirType::LinkedList(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::List { inner, .. } => MirType::List(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Array { inner, size, .. } => MirType::Array(Box::new(ast_type_to_mir(inner, struct_defs)), *size),
        AstType::Slice { inner, .. } => MirType::Slice(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Chan { inner, .. } => MirType::Chan(Box::new(ast_type_to_mir(inner, struct_defs))),
        AstType::Bytes { .. } => MirType::Bytes,
    }
}
