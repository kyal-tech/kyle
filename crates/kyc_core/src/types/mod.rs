// kyc_core::types — Type system representations for the semantic layer
//
// Reference: docs/04-type-system.md
// These are the *semantic* types used after type resolution,
// distinct from AstType which represents type syntax in the AST.

use std::fmt;
use crate::ast::AstType;

/// A resolved type in the KL type system.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Bool, Char, Str, Void,
    Named(String),
    TypeParam(String),
    Generic(String, Vec<Type>),
    TypeVar(usize),
    Ptr,
    Option(Box<Type>),
    Error(Box<Type>),
    Function(FunctionType),
    Array(Box<Type>, usize),
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Queue(Box<Type>),
    Stack(Box<Type>),
    Deque(Box<Type>),
    LinkedList(Box<Type>),
    Object(Vec<(String, Type)>),
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    Box_(Box<Type>),
    Chan(Box<Type>),
    Bytes,
}

impl Type {
    pub fn from_ast_type(ast: &AstType) -> Self {
        match ast {
            AstType::Primitive { name, .. } => Self::from_primitive_name(name),
            AstType::User { name, .. } => Self::from_user_type_name(name),
            AstType::Generic { name, args, .. } => Type::Generic(
                name.clone(),
                args.iter().map(|a| Type::from_ast_type(a)).collect(),
            ),
            AstType::Optional { inner, .. } => {
                Type::Option(Box::new(Type::from_ast_type(inner)))
            }
            AstType::Error { inner, .. } => {
                Type::Error(Box::new(Type::from_ast_type(inner)))
            }
            AstType::Dict { key, value, .. } => {
                Type::Dict(Box::new(Type::from_ast_type(key)), Box::new(Type::from_ast_type(value)))
            }
            AstType::FnPtr { params, return_, .. } => {
                let param_types = params.iter().map(|p| Type::from_ast_type(p)).collect();
                Type::Function(FunctionType {
                    is_async: false,
                    is_const: false,
                    params: param_types,
                    return_: Box::new(Type::from_ast_type(return_)),
                    fallible: false,
                })
            }
            AstType::Array { inner, size, .. } => {
                Type::Array(Box::new(Type::from_ast_type(inner)), *size)
            }
            AstType::Set { inner, .. } => Type::Set(Box::new(Type::from_ast_type(inner))),
            AstType::Queue { inner, .. } => Type::Queue(Box::new(Type::from_ast_type(inner))),
            AstType::Stack { inner, .. } => Type::Stack(Box::new(Type::from_ast_type(inner))),
            AstType::Deque { inner, .. } => Type::Deque(Box::new(Type::from_ast_type(inner))),
            AstType::LinkedList { inner, .. } => Type::LinkedList(Box::new(Type::from_ast_type(inner))),
            AstType::List { inner, .. } => Type::List(Box::new(Type::from_ast_type(inner))),
            AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => Type::from_ast_type(inner),
            AstType::Ptr { .. } => Type::Ptr,
            AstType::Slice { inner, .. } => Type::Slice(Box::new(Type::from_ast_type(inner))),
            AstType::Chan { inner, .. } => Type::Chan(Box::new(Type::from_ast_type(inner))),
            AstType::Bytes { .. } => Type::Bytes,
        }
    }

    fn from_primitive_name(name: &str) -> Self {
        match name {
            "i8" => Type::I8, "i16" => Type::I16,             "i32" | "int" => Type::I32, "i64" => Type::I64,
            "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32, "u64" => Type::U64,
            "f32" => Type::F32, "f64" => Type::F64,
            "bool" => Type::Bool, "char" => Type::Char, "str" => Type::Str,
            "void" => Type::Void,
            "ptr" => Type::Ptr,
            _ => Type::Named(name.to_string()),
        }
    }

    /// Converts a user-written type name to a Type.
    /// Handles the fact that the parser emits AstType::User{name} for primitives.
    fn from_user_type_name(name: &str) -> Self {
        match name {
            "i8" => Type::I8, "i16" => Type::I16,             "i32" | "int" => Type::I32, "i64" => Type::I64,
            "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32, "u64" => Type::U64,
            "f32" => Type::F32, "f64" => Type::F64,
            "bool" => Type::Bool, "char" => Type::Char, "str" => Type::Str,
            "void" => Type::Void,
            "ptr" => Type::Ptr,
            _ => Type::Named(name.to_string()),
        }
    }

