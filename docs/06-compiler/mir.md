# MIR (Mid-Level IR)

> Translation del AST tipado a una representscion de bajo nivel with instruccionis simples.
> Crate: `kyc_mir/src/lower.rs` (7740 lines).

## Responsabilidad

El MIR lowering transforma AST en una representscion intermedia de bajo nivel
que opera with instruccionis similaris a una maquina virtual: alloca, load, store,
call, branch, etc. Esta is fase more grande y compleja del compiler.

## Typis MIR

```rust
 enum MirType {
 I1, I8, I16, I32, I64, // Signed integers
 F32, F64, // Floating point
 Bool, Char, // Boolean, character
 Str, // String pointer
 Void, Ptr(Box<MirType>), // Void, typed pointer
 Array(Box<MirType>, usize), // [T, N]
 List(Box<MirType>), // {T}
 Dict(Box<MirType>, Box<MirType>), // {K: V}
 Struct(String, Vec<(String, MirType)>), // Named struct
}
```

## Instruccionis MIR

```rust
 enum MirInst {
 // Memory
 Alloca { dest: usize, type_: MirType },
 Load { dest: usize, src: usize },
 Store { dest: usize, value: MirValue },
 
 // Operations
 BinaryOp { dest: usize, op: MirBinaryOp, left: MirValue, right: MirValue },
 UnaryOp { dest: usize, op: MirUnaryOp, operand: MirValue },
 Cast { dest: usize, value: MirValue, to_type: MirType },
 
 // Control flow
 Call { dest: Option<usize>, name: String, args: Vec<MirValue> },
 
 // Struct/Array access
 FieldPtr { dest: usize, ptr: usize, field_index: usize, struct_type: Box<MirType> },
 ArrayElemPtr { dest: usize, ptr: usize, index: MirValue, arr_type: Box<MirType>, elem_type: Box<MirType> },
 
 // Pointers
 PtrOffset { dest: usize, ptr: usize, index: MirValue },
 PtrStore { dest: usize, ptr: usize, index: MirValue, value: MirValue },
 
 // Functions
 FnAddr { dest: usize, name: String },
 CallIndirect { dest: Option<usize>, fn_ptr: MirValue, args: Vec<MirValue> },
 
 // Async
 AsyncSpawn { dest: usize, fn_name: String, arg: MirValue },
 AsyncAwait { dest: usize, handle: MirValue },
 
 // Memory copy
 Memcpy { dest_ptr_local: usize, src_alloca_local: usize, struct_type: Box<MirType> },
}
```

## Lowering: example concreto

```ky
fn add(a: i32, b: i32) i32:
 a + b
```

Se traduce a MIR:

```rust
// Alloca for parameters y resultado
Alloca { dest: 0, type: I32 } // a (param)
Alloca { dest: 1, type: I32 } // b (param)
Alloca { dest: 2, type: I32 } // resultado temporal

// Guardar parameters
Store { dest: 0, value: Param(0) } // a
Store { dest: 1, value: Param(1) } // b

// a + b
Load { dest: 3, src: 0 } // cargar a
Load { dest: 4, src: 1 } // cargar b
BinaryOp { dest: 5, op: Add, left: Local(3), right: Local(4) }
Store { dest: 2, value: Local(5) }

// return a + b
Load { dest: 6, src: 2 }
Return { value: Local(6) }
```

## Lowering de estructuras de control

### If/Else

```ky
if x > 0:
 println("pos")
else:
 println("neg")
```

```
 BinaryOp { cond, Gt, x, 0 }
 CondBr { cond, true_block: "bb1", false_block: "bb2" }
bb1:
 Call { "ky_println", ["pos"] }
 Br "bb3"
bb2:
 Call { "ky_println", ["neg"] }
 Br "bb3"
bb3:
 ...
```

### While

```ky
while i < n:
 ...
```

```
 Br "bb_check"
bb_check:
 Load i; Load n; BinaryOp Lt
 CondBr { cond, true_block: "bb_body", false_block: "bb_done" }
bb_body:
 ...
 i += 1
 Br "bb_check"
bb_done:
 ...
```

## Manejo de typis especiales

### ^T (mutable)

El marcador `^` is puramente semantico. En MIR, `^T` se resuelve al type base `T`.
No genera code diferente.

### &T (borrow)

Los borrows se pasan as direccionis de alloca directamente, without generar Load.
El code gen genera GEP about alloca original en vez de about una copia.

### str (Move type)

Los strings se manejan as pointers a heap. El borrow analysis inserta
llamadas a `ky_free` cuando un string sale de scope.

## See also

- `borrow-analysis.md` — Analysis de ownership post-MIR
- `ssa.md` — Transformation SSA post-MIR
- `codegen.md` — Translation de MIR a LLVM IR
