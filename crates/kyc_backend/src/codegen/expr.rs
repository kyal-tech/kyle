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
    /// Convert a BasicValueEnum to an IntValue, converting pointer to int if needed.
    pub(crate) fn to_int_value(&self, val: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        match val {
            BasicValueEnum::IntValue(i) => i,
            BasicValueEnum::PointerValue(p) => {
                self.builder.build_ptr_to_int(p, self.context.i64_type(), "")
                    .expect("ptrtoint")
            }
            _ => self.context.i32_type().const_zero(),
        }
    }

    pub(crate) fn to_float_value(&self, val: BasicValueEnum<'ctx>) -> inkwell::values::FloatValue<'ctx> {
        match val {
            BasicValueEnum::FloatValue(f) => f,
            BasicValueEnum::IntValue(i) => {
                self.builder.build_signed_int_to_float(i, self.context.f64_type(), "")
                    .expect("inttofloat")
            }
            _ => self.context.f64_type().const_zero(),
        }
    }

    pub(crate) fn value_to_llvm(
        &self,
        value: &MirValue,
        last_values: &HashMap<usize, BasicValueEnum<'ctx>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            MirValue::Local(id) => self.load_value(*id, last_values),
            MirValue::Param(id) => {
                Ok(self.param_values.get(id).copied().unwrap_or_else(|| {
                    self.context.i32_type().const_zero().as_basic_value_enum()
                }))
            }
            MirValue::Constant(c) => Ok(self.constant_to_llvm(c)),
        }
    }

    // SSA Helper: binary operation
    pub(crate) fn ssa_binop(&self, op: MirBinaryOp, l: BasicValueEnum<'ctx>, r: BasicValueEnum<'ctx>, is_unsigned: bool) -> Result<BasicValueEnum<'ctx>, String> {
        let to_float = |v: BasicValueEnum<'ctx>| -> inkwell::values::FloatValue<'ctx> {
            if let BasicValueEnum::FloatValue(f) = v { f }
            else { self.builder.build_signed_int_to_float(self.to_int_value(v), self.context.f64_type(), "").unwrap() }
        };
        let to_int = |v: BasicValueEnum<'ctx>| -> inkwell::values::IntValue<'ctx> {
            self.to_int_value(v)
        };
        let l_is_float = matches!(l, BasicValueEnum::FloatValue(_));
        let r_is_float = matches!(r, BasicValueEnum::FloatValue(_));
        if l_is_float || r_is_float {
            let lf = to_float(l); let rf = to_float(r);
            Ok(match op {
                MirBinaryOp::Add => self.builder.build_float_add(lf, rf, "").map_err(|e| format!("fadd: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Sub => self.builder.build_float_sub(lf, rf, "").map_err(|e| format!("fsub: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Mul => self.builder.build_float_mul(lf, rf, "").map_err(|e| format!("fmul: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Div => self.builder.build_float_div(lf, rf, "").map_err(|e| format!("fdiv: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Rem => self.builder.build_float_rem(lf, rf, "").map_err(|e| format!("frem: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Eq => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lf, rf, "").map_err(|e| format!("feq: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("feqe: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Neq => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lf, rf, "").map_err(|e| format!("fne: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fnee: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Lt => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lf, rf, "").map_err(|e| format!("flt: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("flte: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Gt => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lf, rf, "").map_err(|e| format!("fgt: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fgte: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Le => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OLE, lf, rf, "").map_err(|e| format!("fle: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("flee: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                MirBinaryOp::Ge => { let c = self.builder.build_float_compare(inkwell::FloatPredicate::OGE, lf, rf, "").map_err(|e| format!("fge: {}", e))?;
                    let e = self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("fgee: {}", e))?; self.add_bool_range(e); e.as_basic_value_enum() }
                _ => return Err("ssa: unsupported float op".into()),
            })
        } else {
            let li = to_int(l); let ri = to_int(r);
            // Auto-widen mismatched integer widths
            let lw = li.get_type().get_bit_width();
            let rw = ri.get_type().get_bit_width();
            let widen = |val: inkwell::values::IntValue<'ctx>, target_w: u32| -> Option<inkwell::values::IntValue<'ctx>> {
                let ty = match target_w { 8 => self.context.i8_type(), 16 => self.context.i16_type(), 32 => self.context.i32_type(), 64 => self.context.i64_type(), _ => return None };
                if is_unsigned {
                    self.builder.build_int_z_extend(val, ty, "_ssaw").ok()
                } else {
                    self.builder.build_int_s_extend(val, ty, "_ssaw").ok()
                }
            };
            let (li, ri) = if lw < rw {
                (widen(li, rw).unwrap_or(li), ri)
            } else if rw < lw {
                (li, widen(ri, lw).unwrap_or(ri))
            } else { (li, ri) };
            Ok(match op {
                MirBinaryOp::Add => self.int_nsw_nuw_add(li, ri).map_err(|e| format!("iadd: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Sub => self.int_nsw_nuw_sub(li, ri).map_err(|e| format!("isub: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Mul => self.int_nsw_nuw_mul(li, ri).map_err(|e| format!("imul: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Div => {
                    if is_unsigned {
                        self.builder.build_int_unsigned_div(li, ri, "").map_err(|e| format!("udiv: {}", e))?.as_basic_value_enum()
                    } else {
                        self.builder.build_int_signed_div(li, ri, "").map_err(|e| format!("idiv: {}", e))?.as_basic_value_enum()
                    }
                }
                MirBinaryOp::Rem => {
                    if is_unsigned {
                        self.builder.build_int_unsigned_rem(li, ri, "").map_err(|e| format!("urem: {}", e))?.as_basic_value_enum()
                    } else {
                        self.builder.build_int_signed_rem(li, ri, "").map_err(|e| format!("irem: {}", e))?.as_basic_value_enum()
                    }
                }
                MirBinaryOp::And => self.builder.build_and(li, ri, "").map_err(|e| format!("iand: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Or => self.builder.build_or(li, ri, "").map_err(|e| format!("ior: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Xor => self.builder.build_xor(li, ri, "").map_err(|e| format!("ixor: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Shl => self.builder.build_left_shift(li, ri, "").map_err(|e| format!("ishl: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Shr => self.builder.build_right_shift(li, ri, !is_unsigned, "").map_err(|e| format!("ishr: {}", e))?.as_basic_value_enum(),
                MirBinaryOp::Eq => { let c = self.builder.build_int_compare(inkwell::IntPredicate::EQ, li, ri, "").map_err(|e| format!("ieq: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("iez: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Neq => { let c = self.builder.build_int_compare(inkwell::IntPredicate::NE, li, ri, "").map_err(|e| format!("ine: {}", e))?; self.add_bool_range(c); self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("inz: {}", e))?.as_basic_value_enum() }
                MirBinaryOp::Lt => {
                    let p = if is_unsigned { inkwell::IntPredicate::ULT } else { inkwell::IntPredicate::SLT };
                    let c = self.builder.build_int_compare(p, li, ri, "").map_err(|e| format!("ilt: {}", e))?;
                    self.add_bool_range(c);
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("ilz: {}", e))?.as_basic_value_enum()
                }
                MirBinaryOp::Gt => {
                    let p = if is_unsigned { inkwell::IntPredicate::UGT } else { inkwell::IntPredicate::SGT };
                    let c = self.builder.build_int_compare(p, li, ri, "").map_err(|e| format!("igt: {}", e))?;
                    self.add_bool_range(c);
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("igz: {}", e))?.as_basic_value_enum()
                }
                MirBinaryOp::Le => {
                    let p = if is_unsigned { inkwell::IntPredicate::ULE } else { inkwell::IntPredicate::SLE };
                    let c = self.builder.build_int_compare(p, li, ri, "").map_err(|e| format!("ile: {}", e))?;
                    self.add_bool_range(c);
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("ilz2: {}", e))?.as_basic_value_enum()
                }
                MirBinaryOp::Ge => {
                    let p = if is_unsigned { inkwell::IntPredicate::UGE } else { inkwell::IntPredicate::SGE };
                    let c = self.builder.build_int_compare(p, li, ri, "").map_err(|e| format!("ige: {}", e))?;
                    self.add_bool_range(c);
                    self.builder.build_int_z_extend(c, self.context.i32_type(), "").map_err(|e| format!("igz2: {}", e))?.as_basic_value_enum()
                }
            })
        }
    }

    // SSA Helper: type cast
    pub(crate) fn ssa_cast(&self, val: BasicValueEnum<'ctx>, target: &BasicTypeEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        Ok(match (&val, target) {
            (BasicValueEnum::FloatValue(_), BasicTypeEnum::FloatType(t)) =>
                self.builder.build_float_cast(self.to_float_value(val), *t, "").map_err(|e| format!("fcs: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::IntValue(_), BasicTypeEnum::IntType(t)) => {
                let vi = self.to_int_value(val);
                let sw = vi.get_type().get_bit_width();
                let dw = t.get_bit_width();
                (if sw < dw { self.builder.build_int_s_extend(vi, *t, "") }
                 else if sw > dw { self.builder.build_int_truncate(vi, *t, "") }
                 else { self.builder.build_int_cast(vi, *t, "") })
                    .map_err(|e| format!("ics: {}", e))?.as_basic_value_enum()
            }
            (BasicValueEnum::IntValue(_), BasicTypeEnum::FloatType(t)) =>
                self.builder.build_signed_int_to_float(self.to_int_value(val), *t, "").map_err(|e| format!("itof: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::FloatValue(_), BasicTypeEnum::IntType(t)) =>
                self.builder.build_float_to_signed_int(self.to_float_value(val), *t, "").map_err(|e| format!("ftoi: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::PointerValue(_), BasicTypeEnum::IntType(t)) =>
                self.builder.build_ptr_to_int(val.into_pointer_value(), *t, "").map_err(|e| format!("ptoi: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::IntValue(_), BasicTypeEnum::PointerType(t)) =>
                self.builder.build_int_to_ptr(self.to_int_value(val), *t, "").map_err(|e| format!("itop: {}", e))?.as_basic_value_enum(),
            (BasicValueEnum::StructValue(struct_val), BasicTypeEnum::IntType(t)) => {
                // Struct → i64: bitcast the struct pointer to i64 pointer, then load
                let struct_ty = struct_val.get_type();
                let temp_alloca = self.builder.build_alloca(struct_ty, "_tmp_struct")
                    .map_err(|e| format!("alloca: {}", e))?;
                self.builder.build_store(temp_alloca, *struct_val)
                    .map_err(|e| format!("store struct: {}", e))?;
                let ptr_ty = t.ptr_type(Default::default());
                let casted_ptr = self.builder.build_pointer_cast(temp_alloca, ptr_ty, "_bitcast")
                    .map_err(|e| format!("bitcast: {}", e))?;
                let loaded = self.builder.build_load(*t, casted_ptr, "_loadstruct")
                    .map_err(|e| format!("loadstruct: {}", e))?;
                loaded
            }
            _ => val,
        })
    }

    pub(crate) fn constant_to_llvm(&self, c: &MirConstant) -> BasicValueEnum<'ctx> {
        match c {
            MirConstant::I32(n) => self.context.i32_type().const_int(*n as u64, false).as_basic_value_enum(),
            MirConstant::I64(n) => self.context.i64_type().const_int(*n as u64, false).as_basic_value_enum(),
            MirConstant::F64(n) => self.context.f64_type().const_float(*n).as_basic_value_enum(),
            MirConstant::Bool(b) => self.context.bool_type().const_int(*b as u64, false).as_basic_value_enum(),
            MirConstant::String(s) => {
                self.builder.build_global_string_ptr(s, "")
                    .expect("global string ptr")
                    .as_pointer_value()
                    .as_basic_value_enum()
            }
            MirConstant::Void => self.context.i32_type().const_zero().as_basic_value_enum(),
            MirConstant::Null => self.context.ptr_type(Default::default()).const_null().as_basic_value_enum(),
        }
    }

    /// Cast an i64 value to the specified LLVM target type (used for async await results)
    pub(crate) fn cast_to_type(
        &self,
        val: BasicValueEnum<'ctx>,
        target: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match (&val, &target) {
            (BasicValueEnum::IntValue(iv), BasicTypeEnum::IntType(t)) => {
                let src_w = iv.get_type().get_bit_width();
                let dst_w = t.get_bit_width();
                if src_w >= dst_w {
                    self.builder.build_int_truncate(*iv, *t, "")
                        .map(|v| v.as_basic_value_enum())
                        .map_err(|e| format!("cast trunc: {}", e))
                } else {
                    self.builder.build_int_z_extend(*iv, *t, "")
                        .map(|v| v.as_basic_value_enum())
                        .map_err(|e| format!("cast zext: {}", e))
                }
            }
            (BasicValueEnum::IntValue(iv), BasicTypeEnum::PointerType(pt)) => {
                self.builder.build_int_to_ptr(*iv, *pt, "")
                    .map(|v| v.as_basic_value_enum())
                    .map_err(|e| format!("cast inttoptr: {}", e))
            }
            (BasicValueEnum::IntValue(iv), BasicTypeEnum::FloatType(ft)) => {
                // Bitcast the i64 bits to the float type to preserve the exact value
                let i64_ty = self.context.i64_type();
                if iv.get_type().get_bit_width() == 64 {
                    self.builder.build_bit_cast(*iv, *ft, "")
                        .map(|v| v.as_basic_value_enum())
                        .map_err(|e| format!("cast bitcast float: {}", e))
                } else {
                    self.builder.build_signed_int_to_float(*iv, *ft, "")
                        .map(|v| v.as_basic_value_enum())
                        .map_err(|e| format!("cast inttofloat: {}", e))
                }
            }
            _ => Ok(val),  // Pass through if types match or unsupported
        }
    }
}
