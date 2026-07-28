use std::fmt;
use kyc_core::ast::ParamMode;

/// A value in MIR — represents SSA-like values, locals, and constants.
#[derive(Clone, Debug, PartialEq)]
pub enum MirValue {
    Local(usize),
    Param(usize),
    Constant(MirConstant),
}

/// Constants in MIR.
#[derive(Clone, Debug, PartialEq)]
pub enum MirConstant {
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Void,
    Null,
}

/// MIR types — simplified subset for codegen.
#[derive(Clone, Debug, PartialEq)]
pub enum MirType {
    I1,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    Str,
    Void,
    Ptr(Box<MirType>),
    Array(Box<MirType>, usize),
    List(Box<MirType>),
    /// A struct type with name, field names, and field types.
    Struct(String, Vec<(String, MirType)>),
    /// A dictionary/map type with key and value types.
    Dict(Box<MirType>, Box<MirType>),
    Set(Box<MirType>),
    Queue(Box<MirType>),
    Stack(Box<MirType>),
    Deque(Box<MirType>),
    LinkedList(Box<MirType>),
    /// Slice view: `&[T]` — fat pointer {ptr, len}, Copy semantics.
    Slice(Box<MirType>),
    /// Heap-allocated box: `box<T>` — owned pointer to heap T, Move semantics.
    Box(Box<MirType>),
    /// Channel: `chan<T>` — concurrent message passing, Move semantics.
    Chan(Box<MirType>),
    /// Bytes: `bytes` — raw byte buffer, Move semantics.
    Bytes,
}

/// Binary operators in MIR.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MirBinaryOp {
    Add, Sub, Mul, Div, Rem,
    And, Or, Xor,
    Shl, Shr,
    Eq, Neq, Lt, Gt, Le, Ge,
}

/// Unary operators in MIR.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MirUnaryOp {
    Neg, Not, BitNot,
}

/// A single MIR instruction.
#[derive(Clone, Debug, PartialEq)]
pub enum MirInst {
    /// Allocate stack space for a local variable.
    Alloca { dest: usize, type_: MirType, name: String },
    /// Store a value into a local.
    Store { dest: usize, value: MirValue },
    /// Load a value from a local.
    Load { dest: usize, src: usize },
    /// Binary operation: dest = left op right.
    BinaryOp { dest: usize, op: MirBinaryOp, left: MirValue, right: MirValue },
    /// Unary operation: dest = op operand.
    UnaryOp { dest: usize, op: MirUnaryOp, operand: MirValue },
    /// Call a function: dest = func(args...).
    Call { dest: Option<usize>, name: String, args: Vec<MirValue> },
    /// Get pointer to array element: dest = ptr + index * sizeof(elem_type).
    /// elem_type determines the pointee type for subsequent Load instructions.
    PtrOffset { dest: usize, ptr: usize, index: MirValue, elem_type: Box<MirType> },
    /// Store value through computed pointer: ptr[index] = val
    PtrStore { dest: usize, ptr: usize, index: MirValue, value: MirValue },
    /// Get pointer to a struct field by index.
    FieldPtr { dest: usize, ptr: usize, field_index: usize, struct_type: Box<MirType> },
    /// Get pointer to an array element by index: dest = GEP(ptr, 0, index)
    ArrayElemPtr { dest: usize, ptr: usize, index: MirValue, arr_type: Box<MirType>, elem_type: Box<MirType> },
    /// Cast between types.
    Cast { dest: usize, value: MirValue, to_type: MirType },
    /// Copy struct value from a local alloca to a heap pointer (i64).
    Memcpy { dest_ptr_local: usize, src_alloca_local: usize, struct_type: Box<MirType> },
    /// Load address of a named function into a local (for closures).
    FnAddr { dest: usize, name: String },
    /// Get the address (alloca pointer) of a local variable.
    AddressOf { dest: usize, local_id: usize },
    /// Call through a function pointer.
    CallIndirect { dest: Option<usize>, fn_ptr: usize, ret_type: MirType, param_types: Vec<MirType>, args: Vec<MirValue> },
    /// Create a slice struct {ptr, len} from a pointer and length.
    SliceMake { dest: usize, ptr: MirValue, len: MirValue, elem_type: Box<MirType> },
    /// Spawn an async function on a thread: dest = kl_spawn_thread(func_name, arg).
    AsyncSpawn { dest: usize, function_name: String, arg: MirValue },
    /// Await (join) an async thread handle: dest = kl_join_thread(handle).
    AsyncAwait { dest: usize, handle: usize, return_type: MirType },
}

/// How a basic block ends.
#[derive(Clone, Debug, PartialEq)]
pub enum MirTerminator {
    /// Return a value (or void).
    Return(MirValue),
    /// Unconditional branch to another block.
    Br(String),
    /// Conditional branch: if cond, go to true_block, else false_block.
    CondBr { cond: MirValue, true_block: String, false_block: String },
    /// Unreachable terminator.
    Unreachable,
}

