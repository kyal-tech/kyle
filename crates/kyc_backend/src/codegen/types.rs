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
    pub(crate) fn llvm_type(&self, mir_type: &MirType) -> BasicTypeEnum<'ctx> {
        match mir_type {
            MirType::I1 => self.context.bool_type().as_basic_type_enum(),
            MirType::I8 => self.context.i8_type().as_basic_type_enum(),
            MirType::U8 => self.context.i8_type().as_basic_type_enum(),
            MirType::I16 => self.context.i16_type().as_basic_type_enum(),
            MirType::U16 => self.context.i16_type().as_basic_type_enum(),
            MirType::I32 => self.context.i32_type().as_basic_type_enum(),
            MirType::U32 => self.context.i32_type().as_basic_type_enum(),
            MirType::I64 => self.context.i64_type().as_basic_type_enum(),
            MirType::U64 => self.context.i64_type().as_basic_type_enum(),
            MirType::F32 => self.context.f32_type().as_basic_type_enum(),
            MirType::F64 => self.context.f64_type().as_basic_type_enum(),
            MirType::Bool => self.context.i8_type().as_basic_type_enum(),
            MirType::Char => self.context.i32_type().as_basic_type_enum(),
            MirType::Str | MirType::Box(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Deque(_) | MirType::LinkedList(_) | MirType::Chan(_) | MirType::Bytes => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::Void => self.context.i32_type().as_basic_type_enum(),
            MirType::Ptr(_) => self.context.ptr_type(Default::default()).as_basic_type_enum(),
            MirType::Array(inner, size) => {
                let base = self.llvm_type(inner);
                base.array_type(*size as u32).as_basic_type_enum()
            }
            MirType::Struct(name, fields) => {
                // Reuse existing struct type to avoid creating duplicate types
                // with numbered suffixes (e.g. Token.1, Token.2, ...)
                let struct_ty = self.module.get_struct_type(name)
                    .unwrap_or_else(|| {
                        let new_ty = self.context.opaque_struct_type(name);
                        let field_types: Vec<BasicTypeEnum<'ctx>> = fields.iter()
                            .map(|(_, ty)| self.llvm_type(ty))
                            .collect();
                        new_ty.set_body(&field_types, false);
                        new_ty
                    });
                struct_ty.as_basic_type_enum()
            }
            MirType::Slice(inner) => {
                let inner_str = format!("{}", inner);
                let name = format!("__slice_{}", inner_str);
                let struct_ty = self.module.get_struct_type(&name)
                    .unwrap_or_else(|| {
                        let ty = self.context.opaque_struct_type(&name);
                        let ptr_ty = self.llvm_type(inner).ptr_type(Default::default());
                        let len_ty = self.context.i64_type();
                        ty.set_body(&[ptr_ty.into(), len_ty.into()], false);
                        ty
                    });
                struct_ty.as_basic_type_enum()
            }
        }
    }

}
