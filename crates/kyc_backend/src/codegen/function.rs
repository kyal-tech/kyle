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

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn declare_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let ret_type = self.llvm_type(&func.return_type);
        let ptr_ty = self.context.ptr_type(Default::default());
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = func.params
            .iter()
            .map(|p| {
                if matches!(p, MirType::Struct(_, _) | MirType::Slice(_)) {
                    ptr_ty.into()
                } else {
                    self.llvm_type(p).into()
                }
            })
            .collect();

        // If main has a list parameter (e.g. args: [str]), rename to kyle_main
        // and generate a C-compatible main(i32, ptr) wrapper later.
        // In freestanding mode, keep function names as-is (no main wrapper).
        let fn_name = if self.is_freestanding {
            &func.name
        } else if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            self.needs_main_wrapper = true;
            "kyle_main"
        } else {
            &func.name
        };

        let fn_type = ret_type.fn_type(&param_types, false);
        // If function was already declared (e.g. from prelude extern fn),
        // reuse the existing declaration so the body fills it in.
        let fn_value = if let Some(existing) = self.module.get_function(fn_name) {
            existing
        } else {
            self.module.add_function(fn_name, fn_type, None)
        };
        // Parameter attributes: noundef on all, noalias on pointer types
        let noundef_kind = Attribute::get_named_enum_kind_id("noundef");
        let noalias_kind = Attribute::get_named_enum_kind_id("noalias");
        for (i, ptype) in func.params.iter().enumerate() {
            let idx = i as u32;
            if noundef_kind > 0 {
                let attr = self.context.create_enum_attribute(noundef_kind, 0);
                fn_value.add_attribute(AttributeLoc::Param(idx), attr);
            }
            if noalias_kind > 0 {
                if matches!(ptype, MirType::Struct(_, _) | MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Ptr(_) | MirType::Box(_)) {
                    let attr = self.context.create_enum_attribute(noalias_kind, 0);
                    fn_value.add_attribute(AttributeLoc::Param(idx), attr);
                }
            }
        }
        self.fn_value_map.insert(fn_name.to_string(), fn_value);
        Ok(())
    }

    pub(crate) fn compile_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let fn_name = if self.is_freestanding {
            &func.name
        } else if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            "kyle_main"
        } else {
            &func.name
        };
        let fn_value = self.fn_value_map.get(fn_name)
            .ok_or_else(|| format!("Function {} not declared", fn_name))?;

        self.alloca_types.clear();
        self.ref_param_struct_types.clear();
        self.field_ptr_allocas.clear();
        self.field_ptr_types.clear();
        let ptr_ty = self.context.ptr_type(Default::default());
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Alloca { dest, type_, .. } = inst {
                    let llvm_ty = self.llvm_type(type_);
                    let actual_ty = if let MirType::Ptr(_) = type_ {
                        ptr_ty.as_basic_type_enum()
                    } else {
                        llvm_ty
                    };
                    self.alloca_types.entry(*dest).or_insert(actual_ty);
                }
            }
        }

        // Phase 17.5: identify single-block locals to skip allocas for
        // A local is eligble if:
        //   1. Simple type (i32/i64/f32/f64/bool — not struct/string/list)
        //   2. Only used in FieldPtr/Memcpy/PtrOffset → needs alloca (always keep)
        //   3. Defined and used within a single basic block (no cross-block flow)
        let simple_types: [MirType; 9] = [MirType::I32, MirType::I64, MirType::F32, MirType::F64, MirType::Bool, MirType::U8, MirType::U16, MirType::U32, MirType::U64];
        let mut escaping: HashSet<usize> = HashSet::new();
        let mut def_blocks: HashMap<usize, usize> = HashMap::new();
        let mut use_blocks: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (bi, bb) in func.basic_blocks.iter().enumerate() {
            for inst in &bb.insts {
                match inst {
                    MirInst::Alloca { dest, type_, .. } => {
                        if !simple_types.contains(type_) {
                            escaping.insert(*dest);
                        }
                    }
                    MirInst::FieldPtr { ptr, .. } => { escaping.insert(*ptr); }
                    MirInst::ArrayElemPtr { ptr, .. } => { escaping.insert(*ptr); }
                    MirInst::PtrOffset { dest, ptr, .. } => { 
                        escaping.insert(*ptr);
                        escaping.insert(*dest);
                    }
                    MirInst::PtrStore { ptr, .. } => { escaping.insert(*ptr); }
                    MirInst::Memcpy { dest_ptr_local, .. } => { escaping.insert(*dest_ptr_local); }
                    MirInst::SliceMake { dest, .. } => { escaping.insert(*dest); }
                    _ => {}
                }
                // Track definitions (any instruction writing to dest)
                let dest_opt = match inst {
                    MirInst::Store { dest, .. } => Some(*dest),
                    MirInst::Load { dest, .. } => Some(*dest),
                    MirInst::BinaryOp { dest, .. } => Some(*dest),
                    MirInst::UnaryOp { dest, .. } => Some(*dest),
                    MirInst::Cast { dest, .. } => Some(*dest),
                    MirInst::Call { dest, .. } => dest.map(|d| d),
                    MirInst::CallIndirect { dest, .. } => dest.map(|d| d),
                    MirInst::PtrOffset { dest, .. } => Some(*dest),
                    MirInst::PtrStore { dest, .. } => Some(*dest),
                    MirInst::FieldPtr { dest, .. } => Some(*dest),
                    MirInst::ArrayElemPtr { dest, .. } => Some(*dest),
                    MirInst::FnAddr { dest, .. } => Some(*dest),
                    MirInst::AddressOf { dest, .. } => Some(*dest),
                    MirInst::AsyncSpawn { dest, .. } => Some(*dest),
                    MirInst::AsyncAwait { dest, .. } => Some(*dest),
                    MirInst::SliceMake { dest, .. } => Some(*dest),
                    _ => None,
                };
                if let Some(d) = dest_opt {
                    def_blocks.entry(d).or_insert(bi);
                }
                // Track uses (MirValue::Local references)
                self.collect_local_uses(inst, &mut |local_id| {
                    use_blocks.entry(local_id).or_default().insert(bi);
                });
            }
            // Track uses in terminator
            match &bb.terminator {
                MirTerminator::Return(val) => {
                    if let MirValue::Local(lid) = val {
                        use_blocks.entry(*lid).or_default().insert(bi);
                    }
                }
                MirTerminator::CondBr { cond, .. } => {
                    if let MirValue::Local(lid) = cond {
                        use_blocks.entry(*lid).or_default().insert(bi);
                    }
                }
                _ => {}
            }
        }
        let mut skip_allocas: HashSet<usize> = HashSet::new();
        for (local, db) in &def_blocks {
            if escaping.contains(local) { continue; }
            let ub = use_blocks.get(local);
            // Single-block: defined and all uses are in the same block
            let is_single_block = ub.map_or(true, |blocks| blocks.len() == 1 && blocks.contains(db));
            // Also skip locals with no uses (defined but never read — dead code)
            let has_no_uses = ub.map_or(true, |blocks| blocks.is_empty());
            if is_single_block || has_no_uses {
                skip_allocas.insert(*local);
            }
        }
        // Params always need allocas (value flows from entry to other blocks)
        for (bi, bb) in func.basic_blocks.iter().enumerate() {
            for inst in &bb.insts {
                if let MirInst::Store { dest, value: MirValue::Param(_) } = inst {
                    skip_allocas.remove(dest);
                }
            }
        }

        // Pre-scan for struct function params: change their alloca type to `ptr`
        // so they receive a pointer to the caller's struct (pass-by-reference ABI).
        for bb in &func.basic_blocks {
            for inst in &bb.insts {
                if let MirInst::Store { dest, value: MirValue::Param(param_idx) } = inst {
                    if let Some(&llvm_type) = self.alloca_types.get(dest) {
                        let orig_type = if matches!(llvm_type, BasicTypeEnum::StructType(_)) {
                            llvm_type
                        } else if llvm_type.is_pointer_type() {
                            self.llvm_type(&func.params[*param_idx])
                        } else {
                            continue;
                        };
                        self.alloca_types.insert(*dest, ptr_ty.as_basic_type_enum());
                        self.ref_param_struct_types.insert(*dest, orig_type);
                    }
                }
            }
        }

        let mut block_map: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();
        for bb in &func.basic_blocks {
            let llvm_bb = self.context.append_basic_block(*fn_value, &bb.label);
            block_map.insert(bb.label.clone(), llvm_bb);
        }

        self.alloca_map.clear();

        if let Some(entry_bb) = func.basic_blocks.first() {
            if let Some(&llvm_entry) = block_map.get(&entry_bb.label) {
                self.builder.position_at_end(llvm_entry);

                for (dest, llvm_type) in &self.alloca_types {
                    if skip_allocas.contains(dest) {
                        // Ensure the vec is sized correctly even without an alloca
                        while self.alloca_map.len() <= *dest {
                            self.alloca_map.push(None);
                        }
                        continue;
                    }
                    while self.alloca_map.len() <= *dest {
                        self.alloca_map.push(None);
                    }
                    let ptr = self.builder.build_alloca(*llvm_type, "")
                        .map_err(|e| format!("alloca {}: {}", dest, e))?;
                    if let Ok(iv) = inkwell::values::InstructionValue::try_from(AnyValueEnum::PointerValue(ptr)) {
                        let _ = iv.set_alignment(8);
                    }
                    // // self.emit_lifetime_start(ptr, -1); // DEBUG: disabled for mem2reg test // DEBUG: disabled for mem2reg test
                    self.alloca_map[*dest] = Some(ptr);
                }

                let ptr_ty = self.context.ptr_type(Default::default());
                for bb in &func.basic_blocks {
                    for inst in &bb.insts {
                        if let MirInst::FieldPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest {
                                self.field_ptr_allocas.push(None);
                            }
                            if self.field_ptr_allocas[*dest].is_none() {
                                let alloca = self.builder.build_alloca(ptr_ty, "_fgep")
                                    .map_err(|e| format!("fgep alloca {}: {}", dest, e))?;
                                self.field_ptr_allocas[*dest] = Some(alloca);
                            }
                        }
                        if let MirInst::ArrayElemPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest {
                                self.field_ptr_allocas.push(None);
                            }
                            if self.field_ptr_allocas[*dest].is_none() {
                                let alloca = self.builder.build_alloca(ptr_ty, "_aelem")
                                    .map_err(|e| format!("aelem alloca {}: {}", dest, e))?;
                                self.field_ptr_allocas[*dest] = Some(alloca);
                            }
                        }
                        if let MirInst::PtrOffset { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest {
                                self.field_ptr_allocas.push(None);
                            }
                            if self.field_ptr_allocas[*dest].is_none() {
                                let alloca = self.builder.build_alloca(ptr_ty, "_pgep")
                                    .map_err(|e| format!("pgep alloca {}: {}", dest, e))?;
                                self.field_ptr_allocas[*dest] = Some(alloca);
                            }
                        }
                    }
                }

                for (i, param) in fn_value.get_param_iter().enumerate() {
                    self.param_values.insert(i, param);
                }
            }
        }

        let mut last_value_map: HashMap<usize, BasicValueEnum<'ctx>> = HashMap::new();

        for bb in &func.basic_blocks {
            if let Some(&llvm_bb) = block_map.get(&bb.label) {
                self.builder.position_at_end(llvm_bb);

                for inst in &bb.insts {
                    match inst {
                        MirInst::Alloca { .. } => {}
                        MirInst::Store { dest, value } => {
                            let val = self.value_to_llvm(value, &last_value_map)?;
                            // Check if this is a store to a field pointer
                            if *dest < self.field_ptr_allocas.len() && self.field_ptr_allocas[*dest].is_some() {
                                if let Some(field_ptr_alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                    let gep = self.builder.build_load(
                                        self.context.ptr_type(Default::default()),
                                        field_ptr_alloca, "_fgepload"
                                    ).map_err(|e| format!("fptr store load: {}", e))?;
                                    // Auto-cast value to match field type
                                    let casted = if let Some(pointee_type) = self.alloca_types.get(dest) {
                                        self.cast_to_type(val, *pointee_type)?
                                    } else {
                                        val
                                    };
                                    let siv = self.builder.build_store(gep.into_pointer_value(), casted)
                                        .map_err(|e| format!("fptr store: {}", e))?;
                                    if let Some(tbaa_node) = self.alloca_types.get(dest).and_then(|t| self.tbaa_for_llvm_type(t)) {
                                        self.add_tbaa(siv, tbaa_node);
                                    }
                                }
                            } else if let Some(ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                // Auto-cast value to match alloca type for regular stores
                                let casted = if let Some(pointee_type) = self.alloca_types.get(dest) {
                                    self.cast_to_type(val, *pointee_type)?
                                } else {
                                    val
                                };
                                let siv = self.builder.build_store(ptr, casted)
                                    .map_err(|e| format!("store: {}", e))?;
                                if let Some(tbaa_node) = self.alloca_types.get(dest).and_then(|t| self.tbaa_for_llvm_type(t)) {
                                    self.add_tbaa(siv, tbaa_node);
                                }
                            }
                            last_value_map.insert(*dest, val);
                        }
                        MirInst::Load { dest, src } => {
                            // Check if this is a load from a field pointer
                            if *src < self.field_ptr_allocas.len() && self.field_ptr_allocas[*src].is_some() {
                                if let Some(field_ptr_alloca) = self.field_ptr_allocas.get(*src).and_then(|p| *p) {
                                    let gep = self.builder.build_load(
                                        self.context.ptr_type(Default::default()), 
                                        field_ptr_alloca, "_fgepload"
                                    ).map_err(|e| format!("fptr load: {}", e))?;
                                    let field_type = self.field_ptr_types.get(src).or_else(|| self.alloca_types.get(src));
                                    if let Some(pointee_type) = field_type {
                                        let loaded = self.builder.build_load(*pointee_type, gep.into_pointer_value(), "")
                                            .map_err(|e| format!("field load: {}", e))?;
                                        if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                            self.builder.build_store(dest_ptr, loaded)
                                                .map_err(|e| format!("field load-store: {}", e))?;
                                        }
                                        last_value_map.insert(*dest, loaded);
                                    }
                                }
                            } else if let Some(ptr) = self.alloca_map.get(*src).and_then(|p| *p) {
                                if let Some(pointee_type) = self.alloca_types.get(src) {
                                    let loaded = if self.ref_param_struct_types.contains_key(src) {
                                        // Ref param: alloca stores a pointer to the struct.
                                        // Load the pointer from alloca, then load the struct from that pointer.
                                        let struct_ptr = self.builder.build_load(
                                            *pointee_type, ptr, "_ref_load"
                                        ).map_err(|e| format!("ref load: {}", e))?;
                                        if let Some(&orig_struct_type) = self.ref_param_struct_types.get(src) {
                                            self.builder.build_load(
                                                orig_struct_type, struct_ptr.into_pointer_value(), "_ref_val"
                                            ).map_err(|e| format!("ref load val: {}", e))?
                                        } else {
                                            struct_ptr
                                        }
                                    } else {
                                        self.builder.build_load(*pointee_type, ptr, "")
                                            .map_err(|e| format!("load: {}", e))?
                                    };
                                    // Add TBAA metadata to load instruction
                                    if let Some(src_ty) = self.alloca_types.get(src) {
                                        if let Some(tbaa_node) = self.tbaa_for_llvm_type(src_ty) {
                                            if let Ok(liv) = inkwell::values::InstructionValue::try_from(AnyValueEnum::from(loaded.clone())) {
                                                self.add_tbaa(liv, tbaa_node);
                                            }
                                        }
                                    }
                                    // Store to dest alloca for cross-block reads
                                    if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(dest_ptr, loaded)
                                            .map_err(|e| format!("load-store: {}", e))?;
                                    }
                                    last_value_map.insert(*dest, loaded);
                                }
                            } else {
                                // Promoted temp (no alloca): read from last_value_map
                                if let Some(&val) = last_value_map.get(src) {
                                    last_value_map.insert(*dest, val);
                                    if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(dest_ptr, val)
                                            .map_err(|e| format!("promoted-load-store: {}", e))?;
                                    }
                                }
                            }
                        }
                        MirInst::BinaryOp { dest, op, left, right } => {
                            let l = self.value_to_llvm(left, &last_value_map)?;
                            let r = self.value_to_llvm(right, &last_value_map)?;

                            // Check if either operand is float (handles comparison ops whose result is I32)
                            let l_is_float = matches!(l, BasicValueEnum::FloatValue(_));
                            let r_is_float = matches!(r, BasicValueEnum::FloatValue(_));
                            let any_float = l_is_float || r_is_float;
                            // Also check if destination is float type (for arithmetic)
                            let dest_type = self.alloca_types.get(dest).or_else(|| self.field_ptr_types.get(dest));
                            let is_float = dest_type.map_or(false, |t| matches!(t, BasicTypeEnum::FloatType(_)));

                            let result = if any_float || is_float {
                                let lf = self.to_float_value(l);
                                let rf = self.to_float_value(r);
                                match op {
                                    MirBinaryOp::Add => self.builder.build_float_add(lf, rf, "")
                                        .map_err(|e| format!("fadd: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Sub => self.builder.build_float_sub(lf, rf, "")
                                        .map_err(|e| format!("fsub: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Mul => self.builder.build_float_mul(lf, rf, "")
                                        .map_err(|e| format!("fmul: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Div => self.builder.build_float_div(lf, rf, "")
                                        .map_err(|e| format!("fdiv: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Rem => self.builder.build_float_rem(lf, rf, "")
                                        .map_err(|e| format!("frem: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Eq => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OEQ, lf, rf, "")
                                            .map_err(|e| format!("feq: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("feq-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Neq => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::ONE, lf, rf, "")
                                            .map_err(|e| format!("fne: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fne-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Lt => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OLT, lf, rf, "")
                                            .map_err(|e| format!("flt: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("flt-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Gt => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OGT, lf, rf, "")
                                            .map_err(|e| format!("fgt: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fgt-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Le => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OLE, lf, rf, "")
                                            .map_err(|e| format!("fle: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fle-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    MirBinaryOp::Ge => {
                                        let cmp = self.builder.build_float_compare(
                                            inkwell::FloatPredicate::OGE, lf, rf, "")
                                            .map_err(|e| format!("fge: {}", e))?;
                                        self.builder.build_int_z_extend(cmp,
                                            self.context.i32_type(), "")
                                            .map_err(|e| format!("fge-ext: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                    _ => {
                                        // Fallback: use int op for bitwise etc.
                                        let li = self.to_int_value(l);
                                        let ri = self.to_int_value(r);
                                        self.builder.build_int_add(li, ri, "")
                                            .map_err(|e| format!("int_add: {}", e))?
                                            .as_basic_value_enum()
                                    },
                                }
                            } else {
                                let li = self.to_int_value(l);
                                let ri = self.to_int_value(r);

                                 match op {
                                    MirBinaryOp::Add => self.int_nsw_nuw_add(li, ri).map_err(|e| format!("iadd: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Sub => self.int_nsw_nuw_sub(li, ri).map_err(|e| format!("isub: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Mul => self.int_nsw_nuw_mul(li, ri).map_err(|e| format!("imul: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Div => self.builder.build_int_signed_div(li, ri, "")
                                        .map_err(|e| format!("div: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Rem => self.builder.build_int_signed_rem(li, ri, "")
                                        .map_err(|e| format!("rem: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::And => self.builder.build_and(li, ri, "")
                                        .map_err(|e| format!("and: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Or => self.builder.build_or(li, ri, "")
                                        .map_err(|e| format!("or: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Xor => self.builder.build_xor(li, ri, "")
                                        .map_err(|e| format!("xor: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Shl => self.builder.build_left_shift(li, ri, "")
                                        .map_err(|e| format!("shl: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Shr => self.builder.build_right_shift(li, ri, true, "")
                                        .map_err(|e| format!("shr: {}", e))?
                                        .as_basic_value_enum(),
                                    MirBinaryOp::Eq => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::EQ, li, ri, "")
                                            .map_err(|e| format!("eq: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("eqz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Neq => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::NE, li, ri, "")
                                            .map_err(|e| format!("neq: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("nqz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Lt => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SLT, li, ri, "")
                                            .map_err(|e| format!("lt: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("ltz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Gt => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SGT, li, ri, "")
                                            .map_err(|e| format!("gt: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("gtz: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Le => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SLE, li, ri, "")
                                            .map_err(|e| format!("le: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("lez: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    MirBinaryOp::Ge => {
                                        let cmp = self.builder.build_int_compare(IntPredicate::SGE, li, ri, "")
                                            .map_err(|e| format!("ge: {}", e))?;
                                        self.add_bool_range(cmp);
                                        self.builder.build_int_z_extend(cmp, self.context.i32_type(), "")
                                            .map_err(|e| format!("gez: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                }
                            };
                            let result_val = result.as_basic_value_enum();
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                // Auto-extend i1 result to wider int if dest type is wider
                                let extended = match (&result_val, &self.alloca_types.get(dest)) {
                                    (BasicValueEnum::IntValue(iv), Some(BasicTypeEnum::IntType(dt))) => {
                                        let rw = iv.get_type().get_bit_width();
                                        let dw = dt.get_bit_width();
                                        if rw != dw {
                                            if rw == 1 && dw > 1 {
                                                self.builder.build_int_z_extend(*iv, *dt, "")
                                                    .map_err(|e| format!("binop-zext: {}", e))?
                                                    .as_basic_value_enum()
                                            } else {
                                                self.builder.build_int_cast(*iv, *dt, "")
                                                    .map_err(|e| format!("binop-cast: {}", e))?
                                                    .as_basic_value_enum()
                                            }
                                        } else { result_val }
                                    }
                                    _ => result_val,
                                };
                                self.builder.build_store(dest_ptr, extended)
                                    .map_err(|e| format!("binop-store: {}", e))?;
                            }
                            last_value_map.insert(*dest, result_val);
                        }
                        MirInst::UnaryOp { dest, op, operand } => {
                            let val = self.value_to_llvm(operand, &last_value_map)?;
                            let int_val = val.into_int_value();
                            let result = match op {
                                MirUnaryOp::Neg => self.builder.build_int_neg(int_val, "")
                                    .map_err(|e| format!("neg: {}", e))?,
                                MirUnaryOp::Not | MirUnaryOp::BitNot => self.builder.build_not(int_val, "")
                                    .map_err(|e| format!("not: {}", e))?,
                            };
                            let result_val = result.as_basic_value_enum();
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, result_val)
                                    .map_err(|e| format!("unary-store: {}", e))?;
                            }
                            last_value_map.insert(*dest, result_val);
                        }
                        MirInst::Call { dest, name, args } => {
                            // Inline list operations (avoid FFI overhead for tight loops)
                            match name.as_str() {
                                "ky_list_get" | "ky_list_set" | "ky_list_len" | "ky_list_pop" => {
                                    self.emit_inline_list_op(name, dest, args, &mut last_value_map)?;
                                    continue;
                                }
                                "ky_bytes_get" | "ky_bytes_set" => {
                                    self.emit_inline_bytes_op(name, dest, args, &mut last_value_map)?;
                                    continue;
                                }
                                _ => {}
                            }
                            // Don't apply runtime name mapping for user-defined functions
                            let runtime_name = if self.fn_value_map.contains_key(name) {
                                name.clone()
                            } else {
                                match name.as_str() {
                                "print" => "ky_print",
                                "println" => "ky_println",
                                "contains" => "ky_str_contains",
                                "to_upper" => "ky_str_to_upper",
                                "to_lower" => "ky_str_to_lower",
                                "trim" => "ky_str_trim",
                                "replace" => "ky_str_replace",
                                "input" => "ky_input",
                                "open" => "ky_open",
                                "read_str" => "ky_read_str",
                                "write_str" => "ky_write_str",
                                "close" => "ky_close",
                                "sleep" => "ky_sleep",
                                "now" => "ky_now",
                                "char_at" => "ky_char_at",
                                "is_digit" => "ky_is_digit",
                                "is_alpha" => "ky_is_alpha",
                                "is_alnum" => "ky_is_alnum",
                                "is_whitespace" => "ky_is_whitespace",
                                "is_upper" => "ky_is_upper",
                                "is_lower" => "ky_is_lower",
                                "ord" => "ky_ord",
                                "substr" => "ky_substr",
                                "list_new" => "ky_list_new",
                                "list_push" => "ky_list_push",
                                "list_get" => "ky_list_get",
                                "list_set" => "ky_list_set",
                                "list_len" => "ky_list_len",
                                "list_pop" => "ky_list_pop", "reserve" => "ky_list_reserve",
                                "ky_str_builder_new" => "ky_str_builder_new",
                                "ky_str_builder_append" => "ky_str_builder_append",
                                "ky_str_builder_to_str" => "ky_str_builder_to_str",
                                "ky_str_builder_free" => "ky_str_builder_free",
                                _ => name.as_str(),
                                }.to_string()
                            };
                             if self.module.get_function(&runtime_name).is_none() {
                                // Auto-declare extern function on first use with inferred types
                                let ret_type: BasicTypeEnum = if let Some(d) = dest {
                                    let raw = self.alloca_types.get(&d).copied()
                                        .unwrap_or(self.context.i64_type().as_basic_type_enum());
                                    // Struct return types are actually i32 (runtime returns status code)
                                    if matches!(raw, BasicTypeEnum::StructType(_)) {
                                        self.context.i32_type().as_basic_type_enum()
                                    } else {
                                        raw
                                    }
                                } else {
                                    self.context.i64_type().as_basic_type_enum()
                                };
                                let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = args.iter()
                                    .map(|a| {
                                        let t = match a {
                                            MirValue::Local(id) => {
                                                let raw = self.alloca_types.get(id).copied()
                                                    .unwrap_or(self.context.i32_type().as_basic_type_enum());
                                                // Struct allocas are passed by pointer, so use ptr type
                                                if matches!(raw, BasicTypeEnum::StructType(_)) {
                                                    self.context.ptr_type(Default::default()).as_basic_type_enum()
                                                } else {
                                                    raw
                                                }
                                            }
                                            MirValue::Constant(c) => {
                                                match c {
                                                    MirConstant::String(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
                                                    MirConstant::I32(_) => self.context.i32_type().as_basic_type_enum(),
                                                    MirConstant::I64(_) => self.context.i64_type().as_basic_type_enum(),
                                                    MirConstant::Bool(_) => self.context.i8_type().as_basic_type_enum(),
                                                    _ => self.context.i32_type().as_basic_type_enum(),
                                                }
                                            }
                                            _ => self.context.i32_type().as_basic_type_enum(),
                                        };
                                        t.into()
                                    }).collect();
                                let fn_type = ret_type.fn_type(&param_types, false);
                                self.module.add_function(&runtime_name, fn_type, None);
                            }
                            if let Some(callee_fn) = self.module.get_function(&runtime_name) {
                                let fn_ty = callee_fn.get_type();
                                let param_types = fn_ty.get_param_types();
                                let llvm_args: Vec<BasicValueEnum<'ctx>> = args
                                    .iter()
                                    .enumerate()
                                    .map(|(i, a)| {
                                        // Pass struct locals by pointer (pass-by-reference ABI)
                                        if let MirValue::Local(id) = a {
                                            // Regular struct local: pass alloca pointer
                                            if let Some(&struct_type) = self.alloca_types.get(id) {
                                                if matches!(struct_type, BasicTypeEnum::StructType(_)) {
                                                    if let Some(ptr) = self.alloca_map.get(*id).and_then(|p| *p) {
                                                        return Ok(ptr.as_basic_value_enum());
                                                    }
                                                }
                                            }
                                            // Ref param: alloca stores ptr, load it as-is (already a ptr)
                                            if self.ref_param_struct_types.contains_key(id) {
                                                let val = self.load_value(*id, &last_value_map)?;
                                                return Ok(val);
                                            }
                                        }
                                        let val = self.value_to_llvm(a, &last_value_map)?;
                                        // Auto-cast i64 → ptr when function expects ptr
                                        // Auto-cast f64 → f32 when function expects f32
                                        if i < param_types.len() {
                                            let expected = param_types[i];
                                            match expected {
                                                inkwell::types::BasicMetadataTypeEnum::PointerType(_) => {
                                                    if let BasicValueEnum::IntValue(int_val) = val {
                                                        let ptr_ty = self.context.ptr_type(Default::default());
                                                        return Ok(self.builder.build_int_to_ptr(int_val, ptr_ty, "_argptr")
                                                            .map_err(|e| format!("arg inttoptr: {}", e))?
                                                            .as_basic_value_enum());
                                                    }
                                                }
                                                inkwell::types::BasicMetadataTypeEnum::FloatType(ft) => {
                                                    if let BasicValueEnum::FloatValue(fv) = val {
                                                        let src_w = fv.get_type().get_bit_width();
                                                        let dst_w = ft.get_bit_width();
                                                        if src_w > dst_w {
                                                            return Ok(self.builder.build_float_trunc(fv, ft, "")
                                                                .map_err(|e| format!("arg ftrunc: {}", e))?
                                                                .as_basic_value_enum());
                                                        } else if src_w < dst_w {
                                                            return Ok(self.builder.build_float_ext(fv, ft, "")
                                                                .map_err(|e| format!("arg fext: {}", e))?
                                                                .as_basic_value_enum());
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        Ok(val)
                                    })
                                    .collect::<Result<Vec<_>, String>>()?;
                                let llvm_arg_refs: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> =
                                    llvm_args.iter().map(|a| (*a).into()).collect();
                                let call_result = self.builder.build_call(callee_fn, &llvm_arg_refs, "")
                                    .map_err(|e| format!("call {}: {}", name, e))?;
                                if let Some(d) = dest {
                                    if let inkwell::values::ValueKind::Basic(ret_val) = call_result.try_as_basic_value() {
                                        // Store call result to both the last_value_map (for SSA-style use
                                        // within the same basic block) AND the alloca (for cross-block
                                        // references like kl_release in the return block)
                                        if let Some(alloca_ptr) = self.alloca_map.get(*d).and_then(|p| *p) {
                                            self.builder.build_store(alloca_ptr, ret_val)
                                                .map_err(|e| format!("call store {}: {}", name, e))?;
                                        }
                                        last_value_map.insert(*d, ret_val);
                                    }
                                }
                            }
                        }
                        MirInst::PtrOffset { dest, ptr, index, elem_type } => {
                            let base_val = self.load_value(*ptr, &last_value_map)?;
                            let idx = self.value_to_llvm(index, &last_value_map)?;
                            let int_idx = idx.into_int_value();
                            let gep = unsafe {
                                let ptr_val = if let BasicValueEnum::IntValue(iv) = base_val {
                                    self.builder.build_int_to_ptr(iv, self.context.ptr_type(Default::default()), "_inttoptr")
                                        .map_err(|e| format!("ptroff inttoptr: {}", e))?
                                } else {
                                    base_val.into_pointer_value()
                                };
                                let elem_llvm = self.llvm_type(elem_type);
                                // Bitcast i8* to T*, then GEP with proper element type
                                let typed_ptr = self.builder.build_pointer_cast(ptr_val, elem_llvm.ptr_type(Default::default()), "_pcast")
                                    .map_err(|e| format!("pcast: {}", e))?;
                                self.builder.build_in_bounds_gep(elem_llvm, typed_ptr, &[int_idx], "")
                                    .map_err(|e| format!("ptroff gep: {}", e))?
                            };
                            // Store GEP result in last_value_map, dest alloca, AND field_ptr_allocas
                            last_value_map.insert(*dest, gep.as_basic_value_enum());
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, gep.as_basic_value_enum())
                                    .map_err(|e| format!("ptroff-store: {}", e))?;
                            }
                            if let Some(fpa) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(fpa, gep.as_basic_value_enum())
                                    .map_err(|e| format!("ptroff-fpa: {}", e))?;
                            }
                            let elem_llvm = self.llvm_type(elem_type);
                            self.field_ptr_types.insert(*dest, elem_llvm);
                        }
                        MirInst::PtrStore { dest, ptr, index, value } => {
                            let base_val = self.load_value(*ptr, &last_value_map)?;
                            let idx = self.value_to_llvm(index, &last_value_map)?;
                            let int_idx = idx.into_int_value();
                            let pointee_type = self.context.i8_type().as_basic_type_enum();
                            let gep = unsafe {
                                let ptr_val = if let BasicValueEnum::IntValue(iv) = base_val {
                                    self.builder.build_int_to_ptr(iv, self.context.ptr_type(Default::default()), "_psint")
                                        .map_err(|e| format!("ps inttoptr: {}", e))?
                                } else {
                                    base_val.into_pointer_value()
                                };
                                self.builder.build_in_bounds_gep(pointee_type, ptr_val, &[int_idx], "")
                                    .map_err(|e| format!("ps gep: {}", e))?
                            };
                            let val = self.value_to_llvm(value, &last_value_map)?;
                            let siv = self.builder.build_store(gep, val)
                                .map_err(|e| format!("ps store: {}", e))?;
                            last_value_map.insert(*dest, val);
                        }
                        MirInst::FieldPtr { dest, ptr, field_index, struct_type } => {
                            if let Some(base_ptr) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                // Determine field type for later loads from this field pointer
                                if let MirType::Struct(_, fields) = struct_type.as_ref() {
                                    if let Some((_, field_mir_type)) = fields.get(*field_index) {
                                        let field_llvm = self.llvm_type(field_mir_type);
                                        self.field_ptr_types.insert(*dest, field_llvm);
                                    }
                                }
                                let zero = self.context.i32_type().const_zero();
                                let idx_val = self.context.i32_type().const_int(*field_index as u64, false);
                                // Ref param: alloca stores pointer-to-struct, load it first
                                if let Some(&orig_struct_type) = self.ref_param_struct_types.get(ptr) {
                                    let struct_ptr = self.builder.build_load(
                                        self.context.ptr_type(Default::default()),
                                        base_ptr, "_ref_load"
                                    ).map_err(|e| format!("ref_field_ptr load: {}", e))?;
                                    let gep = unsafe {
                                        self.builder.build_in_bounds_gep(orig_struct_type, struct_ptr.into_pointer_value(), &[zero, idx_val], "")
                                            .map_err(|e| format!("ref_field_ptr: {}", e))?
                                    };
                                    if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                        self.builder.build_store(alloca, gep)
                                            .map_err(|e| format!("ref_fgep store: {}", e))?;
                                    }
                                } else if let Some(ptr_type) = self.alloca_types.get(ptr) {
                                    if ptr_type.is_pointer_type() {
                                        // Ptr(Struct) closure param: alloca stores a pointer to the struct.
                                        // Load the pointer, then GEP on the pointed-to struct.
                                        if let MirType::Struct(sname, fields) = struct_type.as_ref() {
                                            if !fields.is_empty() {
                                                let struct_llvm = self.llvm_type(&MirType::Struct(sname.clone(), fields.clone()));
                                                let struct_ptr = self.builder.build_load(
                                                    self.context.ptr_type(Default::default()),
                                                    base_ptr, "_ptr_param_load"
                                                ).map_err(|e| format!("ptr_param_field_ptr load: {}", e))?;
                                                let gep = unsafe {
                                                    self.builder.build_in_bounds_gep(struct_llvm, struct_ptr.into_pointer_value(), &[zero, idx_val], "")
                                                        .map_err(|e| format!("ptr_param_field_ptr: {}", e))?
                                                };
                                                if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                                    self.builder.build_store(alloca, gep)
                                                        .map_err(|e| format!("ptr_param_fgep store: {}", e))?;
                                                }
                                            }
                                        }
                                    } else {
                                        let gep = unsafe {
                                            self.builder.build_in_bounds_gep(*ptr_type, base_ptr, &[zero, idx_val], "")
                                                .map_err(|e| format!("field_ptr: {}", e))?
                                        };
                                        if let Some(alloca) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                            self.builder.build_store(alloca, gep)
                                                .map_err(|e| format!("fgep store: {}", e))?;
                                        }
                                    }
                                }
                            }
                        }
                        MirInst::ArrayElemPtr { dest, ptr, index, arr_type, elem_type } => {
                            // Check if ptr is a previously-computed GEP pointer (chained GEP)
                            let fpa_found = *ptr < self.field_ptr_allocas.len() && self.field_ptr_allocas[*ptr].is_some();
                            let alloca_found = self.alloca_map.get(*ptr).and_then(|p| *p);
                            let base_ptr = if fpa_found {
                                let p = self.field_ptr_allocas[*ptr].unwrap();
                                let loaded = self.builder.build_load(
                                    self.context.ptr_type(Default::default()), p, "_aebase"
                                ).map_err(|e| format!("aebase load: {}", e))?;
                                loaded.as_basic_value_enum()
                            } else if let Some(p) = alloca_found {
                                p.as_basic_value_enum()
                            } else { continue; };
                            if let BasicValueEnum::PointerValue(base_ptr) = base_ptr {
                                let arr_llvm = self.llvm_type(arr_type);
                                let zero = self.context.i32_type().const_zero();
                                let idx_val = self.value_to_llvm(index, &last_value_map)
                                    .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum());
                                let idx_i32 = if let BasicValueEnum::IntValue(iv) = idx_val {
                                    if iv.get_type().get_bit_width() != 32 {
                                        self.builder.build_int_truncate(iv, self.context.i32_type(), "_aeptrunc")
                                            .map_err(|e| format!("aeptrunc: {}", e))?
                                    } else { iv }
                                } else {
                                    self.context.i32_type().const_zero()
                                };
                                let gep = unsafe {
                                    self.builder.build_in_bounds_gep(arr_llvm, base_ptr, &[zero, idx_i32], "_aelem")
                                        .map_err(|e| format!("aelem: {}", e))?
                                };
                                if let Some(fpa) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(fpa, gep)
                                        .map_err(|e| format!("aelem store gep: {}", e))?;
                                }
                                let elem_llvm = self.llvm_type(elem_type);
                                self.field_ptr_types.insert(*dest, elem_llvm);
                            }
                        }
                        MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                            if let Some(dest_ptr) = last_value_map.get(dest_ptr_local) {
                                if let Some(src_val) = last_value_map.get(src_alloca_local) {
                                    if let BasicValueEnum::StructValue(struct_val) = src_val {
                                        let heap_ptr = dest_ptr.into_pointer_value();
                                        let struct_ptr = self.builder.build_pointer_cast(heap_ptr, self.context.ptr_type(Default::default()), "_mc")
                                            .map_err(|e| format!("memcpy bitcast: {}", e))?;
                                        self.builder.build_store(struct_ptr, *struct_val)
                                            .map_err(|e| format!("memcpy store: {}", e))?;
                                    }
                                }
                            }
                        }
                        MirInst::Cast { dest, value, to_type } => {
                            let val = self.value_to_llvm(value, &last_value_map)?;
                            let target_type = self.llvm_type(to_type);
                            let result = match (&val, &target_type) {
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(t)) => {
                                    let src_width = int_val.get_type().get_bit_width();
                                    let dst_width = t.get_bit_width();
                                    let result = if src_width < dst_width && src_width > 1 {
                                        self.builder.build_int_s_extend(*int_val, *t, "")
                                            .map_err(|e| format!("sext: {}", e))?
                                    } else if src_width == 1 && dst_width > 1 {
                                        self.builder.build_int_z_extend(*int_val, *t, "")
                                            .map_err(|e| format!("zext: {}", e))?
                                    } else if dst_width == 1 && src_width > 1 {
                                        // Int → Bool: compare with zero, not truncate
                                        let zero = self.context.i32_type().const_zero();
                                        let widened = if src_width < 32 {
                                            self.builder.build_int_z_extend(*int_val, self.context.i32_type(), "_widen")
                                                .map_err(|e| format!("widen: {}", e))?
                                        } else { *int_val };
                                        self.builder.build_int_compare(inkwell::IntPredicate::NE, widened, zero, "_tobool")
                                            .map_err(|e| format!("tobool: {}", e))?
                                    } else {
                                        self.builder.build_int_cast(*int_val, *t, "")
                                            .map_err(|e| format!("cast: {}", e))?
                                    };
                                    result.as_basic_value_enum()
                                }
                                (BasicValueEnum::PointerValue(ptr_val), BasicTypeEnum::IntType(t)) => {
                                    self.builder.build_ptr_to_int(*ptr_val, *t, "")
                                        .map_err(|e| format!("ptrtoint: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::PointerType(t)) => {
                                    self.builder.build_int_to_ptr(*int_val, *t, "")
                                        .map_err(|e| format!("inttoptr: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::StructType(s)) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    let ptr_val = self.builder.build_int_to_ptr(*int_val, ptr_ty, "_ptr")
                                        .map_err(|e| format!("inttoptr: {}", e))?;
                                    self.builder.build_load(*s, ptr_val, "_struct")
                                        .map_err(|e| format!("load struct: {}", e))?
                                }
                                (BasicValueEnum::StructValue(struct_val), BasicTypeEnum::IntType(i)) => {
                                    let struct_ty = struct_val.get_type();
                                    let temp_alloca = self.builder.build_alloca(struct_ty, "_tmp_struct")
                                        .map_err(|e| format!("alloca: {}", e))?;
                                    self.builder.build_store(temp_alloca, *struct_val)
                                        .map_err(|e| format!("store struct: {}", e))?;
                                    let ptr = temp_alloca.as_basic_value_enum();
                                    self.builder.build_ptr_to_int(ptr.into_pointer_value(), *i, "_ptrint")
                                        .map_err(|e| format!("ptrtoint: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::FloatType(f)) => {
                                    // Integer → Float: sitofp
                                    self.builder.build_signed_int_to_float(*int_val, *f, "_sitofp")
                                        .map_err(|e| format!("sitofp: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::FloatType(t)) => {
                                    // Float → Float: fpext/fptrunc
                                    self.builder.build_float_cast(*float_val, *t, "_ffcast")
                                        .map_err(|e| format!("ffcast: {}", e))?
                                        .as_basic_value_enum()
                                }
                                (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::IntType(i)) => {
                                    // Float → Integer: use bitcast when sizes match, fptosi otherwise
                                    let f_size = float_val.get_type().get_bit_width();
                                    let i_size = i.get_bit_width();
                                    if f_size == i_size {
                                        self.builder.build_bit_cast(*float_val, *i, "_fcast")
                                            .map_err(|e| format!("fbitcast: {}", e))?
                                            .as_basic_value_enum()
                                    } else {
                                        self.builder.build_float_to_signed_int(*float_val, *i, "_fptosi")
                                            .map_err(|e| format!("fptosi: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                }
                                // Pointer → Pointer: identity (no-op cast)
                                (BasicValueEnum::PointerValue(ptr_val), BasicTypeEnum::PointerType(_)) => {
                                    ptr_val.as_basic_value_enum()
                                }
                                _ => self.context.i32_type().const_zero().as_basic_value_enum(),
                            };
                            if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                self.builder.build_store(dest_ptr, result)
                                    .map_err(|e| format!("cast-store: {}", e))?;
                            }
                            last_value_map.insert(*dest, result);
                        }
                        MirInst::FnAddr { dest, name } => {
                            if let Some(fn_val) = self.fn_value_map.get(name) {
                                let global = fn_val.as_global_value();
                                let ptr = global.as_pointer_value();
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ptr)
                                        .map_err(|e| format!("fnaddr store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ptr.as_basic_value_enum());
                            }
                        }
                        MirInst::AddressOf { dest, local_id } => {
                            if let Some(ptr) = self.alloca_map.get(*local_id).and_then(|p| *p) {
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ptr)
                                        .map_err(|e| format!("addr store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ptr.as_basic_value_enum());
                            }
                        }
                        MirInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                            let ptr_val = self.load_value(*fn_ptr, &last_value_map)?;
                            let fn_ptr = match ptr_val {
                                BasicValueEnum::IntValue(iv) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    self.builder.build_int_to_ptr(iv, ptr_ty, "_fnptr")
                                        .map_err(|e| format!("callindirect inttoptr: {}", e))?
                                }
                                _ => ptr_val.into_pointer_value(),
                            };
                            let llvm_ret = self.llvm_type(ret_type);
                            let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types.iter()
                                .map(|p| self.llvm_type(p).into())
                                .collect();
                            let fn_ty = llvm_ret.fn_type(&llvm_params, false);
                            let fn_param_types = fn_ty.get_param_types();
                            let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args.iter()
                                .enumerate()
                                .map(|(i, a)| {
                                    // If the function expects ptr but the MIR arg is a struct alloca,
                                    // pass the alloca pointer instead of loading the struct value.
                                    if i < fn_param_types.len() {
                                        if let inkwell::types::BasicMetadataTypeEnum::PointerType(_) = fn_param_types[i] {
                                            if let MirValue::Local(id) = a {
                                                if let Some(Some(alloca)) = self.alloca_map.get(*id) {
                                                    if let Some(pointee_type) = self.alloca_types.get(id) {
                                                        if matches!(pointee_type, BasicTypeEnum::StructType(_)) {
                                                            return alloca.as_basic_value_enum().into();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let val: BasicValueEnum = self.value_to_llvm(a, &last_value_map)
                                        .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum());
                                    // Auto-truncate i64 args to i32 if needed (for closure calls)
                                    if i < fn_param_types.len() {
                                        if let BasicValueEnum::IntValue(iv) = val {
                                            let expected_ty = fn_param_types[i];
                                            let actual_w = iv.get_type().get_bit_width();
                                            if let inkwell::types::BasicMetadataTypeEnum::IntType(eit) = expected_ty {
                                                let expected_w = eit.get_bit_width();
                                                if actual_w > expected_w {
                                                    if let Ok(trunc) = self.builder.build_int_truncate(iv, eit, "") {
                                                        return trunc.as_basic_value_enum().into();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    val.into()
                                })
                                .collect();
                            let call_result = self.builder.build_indirect_call(fn_ty, fn_ptr, &llvm_args, "_icl")
                                .map_err(|e| format!("indirect call: {}", e))?;
                            if let Some(d) = dest {
                                if let inkwell::values::ValueKind::Basic(result) = call_result.try_as_basic_value() {
                                    if let Some(alloca) = self.alloca_map.get(*d).and_then(|p| *p) {
                                        self.builder.build_store(alloca, result)
                                            .map_err(|e| format!("icall store: {}", e))?;
                                    }
                                    last_value_map.insert(*d, result);
                                }
                            }
                        }
                        MirInst::AsyncSpawn { dest, function_name, arg } => {
                            let arg_val = self.value_to_llvm(arg, &last_value_map)?;
                            let spawn_fn = self.module.get_function("ky_spawn_task")
                                .ok_or_else(|| "ky_spawn_task not declared".to_string())?;
                            // Get the function pointer of the async wrapper
                            let fn_val = self.fn_value_map.get(function_name)
                                .ok_or_else(|| format!("async function {} not found", function_name))?;
                            let fn_global = fn_val.as_global_value();
                            let fn_ptr = fn_global.as_pointer_value();
                            let args_meta: Vec<inkwell::values::BasicMetadataValueEnum> = vec![
                                fn_ptr.into(),
                                arg_val.into(),
                            ];
                            let call_result = self.builder.build_call(spawn_fn, &args_meta, "_async_spawn")
                                .map_err(|e| format!("async_spawn: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(ret_val) = call_result.try_as_basic_value() {
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, ret_val)
                                        .map_err(|e| format!("async_spawn store: {}", e))?;
                                }
                                last_value_map.insert(*dest, ret_val);
                            }
                        }
                        MirInst::SliceMake { dest, ptr, len, elem_type } => {
                            let ptr_val = self.value_to_llvm(ptr, &last_value_map)?;
                            let len_val = self.value_to_llvm(len, &last_value_map)?;
                            let slice_type = self.llvm_type(&MirType::Slice(elem_type.clone()));
                            if let BasicTypeEnum::StructType(st) = slice_type {
                                let undef = st.get_undef();
                                let sv = unsafe {
                                    self.builder.build_insert_value(undef, ptr_val, 0, "_msmi0")
                                        .map_err(|e| format!("msmi0: {}", e))?
                                };
                                let sv = unsafe {
                                    self.builder.build_insert_value(sv, len_val, 1, "_msmi1")
                                        .map_err(|e| format!("msmi1: {}", e))?
                                };
                                let bv = sv.as_basic_value_enum();
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, bv)
                                        .map_err(|e| format!("msmst: {}", e))?;
                                }
                                last_value_map.insert(*dest, bv);
                            }
                        }
                        MirInst::AsyncAwait { dest, handle, return_type } => {
                            let handle_val = self.load_value(*handle, &last_value_map)?;
                            let join_fn = self.module.get_function("ky_await_task")
                                .ok_or_else(|| "ky_await_task not declared".to_string())?;
                            let args_meta: Vec<inkwell::values::BasicMetadataValueEnum> = vec![handle_val.into()];
                            let call_result = self.builder.build_call(join_fn, &args_meta, "_async_join")
                                .map_err(|e| format!("async_join: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(ret_val) = call_result.try_as_basic_value() {
                                // Cast i64 result to the actual return type
                                let target_type = self.llvm_type(return_type);
                                let casted = self.cast_to_type(ret_val, target_type)?;
                                if let Some(alloca) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    self.builder.build_store(alloca, casted)
                                        .map_err(|e| format!("async_join store: {}", e))?;
                                }
                                last_value_map.insert(*dest, casted);
                            }
                        }
                    }
                }

                match &bb.terminator {
                    MirTerminator::Return(value) => {
                        let val = match value {
                            MirValue::Local(id) => {
                                // Ref params: the alloca stores a ptr, dereference to get struct value
                                if let Some(&struct_type) = self.ref_param_struct_types.get(id) {
                                    let ptr_val = self.load_value(*id, &last_value_map)?;
                                    self.builder.build_load(struct_type, ptr_val.into_pointer_value(), "_retderef")
                                        .map_err(|e| format!("ret ref deref: {}", e))?
                                } else {
                                    self.load_value(*id, &last_value_map)?
                                }
                            }
                            _ => self.value_to_llvm(value, &last_value_map)?,
                        };
                        // Auto-cast return value if it doesn't match function return type
                        let fn_ret_type = fn_value.get_type().get_return_type();
                        let val = if let Some(expected_ret_ty) = fn_ret_type {
                            if val.get_type() != expected_ret_ty.as_basic_type_enum() {
                                match (&val, &expected_ret_ty) {
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::PointerType(pt)) =>
                                        self.builder.build_int_to_ptr(*iv, *pt, "_retptr")
                                            .map_err(|e| format!("ret inttoptr: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::PointerValue(pv), BasicTypeEnum::IntType(it)) =>
                                        self.builder.build_ptr_to_int(*pv, *it, "_retint")
                                            .map_err(|e| format!("ret ptrtoint: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(it)) => {
                                        let sw = iv.get_type().get_bit_width();
                                        let dw = it.get_bit_width();
                                        if sw == 1 && dw > 1 {
                                            self.builder.build_int_z_extend(*iv, *it, "")
                                                .map_err(|e| format!("ret zext: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            self.builder.build_int_cast(*iv, *it, "")
                                                .map_err(|e| format!("ret intcast: {}", e))?
                                                .as_basic_value_enum()
                                        }
                                    }
                                    (BasicValueEnum::FloatValue(fv), BasicTypeEnum::IntType(it)) => {
                                        let fw = fv.get_type().get_bit_width();
                                        let dw = it.get_bit_width();
                                        if fw == dw as u32 {
                                            self.builder.build_bit_cast(*fv, *it, "")
                                                .map_err(|e| format!("ret fbitcast: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            self.builder.build_float_to_signed_int(*fv, *it, "")
                                                .map_err(|e| format!("ret fptosi: {}", e))?
                                                .as_basic_value_enum()
                                        }
                                    }
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(ft)) => {
                                        self.builder.build_signed_int_to_float(*iv, *ft, "")
                                            .map_err(|e| format!("ret sitofp: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(ft)) => {
                                        let fw = fv.get_type().get_bit_width();
                                        let dw = ft.get_bit_width();
                                        if fw > dw {
                                            self.builder.build_float_trunc(*fv, *ft, "")
                                                .map_err(|e| format!("ret ftrunc: {}", e))?
                                                .as_basic_value_enum()
                                        } else if fw < dw {
                                            self.builder.build_float_ext(*fv, *ft, "")
                                                .map_err(|e| format!("ret fext: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            val
                                        }
                                    }
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::StructType(st)) => {
                                        // Heap pointer (i64) → dereference to struct value
                                        let ptr_ty = self.context.ptr_type(Default::default());
                                        let ptr_val = self.builder.build_int_to_ptr(*iv, ptr_ty, "_retptr")
                                            .map_err(|e| format!("ret inttoptr: {}", e))?;
                                        self.builder.build_load(*st, ptr_val, "_retstruct")
                                            .map_err(|e| format!("ret load struct: {}", e))?
                                    }
                                    _ => val,
                                }
                            } else { val }
                        } else { val };
                        self.builder.build_return(Some(&val))
                            .map_err(|e| format!("ret: {}", e))?;
                    }
                    MirTerminator::Br(label) => {
                        if let Some(&target) = block_map.get(label) {
                            let _ = self.builder.build_unconditional_branch(target);
                        }
                    }
                    MirTerminator::CondBr { cond, true_block, false_block } => {
                        let cond_val = match cond {
                            MirValue::Local(id) => self.load_value(*id, &last_value_map)?,
                            _ => self.value_to_llvm(cond, &last_value_map)?,
                        };
                        let cond_int = cond_val.into_int_value();
                        // Truncate to i1 if needed (e.g. string eq returns i32)
                        let i1_cond = if cond_int.get_type().get_bit_width() > 1 {
                            let i1_ty = self.context.bool_type();
                            self.builder.build_int_truncate(cond_int, i1_ty, "")
                                .map_err(|e| format!("cond trunc: {}", e))?
                        } else {
                            cond_int
                        };
                        if let (Some(&t), Some(&f)) = (block_map.get(true_block), block_map.get(false_block)) {
                            let _ = self.builder.build_conditional_branch(i1_cond, t, f);
                        }
                    }
                    MirTerminator::Unreachable => {
                        self.builder.build_unreachable()
                            .map_err(|e| format!("unreachable: {}", e))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a C-compatible main(i32, ptr) wrapper that:
    /// 1. Calls kl_init_args(argc, argv) to build a Kyle list<str>
    /// 2. Calls kyle_main(list) with the original function's logic
    pub(crate) fn generate_main_wrapper(&mut self) -> Result<(), String> {
        let i32_ty = self.context.i32_type();
        let ptr_ty = self.context.ptr_type(Default::default());

        // Get the kyle_main function that was declared instead of main
        let kyle_main = self.fn_value_map.get("kyle_main")
            .ok_or_else(|| "kyle_main not declared for wrapper".to_string())?;

        // Declare i32 @main(i32, ptr)
        let param_tys = [i32_ty.into(), ptr_ty.into()];
        let main_type = i32_ty.fn_type(&param_tys, false);
        let main_fn = self.module.add_function("main", main_type, None);

        let bb = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(bb);

        // Convert parameters to BasicMetadataValueEnum
        let argc = main_fn.get_nth_param(0).unwrap();
        let argv = main_fn.get_nth_param(1).unwrap();
        let argc_meta: inkwell::values::BasicMetadataValueEnum = argc.into();
        let argv_meta: inkwell::values::BasicMetadataValueEnum = argv.into();

        // Call kl_init_args(argc, argv) -> ptr (list handle)
        let init_args_fn = self.module.get_function("ky_init_args")
            .ok_or_else(|| "ky_init_args not declared".to_string())?;
        let args_call = self.builder.build_call(init_args_fn, &[argc_meta, argv_meta], "args")
            .map_err(|e| format!("call kl_init_args: {}", e))?;
        let args_list = match args_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => bv,
            _ => return Err("ky_init_args did not return a basic value".to_string()),
        };
        let args_meta: inkwell::values::BasicMetadataValueEnum = args_list.into();

        // Call kyle_main(args_list)
        let result_call = self.builder.build_call(*kyle_main, &[args_meta], "result")
            .map_err(|e| format!("call kyle_main: {}", e))?;
        match result_call.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(bv) => {
                self.builder.build_return(Some(&bv))
                    .map_err(|e| format!("main_wrapper ret: {}", e))?;
            }
            _ => {
                // kyle_main returns void — return 0
                self.builder.build_return(Some(&i32_ty.const_zero()))
                    .map_err(|e| format!("main_wrapper ret void: {}", e))?;
            }
        }

        Ok(())
    }

}
