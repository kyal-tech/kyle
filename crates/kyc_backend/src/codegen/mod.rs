use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::AnyValueEnum;
use inkwell::values::AsValueRef;
use inkwell::values::InstructionValue as InkwellInstructionValue;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use llvm_sys::core::{LLVMSetNSW, LLVMSetNUW};
use std::collections::{HashMap, HashSet};

use kyc_mir::mir::*;
use kyc_mir::ssa::{SsaFunction, SsaInst, SsaValueId};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    target_triple: Option<String>,
    fn_value_map: HashMap<String, inkwell::values::FunctionValue<'ctx>>,
    param_values: HashMap<usize, BasicValueEnum<'ctx>>,
    alloca_map: Vec<Option<PointerValue<'ctx>>>,
    alloca_types: HashMap<usize, BasicTypeEnum<'ctx>>,
    field_ptr_allocas: Vec<Option<PointerValue<'ctx>>>,
    field_ptr_types: HashMap<usize, BasicTypeEnum<'ctx>>,
    needs_main_wrapper: bool,
    pub is_freestanding: bool,
    /// Local IDs holding struct values via pointer (pass-by-reference semantics).
    /// Maps local_id → the original LLVM struct type (used for GEP/load).
    ref_param_struct_types: HashMap<usize, BasicTypeEnum<'ctx>>,
    /// TBAA metadata nodes: root, then per-type descriptors.
    tbaa_nodes: HashMap<String, inkwell::values::MetadataValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    /// Add `!range !{i32 0, i32 2}` metadata to an integer comparison result,
    /// indicating the value is always 0 or 1 (bool-like).
    fn add_bool_range(&self, int_val: inkwell::values::IntValue<'ctx>) {
        let range_kind = self.context.get_kind_id("range");
        let lo = self.context.i32_type().const_zero();
        let hi = self.context.i32_type().const_int(2, false);
        let md = self.context.metadata_node(&[lo.into(), hi.into()]);
        if let Ok(iv) = inkwell::values::InstructionValue::try_from(AnyValueEnum::IntValue(int_val)) {
            let _ = iv.set_metadata(md, range_kind);
        }
    }

    /// Add `!tbaa` metadata to a load or store instruction.
    fn add_tbaa(&self, inst: inkwell::values::InstructionValue<'ctx>, tbaa_node: inkwell::values::MetadataValue<'ctx>) {
        let tbaa_kind = self.context.get_kind_id("tbaa");
        let _ = inst.set_metadata(tbaa_node, tbaa_kind);
    }
    /// Load a local's value, always preferring the alloca for cross-block correctness.
    /// Falls back to last_value_map for values that weren't stored to an alloca.
    fn load_value(
        &self,
        id: usize,
        last_values: &HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if this is a field pointer: single-load (get the pointer, not the value at it)
        if id < self.field_ptr_allocas.len() && self.field_ptr_allocas[id].is_some() {
            if let Some(field_ptr_alloca) = self.field_ptr_allocas[id] {
                let gep = self.builder.build_load(
                    self.context.ptr_type(Default::default()),
                    field_ptr_alloca, "_fgepload"
                ).map_err(|e| format!("load_value fptr {}: {}", id, e))?;
                return Ok(gep);  // Return the POINTER, not the value at the pointer
            }
        }
        if let Some(Some(ptr)) = self.alloca_map.get(id) {
            if let Some(pointee_type) = self.alloca_types.get(&id) {
                let loaded = self.builder.build_load(*pointee_type, *ptr, "")
                    .map_err(|e| format!("load_value {}: {}", id, e))?;
                return Ok(loaded);
            }
        }
        if let Some(val) = last_values.get(&id) {
            return Ok(*val);
        }
        Ok(self.context.i32_type().const_zero().as_basic_value_enum())
    }

    /// Collect all MirValue::Local references from an instruction.
    fn collect_local_uses<F: FnMut(usize)>(&self, inst: &MirInst, callback: &mut F) {
        match inst {
            MirInst::Alloca { .. } => {}
            MirInst::Store { value, .. } => {
                if let MirValue::Local(id) = value { callback(*id); }
            }
            MirInst::Load { src, .. } => { callback(*src); }
            MirInst::BinaryOp { left, right, .. } => {
                if let MirValue::Local(id) = left { callback(*id); }
                if let MirValue::Local(id) = right { callback(*id); }
            }
            MirInst::UnaryOp { operand, .. } => {
                if let MirValue::Local(id) = operand { callback(*id); }
            }
            MirInst::Call { args, .. } => {
                for a in args { if let MirValue::Local(id) = a { callback(*id); } }
            }
            MirInst::PtrOffset { index, .. } => {
                if let MirValue::Local(id) = index { callback(*id); }
            }
            MirInst::PtrStore { index, value, .. } => {
                if let MirValue::Local(id) = index { callback(*id); }
                if let MirValue::Local(id) = value { callback(*id); }
            }
            MirInst::FieldPtr { .. } => {}
            MirInst::ArrayElemPtr { .. } => {}
            MirInst::Cast { value, .. } => {
                if let MirValue::Local(id) = value { callback(*id); }
            }
            MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                callback(*dest_ptr_local);
                callback(*src_alloca_local);
            }
            MirInst::FnAddr { .. } => {}
            MirInst::AddressOf { local_id, .. } => { callback(*local_id); }
            MirInst::CallIndirect { fn_ptr, args, .. } => {
                callback(*fn_ptr);
                for a in args { if let MirValue::Local(id) = a { callback(*id); } }
            }
            MirInst::AsyncSpawn { arg, .. } => {
                if let MirValue::Local(id) = arg { callback(*id); }
            }
            MirInst::AsyncAwait { handle, .. } => { callback(*handle); }
            MirInst::SliceMake { ptr, len, .. } => {
                if let MirValue::Local(id) = ptr { callback(*id); }
                if let MirValue::Local(id) = len { callback(*id); }
            }
        }
    }

    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module_name);
        // Set target triple and data layout for LLVM optimization passes
        let triple = inkwell::targets::TargetMachine::get_default_triple();

        module.set_triple(&triple);
        Self {
            context,
            builder,
            module,
            target_triple: None,
            fn_value_map: HashMap::new(),
            param_values: HashMap::new(),
            alloca_map: Vec::new(),
            alloca_types: HashMap::new(),
            field_ptr_allocas: Vec::new(),
            field_ptr_types: HashMap::new(),
            needs_main_wrapper: false,
            is_freestanding: false,
            ref_param_struct_types: HashMap::new(),
            tbaa_nodes: HashMap::new(),
        }
    }

    pub fn new_with_target(context: &'ctx Context, module_name: &str, target_triple: &str) -> Self {
        let is_freestanding = target_triple == "freestanding";
        let mut cg = Self::new(context, module_name);
        cg.target_triple = Some(target_triple.to_string());
        cg.is_freestanding = is_freestanding;
        // For freestanding, don't set host triple (use the native one)
        if !is_freestanding {
            let triple = inkwell::targets::TargetTriple::create(target_triple);
            cg.module.set_triple(&triple);
        }
        cg
    }

    /// Initialize TBAA metadata nodes for type-based alias analysis.
    fn init_tbaa(&mut self) {
        // Create access tags for struct-path TBAA (LLVM 18+).
        // Type descriptor: {name, parent, i64_offset}
        // Access tag: {type_descriptor, type_descriptor, 0}
        // The !tbaa metadata on loads/stores references the ACCESS TAG.
        let md_str = |s: &str| self.context.metadata_string(s);
        let i64_0 = || self.context.i64_type().const_int(0, false);

        let root = self.context.metadata_node(&[md_str("any level").into()]);
        let int_desc = self.context.metadata_node(&[md_str("int").into(), root.into(), i64_0().into()]);
        let float_desc = self.context.metadata_node(&[md_str("float").into(), root.into(), i64_0().into()]);
        let ptr_desc = self.context.metadata_node(&[md_str("ptr").into(), root.into(), i64_0().into()]);
        let list_desc = self.context.metadata_node(&[md_str("list").into(), root.into(), i64_0().into()]);
        let list_data_desc = self.context.metadata_node(&[md_str("list_data").into(), list_desc.into(), i64_0().into()]);
        let list_len_desc = self.context.metadata_node(&[md_str("list_len").into(), list_desc.into(), i64_0().into()]);

        // Access tags (what !tbaa references)
        let int_tag = self.context.metadata_node(&[int_desc.into(), int_desc.into(), i64_0().into()]);
        let float_tag = self.context.metadata_node(&[float_desc.into(), float_desc.into(), i64_0().into()]);
        let ptr_tag = self.context.metadata_node(&[ptr_desc.into(), ptr_desc.into(), i64_0().into()]);
        let list_data_tag = self.context.metadata_node(&[list_data_desc.into(), list_data_desc.into(), i64_0().into()]);
        let list_len_tag = self.context.metadata_node(&[list_len_desc.into(), list_len_desc.into(), i64_0().into()]);

        self.tbaa_nodes.insert("root".to_string(), root);
        self.tbaa_nodes.insert("int.desc".to_string(), int_desc);
        self.tbaa_nodes.insert("int".to_string(), int_tag);
        self.tbaa_nodes.insert("float".to_string(), float_tag);
        self.tbaa_nodes.insert("ptr".to_string(), ptr_tag);
        self.tbaa_nodes.insert("list_data".to_string(), list_data_tag);
        self.tbaa_nodes.insert("list_len".to_string(), list_len_tag);
    }

    fn tbaa_md(&self, name: &str) -> Option<inkwell::values::MetadataValue<'ctx>> {
        self.tbaa_nodes.get(name).copied()
    }

    /// Add struct-path TBAA metadata to a load instruction.
    fn tbaa_load(&self, val: inkwell::values::BasicValueEnum<'ctx>, node: &str) {
        if let Some(n) = self.tbaa_md(node) {
            if let Ok(inst) = inkwell::values::InstructionValue::try_from(inkwell::values::AnyValueEnum::from(val)) {
                self.add_tbaa(inst, n);
            }
        }
    }

    /// Add struct-path TBAA metadata to a store instruction.
    fn tbaa_store(&self, sv: impl Into<inkwell::values::InstructionValue<'ctx>>, node: &str) {
        if let Some(n) = self.tbaa_md(node) {
            self.add_tbaa(sv.into(), n);
        }
    }

    /// Get the TBAA access tag for a given MirType.
    fn tbaa_for_type(&self, ty: &MirType) -> Option<inkwell::values::MetadataValue<'ctx>> {
        match ty {
        MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64 | MirType::I1
        | MirType::U8 | MirType::U16 | MirType::U32 | MirType::U64
        | MirType::Bool | MirType::Char => self.tbaa_nodes.get("int").copied(),
            MirType::F32 | MirType::F64 => self.tbaa_nodes.get("float").copied(),
            MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Deque(_) | MirType::LinkedList(_) | MirType::Chan(_) | MirType::Bytes => self.tbaa_nodes.get("ptr").copied(),
            MirType::Str | MirType::Ptr(_) | MirType::Struct(_, _) | MirType::Array(_, _) | MirType::Slice(_) | MirType::Box(_) => self.tbaa_nodes.get("ptr").copied(),
            MirType::Void => None,
        }
    }

    /// Create an integer add with nsw+nuw flags for signed/unsigned overflow UB optimization.
    fn int_nsw_nuw_add(&self, lhs: inkwell::values::IntValue<'ctx>, rhs: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let val = self.builder.build_int_nsw_add(lhs, rhs, "").map_err(|e| format!("add: {}", e))?;
        unsafe { LLVMSetNUW(val.as_value_ref(), true.into()); }
        Ok(val)
    }

    /// Create an integer sub with nsw+nuw flags.
    fn int_nsw_nuw_sub(&self, lhs: inkwell::values::IntValue<'ctx>, rhs: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let val = self.builder.build_int_nsw_sub(lhs, rhs, "").map_err(|e| format!("sub: {}", e))?;
        unsafe { LLVMSetNUW(val.as_value_ref(), true.into()); }
        Ok(val)
    }

    /// Create an integer mul with nsw+nuw flags.
    fn int_nsw_nuw_mul(&self, lhs: inkwell::values::IntValue<'ctx>, rhs: inkwell::values::IntValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let val = self.builder.build_int_nsw_mul(lhs, rhs, "").map_err(|e| format!("mul: {}", e))?;
        unsafe { LLVMSetNUW(val.as_value_ref(), true.into()); }
        Ok(val)
    }

    /// Emit `llvm.lifetime.start` for an alloca to help LLVM reuse stack slots.
    fn emit_lifetime_start(&self, ptr: inkwell::values::PointerValue<'ctx>, size: i64) {
        if let Some(func) = self.module.get_function("llvm.lifetime.start.p0") {
            let size_val = self.context.i64_type().const_int(size as u64, false);
            if let Ok(call) = self.builder.build_call(func, &[size_val.into(), ptr.into()], "") {
                let _ = call.try_as_basic_value();
            }
        }
    }

    /// Emit `llvm.lifetime.end` for an alloca.
    fn emit_lifetime_end(&self, ptr: inkwell::values::PointerValue<'ctx>, size: i64) {
        if let Some(func) = self.module.get_function("llvm.lifetime.end.p0") {
            let size_val = self.context.i64_type().const_int(size as u64, false);
            if let Ok(call) = self.builder.build_call(func, &[size_val.into(), ptr.into()], "") {
                let _ = call.try_as_basic_value();
            }
        }
    }

    /// Map an LLVM BasicTypeEnum to a TBAA metadata node.
    fn tbaa_for_llvm_type(&self, t: &BasicTypeEnum<'ctx>) -> Option<inkwell::values::MetadataValue<'ctx>> {
        match t {
            BasicTypeEnum::IntType(_) => self.tbaa_nodes.get("int").copied(),
            BasicTypeEnum::FloatType(_) => self.tbaa_nodes.get("float").copied(),
            BasicTypeEnum::PointerType(_) | BasicTypeEnum::StructType(_) | BasicTypeEnum::ArrayType(_) => self.tbaa_nodes.get("ptr").copied(),
            _ => None,
        }
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }


}

pub mod ssa;
pub mod runtime;
pub mod function;
pub mod expr;
pub mod types;