    pub fn default_for_literal(lit: &crate::ast::Literal) -> Self {
        match lit {
            crate::ast::Literal::Integer(n) => {
                if *n > i32::MAX as i64 || *n < i32::MIN as i64 {
                    Type::I64
                } else {
                    Type::I32
                }
            }
            crate::ast::Literal::Float(_) => Type::F64,
            crate::ast::Literal::String(_) => Type::Str,
            crate::ast::Literal::Boolean(_) => Type::Bool,
            crate::ast::Literal::Char(_) => Type::Char,
            crate::ast::Literal::None => Type::Option(Box::new(Type::TypeVar(0))),
            crate::ast::Literal::Null => Type::Ptr,
        }
    }

    pub fn can_assign_to(&self, target: &Type) -> bool {
        if self == target { return true; }
        match (self, target) {
            (Type::Char, Type::I8 | Type::I16 | Type::I32 | Type::I64) => true,
            (Type::I8, Type::I16 | Type::I32 | Type::I64) => true,
            (Type::I16, Type::I32 | Type::I64) => true,
            (Type::I32, Type::I64) => true,
            (Type::U8, Type::U16 | Type::U32 | Type::U64) => true,
            (Type::U16, Type::U32 | Type::U64) => true,
            (Type::U32, Type::U64) => true,
            (Type::F32, Type::F64) => true,
            _ => false,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self,
            Type::I8 | Type::I16 | Type::I32 | Type::I64
            | Type::U8 | Type::U16 | Type::U32 | Type::U64
            | Type::F32 | Type::F64
            | Type::Char
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(self,
            Type::I8 | Type::I16 | Type::I32 | Type::I64
            | Type::U8 | Type::U16 | Type::U32 | Type::U64
        )
    }

    pub fn is_fallible(&self) -> bool {
        matches!(self, Type::Error(_))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::Str => write!(f, "str"),
            Type::Void => write!(f, "void"),
            Type::Ptr => write!(f, "ptr"),
            Type::Named(n) => write!(f, "{}", n),
            Type::TypeParam(n) => write!(f, "{}", n),
            Type::Generic(name, args) => {
                write!(f, "{}[", name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", a)?;
                }
                write!(f, "]")
            }
            Type::TypeVar(id) => write!(f, "?{}", id),
            Type::Function(ft) => {
                if ft.is_async { write!(f, "async ")?; }
                write!(f, "fn(")?;
                for (i, p) in ft.params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") {}", ft.return_)?;
                if ft.fallible { write!(f, "!")?; }
                Ok(())
            }
            Type::Error(inner) => write!(f, "{}!", inner),
            Type::Option(inner) => write!(f, "Option<{}>", inner),
            Type::Array(inner, size) => write!(f, "[{}; {}]", inner, size),
            Type::List(inner) => write!(f, "list<{}>", inner),
            Type::Dict(k, v) => write!(f, "dict<{}, {}>", k, v),
            Type::Set(inner) => write!(f, "Set<{}>", inner),
            Type::Queue(inner) => write!(f, "Queue<{}>", inner),
            Type::Stack(inner) => write!(f, "Stack<{}>", inner),
            Type::Deque(inner) => write!(f, "Deque<{}>", inner),
            Type::LinkedList(inner) => write!(f, "LinkedList<{}>", inner),
            Type::Object(fields) => {
                write!(f, "{{")?;
                for (i, (n, t)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", n, t)?;
                }
                write!(f, "}}")
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Slice(inner) => write!(f, "&[{}]", inner),
            Type::Box_(inner) => write!(f, "box<{}>", inner),
            Type::Chan(inner) => write!(f, "chan<{}>", inner),
            Type::Bytes => write!(f, "bytes"),
        }
    }
}

/// Represents a function signature in the type system.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionType {
    pub is_async: bool,
    pub is_const: bool,
    pub params: Vec<Type>,
    pub return_: Box<Type>,
    pub fallible: bool,
}

/// Overflow behavior for integer operations.
#[derive(Clone, Debug, PartialEq)]
pub enum OverflowBehavior {
    DebugPanicReleaseWrap,
    ExplicitWrap,
}
