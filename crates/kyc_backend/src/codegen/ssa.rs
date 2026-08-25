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
use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub fn compile(&mut self, mir_module: &MirModule) -> Result<(), String> {
        self.init_tbaa();
        self.declare_runtime_externs();
        for func in &mir_module.functions {
            self.declare_function(func)?;
        }
        for func in &mir_module.functions {
            self.compile_function(func)?;
        }
        if self.needs_main_wrapper {
            self.generate_main_wrapper()?;
        }
        Ok(())
    }

    // ===================================================================
    // SSA Codegen (Phase 15 — experimental)
    // ===================================================================

    pub fn compile_with_ssa(&mut self, mir_module: &MirModule) -> Result<(), String> {
        self.init_tbaa();
        let result = kyc_mir::ssa::convert_module(mir_module);
        let ssa_fns = result.ssa_functions;
        let non_ssa_fns = result.non_ssa_functions;

        // Always declare runtime externs so LLVM IR is valid.
        // In freestanding mode, the kernel must provide its own implementations.
        self.declare_runtime_externs();

        for ssa_fn in &ssa_fns {
            self.declare_ssa_function(ssa_fn)?;
        }
        for func in &non_ssa_fns {
            self.declare_function(func)?;
        }
        for ssa_fn in &ssa_fns {
            self.compile_ssa_function(ssa_fn)?;
        }
        for func in &non_ssa_fns {
            self.compile_function(func)?;
        }
        if self.needs_main_wrapper && !self.is_freestanding {
            self.generate_main_wrapper()?;
        }
        if let Err(e) = self.module.verify() {
            return Err(format!("SSA verify: {}", e));
        }
        Ok(())
    }

    pub(crate) fn declare_ssa_function(&mut self, func: &SsaFunction) -> Result<(), String> {
        let ret_type = self.llvm_type(&func.return_type);
        let ptr_ty = self.context.ptr_type(Default::default());
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = func.params
            .iter()
            .map(|p| if matches!(p, MirType::Struct(_, _) | MirType::Slice(_)) { ptr_ty.into() }
                 else { self.llvm_type(p).into() })
            .collect();

        let fn_name = if self.is_freestanding {
            &func.name
        } else if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            self.needs_main_wrapper = true; "kyle_main"
        } else { &func.name };
        let fn_type = ret_type.fn_type(&param_types, false);
        // If function was already declared (e.g. from prelude extern fn),
        // reuse the existing declaration so the body fills it in.
        // This allows Kyle runtime code to define prelude-declared extern fns.
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
                if matches!(ptype, MirType::Struct(_, _) | MirType::Slice(_) | MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Ptr(_) | MirType::Box(_)) {
                    let attr = self.context.create_enum_attribute(noalias_kind, 0);
                    fn_value.add_attribute(AttributeLoc::Param(idx), attr);
                }
            }
        }
        self.fn_value_map.insert(fn_name.to_string(), fn_value);
        Ok(())
    }

    /// Compile an SSA function, generating pure SSA LLVM IR (no allocas for promoted vars).
    ///
    /// Key design:
    /// - `block_vals[bi]` maps SsaValueId → LLVM value for each block
    /// - Each instruction reads operands directly from `block_vals[bi]` (no stale snapshot)
    /// - `alloca_current` (global) tracks the current LLVM value for each promoted alloca
    /// - Phi nodes in LLVM carry cross-block values; `alloca_current` seeds per-block start
    pub(crate) fn compile_ssa_function(&mut self, func: &SsaFunction) -> Result<(), String> {
        let fn_name = if self.is_freestanding {
            &func.name
        } else if func.name == "main" && func.params.len() == 1 && matches!(&func.params[0], MirType::List(_)) {
            "kyle_main"
        } else { &func.name };
        let fn_value = *self.fn_value_map.get(fn_name)
            .ok_or_else(|| format!("SSA fn '{}' not declared", fn_name))?;
        let ptr_ty = self.context.ptr_type(Default::default());

        // Pre-scan non-promotable allocas (escaping: field_ptr, heap types)
        self.alloca_types.clear();
        self.ref_param_struct_types.clear();
        self.field_ptr_allocas.clear();
        self.field_ptr_types.clear();
        for block in &func.blocks {
            for inst in &block.insts {
                if let SsaInst::Alloca { dest, type_, .. } = inst {
                    let llvm_ty = self.llvm_type(type_);
                    let actual = if let MirType::Ptr(_) = type_ { ptr_ty.as_basic_type_enum() } else { llvm_ty };
                    self.alloca_types.entry(*dest).or_insert(actual);
                }
            }
        }

        // Ref params: change struct param allocas from struct type to ptr
        for block in &func.blocks {
            for inst in &block.insts {
                if let SsaInst::Store { dest, value } = inst {
                    if let Some(param_idx) = func.param_value_ids.iter().position(|&p| p == *value) {
                        if matches!(&func.params[param_idx], MirType::Struct(_, _) | MirType::Slice(_)) {
                            // Struct params are passed by pointer in LLVM.
                            // The alloca type may be Struct (direct) or Ptr(Struct) (indirect).
                            // In either case, change the alloca to store ptr_ty and track the original.
                            if let Some(&llvm_type) = self.alloca_types.get(dest) {
                                let orig_type = if matches!(llvm_type, BasicTypeEnum::StructType(_)) {
                                    llvm_type
                                } else if llvm_type.is_pointer_type() {
                                    // For Ptr(Struct) allocas, the actual struct type comes from func.params
                                    self.llvm_type(&func.params[param_idx])
                                } else {
                                    continue;
                                };
                                self.alloca_types.insert(*dest, ptr_ty.as_basic_type_enum());
                                self.ref_param_struct_types.insert(*dest, orig_type);
                            }
                        }
                    }
                }
            }
        }

        // Create LLVM basic blocks and phi nodes
        let mut block_map: HashMap<String, inkwell::basic_block::BasicBlock<'ctx>> = HashMap::new();
        let mut block_indices: HashMap<String, usize> = HashMap::new();
        let mut phi_map: Vec<(usize, inkwell::values::PhiValue<'ctx>)> = Vec::new();
        for (i, block) in func.blocks.iter().enumerate() {
            let llvm_bb = self.context.append_basic_block(fn_value, &block.label);
            block_map.insert(block.label.clone(), llvm_bb);
            block_indices.insert(block.label.clone(), i);
        }

        // Entry block: allocate non-promotable stack slots + params
        self.alloca_map.clear();
        if let Some(entry) = func.blocks.first() {
            if let Some(&entry_bb) = block_map.get(&entry.label) {
                self.builder.position_at_end(entry_bb);
                for (&dest, &ty) in &self.alloca_types {
                    while self.alloca_map.len() <= dest { self.alloca_map.push(None); }
                    let ptr = self.builder.build_alloca(ty, "")
                        .map_err(|e| format!("ssa alloca {}: {}", dest, e))?;
                    if let Ok(iv) = inkwell::values::InstructionValue::try_from(inkwell::values::AnyValueEnum::PointerValue(ptr)) {
                        let _ = iv.set_alignment(8);
                    }
                    // // self.emit_lifetime_start(ptr, -1); // DEBUG: disabled for mem2reg test // DEBUG: disabled for mem2reg test
                    self.alloca_map[dest] = Some(ptr);
                }
                for block in &func.blocks {
                    for inst in &block.insts {
                        if let SsaInst::FieldPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest { self.field_ptr_allocas.push(None); }
                            if self.field_ptr_allocas[*dest].is_none() {
                                self.field_ptr_allocas[*dest] = Some(
                                    self.builder.build_alloca(ptr_ty, "_fgep")
                                        .map_err(|e| format!("ssa fgep: {}", e))?
                                );
                            }
                        }
                        if let SsaInst::ArrayElemPtr { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest { self.field_ptr_allocas.push(None); }
                            if self.field_ptr_allocas[*dest].is_none() {
                                self.field_ptr_allocas[*dest] = Some(
                                    self.builder.build_alloca(ptr_ty, "_aelem")
                                        .map_err(|e| format!("ssa aep: {}", e))?
                                );
                            }
                        }
                        if let SsaInst::PtrOffset { dest, .. } = inst {
                            while self.field_ptr_allocas.len() <= *dest { self.field_ptr_allocas.push(None); }
                            if self.field_ptr_allocas[*dest].is_none() {
                                self.field_ptr_allocas[*dest] = Some(
                                    self.builder.build_alloca(ptr_ty, "_pgep")
                                        .map_err(|e| format!("ssa pgep: {}", e))?
                                );
                            }
                        }
                        if let SsaInst::Load { src, .. } = inst {
                            while self.field_ptr_allocas.len() <= *src { self.field_ptr_allocas.push(None); }
                        }
                    }
                }
                self.param_values.clear();
                for (i, p) in fn_value.get_param_iter().enumerate() {
                    self.param_values.insert(i, p);
                }
            }
        }

        // Pass 1: create phi nodes at block starts
        for block in &func.blocks {
            if let Some(&llvm_bb) = block_map.get(&block.label) {
                self.builder.position_at_end(llvm_bb);
                for phi in &block.phis {
                    let phi_type = self.llvm_type(&phi.type_);
                    if let Ok(llvm_phi) = self.builder.build_phi(phi_type, "_phi") {
                        phi_map.push((phi.dest, llvm_phi));
                    }
                }
            }
        }

        // Pass 2: compile instructions — main SSA codegen
        // GLOBAL alloca_current: tracks current LLVM value for each promoted alloca.
        // This persists across ALL blocks so values flow through loops correctly.
        let mut alloca_current: HashMap<usize, BasicValueEnum<'ctx>> = HashMap::new();
        let mut block_vals: Vec<HashMap<usize, BasicValueEnum<'ctx>>> = vec![HashMap::new(); func.blocks.len()];

        // Seed block_vals[0] with param values for ssa_read! in the entry block
        for (&i, &p) in &self.param_values {
            if let Some(&pid) = func.param_value_ids.get(i) {
                block_vals[0].insert(pid, p);
            }
        }

        for (bi, block) in func.blocks.iter().enumerate() {
            if let Some(&llvm_bb) = block_map.get(&block.label) {
                self.builder.position_at_end(llvm_bb);

                // Seed block_vals with phi values for THIS block
                // AND update alloca_current with phi values (for succeeding blocks)
                for &(phi_id, ref phi_val) in &phi_map {
                    let bv = phi_val.as_basic_value();
                    if let Some(phi) = block.phis.iter().find(|p| p.dest == phi_id) {
                        block_vals[bi].insert(phi_id, bv);
                        alloca_current.insert(phi.alloca_id, bv);
                    }
                }

                // Compile instructions.
                let insts = block.insts.clone();

                // Inline helper: read SsaValueId from block_vals, alloca_current, const_values
                macro_rules! ssa_read {
                    ($id:expr) => {{
                        let id: SsaValueId = $id;
                        // 1. Current block's computed values
                        if let Some(&v) = block_vals[bi].get(&id) { v }
                        // 2. Search ALL prior blocks' block_vals (cross-block phis/values)
                        else {
                            let mut found = false;
                            let mut val = self.context.i32_type().const_zero().as_basic_value_enum();
                            for bv in block_vals[..=bi].iter().rev() {
                                if let Some(&v) = bv.get(&id) { val = v; found = true; break; }
                            }
                            if found { val }
                            // 3. Constant values from const_values
                            else if let Some(c) = func.const_values.get(&id) { self.constant_to_llvm(c) }
                            // 4. Non-promotable dummy (_np{mir_id}): load from the actual MIR alloca.
                            //    Must run BEFORE the alloca_map fallback: SsaValueIds and MIR local
                            //    ids share a numeric namespace, so alloca_map.get(id) can collide
                            //    with an unrelated MIR alloca for dummy SsaValueIds.
                            else if let Some(sv) = func.values.get(id) {
                                let mir_id = sv.name.strip_prefix("_np")
                                    .and_then(|s| s.parse::<usize>().ok());
                                if let Some(mir_id) = mir_id {
                                    self.alloca_map.get(mir_id).and_then(|p| *p).and_then(|ptr| {
                                        self.alloca_types.get(&mir_id).and_then(|pointee_type| {
                                            self.builder.build_load(*pointee_type, ptr, "_ssaload_np").ok()
                                        })
                                    }).unwrap_or_else(|| self.context.i32_type().const_zero().as_basic_value_enum())
                                } else {
                                    self.context.i32_type().const_zero().as_basic_value_enum()
                                }
                            }
                            // 5. Fallback to alloca_map (non-promotable allocas like structs, slices)
                            else if let Some(Some(ptr)) = self.alloca_map.get(id) {
                                self.alloca_types.get(&id).and_then(|pointee_type| {
                                    self.builder.build_load(*pointee_type, *ptr, "_ssaload").ok()
                                }).unwrap_or_else(|| self.context.i32_type().const_zero().as_basic_value_enum())
                            }
                            // 6. Default zero
                            else { self.context.i32_type().const_zero().as_basic_value_enum() }
                        }
                    }};
                }

                // Helper: resolve MirValue to LLVM value using block_local_map
                macro_rules! resolve_mir {
                    ($val:expr) => {{
                        let val: &MirValue = $val;
                        match val {
                            MirValue::Constant(c) => self.constant_to_llvm(c),
                            MirValue::Local(mir_id) => {
                                // For non-promotable allocas (str types), load from alloca directly.
                                // Promotable allocas use block_local_map for SSA-tracked values.
                                if let Some(Some(ptr)) = self.alloca_map.get(*mir_id) {
                                    if let Some(pointee_type) = self.alloca_types.get(mir_id) {
                                        self.builder.build_load(*pointee_type, *ptr, "_ssaload")
                                            .map_err(|e| format!("ssa-load {}: {}", mir_id, e))?
                                            .as_basic_value_enum()
                                    } else {
                                        self.context.i32_type().const_zero().as_basic_value_enum()
                                    }
                                } else if let Some(ssa_id) = func.block_local_map.get(bi).and_then(|m| m.get(mir_id)).copied() {
                                    ssa_read!(ssa_id)
                                } else if let Some(&v) = alloca_current.get(mir_id) {
                                    v
                                } else if let Some(v) = block_vals.get(bi).and_then(|bv| bv.get(mir_id)).copied() {
                                    v
                                } else {
                                    self.context.i32_type().const_zero().as_basic_value_enum()
                                }
                            }
                            MirValue::Param(id) => self.param_values.get(id).copied()
                                .unwrap_or_else(|| self.context.i32_type().const_zero().as_basic_value_enum()),
                        }
                    }};
                }

                for inst in &insts {
                    match inst {
                        SsaInst::Alloca { .. } => {}
                        SsaInst::Store { dest, value } => {
                            let val = ssa_read!(*value);
                            let mut stored = false;
                            // Try field_ptr_allocas first (used by ArrayElemPtr, FieldPtr)
                            if *dest < self.field_ptr_allocas.len() {
                                if let Some(fpa) = self.field_ptr_allocas[*dest] {
                                    let gep = self.builder.build_load(ptr_ty, fpa, "_fgepl")
                                        .map_err(|e| format!("sfst: {}", e))?;
                                    self.builder.build_store(gep.into_pointer_value(), val)
                                        .map_err(|e| format!("sfst2: {}", e))?;
                                    stored = true;
                                }
                            }
                            if !stored {
                                if let Some(ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    let val = if let Some(dest_type) = self.alloca_types.get(dest) {
                                        if val.get_type() != *dest_type {
                                            self.ssa_cast(val, dest_type).unwrap_or(val)
                                        } else { val }
                                    } else { val };
                                    self.builder.build_store(ptr, val)
                                        .map_err(|e| format!("ssast: {}", e))?;
                                    // Also track in block_vals so ssa_read! can find it for Call args
                                    block_vals[bi].insert(*value, val);
                                    alloca_current.insert(*dest, val);
                                } else {
                                    // Promoted alloca: track in global map AND block_vals
                                    alloca_current.insert(*dest, val);
                                    // KEY FIX: Also insert into block_vals so phi incomings can find it
                                    block_vals[bi].insert(*value, val);
                                }
                            }
                        }
                        SsaInst::Load { dest, src } => {
                            let loaded = if *src < self.field_ptr_allocas.len() && self.field_ptr_allocas[*src].is_some() {
                                if let Some(fpa) = self.field_ptr_allocas[*src] {
                                    let gep = self.builder.build_load(ptr_ty, fpa, "_fgepl")
                                        .map_err(|e| format!("sfld: {}", e))?;
                                    let ft = self.field_ptr_types.get(src).cloned();
                                    let lt = ft.or_else(|| {
                                        let t = func.values.get(*dest).map(|sv| {
                                            eprintln!("DBG Load fallback: dest={} sv_type={}", dest, sv.type_);
                                            self.llvm_type(&sv.type_)
                                        });
                                        if t.is_some() { eprintln!("DBG Load fallback FOUND"); }
                                        t
                                    }).or_else(|| {
                                        self.alloca_types.get(src).copied()
                                    }).unwrap_or(self.context.i64_type().as_basic_type_enum());
                                    Some(self.builder.build_load(lt, gep.into_pointer_value(), "")
                                        .map_err(|e| format!("sfld2: {}", e))?)
                                } else { None }
                            } else if let Some(ptr) = self.alloca_map.get(*src).and_then(|p| *p) {
                                if let Some(lt) = self.alloca_types.get(src) {
                                    Some(self.builder.build_load(*lt, ptr, "")
                                        .map_err(|e| format!("ssald: {}", e))?)
                                } else { None }
                            } else {
                                // Promoted alloca: read from block_vals or global map
                                block_vals[bi].get(src).copied()
                                    .or_else(|| alloca_current.get(src).copied())
                            };
                            if let Some(v) = loaded {
                                block_vals[bi].insert(*dest, v);
                                // Also store to alloca if dest is non-promotable (for resolve_mir! fallback)
                                if let Some(dptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                    if let Some(dt) = self.alloca_types.get(dest) {
                                        let _ = self.builder.build_store(dptr, v);
                                    }
                                }
                            }
                        }
                        SsaInst::BinaryOp { dest, op, left, right } => {
                            let l = ssa_read!(*left);
                            let r = ssa_read!(*right);
                            let is_unsigned = func.values.get(*left).map_or(false, |v| kyc_mir::mir::is_unsigned_type(&v.type_))
                                || func.values.get(*right).map_or(false, |v| kyc_mir::mir::is_unsigned_type(&v.type_));
                            let result = self.ssa_binop(*op, l, r, is_unsigned)?;
                            block_vals[bi].insert(*dest, result);
                        }
                        SsaInst::UnaryOp { dest, op, operand } => {
                            let val = ssa_read!(*operand);
                            let result = match *op {
                                MirUnaryOp::Neg => self.builder.build_int_neg(self.to_int_value(val), ""),
                                MirUnaryOp::Not | MirUnaryOp::BitNot => self.builder.build_not(self.to_int_value(val), ""),
                            }.map_err(|e| format!("ssaun: {}", e))?.as_basic_value_enum();
                            block_vals[bi].insert(*dest, result);
                        }
                        SsaInst::Call { dest, name, args } => {
                            // Inline list operations in SSA path (same as MIR path)
                            match name.as_str() {
                                 "ky_list_get" | "ky_list_set" | "ky_list_len" | "ky_list_pop" => {
                                     let val = self.emit_ssa_inline_list_op(name, &block_vals[bi], args)?;
                                     if let Some(d) = dest {
                                         if let Some(v) = val {
                                             block_vals[bi].insert(*d, v);
                                         }
                                     }
                                     continue;
                                 }
                                 "ky_bytes_get" | "ky_bytes_set" => {
                                     let val = self.emit_ssa_inline_bytes_op(name, args, &block_vals[bi])?;
                                     if let Some(d) = dest {
                                         if let Some(v) = val {
                                             block_vals[bi].insert(*d, v);
                                         }
                                     }
                                     continue;
                                 }
                                 _ => {}
                             }
                            let runtime_name = if self.fn_value_map.contains_key(name) {
                                name.clone()
                            } else {
                                match name.as_str() {
                                "print" => "ky_print", "println" => "ky_println",
                                "contains" => "ky_str_contains", "to_upper" => "ky_str_to_upper",
                                "to_lower" => "ky_str_to_lower", "trim" => "ky_str_trim",
                                "replace" => "ky_str_replace", "input" => "ky_input",
                                "open" => "ky_open", "read_str" => "ky_read_str",
                                "write_str" => "ky_write_str", "close" => "ky_close",
                                "sleep" => "ky_sleep", "now" => "ky_now",
                                "char_at" => "ky_char_at", "is_digit" => "ky_is_digit",
                                "is_alpha" => "ky_is_alpha", "is_alnum" => "ky_is_alnum",
                                "is_whitespace" => "ky_is_whitespace", "is_upper" => "ky_is_upper",
                                "is_lower" => "ky_is_lower", "ord" => "ky_ord",
                                "substr" => "ky_substr",
                                // json_parse/stringify handled via prelude wrappers
                                "assert" => "ky_assert", "assert_eq" => "ky_assert_eq",
                                "assert_ne" => "ky_assert_ne", "assert_str" => "ky_assert_str_eq",
                                "list_new" => "ky_list_new", "list_push" => "ky_list_push",
                                "list_get" => "ky_list_get", "list_set" => "ky_list_set",
                                "list_len" => "ky_list_len", "list_pop" => "ky_list_pop", "reserve" => "ky_list_reserve",
                                "ky_str_builder_new" => "ky_str_builder_new",
                                "ky_str_builder_append" => "ky_str_builder_append",
                                "ky_str_builder_to_str" => "ky_str_builder_to_str",
                                "ky_str_builder_free" => "ky_str_builder_free",
                                _ => name.as_str(),
                                }.to_string()
                            };
                             if self.module.get_function(&runtime_name).is_none() {
                                 let ret_type = if let Some(d) = dest {
                                     let mir_type = func.values.get(*d).map(|v| &v.type_).unwrap_or(&MirType::I64);
                                     self.llvm_type(mir_type).as_basic_type_enum()
                                 } else {
                                     self.context.i64_type().as_basic_type_enum()
                                 };
                                 let param_tys: Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>> = args.iter()
                                     .filter_map(|a| func.values.get(*a))
                                     .map(|v| self.llvm_type(&v.type_).into())
                                     .collect();
                                 let fn_type = ret_type.fn_type(&param_tys, false);
                                 self.module.add_function(&runtime_name, fn_type, None);
                             }
                            if let Some(callee) = self.module.get_function(&runtime_name) {
                                let fn_ty = callee.get_type();
                                let param_tys = fn_ty.get_param_types();
                                let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = args.iter()
                                    .enumerate().map(|(i, a)| {
                                        let v = ssa_read!(*a);
                                        if i < param_tys.len() {
                                            match param_tys[i] {
                                                inkwell::types::BasicMetadataTypeEnum::PointerType(_) => {
                                                    if let BasicValueEnum::IntValue(iv) = v {
                                                        let ptr_ty = self.context.ptr_type(Default::default());
                                                        if let Ok(pv) = self.builder.build_int_to_ptr(iv, ptr_ty, "_aptr") {
                                                            return pv.into();
                                                        }
                                                    }
                                                    // Struct value → pass alloca pointer if available
                                                    if let BasicValueEnum::StructValue(sv) = v {
                                                        // Check if the arg has a non-promotable alloca to pass by reference
                                                        if let Some(Some(ptr)) = self.alloca_map.get(*a) {
                                                            return ptr.as_basic_value_enum().into();
                                                        }
                                                        // Otherwise create temp alloca (pass-by-ref ABI)
                                                        let st = sv.get_type();
                                                        if let Ok(temp) = self.builder.build_alloca(st, "_stmp") {
                                                            let _ = self.builder.build_store(temp, sv);
                                                            return temp.as_basic_value_enum().into();
                                                        }
                                                    }
                                                }
                                                inkwell::types::BasicMetadataTypeEnum::IntType(it) => {
                                                    if let BasicValueEnum::IntValue(iv) = v {
                                                        let vw = iv.get_type().get_bit_width();
                                                        let ew = it.get_bit_width();
                                                        if vw < ew {
                                                            let is_unsigned = func.values.get(*a).map_or(false, |sv| kyc_mir::mir::is_unsigned_type(&sv.type_));
                                                            if is_unsigned {
                                                                if let Ok(ext) = self.builder.build_int_z_extend(iv, it, "_ssac") {
                                                                    return ext.into();
                                                                }
                                                            } else {
                                                                if let Ok(ext) = self.builder.build_int_s_extend(iv, it, "_ssac") {
                                                                    return ext.into();
                                                                }
                                                            }
                                                        } else if vw > ew {
                                                            if let Ok(trunc) = self.builder.build_int_truncate(iv, it, "_sstrunc") {
                                                                return trunc.into();
                                                            }
                                                        }
                                                    }
                                                }
                                                inkwell::types::BasicMetadataTypeEnum::FloatType(ft) => {
                                                    if let BasicValueEnum::FloatValue(fv) = v {
                                                        let src_w = fv.get_type().get_bit_width();
                                                        let dst_w = ft.get_bit_width();
                                                        if src_w > dst_w {
                                                            if let Ok(trunc) = self.builder.build_float_trunc(fv, ft, "_ssaf") {
                                                                return trunc.into();
                                                            }
                                                        } else if src_w < dst_w {
                                                            if let Ok(ext) = self.builder.build_float_ext(fv, ft, "_ssaf") {
                                                                return ext.into();
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        v.into()
                                    }).collect();
                                let call_res = self.builder.build_call(callee, &llvm_args, "")
                                    .map_err(|e| format!("ssacl {}: {}", name, e))?;
                                if let Some(d) = dest {
                                    if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                        block_vals[bi].insert(*d, rv);
                                    }
                                }
                            }
                        }
                        SsaInst::Cast { dest, value, to_type } => {
                            let val = ssa_read!(*value);
                            let target = self.llvm_type(to_type);
                            let result = self.ssa_cast(val, &target)?;
                            block_vals[bi].insert(*dest, result);
                        }
                        SsaInst::FnAddr { dest, name } => {
                            if let Some(fv) = self.fn_value_map.get(name) {
                                block_vals[bi].insert(*dest, fv.as_global_value().as_basic_value_enum());
                            }
                        }
                        SsaInst::AddressOf { dest, local_id } => {
                            if let Some(Some(ptr)) = self.alloca_map.get(*local_id) {
                                block_vals[bi].insert(*dest, ptr.as_basic_value_enum());
                            }
                        }
                        SsaInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                            let raw_val = ssa_read!(*fn_ptr);
                            let fn_ptr = match raw_val {
                                BasicValueEnum::IntValue(iv) => {
                                    let ptr_ty = self.context.ptr_type(Default::default());
                                    self.builder.build_int_to_ptr(iv, ptr_ty, "_ssa_fnptr")
                                        .map_err(|e| format!("ssaic inttoptr: {}", e))?
                                }
                                _ => raw_val.into_pointer_value(),
                            };
                            let llvm_ret = self.llvm_type(ret_type);
                            let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types.iter()
                                .map(|p| self.llvm_type(p).into()).collect();
                            let fn_ty = llvm_ret.fn_type(&llvm_params, false);
                            let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum> = args.iter()
                                .map(|a| { let v = ssa_read!(*a); v.into() }).collect();
                            let call_res = unsafe {
                                self.builder.build_indirect_call(fn_ty, fn_ptr, &llvm_args, "_sicl")
                                    .map_err(|e| format!("ssaic: {}", e))?
                            };
                            if let Some(d) = dest {
                                if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                    block_vals[bi].insert(*d, rv);
                                }
                            }
                        }
                        SsaInst::AsyncSpawn { dest, function_name, arg } => {
                            let arg_val = ssa_read!(*arg);
                            let spawn_fn = self.module.get_function("ky_spawn_task")
                                .ok_or_else(|| "no kl_spawn_task".to_string())?;
                            let fn_val = self.fn_value_map.get(function_name)
                                .ok_or_else(|| format!("async '{}' not found", function_name))?;
                            let args_m: Vec<inkwell::values::BasicMetadataValueEnum> = vec![
                                fn_val.as_global_value().as_pointer_value().into(),
                                arg_val.into(),
                            ];
                            let call_res = self.builder.build_call(spawn_fn, &args_m, "_ssasp")
                                .map_err(|e| format!("ssasp: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                block_vals[bi].insert(*dest, rv);
                            }
                        }
                        SsaInst::AsyncAwait { dest, handle, return_type } => {
                            let handle_val = ssa_read!(*handle);
                            let join_fn = self.module.get_function("ky_await_task")
                                .ok_or_else(|| "no kl_await_task".to_string())?;
                            let args_m: Vec<inkwell::values::BasicMetadataValueEnum> = vec![handle_val.into()];
                            let call_res = self.builder.build_call(join_fn, &args_m, "_ssaaw")
                                .map_err(|e| format!("ssaaw: {}", e))?;
                            if let inkwell::values::ValueKind::Basic(rv) = call_res.try_as_basic_value() {
                                let target_type = self.llvm_type(return_type);
                                let casted = self.cast_to_type(rv, target_type)?;
                                block_vals[bi].insert(*dest, casted);
                            }
                        }
                        SsaInst::PtrOffset { dest, ptr, index, elem_type } => {
                            let base_val = ssa_read!(*ptr);
                            let idx_val = ssa_read!(*index);
                            let ptr_val = match base_val {
                                BasicValueEnum::IntValue(iv) => self.builder.build_int_to_ptr(iv, self.context.ptr_type(Default::default()), "_ptttr")
                                    .map_err(|e| format!("ptroff inttoptr: {}", e))?,
                                BasicValueEnum::PointerValue(pv) => pv,
                                _ => return Err(format!("ptroff: unexpected base for ptr={}", ptr)),
                            };
                            let gep = unsafe {
                                let elem_llvm = self.llvm_type(elem_type);
                                let idx = self.to_int_value(idx_val);
                                // Bitcast i8* to T*, then GEP with proper element type
                                let typed_ptr = self.builder.build_pointer_cast(ptr_val, elem_llvm.ptr_type(Default::default()), "_ssgcast")
                                    .map_err(|e| format!("ssgcast: {}", e))?;
                                self.builder.build_in_bounds_gep(elem_llvm, typed_ptr, &[idx], "_ssgep")
                                    .map_err(|e| format!("ssgep: {}", e))?
                            };
                                     block_vals[bi].insert(*dest, gep.as_basic_value_enum());
                                     if *dest < self.field_ptr_allocas.len() {
                                if let Some(fpa) = self.field_ptr_allocas[*dest] {
                                    let _ = self.builder.build_store(fpa, gep.as_basic_value_enum());
                                }
                            }
                            let elem_llvm = self.llvm_type(elem_type);
                            self.field_ptr_types.insert(*dest, elem_llvm);
                        }
                        SsaInst::PtrStore { ptr, index, value } => {
                            if let Some(base) = self.alloca_map.get(*ptr).and_then(|p| *p) {
                                let idx = ssa_read!(*index);
                                let val = ssa_read!(*value);
                                // If base alloca holds a pointer (^&T ref param), load the pointer first
                                let gep_base = if let Some(ty) = self.alloca_types.get(ptr) {
                                    if ty.is_pointer_type() {
                                        self.builder.build_load(self.context.ptr_type(Default::default()), base, "_ssps_pl")
                                            .ok().map(|v| v.into_pointer_value())
                                    } else { None }
                                } else { None };
                                let gep = unsafe {
                                    self.builder.build_in_bounds_gep(self.context.i8_type(), gep_base.unwrap_or(base), &[self.to_int_value(idx)], "_ssps")
                                        .map_err(|e| format!("ssps gep: {}", e))?
                                };
                                self.builder.build_store(gep, val)
                                    .map_err(|e| format!("ssps store: {}", e))?;
                            }
                        }
                        SsaInst::FieldPtr { dest, ptr, field_index, struct_type } => {
                            let base_ptr = self.alloca_map.get(*ptr).and_then(|p| *p)
                                .or_else(|| {
                                    let val = block_vals.get(bi).and_then(|m| {
                                        func.block_local_map.get(bi)
                                            .and_then(|m2| m2.get(ptr))
                                            .and_then(|sid| m.get(sid))
                                    }).or_else(|| alloca_current.get(ptr));
                                    match val.copied() {
                                        Some(BasicValueEnum::StructValue(sv)) => {
                                            let st = sv.get_type();
                                            self.builder.build_alloca(st, "_stmp").ok()
                                        }
                                        _ => None,
                                    }
                                });
                            if let Some(base) = base_ptr {
                                if let Some(fpa) = self.field_ptr_allocas.get(*dest).and_then(|p| *p) {
                                    let st = self.llvm_type(struct_type);
                                    if let BasicTypeEnum::StructType(s) = st {
                                        let fts = s.get_field_types();
                                        if *field_index < fts.len() {
                                            self.field_ptr_types.insert(*dest, fts[*field_index]);
                                        }
                                    }
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_v = self.context.i32_type().const_int(*field_index as u64, false);
                                    // Check if the alloca stores a pointer to the struct (Ptr(Struct) closure param)
                                    let gep = if self.ref_param_struct_types.contains_key(ptr) {
                                        // Ref param: load pointer from alloca, GEP on struct
                                        let struct_ptr = self.builder.build_load(
                                            self.context.ptr_type(Default::default()), base, "_ssref_load"
                                        ).map_err(|e| format!("ssrefld: {}", e))?;
                                        unsafe {
                                            self.builder.build_in_bounds_gep(st, struct_ptr.into_pointer_value(), &[zero, idx_v], "_ssrfptr")
                                                .map_err(|e| format!("ssrfptr: {}", e))?
                                        }
                                    } else if let Some(&alloca_ty) = self.alloca_types.get(ptr) {
                                        if alloca_ty.is_pointer_type() {
                                            // Ptr(Struct): load pointer from alloca, GEP on struct
                                            if let MirType::Struct(sname, fields) = struct_type.as_ref() {
                                                if !fields.is_empty() {
                                                    let struct_llvm = self.llvm_type(&MirType::Struct(sname.clone(), fields.clone()));
                                                    let struct_ptr = self.builder.build_load(
                                                        self.context.ptr_type(Default::default()), base, "_ssptrld"
                                                    ).map_err(|e| format!("ssptrld: {}", e))?;
                                                    unsafe {
                                                        self.builder.build_in_bounds_gep(struct_llvm, struct_ptr.into_pointer_value(), &[zero, idx_v], "_sspptr")
                                                            .map_err(|e| format!("sspptr: {}", e))?
                                                    }
                                                } else {
                                                    unsafe {
                                                        self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                            .map_err(|e| format!("ssfptr: {}", e))?
                                                    }
                                                }
                                            } else {
                                                unsafe {
                                                    self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                        .map_err(|e| format!("ssfptr: {}", e))?
                                                }
                                            }
                                        } else {
                                            unsafe {
                                                self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                    .map_err(|e| format!("ssfptr: {}", e))?
                                            }
                                        }
                                    } else {
                                        unsafe {
                                            self.builder.build_in_bounds_gep(st, base, &[zero, idx_v], "_ssfptr")
                                                .map_err(|e| format!("ssfptr: {}", e))?
                                        }
                                    };
                                    self.builder.build_store(fpa, gep)
                                        .map_err(|e| format!("ssfptr2: {}", e))?;
                                }
                            }
                        }
                        SsaInst::ArrayElemPtr { dest, mir_dest, ptr, index, arr_type, elem_type } => {
                            let base_ptr = if *ptr < self.field_ptr_allocas.len() && self.field_ptr_allocas[*ptr].is_some() {
                                let p = self.field_ptr_allocas[*ptr].unwrap();
                                Some(self.builder.build_load(
                                    self.context.ptr_type(Default::default()), p, "_ssaebase"
                                ).map_err(|e| format!("ssaebase: {}", e))?.as_basic_value_enum())
                            } else {
                                self.alloca_map.get(*ptr).and_then(|p| *p).map(|p| p.as_basic_value_enum())
                            };
                            if let Some(base) = base_ptr {
                                if let BasicValueEnum::PointerValue(base_pv) = base {
                                    let arr_llvm = self.llvm_type(arr_type);
                                    let zero = self.context.i32_type().const_zero();
                                    let idx_val = block_vals[bi].get(index).copied()
                                        .or_else(|| func.const_values.get(index).map(|c| self.constant_to_llvm(c)))
                                        .or_else(|| self.alloca_map.get(*index).and_then(|p| *p).map(|p| p.as_basic_value_enum()))
                                        .unwrap_or(self.context.i32_type().const_zero().as_basic_value_enum());
                                    let idx_i32 = if let BasicValueEnum::IntValue(iv) = idx_val {
                                        if iv.get_type().get_bit_width() != 32 {
                                            self.builder.build_int_truncate(iv, self.context.i32_type(), "_ssaetrunc")
                                                .map_err(|e| format!("ssaetrunc: {}", e))?
                                        } else { iv }
                                    } else {
                                        self.context.i32_type().const_zero()
                                    };
                                    let gep = unsafe {
                                        self.builder.build_in_bounds_gep(arr_llvm, base_pv, &[zero, idx_i32], "_ssaelem")
                                            .map_err(|e| format!("ssaelem: {}", e))?
                                    };
                                    block_vals[bi].insert(*dest, gep.as_basic_value_enum());
                                    if *dest < self.field_ptr_allocas.len() {
                                        if let Some(fpa) = self.field_ptr_allocas[*dest] {
                                            self.builder.build_store(fpa, gep)
                                                .map_err(|e| format!("aelem fpa store: {}", e))?;
                                        }
                                    }
                                     // Also store to alloca_map for ssa_read! fallback
                                     if let Some(dest_ptr) = self.alloca_map.get(*dest).and_then(|p| *p) {
                                         let _ = self.builder.build_store(dest_ptr, gep.as_basic_value_enum());
                                     }
                                     // Track at mir_dest for SsaInst::Store fallback
                                     while self.field_ptr_allocas.len() <= *mir_dest { self.field_ptr_allocas.push(None); }
                                     if self.field_ptr_allocas[*mir_dest].is_none() {
                                         self.field_ptr_allocas[*mir_dest] = Some(
                                             self.builder.build_alloca(self.context.ptr_type(Default::default()), "_aelem_mir")
                                                 .map_err(|e| format!("ssa aep mir: {}", e))?
                                         );
                                     }
                                     if let Some(fpa) = self.field_ptr_allocas[*mir_dest] {
                                         self.builder.build_store(fpa, gep)
                                             .map_err(|e| format!("aelem mir store: {}", e))?;
                                     }
                                     let elem_llvm = self.llvm_type(elem_type);
                                     self.field_ptr_types.insert(*dest, elem_llvm);
                                     self.field_ptr_types.insert(*mir_dest, elem_llvm);
                                }
                            }
                        }
                        SsaInst::Memcpy { dest_ptr_local, src_alloca_local, struct_type } => {
                            if let Some(dp) = self.alloca_map.get(*dest_ptr_local).and_then(|p| *p) {
                                if let Some(sp) = self.alloca_map.get(*src_alloca_local).and_then(|p| *p) {
                                    let st = self.llvm_type(struct_type);
                                    let src_val = self.builder.build_load(st, sp, "_ssmc")
                                        .map_err(|e| format!("ssmcld: {}", e))?;
                                    let dst_gep = self.builder.build_load(ptr_ty, dp, "_ssmd")
                                        .map_err(|e| format!("ssmcds: {}", e))?;
                                    self.builder.build_store(dst_gep.into_pointer_value(), src_val)
                                        .map_err(|e| format!("ssmcst: {}", e))?;
                                }
                            }
                        }
                        SsaInst::SliceMake { dest, ptr, len, elem_type } => {
                            let ptr_val = ssa_read!(*ptr);
                            let len_val = ssa_read!(*len);
                            let slice_type = self.llvm_type(&MirType::Slice(elem_type.clone()));
                            if let BasicTypeEnum::StructType(st) = slice_type {
                                let undef = st.get_undef();
                                let sv = unsafe {
                                    self.builder.build_insert_value(undef, ptr_val, 0, "_smi0")
                                        .map_err(|e| format!("smi0: {}", e))?
                                };
                                let sv = unsafe {
                                    self.builder.build_insert_value(sv, len_val, 1, "_smi1")
                                        .map_err(|e| format!("smi1: {}", e))?
                                };
                                let bv = sv.as_basic_value_enum();
                                block_vals[bi].insert(*dest, bv);
                            }
                        }
                    }
                }

                // Terminator
                match &block.terminator {
                    MirTerminator::Return(val) => {
                        let ret = resolve_mir!(val);
                        // Auto-cast return value if it doesn't match function return type
                        let ret_ty = fn_value.get_type().get_return_type();
                        let ret = if let Some(expected_ret_ty) = ret_ty {
                            if ret.get_type() != expected_ret_ty.as_basic_type_enum() {
                                match (&ret, &expected_ret_ty) {
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::PointerType(pt)) =>
                                        self.builder.build_int_to_ptr(*iv, *pt, "_retptr")
                                            .map_err(|e| format!("ssa ret inttoptr: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::PointerValue(pv), BasicTypeEnum::IntType(it)) =>
                                        self.builder.build_ptr_to_int(*pv, *it, "_retint")
                                            .map_err(|e| format!("ssa ret ptrtoint: {}", e))?
                                            .as_basic_value_enum(),
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(it)) => {
                                        let sw = iv.get_type().get_bit_width();
                                        let dw = it.get_bit_width();
                                        if sw == 1 && dw > 1 {
                                            self.builder.build_int_z_extend(*iv, *it, "_retzext")
                                                .map_err(|e| format!("ssa ret zext: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            self.builder.build_int_cast(*iv, *it, "_retcast")
                                                .map_err(|e| format!("ssa ret cast: {}", e))?
                                                .as_basic_value_enum()
                                        }
                                    }
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::StructType(st)) => {
                                        let ptr_ty = self.context.ptr_type(Default::default());
                                        let ptr_val = self.builder.build_int_to_ptr(*iv, ptr_ty, "_retptr")
                                            .map_err(|e| format!("ssa ret inttoptr: {}", e))?;
                                        self.builder.build_load(*st, ptr_val, "_retstruct")
                                            .map_err(|e| format!("ssa ret load struct: {}", e))?
                                    }
                                    (BasicValueEnum::FloatValue(fv), BasicTypeEnum::IntType(it)) => {
                                        let fw = fv.get_type().get_bit_width();
                                        let dw = it.get_bit_width();
                                        if fw == dw as u32 {
                                            self.builder.build_bit_cast(*fv, *it, "_retfbc")
                                                .map_err(|e| format!("ssa ret fbitcast: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            self.builder.build_float_to_signed_int(*fv, *it, "_retfptosi")
                                                .map_err(|e| format!("ssa ret fptosi: {}", e))?
                                                .as_basic_value_enum()
                                        }
                                    }
                                    (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(ft)) => {
                                        self.builder.build_signed_int_to_float(*iv, *ft, "_retsitofp")
                                            .map_err(|e| format!("ssa ret sitofp: {}", e))?
                                            .as_basic_value_enum()
                                    }
                                    (BasicValueEnum::FloatValue(fv), BasicTypeEnum::FloatType(ft)) => {
                                        let fw = fv.get_type().get_bit_width();
                                        let dw = ft.get_bit_width();
                                        if fw > dw {
                                            self.builder.build_float_trunc(*fv, *ft, "_retftrunc")
                                                .map_err(|e| format!("ssa ret ftrunc: {}", e))?
                                                .as_basic_value_enum()
                                        } else if fw < dw {
                                            self.builder.build_float_ext(*fv, *ft, "_retfext")
                                                .map_err(|e| format!("ssa ret fext: {}", e))?
                                                .as_basic_value_enum()
                                        } else {
                                            ret
                                        }
                                    }
                                    _ => ret,
                                }
                            } else { ret }
                        } else { ret };
                        self.builder.build_return(Some(&ret))
                            .map_err(|e| format!("ssaret: {}", e))?;
                    }
                    MirTerminator::Br(label) => {
                        if let Some(&tb) = block_map.get(label) {
                            let br = self.builder.build_unconditional_branch(tb)
                                .map_err(|e| format!("ssabr: {}", e))?;
                            // Mark loop back-edges for LLVM optimizations (vectorize, unroll)
                            if let Some(target_idx) = block_indices.get(label) {
                                if *target_idx < bi {
                                    let loop_md = self.context.metadata_node(&[
                                        self.context.metadata_string("llvm.loop.vectorize.enable").into(),
                                    ]);
                                    let kind = self.context.get_kind_id("llvm.loop");
                                    let _ = br.set_metadata(loop_md, kind);
                                }
                            }
                        }
                    }
                    MirTerminator::CondBr { cond, true_block, false_block } => {
                        let cond_val = resolve_mir!(cond);
                        let cond_int = self.to_int_value(cond_val);
                        let i1_cond = if cond_int.get_type().get_bit_width() > 1 {
                            let i1_ty = self.context.bool_type();
                            self.builder.build_int_truncate(cond_int, i1_ty, "")
                                .map_err(|e| format!("cond trunc: {}", e))?
                        } else {
                            cond_int
                        };
                        if let (Some(&tb), Some(&fb)) = (block_map.get(true_block), block_map.get(false_block)) {
                            let br = self.builder.build_conditional_branch(i1_cond, tb, fb)
                                .map_err(|e| format!("ssacbr: {}", e))?;
                            // Mark loop back-edges (conditional branch to previous block)
                            if let Some(target_idx) = block_indices.get(true_block) {
                                if *target_idx < bi {
                                    let loop_md = self.context.metadata_node(&[
                                        self.context.metadata_string("llvm.loop.vectorize.enable").into(),
                                    ]);
                                    let kind = self.context.get_kind_id("llvm.loop");
                                    let _ = br.set_metadata(loop_md, kind);
                                }
                            }
                            if let Some(target_idx) = block_indices.get(false_block) {
                                if *target_idx < bi {
                                    let loop_md = self.context.metadata_node(&[
                                        self.context.metadata_string("llvm.loop.vectorize.enable").into(),
                                    ]);
                                    let kind = self.context.get_kind_id("llvm.loop");
                                    let _ = br.set_metadata(loop_md, kind);
                                }
                            }
                        }
                    }
                    MirTerminator::Unreachable => {
                        self.builder.build_unreachable()
                            .map_err(|e| format!("unreachable: {}", e))?;
                    }
                }
            }
        }

        // Pass 3: phi incomings — fill from block_vals + alloca_current
        for (bi, block) in func.blocks.iter().enumerate() {
            for phi in &block.phis {
                if let Some(phi_entry) = phi_map.iter().find(|(id, _)| *id == phi.dest) {
                    let mut incomings: Vec<(BasicValueEnum<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> = Vec::new();
                    for &(val_id, ref pred_label) in &phi.incomings {
                        if let Some(pred_bb) = block_map.get(pred_label).copied() {
                            if let Some(pred_bi) = func.blocks.iter().position(|b| &b.label == pred_label) {
                                 let mut val = block_vals[pred_bi].get(&val_id).copied()
                                     .or_else(|| {
                                         block_vals[..pred_bi].iter().rev()
                                             .filter_map(|bv| bv.get(&val_id).copied()).next()
                                     })
                                     .or_else(|| {
                                         func.const_values.get(&val_id)
                                             .map(|c| self.constant_to_llvm(c))
                                     })
                                     .unwrap_or_else(|| {
                                         let phi_type = self.llvm_type(&phi.type_);
                                         match phi_type {
                                             BasicTypeEnum::IntType(it) => it.const_zero().as_basic_value_enum(),
                                             BasicTypeEnum::FloatType(ft) => ft.const_zero().as_basic_value_enum(),
                                             _ => self.context.i32_type().const_zero().as_basic_value_enum(),
                                         }
                                     });
                                 let phi_type = self.llvm_type(&phi.type_);
                                 if val.get_type() != phi_type {
                                     if let Some(term) = pred_bb.get_terminator() {
                                         self.builder.position_before(&term);
                                     }
                                     val = self.ssa_cast(val, &phi_type).unwrap_or(val);
                                 }
                                 incomings.push((val, pred_bb));
                            }
                        }
                    }
                    if !incomings.is_empty() {
                        let refs: Vec<(&dyn BasicValue<'ctx>, inkwell::basic_block::BasicBlock<'ctx>)> =
                            incomings.iter().map(|(v, b)| {
                                let bv: &dyn BasicValue = v;
                                (bv, *b)
                            }).collect();
                        phi_entry.1.add_incoming(&refs);
                    }
                }
            }
        }

        Ok(())
    }
}