/// A basic block: label, instructions, terminator.
#[derive(Clone, Debug, PartialEq)]
pub struct MirBasicBlock {
    pub label: String,
    pub insts: Vec<MirInst>,
    pub terminator: MirTerminator,
}

impl MirBasicBlock {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            insts: Vec::new(),
            terminator: MirTerminator::Unreachable,
        }
    }
}

/// An MIR function.
#[derive(Clone, Debug, PartialEq)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<MirType>,
    pub param_modes: Vec<ParamMode>,
    pub return_type: MirType,
    pub is_fallible: bool,
    pub is_const: bool,
    pub basic_blocks: Vec<MirBasicBlock>,
    pub local_count: usize,
}

impl MirFunction {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: Vec::new(),
            param_modes: Vec::new(),
            return_type: MirType::Void,
            is_fallible: false,
            is_const: false,
            basic_blocks: Vec::new(),
            local_count: 0,
        }
    }
}

/// Returns true if the MirType is a Move type (heap-allocated, ownership-transfer semantics).
/// Return true if the type is unsigned integer.
pub fn is_unsigned_type(t: &MirType) -> bool {
    matches!(t, MirType::U8 | MirType::U16 | MirType::U32 | MirType::U64)
}

/// Return true if the type is move-semantics (ownership transfer).
pub fn is_move_type(t: &MirType) -> bool {
    match t {
        MirType::Str => true,
        MirType::List(_) => true,
        MirType::Dict(_, _) => true,
        MirType::Set(_) => true,
        MirType::Queue(_) | MirType::Stack(_) | MirType::Deque(_) | MirType::LinkedList(_) => true,
        MirType::Box(_) => true,
        MirType::Chan(_) | MirType::Bytes => true,
        // Array is NOT heap-allocated — value type on stack. No ky_free.
        // Struct is NOT heap-allocated — it's a value type on the stack.
        // Excluding it from Move prevents kl_free of stack addresses (crash).
        MirType::Struct(_, _) => false,
        _ => false,
    }
}

/// Top-level MIR module.
#[derive(Clone, Debug, PartialEq)]
pub struct MirModule {
    pub functions: Vec<MirFunction>,
    pub globals: Vec<(String, MirType, MirConstant)>,
    pub links: Vec<String>,
}

impl MirModule {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            globals: Vec::new(),
            links: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for MirType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirType::I1 => write!(f, "i1"),
            MirType::I8 => write!(f, "i8"),
            MirType::I16 => write!(f, "i16"),
            MirType::I32 => write!(f, "i32"),
            MirType::I64 => write!(f, "i64"),
            MirType::U8 => write!(f, "u8"),
            MirType::U16 => write!(f, "u16"),
            MirType::U32 => write!(f, "u32"),
            MirType::U64 => write!(f, "u64"),
            MirType::F32 => write!(f, "f32"),
            MirType::F64 => write!(f, "f64"),
            MirType::Bool => write!(f, "bool"),
            MirType::Char => write!(f, "char"),
            MirType::Str => write!(f, "str"),
            MirType::Void => write!(f, "void"),
            MirType::Ptr(inner) => write!(f, "{}*", inner),
            MirType::Array(inner, size) => write!(f, "[{}; {}]", inner, size),
            MirType::List(inner) => write!(f, "list<{}>", inner),
            MirType::Struct(name, fields) => {
                write!(f, "{} {{ ", name)?;
                for (i, (fname, t)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", fname, t)?;
                }
                write!(f, " }}")
            }
            MirType::Dict(key, val) => write!(f, "dict<{}, {}>", key, val),
            MirType::Set(inner) => write!(f, "set<{}>", inner),
            MirType::Queue(inner) => write!(f, "queue<{}>", inner),
            MirType::Stack(inner) => write!(f, "stack<{}>", inner),
            MirType::Deque(inner) => write!(f, "deque<{}>", inner),
            MirType::LinkedList(inner) => write!(f, "linked_list<{}>", inner),
            MirType::Slice(inner) => write!(f, "&[{}]", inner),
            MirType::Box(inner) => write!(f, "box<{}>", inner),
            MirType::Chan(inner) => write!(f, "chan<{}>", inner),
            MirType::Bytes => write!(f, "bytes"),
        }
    }
}

impl fmt::Display for MirValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirValue::Local(id) => write!(f, "%{}", id),
            MirValue::Param(id) => write!(f, "param{}", id),
            MirValue::Constant(c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Display for MirConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirConstant::I32(n) => write!(f, "{}", n),
            MirConstant::I64(n) => write!(f, "{}", n),
            MirConstant::F64(n) => write!(f, "{}", n),
            MirConstant::Bool(b) => write!(f, "{}", b),
            MirConstant::String(s) => write!(f, "\"{}\"", s),
            MirConstant::Void => write!(f, "void"),
            MirConstant::Null => write!(f, "null"),
        }
    }
}

