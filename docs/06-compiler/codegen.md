# LLVM Codegen

> Generation de code LLVM IR from MIR/SSA.
> Crate: `kyc_backend/src/codegen.rs` (2942 lines).

## Responsabilidad

Traduce instruccionis MIR a LLVM IR, que luego LLVM compila a code maquina
nativo. Es fase final del compiler before del linking.

## Arquitectura

```rust
 struct Codegen<'ctx> {
 context: &'ctx Context,
 module: Module<'ctx>,
 builder: Builder<'ctx>,
 alloca_map: Vec<Option<PointerValue<'ctx>>>,
 alloca_types: HashMap<usize, BasicTypeEnum<'ctx>>,
 fn_value_map: HashMap<String, FunctionValue<'ctx>>,
 field_ptr_allocas: Vec<Option<PointerValue<'ctx>>>,
 field_ptr_types: HashMap<usize, BasicTypeEnum<'ctx>>,
 param_values: HashMap<usize, BasicValueEnum<'ctx>>,
}
```

## Type lowering

```rust
fn llvm_type(&self, mir_type: &MirType) -> BasicTypeEnum<'ctx> {
 match mir_type {
 MirType::I32 => context.i32_type().as_basic_type_enum(),
 MirType::I64 => context.i64_type().as_basic_type_enum(),
 MirType::F64 => context.f64_type().as_basic_type_enum(),
 MirType::Bool => context.bool_type().as_basic_type_enum(),
 MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Ptr(_) => {
 context.ptr_type(Default::default()).as_basic_type_enum()
 }
 MirType::Array(inner, size) => {
 let base = self.llvm_type(inner);
 base.array_type(*size as u32).as_basic_type_enum()
 }
 MirType::Struct(name, fields) => {
 // Create or reuse LLVM struct type
 let struct_ty = module.get_struct_type(name)
 .unwrap_or_else(|| {
 let new_ty = context.opaque_struct_type(name);
 let field_typis = fields.iter().map(|(_, ty)| self.llvm_type(ty)).collect();
 new_ty.set_body(&field_types, false);
 new_ty
 });
 struct_ty.as_basic_type_enum()
 }
 }
}
```

## Instruction lowering

| MIR Instruction | LLVM IR |
|----------------|---------|
| `Alloca` | `alloca type` |
| `Load` | `load type, ptr` |
| `Store` | `store value, ptr` |
| `BinaryOp(Add)` | `add nsw i32 %a, %b` |
| `BinaryOp(FAdd)` | `fadd double %a, %b` |
| `Cast(I32→I64)` | `sext i32 %v to i64` |
| `Cast(F64→I32)` | `fptosi double %v to i32` |
| `Call` | `call ret_type @func(args)` |
| `FieldPtr` | `getelementptr inbounds %struct, ptr %base, i32 0, i32 N` |
| `ArrayElemPtr` | `getelementptr inbounds [N x T], ptr %arr, i32 0, i32 idx` |
| `CondBr` | `br i1 %cond, label %true, label %false` |
| `Return` | `ret type %val` |

## LLVM IR Quality

El codegen aplica metadatos for improvementr calidad del IR generado:

### TBAA (Type-Based Alias Analysis)

```llvm
store i32 %val, ptr %ptr, align 4, !tbaa !0
!0 = !{!"int", !1}
!1 = !{!"any level"}
```

### nsw (No Signed Wrap)

```llvm
%result = add nsw i32 %a, %b # allows optimizacionis asumiendo without overflow
```

### inbounds GEP

```llvm
%elem = getelementptr inbounds [3 x i32], ptr %arr, i32 0, i32 %idx
```

### noalias en parameters

```llvm
declare void @process(ptr noalias %input, ptr noalias %output)
```

## Calling convention

- Parameters struct se pasan by referencia (ptr)
- Parameters primitivos by value
- Strings as `ptr` (raw pointer)
- Return valuis primitivos en registros
- Struct returns via sret pointer

## See also

- `ssa.md` — SSA input del codegen
- `backend.md` — LLVM backend configuration
- `linker.md` — Linking post-codegen
