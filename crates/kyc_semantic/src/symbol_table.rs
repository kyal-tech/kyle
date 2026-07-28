use std::collections::HashMap;
use kyc_core::ast::{FunctionDecl, ClassDecl, StructDecl, EnumDecl, ContractDecl, TypeAlias, Visibility};
use kyc_core::types::Type;

#[derive(Clone, Debug)]
pub enum SymKind {
    Variable { type_: Option<Type>, is_mutable: bool, is_auto: bool },
    Constant(Type),
    Function(FunctionDecl),
    Class(ClassDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Contract(ContractDecl),
    TypeAlias(TypeAlias),
    Module(Vec<(String, SymKind)>),
    TypeParam,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymKind,
}

impl Symbol {
    pub fn new(name: String, kind: SymKind) -> Self {
        Self { name, kind }
    }

    pub fn new_var(name: String, type_: Option<Type>, is_mutable: bool) -> Self {
        Self { name, kind: SymKind::Variable { type_, is_mutable, is_auto: false } }
    }

    pub fn new_auto(name: String, type_: Option<Type>) -> Self {
        Self { name, kind: SymKind::Variable { type_, is_mutable: false, is_auto: true } }
    }
}

#[derive(Default)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    pub type_defs: HashMap<String, Type>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut st = Self::default();
        st.scopes.push(HashMap::new());
        st.register_builtins();
        st
    }

    fn register_builtins(&mut self) {
        let builtin_types = [
            ("i8", Type::I8), ("i16", Type::I16), ("i32", Type::I32), ("i64", Type::I64),
            ("u8", Type::U8), ("u16", Type::U16), ("u32", Type::U32), ("u64", Type::U64),
            ("f32", Type::F32), ("f64", Type::F64),
            ("bool", Type::Bool), ("char", Type::Char), ("str", Type::Str), ("void", Type::Void),
            ("ptr", Type::Ptr),
            ("Result", Type::Object(vec![
                ("disc".to_string(), Type::I32),
                ("payload".to_string(), Type::I64),
            ])),
        ];
        for (name, ty) in &builtin_types {
            self.type_defs.insert(name.to_string(), ty.clone());
        }
        // Runtime built-in functions (resolved by type_checker, registered here
        // to avoid "undefined symbol" errors from the scope resolver).
        let runtime_fns = [
            "print", "println", "print_err", "len", "input", "range",
            "json_parse", "json_stringify", "json_stringify_str", "serialize", "deserialize", "type",
            "ky_struct_to_json", "ky_json_to_struct",
            "ky_ptr_read_i32", "ky_ptr_read_ptr",
            "open", "read_str", "write_str", "close", "sleep", "now",
            "assert", "assert_eq", "assert_ne", "assert_str",
            "contains", "to_upper", "to_lower", "trim", "replace", "substr",
            "char_at", "is_digit", "is_alpha", "is_alnum", "is_whitespace", "is_upper", "is_lower",
            "ord",
            "error",
            "ceil", "floor", "round",
            "push", "list_new", "list_len", "list_get", "list_set", "list_pop", "list_push", "reserve",

            "ky_spawn_thread", "ky_join_thread", "ky_parallel_for", "ok", "some",
            "ky_channel_new", "ky_channel_send", "ky_channel_recv",
            "ky_channel_close", "ky_channel_len", "ky_channel_free",
            "ky_bytes_new", "ky_bytes_free", "ky_bytes_get", "ky_bytes_set",
            "ky_bytes_to_hex", "ky_bytes_from_hex", "ky_bytes_to_base64",
            "ky_str_builder_new", "ky_str_builder_append",
            "ky_str_builder_to_str", "ky_str_builder_free",
            "str_builder_new", "str_builder_append",
            "str_builder_to_str", "str_builder_free",
            "ky_dict_contains", "ky_dict_remove",
            "ky_fs_exists", "ky_fs_is_dir", "ky_fs_is_file",
            "ky_fs_size", "ky_fs_copy", "ky_fs_remove",
            "ky_fs_create_dir", "ky_fs_remove_dir", "ky_fs_rename",
            "ky_fs_read_to_string", "ky_fs_write_string", "ky_fs_list_dir",
            "ky_time_now_ms", "ky_time_now_us",
            "ky_set_new", "ky_set_free", "ky_set_add",
            "ky_set_contains", "ky_set_remove", "ky_set_len",
            "box",
        ];
        for &name in &runtime_fns {
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert(name.to_string(), Symbol::new(
                    name.to_string(),
                        SymKind::Function(FunctionDecl {
                            name: name.to_string(),
                            type_params: Vec::new(),
                            params: Vec::new(),
                            return_type: None,
                            is_async: false,
                            is_const: false,
                            is_static: false,
                            is_abstract: false,
                            is_extern: false,
                            is_test: false,
                            is_bench: false,
                            visibility: Visibility::Public,
                            body: None,
                            span: kyc_core::span::Span::dummy(),
                        }),
                ));
            }
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol);
            Ok(())
        } else {
            Err("no scope".to_string())
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    /// Return every class-name registered in the global (bottom) scope.
    /// Used by the visibility check to scan for private members.
    pub fn all_top_level_names(&self) -> Vec<String> {
        self.scopes
            .first()
            .map(|scope| scope.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn lookup_type(&self, name: &str) -> Option<Type> {
        if let Some(ty) = self.type_defs.get(name) {
            return Some(ty.clone());
        }
        if let Some(sym) = self.lookup(name) {
            match &sym.kind {
                SymKind::Class(c) => return Some(Type::Named(c.name.clone())),
                SymKind::Struct(s) => return Some(Type::Named(s.name.clone())),
                SymKind::Enum(e) => return Some(Type::Named(e.name.clone())),
                SymKind::TypeAlias(t) => return Some(Type::Named(t.name.clone())),
                SymKind::TypeParam => return Some(Type::TypeVar(0)),
                _ => {}
            }
        }
        None
    }
}