impl fmt::Display for MirBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirBinaryOp::Add => write!(f, "add"),
            MirBinaryOp::Sub => write!(f, "sub"),
            MirBinaryOp::Mul => write!(f, "mul"),
            MirBinaryOp::Div => write!(f, "div"),
            MirBinaryOp::Rem => write!(f, "rem"),
            MirBinaryOp::And => write!(f, "and"),
            MirBinaryOp::Or => write!(f, "or"),
            MirBinaryOp::Xor => write!(f, "xor"),
            MirBinaryOp::Shl => write!(f, "shl"),
            MirBinaryOp::Shr => write!(f, "shr"),
            MirBinaryOp::Eq => write!(f, "eq"),
            MirBinaryOp::Neq => write!(f, "neq"),
            MirBinaryOp::Lt => write!(f, "lt"),
            MirBinaryOp::Gt => write!(f, "gt"),
            MirBinaryOp::Le => write!(f, "le"),
            MirBinaryOp::Ge => write!(f, "ge"),
        }
    }
}

impl fmt::Display for MirUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirUnaryOp::Neg => write!(f, "neg"),
            MirUnaryOp::Not => write!(f, "not"),
            MirUnaryOp::BitNot => write!(f, "bitnot"),
        }
    }
}

impl fmt::Display for MirInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirInst::Alloca { dest, type_, name } => {
                write!(f, "  %{} = alloca {}, \"{}\"", dest, type_, name)
            }
            MirInst::Store { dest, value } => {
                write!(f, "  store %{} <- {}", dest, value)
            }
            MirInst::Load { dest, src } => {
                write!(f, "  %{} = load %{}", dest, src)
            }
            MirInst::BinaryOp { dest, op, left, right } => {
                write!(f, "  %{} = {} {}, {}", dest, op, left, right)
            }
            MirInst::UnaryOp { dest, op, operand } => {
                write!(f, "  %{} = {} {}", dest, op, operand)
            }
            MirInst::Call { dest, name, args } => {
                if let Some(d) = dest {
                    write!(f, "  %{} = call {}({})", d, name, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))
                } else {
                    write!(f, "  call {}({})", name, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))
                }
            }
            MirInst::PtrOffset { dest, ptr, index, .. } => {
                write!(f, "  {} = ptr_offset {}[{}]", dest, ptr, index)
            }
            MirInst::PtrStore { dest, ptr, index, value } => {
                write!(f, "  {} = ptr_store {}[{}] <- {}", dest, ptr, index, value)
            }
            MirInst::FieldPtr { dest, ptr, field_index, .. } => {
                write!(f, "  %{} = field_ptr %{}, field {}", dest, ptr, field_index)
            }
            MirInst::ArrayElemPtr { dest, ptr, index, .. } => {
                write!(f, "  %{} = elem_ptr %{}[{}]", dest, ptr, index)
            }
            MirInst::Cast { dest, value, to_type } => {
                write!(f, "  %{} = cast {} to {}", dest, value, to_type)
            }
            MirInst::Memcpy { dest_ptr_local, src_alloca_local, struct_type } => {
                write!(f, "  memcpy %{} <- %{}, type {}", dest_ptr_local, src_alloca_local, struct_type)
            }
            MirInst::FnAddr { dest, name } => {
                write!(f, "  %{} = fn_addr {}", dest, name)
            }
            MirInst::AddressOf { dest, local_id } => {
                write!(f, "  %{} = addr_of %{}", dest, local_id)
            }
            MirInst::CallIndirect { dest, fn_ptr, ret_type: _, param_types: _, args } => {
                if let Some(d) = dest {
                    write!(f, "  %{} = call_indirect %{}({})", d, fn_ptr, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))
                } else {
                    write!(f, "  call_indirect %{}({})", fn_ptr, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))
                }
            }
            MirInst::AsyncSpawn { dest, function_name, arg } => {
                write!(f, "  %{} = async_spawn {} ({})", dest, function_name, arg)
            }
            MirInst::AsyncAwait { dest, handle, return_type: _ } => {
                write!(f, "  %{} = async_await %{}", dest, handle)
            }
            MirInst::SliceMake { dest, ptr, len, elem_type } => {
                write!(f, "  %{} = slice_make({}, {}, {})", dest, ptr, len, elem_type)
            }
        }
    }
}

impl fmt::Display for MirTerminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MirTerminator::Return(v) => write!(f, "  ret {}", v),
            MirTerminator::Br(label) => write!(f, "  br {}", label),
            MirTerminator::CondBr { cond, true_block, false_block } => {
                write!(f, "  br {}, {}, {}", cond, true_block, false_block)
            }
            MirTerminator::Unreachable => write!(f, "  unreachable"),
        }
    }
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_const { write!(f, "const ")?; }
        write!(f, "fn {}({}) -> {}", self.name,
            self.params.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
            self.return_type)?;
        if self.is_fallible { write!(f, "!")?; }
        writeln!(f, ":")?;
        for bb in &self.basic_blocks {
            writeln!(f, "{}:", bb.label)?;
            for inst in &bb.insts {
                writeln!(f, "{}", inst)?;
            }
            writeln!(f, "{}", bb.terminator)?;
        }
        Ok(())
    }
}

impl fmt::Display for MirModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, type_, init) in &self.globals {
            writeln!(f, "global {}: {} = {}", name, type_, init)?;
        }
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}
