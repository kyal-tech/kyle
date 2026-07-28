// kyc_mir::ssa — SSA Form conversion and representation
//
// Transforms MirFunction (alloca/load/store) into SsaFunction (values + phi).
// This is the key optimization in Phase 15 — eliminating allocas lets LLVM
// optimize Kyle code as aggressively as Rust.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::mir::*;

// ---------------------------------------------------------------------------
// SSA Types
// ---------------------------------------------------------------------------

/// A reference to an SSA value (index into SsaFunction::values).
pub type SsaValueId = usize;

/// An SSA value — a single assignment in the function.
#[derive(Clone, Debug)]
pub struct SsaValue {
    pub type_: MirType,
    pub name: String,
}

/// A phi node at the start of a basic block.
#[derive(Clone, Debug)]
pub struct PhiNode {
    pub dest: SsaValueId,
    pub alloca_id: usize,  // original MIR alloca index (for stack lookups)
    pub incomings: Vec<(SsaValueId, String)>, // (value, from_block_label)
    pub type_: MirType,
}

/// SSA instructions — no load/store for scalar values.
#[derive(Clone, Debug)]
pub enum SsaInst {
    /// dest = left op right
    BinaryOp { dest: SsaValueId, op: MirBinaryOp, left: SsaValueId, right: SsaValueId },
    /// dest = op operand
    UnaryOp { dest: SsaValueId, op: MirUnaryOp, operand: SsaValueId },
    /// dest = call name(args...)
    Call { dest: Option<SsaValueId>, name: String, args: Vec<SsaValueId> },
    /// dest = cast(value)
    Cast { dest: SsaValueId, value: SsaValueId, to_type: MirType },
    /// dest = &function (for closures)
    FnAddr { dest: SsaValueId, name: String },
    /// dest = address of local (alloca pointer)
    AddressOf { dest: SsaValueId, local_id: usize },
    /// dest = call_indirect(fn_ptr, args)
    CallIndirect { dest: Option<SsaValueId>, fn_ptr: SsaValueId, ret_type: MirType, param_types: Vec<MirType>, args: Vec<SsaValueId> },
    /// dest = spawn_task(func_name, arg)
    AsyncSpawn { dest: SsaValueId, function_name: String, arg: SsaValueId },
    /// dest = await_task(handle)
    AsyncAwait { dest: SsaValueId, handle: SsaValueId, return_type: MirType },
    /// For non-promotable allocas (heap types, field_ptr): store value to alloca
    Store { dest: usize, value: SsaValueId },
    /// For non-promotable allocas: load value from alloca
    Load { dest: SsaValueId, src: usize },
    /// Get pointer to array element (non-promotable)
    PtrOffset { dest: SsaValueId, ptr: usize, index: SsaValueId, elem_type: Box<MirType> },
    /// Store through pointer: ptr[index] = val (non-promotable)
    PtrStore { ptr: usize, index: SsaValueId, value: SsaValueId },
    /// Get pointer to struct field (non-promotable)
    FieldPtr { dest: usize, ptr: usize, field_index: usize, struct_type: Box<MirType> },
    /// Get pointer to array element (non-promotable)
    ArrayElemPtr { dest: usize, mir_dest: usize, ptr: usize, index: SsaValueId, arr_type: Box<MirType>, elem_type: Box<MirType> },
    /// Copy struct memory (non-promotable)
    Memcpy { dest_ptr_local: usize, src_alloca_local: usize, struct_type: Box<MirType> },
    /// Allocate stack space (non-promotable)
    Alloca { dest: usize, type_: MirType, name: String },
    /// Create a slice struct {ptr, len} from ptr and len values
    SliceMake { dest: SsaValueId, ptr: SsaValueId, len: SsaValueId, elem_type: Box<MirType> },
}

/// A basic block in SSA form.
#[derive(Clone, Debug)]
pub struct SsaBlock {
    pub label: String,
    pub phis: Vec<PhiNode>,
    pub insts: Vec<SsaInst>,
    pub terminator: MirTerminator,
}

/// A function in SSA form.
#[derive(Clone, Debug)]
pub struct SsaFunction {
    pub name: String,
    pub params: Vec<MirType>,
    pub return_type: MirType,
    pub is_fallible: bool,
    pub blocks: Vec<SsaBlock>,
    /// All SSA values (results of instructions + params + phi nodes)
    pub values: Vec<SsaValue>,
    /// Local count from the original MirFunction (for non-promotable allocas)
    pub local_count: usize,
    /// Constant values keyed by SsaValueId (created by resolve_value)
    pub const_values: HashMap<SsaValueId, MirConstant>,
    /// Per-block mapping: MIR local ID → SsaValueId at block end (for terminator resolution)
    pub block_local_map: Vec<HashMap<usize, SsaValueId>>,
    /// SsaValueId for each parameter (index = param index)
    pub param_value_ids: Vec<SsaValueId>,
}

/// Module containing SSA functions.
#[derive(Clone, Debug)]
pub struct SsaModule {
    pub functions: Vec<SsaFunction>,
}

// ---------------------------------------------------------------------------
// SSA Conversion (Mem2Reg)
// ---------------------------------------------------------------------------

/// Result of SSA conversion: SSA functions + remaining non-SSA functions
pub struct SsaConversionResult {
    pub ssa_functions: Vec<SsaFunction>,
    pub non_ssa_functions: Vec<MirFunction>,
}

/// Convert a MirModule to SSA form.
/// Functions with only promotable allocas become SsaFunctions.
/// Functions with escaping allocas (field_ptr, heap types) stay as MirFunctions.
pub fn convert_module(module: &MirModule) -> SsaConversionResult {
    let mut ssa_fns = Vec::new();
    let mut non_ssa_fns = Vec::new();

    for func in &module.functions {
        if let Some(ssa_fn) = convert_function(func) {
            ssa_fns.push(ssa_fn);
        } else {
            non_ssa_fns.push(func.clone());
        }
    }

    SsaConversionResult {
        ssa_functions: ssa_fns,
        non_ssa_functions: non_ssa_fns,
    }
}

/// Try to convert a MirFunction to SsaFunction.
/// Returns None if the function has escaping allocas that prevent SSA conversion.
pub fn convert_function(func: &MirFunction) -> Option<SsaFunction> {
    // Step 1: Identify promotable allocas
    let promotable = find_promotable_allocas(func);
    let _non_promotable: HashSet<usize> = func.basic_blocks.iter()
        .flat_map(|b| b.insts.iter())
        .filter_map(|inst| {
            if let MirInst::Alloca { dest, .. } = inst {
                if !promotable.contains(dest) { Some(*dest) } else { None }
            } else { None }
        })
        .collect();

    // If no promotable allocas, skip SSA conversion (no benefit)
    if promotable.is_empty() {
        return None;
    }

    // Step 2: Compute dominators
    let n = func.basic_blocks.len();
    let doms = compute_dominators(func);
    let dom_frontier = compute_dominance_frontier(func, &doms);

    // Step 3: Place phi nodes for each promotable alloca
    let mut phi_blocks: HashMap<usize, HashSet<usize>> = HashMap::new();
    let def_blocks = find_def_blocks(func, &promotable);

    for (&alloca, defs) in &def_blocks {
        let mut worklist: VecDeque<usize> = defs.iter().copied().collect();
        let mut added = HashSet::new();
        while let Some(b) = worklist.pop_front() {
            if let Some(df) = dom_frontier.get(&b) {
                for &df_block in df {
                    if added.insert(df_block) {
                        phi_blocks.entry(alloca).or_default().insert(df_block);
                        worklist.push_back(df_block);
                    }
                }
            }
        }
    }

    // Step 4: Build SsaFunction
    let mut ssa = SsaFunction {
        name: func.name.clone(),
        params: func.params.clone(),
        return_type: func.return_type.clone(),
        is_fallible: func.is_fallible,
        blocks: Vec::new(),
        values: Vec::new(),
        local_count: func.local_count,
        const_values: HashMap::new(),
        block_local_map: Vec::new(),
        param_value_ids: Vec::new(),
    };

    // Create SSA values for params
    let param_value_ids: Vec<SsaValueId> = func.params.iter().enumerate().map(|(i, ptype)| {
        let id = ssa.values.len();
        ssa.values.push(SsaValue { type_: ptype.clone(), name: format!("p{}", i) });
        id
    }).collect();

    // Create initial SSA values for promotable allocas (will be overwritten by phis/stores)
    let mut alloca_current: HashMap<usize, SsaValueId> = HashMap::new();
    for &alloca in &promotable {
        let alloc_type = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .find_map(|inst| {
                if let MirInst::Alloca { dest, type_, .. } = inst {
                    if *dest == alloca { Some(type_.clone()) } else { None }
                } else { None }
            })
            .unwrap_or(MirType::I32);
        let id = ssa.values.len();
        ssa.values.push(SsaValue { type_: alloc_type, name: format!("_a{}", alloca) });
        alloca_current.insert(alloca, id);
    }

    // Step 5: Walk blocks and rename
    let mut stacks: HashMap<usize, Vec<SsaValueId>> = HashMap::new();
    // Initialize param stacks
    for (i, &vid) in param_value_ids.iter().enumerate() {
        stacks.entry(i).or_default().push(vid);
    }
    // Initialize alloca stacks with the initial value (undef-like)
    for (&alloca, &vid) in &alloca_current {
        stacks.entry(alloca).or_default().push(vid);
    }

    // Build SSA blocks in order. Since blocks are processed linearly (not in
    // dominator-tree order), we must restore stacks/alloca_current to the state
    // at the end of the immediate dominator before each block (except bb0).
    // Otherwise values from a prior processed block leak into non-dominated
    // successor blocks (e.g. loop body value leaks into exit block).
    // Save stack/alloca state per block for proper restoration at dominator boundaries.
    let mut saved_stacks: HashMap<usize, HashMap<usize, Vec<SsaValueId>>> = HashMap::new();
    let mut saved_alloca: HashMap<usize, HashMap<usize, SsaValueId>> = HashMap::new();
    for (block_idx, block) in func.basic_blocks.iter().enumerate() {
        // Restore stacks + alloca_current to the state at the end of the immediate
        // dominator. This prevents values from a non-dominating prior-processed block
        // (e.g. loop body) from leaking into a successor (e.g. exit block).
        if block_idx > 0 {
            let idom = doms[block_idx];
            if idom != usize::MAX {
                if let Some(saved) = saved_stacks.get(&idom) {
                    stacks = saved.clone();
                    // Prune non-promotable entries from stacks (they'd give stale SsaValueIds).
                    stacks.retain(|k, _| promotable.contains(k));
                }
                if let Some(saved) = saved_alloca.get(&idom) {
                    alloca_current = saved.clone();
                    // Remove non-promotable entries from alloca_current so
                    // resolve_value creates _np dummies (force alloca load in codegen)
                    // instead of reusing stale SsaValueIds from the dominator.
                    alloca_current.retain(|k, _| promotable.contains(k));
                }
            }
        }
        let mut ssa_block = SsaBlock {
            label: block.label.clone(),
            phis: Vec::new(),
            insts: Vec::new(),
            terminator: block.terminator.clone(),
        };

        // Add phi nodes for this block
        for (&alloca, phi_blocks_set) in &phi_blocks {
            if phi_blocks_set.contains(&block_idx) {
                let alloc_type = func.basic_blocks.iter()
                    .flat_map(|b| b.insts.iter())
                    .find_map(|inst| {
                        if let MirInst::Alloca { dest, type_, .. } = inst {
                            if *dest == alloca { Some(type_.clone()) } else { None }
                        } else { None }
                    })
                    .unwrap_or(MirType::I32);
                let phi_dest = ssa.values.len();
                ssa.values.push(SsaValue { type_: alloc_type.clone(), name: format!("_phi{}_{}", alloca, block_idx) });
                alloca_current.insert(alloca, phi_dest);
                stacks.entry(alloca).or_default().push(phi_dest);
                ssa_block.phis.push(PhiNode {
                    dest: phi_dest,
                    alloca_id: alloca,
                    incomings: Vec::new(),
                    type_: alloc_type,
                });
            }
        }

        // Build map: AddressOf dest local -> target local, for detecting aliasing calls
        let addr_of_map: HashMap<usize, usize> = block.insts.iter()
            .filter_map(|inst| {
                if let MirInst::AddressOf { dest, local_id } = inst {
                    Some((*dest, *local_id))
                } else { None }
            })
            .collect();

        // Process instructions
        for inst in &block.insts {
            match inst {
                MirInst::Alloca { dest, .. } if promotable.contains(dest) => {
                    // Already handled above — skip
                }
                MirInst::Alloca { dest, type_, name } => {
                    // Non-promotable alloca — keep as Alloca
                    ssa_block.insts.push(SsaInst::Alloca { dest: *dest, type_: type_.clone(), name: name.clone() });
                }
                MirInst::Store { dest, value } if promotable.contains(dest) => {
                    // Create a new SSA value for this def
                    let val_id = resolve_value(value, &mut ssa, &stacks, &param_value_ids);
                    // Track the stored value's SsaValueId (not an abstract Store dest)
                    alloca_current.insert(*dest, val_id);
                    stacks.entry(*dest).or_default().push(val_id);

                    // Emit the store — codegen will update block_vals with val_id
                    ssa_block.insts.push(SsaInst::Store { dest: *dest, value: val_id });
                }
                MirInst::Store { dest, value } => {
                    let val_id = resolve_value(value, &mut ssa, &stacks, &param_value_ids);
                    // Track in alloca_current and stacks even for non-promotable allocas,
                    // so later references to this MIR local find the correct SsaValueId.
                    alloca_current.insert(*dest, val_id);
                    stacks.entry(*dest).or_default().push(val_id);
                    ssa_block.insts.push(SsaInst::Store { dest: *dest, value: val_id });
                }
                MirInst::Load { dest, src } if promotable.contains(src) => {
                    // Check if the src is a FieldPtr dest — if so, the codegen handles
                    // it via field_ptr_allocas, not through promoted SSA values.
                    let is_field_ptr_dest = func.basic_blocks.iter()
                        .flat_map(|b| b.insts.iter())
                        .any(|inst| matches!(inst, MirInst::FieldPtr { dest: d, .. } if *d == *src));
                    if is_field_ptr_dest {
                        // Emit Load instruction so codegen uses field_ptr_allocas.
                        // The codegen stores the loaded value in block_vals[dest].
                        // Create a fresh SSA value for the result and track it.
                        let new_dest = ssa.values.len();
                        ssa.values.push(SsaValue { type_: MirType::I64, name: format!("_fl{}", dest) });
                        ssa_block.insts.push(SsaInst::Load { dest: new_dest, src: *src });
                        stacks.entry(*dest).or_default().push(new_dest);
                        alloca_current.insert(*dest, new_dest);
                    } else {
                        // Load from promotable alloca → read current SSA value
                        let cur = alloca_current.get(src).copied().unwrap_or(0);
                        // Track the loaded value in stacks so subsequent instructions can find it
                        stacks.entry(*dest).or_default().push(cur);
                        alloca_current.insert(*dest, cur);
                        // Emit a Store that codegen treats as promoted update
                        ssa_block.insts.push(SsaInst::Store { dest: *dest, value: cur });
                    }
                }
                MirInst::Load { dest, src } => {
                    let load_type = func.basic_blocks.iter()
                        .flat_map(|b| b.insts.iter())
                        .find_map(|inst| {
                            if let MirInst::Load { dest: d, src: s, .. } = inst {
                                if *d == *dest && *s == *src {
                                    // Get type from alloca
                                    func.basic_blocks.iter()
                                        .flat_map(|b| b.insts.iter())
                                        .find_map(|inst2| {
                                            if let MirInst::Alloca { dest: d2, type_, .. } = inst2 {
                                                if *d2 == *src { Some(type_.clone()) } else { None }
                                            } else { None }
                                        })
                                } else { None }
                            } else { None }
                        })
                        .unwrap_or(MirType::I32);
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: load_type, name: format!("_l{}", dest) });
                     ssa_block.insts.push(SsaInst::Load { dest: new_dest, src: *src });
                     alloca_current.insert(*dest, new_dest);
                     stacks.entry(*dest).or_default().push(new_dest);
                     // Emit Store for non-promotable load dests so the codegen's
                     // resolve_mir! fallback (which checks alloca_map) finds the value.
                     if !promotable.contains(dest) {
                         ssa_block.insts.push(SsaInst::Store { dest: *dest, value: new_dest });
                     }
                }
                MirInst::BinaryOp { dest, op, left, right } => {
                    let left_id = resolve_value(left, &mut ssa, &stacks, &param_value_ids);
                    let right_id = resolve_value(right, &mut ssa, &stacks, &param_value_ids);
                    let result_type = binary_op_result_type(op, left, right, func);
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: result_type, name: format!("_b{}", dest) });
                    ssa_block.insts.push(SsaInst::BinaryOp { dest: new_dest, op: op.clone(), left: left_id, right: right_id });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
                MirInst::UnaryOp { dest, op, operand } => {
                    let op_id = resolve_value(operand, &mut ssa, &stacks, &param_value_ids);
                    let result_type = func.basic_blocks.iter()
                        .flat_map(|b| b.insts.iter())
                        .find_map(|inst| {
                            if let MirInst::UnaryOp { dest: d, .. } = inst {
                                if *d == *dest { Some(MirType::I32) } else { None }
                            } else { None }
                        })
                        .unwrap_or(MirType::I32);
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: result_type, name: format!("_u{}", dest) });
                    ssa_block.insts.push(SsaInst::UnaryOp { dest: new_dest, op: op.clone(), operand: op_id });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
                MirInst::Call { dest, name, args } => {
                    // Create SSA values for constant arguments and collect IDs
                    let mut const_ids: Vec<SsaValueId> = Vec::new();
                    for arg in args {
                        if let MirValue::Constant(c) = arg {
                            let const_type = match c {
                                MirConstant::I32(_) => MirType::I32,
                                MirConstant::I64(_) => MirType::I64,
                                MirConstant::F64(_) => MirType::F64,
                                MirConstant::Bool(_) => MirType::Bool,
                                MirConstant::String(_) => MirType::Str,
                                                                MirConstant::Void => MirType::Void,
                                MirConstant::Null => MirType::Ptr(Box::new(MirType::Void)),
                            };


                            let id = ssa.values.len();
                            ssa.values.push(SsaValue { type_: const_type, name: format!("_carg{}", id) });
                            ssa.const_values.insert(id, c.clone());
                            const_ids.push(id);
                        }
                    }
                    let mut const_idx = 0;
                    let arg_ids: Vec<SsaValueId> = args.iter()
                        .map(|a| match a {
                            MirValue::Constant(_) => {
                                let id = const_ids[const_idx];
                                const_idx += 1;
                                id
                            }
                            _ => resolve_value(a, &mut ssa, &stacks, &param_value_ids),
                        })
                        .collect();
                    let dest_val = *dest;
                    let new_dest = dest_val.map(|d| {
                        let id = ssa.values.len();
                        let call_type = func.basic_blocks.iter()
                            .flat_map(|b| b.insts.iter())
                            .find_map(|inst| {
                                // Look up the alloca type for the dest local (actual return type)
                                if let MirInst::Alloca { dest: dd, type_, .. } = inst {
                                    if *dd == d { return Some(type_.clone()); }
                                }
                                None
                            })
                            .unwrap_or(MirType::I64);
                        ssa.values.push(SsaValue { type_: call_type, name: format!("_c{}", d) });
                        alloca_current.insert(d, id);
                        stacks.entry(d).or_default().push(id);
                        id
                    });
                    ssa_block.insts.push(SsaInst::Call { dest: new_dest, name: name.clone(), args: arg_ids });
                    // Emit Store for non-promotable call dests AFTER the Call instruction,
                    // so codegen processes the Store after the Call has set block_vals.
                    if let Some(d) = dest_val {
                        if !promotable.contains(&d) {
                            if let Some(nid) = new_dest {
                                ssa_block.insts.push(SsaInst::Store { dest: d, value: nid });
                            }
                        }
                    }
                    // Invalidate locals whose address was passed to the call:
                    // the call may have modified them through the pointer.
                    for arg in args {
                        if let MirValue::Local(arg_local) = arg {
                            if let Some(&orig_local) = addr_of_map.get(arg_local) {
                                if !promotable.contains(&orig_local) {
                                    // Create a fresh Load to replace the stale stacks entry
                                    let fresh_val = ssa.values.len();
                                    let src_type = func.basic_blocks.iter()
                                        .flat_map(|b| b.insts.iter())
                                        .find_map(|inst| {
                                            if let MirInst::Alloca { dest: d, type_, .. } = inst {
                                                if *d == orig_local { Some(type_.clone()) } else { None }
                                            } else { None }
                                        })
                                        .unwrap_or(MirType::I32);
                                    ssa.values.push(SsaValue { type_: src_type, name: format!("_ref{}", orig_local) });
                                    ssa_block.insts.push(SsaInst::Load { dest: fresh_val, src: orig_local });
                                    // Also store back to the alloca for the codegen's resolve_mir! fallback
                                    ssa_block.insts.push(SsaInst::Store { dest: orig_local, value: fresh_val });
                                    alloca_current.insert(orig_local, fresh_val);
                                    stacks.entry(orig_local).or_default().push(fresh_val);
                                }
                            }
                        }
                    }
                }
                MirInst::Cast { dest, value, to_type } => {
                    let val_id = resolve_value(value, &mut ssa, &stacks, &param_value_ids);
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: to_type.clone(), name: format!("_cast{}", dest) });
                    ssa_block.insts.push(SsaInst::Cast { dest: new_dest, value: val_id, to_type: to_type.clone() });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                    // Emit Store for non-promotable cast dests so alloca is updated
                    if !promotable.contains(dest) {
                        ssa_block.insts.push(SsaInst::Store { dest: *dest, value: new_dest });
                    }
                }
                MirInst::PtrOffset { dest, ptr, index, elem_type } => {
                    let idx_id = resolve_value(index, &mut ssa, &stacks, &param_value_ids);
                    let ptr_id = alloca_current.get(ptr).copied().unwrap_or(*ptr);
                    ssa_block.insts.push(SsaInst::PtrOffset { dest: *dest, ptr: ptr_id, index: idx_id, elem_type: elem_type.clone() });
                }
                MirInst::PtrStore { dest: _dest, ptr, index, value } => {
                    let idx_id = resolve_value(index, &mut ssa, &stacks, &param_value_ids);
                    let val_id = resolve_value(value, &mut ssa, &stacks, &param_value_ids);
                    ssa_block.insts.push(SsaInst::PtrStore { ptr: *ptr, index: idx_id, value: val_id });
                }
                MirInst::FieldPtr { dest, ptr, field_index, struct_type } => {
                    ssa_block.insts.push(SsaInst::FieldPtr { dest: *dest, ptr: *ptr, field_index: *field_index, struct_type: struct_type.clone() });
                }
                MirInst::ArrayElemPtr { dest, ptr, index, arr_type, elem_type } => {
                    let index_id = match index {
                        MirValue::Local(id) => alloca_current.get(id).copied().unwrap_or(*id),
                        MirValue::Param(id) => param_value_ids.get(*id).copied().unwrap_or(*id),
                        MirValue::Constant(c) => {
                            let val_id = ssa.values.len();
                            ssa.values.push(SsaValue {
                                type_: MirType::I32, name: format!("_idx{}", *dest)
                            });
                            ssa.const_values.insert(val_id, c.clone());
                            val_id
                        }
                    };
                    // Create a new SsaValueId to avoid ID collision between MIR locals and SSA values
                    let result_id = ssa.values.len();
                    ssa.values.push(SsaValue {
                        type_: MirType::Ptr(Box::new(MirType::I8)),
                        name: format!("_aep{}", *dest),
                    });
                    ssa_block.insts.push(SsaInst::ArrayElemPtr { dest: result_id, mir_dest: *dest, ptr: *ptr, index: index_id, arr_type: arr_type.clone(), elem_type: elem_type.clone() });
                    alloca_current.insert(*dest, result_id);
                    stacks.entry(*dest).or_default().push(result_id);
                }
                MirInst::Memcpy { dest_ptr_local, src_alloca_local, struct_type } => {
                    ssa_block.insts.push(SsaInst::Memcpy { dest_ptr_local: *dest_ptr_local, src_alloca_local: *src_alloca_local, struct_type: struct_type.clone() });
                }
                MirInst::FnAddr { dest, name } => {
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: MirType::Ptr(Box::new(MirType::I8)), name: format!("_fn{}", dest) });
                    ssa_block.insts.push(SsaInst::FnAddr { dest: new_dest, name: name.clone() });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
                MirInst::AddressOf { dest, local_id } => {
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: MirType::Ptr(Box::new(MirType::I8)), name: format!("_ad{}", dest) });
                    ssa_block.insts.push(SsaInst::AddressOf { dest: new_dest, local_id: *local_id });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
                MirInst::CallIndirect { dest, fn_ptr, ret_type, param_types, args } => {
                    let fn_ptr_id = alloca_current.get(fn_ptr).copied().unwrap_or(*fn_ptr);
                    let arg_ids: Vec<SsaValueId> = args.iter()
                        .map(|a| resolve_value(a, &mut ssa, &stacks, &param_value_ids))
                        .collect();
                    let new_dest = dest.map(|_| {
                        let id = ssa.values.len();
                        ssa.values.push(SsaValue { type_: ret_type.clone(), name: format!("_ci{}", dest.unwrap()) });
                        if let Some(d) = dest { alloca_current.insert(*d, id); }
                        if let Some(d) = dest { stacks.entry(*d).or_default().push(id); }
                        id
                    });
                    ssa_block.insts.push(SsaInst::CallIndirect { dest: new_dest, fn_ptr: fn_ptr_id, ret_type: ret_type.clone(), param_types: param_types.clone(), args: arg_ids });
                }
                MirInst::AsyncSpawn { dest, function_name, arg } => {
                    let arg_id = resolve_value(arg, &mut ssa, &stacks, &param_value_ids);
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: MirType::I64, name: format!("_as{}", dest) });
                    ssa_block.insts.push(SsaInst::AsyncSpawn { dest: new_dest, function_name: function_name.clone(), arg: arg_id });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
                MirInst::AsyncAwait { dest, handle, return_type } => {
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: return_type.clone(), name: format!("_aa{}", dest) });
                    let handle_id = alloca_current.get(handle).copied().unwrap_or(*handle);
                    ssa_block.insts.push(SsaInst::AsyncAwait { dest: new_dest, handle: handle_id, return_type: return_type.clone() });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
                MirInst::SliceMake { dest, ptr, len, elem_type } => {
                    let ptr_id = resolve_value(ptr, &mut ssa, &stacks, &param_value_ids);
                    let len_id = resolve_value(len, &mut ssa, &stacks, &param_value_ids);
                    let new_dest = ssa.values.len();
                    ssa.values.push(SsaValue { type_: MirType::Slice(elem_type.clone()), name: format!("_sm{}", dest) });
                    ssa_block.insts.push(SsaInst::SliceMake { dest: new_dest, ptr: ptr_id, len: len_id, elem_type: elem_type.clone() });
                    alloca_current.insert(*dest, new_dest);
                    stacks.entry(*dest).or_default().push(new_dest);
                }
            }
        }

        // Save per-block state for terminator resolution (MirValue::Local → SsaValueId)
        ssa.block_local_map.push(alloca_current.clone());
        // Save state for dominator-based restoration in subsequent blocks
        saved_stacks.insert(block_idx, stacks.clone());
        saved_alloca.insert(block_idx, alloca_current.clone());
        ssa.blocks.push(ssa_block);
    }

    // Step 6: Fill phi incomings from predecessor blocks.
    // Use per-block END values: record each alloca's SSA value at the END of each block.
    // First pass: collect all last-stores per block from the SSA instructions.
    let mut block_end_values: HashMap<(usize, usize), SsaValueId> = HashMap::new();
    for (block_idx, block) in ssa.blocks.iter().enumerate() {
        for inst in &block.insts {
            if let SsaInst::Store { dest, value } = inst {
                let key = (block_idx, *dest);
                // Last store to this alloca in this block wins
                block_end_values.insert(key, *value);
            }
        }
    }
    // Ensure every promotable alloca has at least the initial value in block 0
    for &alloca in &promotable {
        let key = (0, alloca);
        block_end_values.entry(key).or_insert_with(|| {
            alloca_current.get(&alloca).copied().unwrap_or(0)
        });
    }

    // Second pass: fill phi incomings from block_end_values per predecessor.
    // For predecessors with no store to the alloca, use the reaching definition
    // from saved_alloca (the alloca's value at end of that predecessor block)
    // instead of falling back to the entry block's value.
    for (block_idx, block) in ssa.blocks.iter_mut().enumerate() {
        let preds = predecessors(func, block_idx);
        for phi in &mut block.phis {
            for pred_label in &preds {
                if let Some(pred_idx) = func.basic_blocks.iter().position(|b| &b.label == pred_label) {
                    let val = block_end_values.get(&(pred_idx, phi.alloca_id)).copied()
                        .or_else(|| saved_alloca.get(&pred_idx).and_then(|m| m.get(&phi.alloca_id)).copied())
                        .or_else(|| block_end_values.get(&(0, phi.alloca_id)).copied());
                    if let Some(v) = val {
                        phi.incomings.push((v, pred_label.clone()));
                    }
                }
            }
            // Ensure phi has at least one incoming (LLVM requirement).
            if phi.incomings.is_empty() {
                let entry_label = func.basic_blocks.first().map(|b| b.label.clone()).unwrap_or_else(|| "entry".to_string());
                let entry_val = block_end_values.get(&(0, phi.alloca_id)).copied()
                    .unwrap_or(phi.dest);
                phi.incomings.push((entry_val, entry_label));
            }
        }
    }

    // Step 7: Store param_value_ids
    ssa.param_value_ids = param_value_ids;

    // Step 8: Run GVN (Global Value Numbering) on the SSA function
    optimize_gvn(&mut ssa);

    Some(ssa)
}

// ---------------------------------------------------------------------------
// SSA Optimizations
// ---------------------------------------------------------------------------

/// Global Value Numbering (GVN) on an SSA function.
/// In SSA, each `dest` is assigned exactly once, so redundant computations
/// can be safely eliminated by replacing them with the previous result.
///
/// This pass detects:
///   a = x + y
///   b = x + y   // redundant → replace all uses of b with a
fn optimize_gvn(ssa: &mut SsaFunction) {
    // For each block, track seen computations and their result SSA value IDs.
    // A computation is identified by (type_tag, op_data, operand1, operand2).
    // If the same computation appears again, replace it with a copy of the first.
    for block in &mut ssa.blocks {
        let mut seen: HashMap<(u64, u64, SsaValueId, SsaValueId), SsaValueId> = HashMap::new();
        let mut replacements: HashMap<SsaValueId, SsaValueId> = HashMap::new();

        for inst in &block.insts {
            match inst {
                SsaInst::BinaryOp { dest, op, left, right } => {
                    let key = (
                        gvn_binary_tag(op),
                        0,  // no extra data
                        *left,
                        *right,
                    );
                    if let Some(&prev) = seen.get(&key) {
                        replacements.insert(*dest, prev);
                    } else {
                        seen.insert(key, *dest);
                    }
                }
                SsaInst::UnaryOp { dest, op, operand } => {
                    let key = (
                        1000 + match op { MirUnaryOp::Neg => 1, MirUnaryOp::Not => 2, MirUnaryOp::BitNot => 3 },
                        0,
                        *operand,
                        0,
                    );
                    if let Some(&prev) = seen.get(&key) {
                        replacements.insert(*dest, prev);
                    } else {
                        seen.insert(key, *dest);
                    }
                }
                SsaInst::Cast { dest, value, to_type } => {
                    let type_code = format!("{:?}", to_type).bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                    let key = (
                        2000 + type_code,
                        0,
                        *value,
                        0,
                    );
                    if let Some(&prev) = seen.get(&key) {
                        replacements.insert(*dest, prev);
                    } else {
                        seen.insert(key, *dest);
                    }
                }
                _ => {}
            }
        }

        // Apply replacements: replace all uses of redundant values with the canonical one.
        if !replacements.is_empty() {
            for inst in &mut block.insts {
                gvn_replace_inst(inst, &replacements);
            }
            for phi in &mut block.phis {
                for (val_id, _) in &mut phi.incomings {
                    if let Some(&replacement) = replacements.get(val_id) {
                        *val_id = replacement;
                    }
                }
            }
        }
    }
}

fn gvn_binary_tag(op: &MirBinaryOp) -> u64 {
    match op {
        MirBinaryOp::Add => 1, MirBinaryOp::Sub => 2, MirBinaryOp::Mul => 3,
        MirBinaryOp::Div => 4, MirBinaryOp::Rem => 5,
        MirBinaryOp::And => 6, MirBinaryOp::Or => 7, MirBinaryOp::Xor => 8,
        MirBinaryOp::Shl => 9, MirBinaryOp::Shr => 10,
        MirBinaryOp::Eq => 11, MirBinaryOp::Neq => 12,
        MirBinaryOp::Lt => 13, MirBinaryOp::Gt => 14,
        MirBinaryOp::Le => 15, MirBinaryOp::Ge => 16,
    }
}

/// Replace direct references to redundant SSA values inside an instruction.
fn gvn_replace_inst(inst: &mut SsaInst, replacements: &HashMap<SsaValueId, SsaValueId>) {
    let replace = |id: &mut SsaValueId| {
        if let Some(&repl) = replacements.get(id) {
            *id = repl;
        }
    };
    match inst {
        SsaInst::BinaryOp { left, right, .. } => { replace(left); replace(right); }
        SsaInst::UnaryOp { operand, .. } => { replace(operand); }
        SsaInst::Cast { value, .. } => { replace(value); }
        SsaInst::Call { args, .. } => { for a in args { replace(a); } }
        SsaInst::CallIndirect { fn_ptr, args, .. } => { replace(fn_ptr); for a in args { replace(a); } }
        SsaInst::Store { value, .. } => { replace(value); }
        SsaInst::PtrOffset { index, .. } => { replace(index); }
        SsaInst::PtrStore { index, value, .. } => { replace(index); replace(value); }
        SsaInst::AsyncSpawn { arg, .. } => { replace(arg); }
        SsaInst::AsyncAwait { handle, .. } => { replace(handle); }
        SsaInst::AddressOf { .. } => {}
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Find allocas that can be promoted to SSA values.
fn find_promotable_allocas(func: &MirFunction) -> HashSet<usize> {
    let mut escaped: HashSet<usize> = HashSet::new();
    let mut allocas: HashSet<usize> = HashSet::new();

    for bb in &func.basic_blocks {
        for inst in &bb.insts {
            match inst {
                MirInst::Alloca { dest, type_, .. } => {
                    allocas.insert(*dest);
                    // Move types, structs, ptrs, slices cannot be promoted (SSA codegen handles them poorly)
                    if is_move_type(type_) || matches!(type_, MirType::Struct(_, _) | MirType::Ptr(_) | MirType::Slice(_) | MirType::Box(_) | MirType::Array(_, _)) {
                        escaped.insert(*dest);
                    }
                }
                MirInst::FieldPtr { ptr, .. } => { escaped.insert(*ptr); }
                MirInst::ArrayElemPtr { ptr, .. } => { escaped.insert(*ptr); }
                MirInst::PtrOffset { ptr, .. } => { escaped.insert(*ptr); }
                MirInst::PtrStore { ptr, .. } => { escaped.insert(*ptr); }
                MirInst::Memcpy { .. } => {
                    // These reference heap-allocated memory, not promotable
                }
                MirInst::SliceMake { dest, .. } => { escaped.insert(*dest); }
                MirInst::AddressOf { local_id, .. } => { escaped.insert(*local_id); }
                // Store to a non-escaping dest from a SliceMake: mark dest as escaping too
                MirInst::Store { dest, value: MirValue::Local(src) } => {
                    if escaped.contains(src) && !escaped.contains(dest) {
                        // Check if the src is a SliceMake or other escaping instruction
                        // by looking at the source local's type
                        let src_type = func.basic_blocks.iter()
                            .flat_map(|b| b.insts.iter())
                            .find_map(|inst| {
                                if let MirInst::Alloca { dest: d, type_, .. } = inst {
                                    if *d == *src { Some(type_.clone()) } else { None }
                                } else { None }
                            });
                        if let Some(t) = src_type {
                            if matches!(t, MirType::Slice(_)) {
                                escaped.insert(*dest);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    allocas.difference(&escaped).copied().collect()
}

/// Find blocks that define (store to) each promotable alloca.
fn find_def_blocks(func: &MirFunction, promotable: &HashSet<usize>) -> HashMap<usize, HashSet<usize>> {
    let mut defs: HashMap<usize, HashSet<usize>> = HashMap::new();
    for (bi, bb) in func.basic_blocks.iter().enumerate() {
        for inst in &bb.insts {
            if let MirInst::Store { dest, .. } = inst {
                if promotable.contains(dest) {
                    defs.entry(*dest).or_default().insert(bi);
                }
            }
        }
    }
    // Params count as definitions at entry (block 0)
    for &alloca in promotable {
        defs.entry(alloca).or_default().insert(0);
    }
    defs
}

/// Compute immediate dominators for each block.
/// Compute immediate dominators using the standard iterative algorithm.
/// Uses a worklist approach that is guaranteed to converge.
fn compute_dominators(func: &MirFunction) -> Vec<usize> {
    let n = func.basic_blocks.len();
    if n == 0 { return vec![]; }
    if n == 1 { return vec![0]; }

    // Map block label to index for predecessor lookup
    let label_to_idx: HashMap<&str, usize> = func.basic_blocks.iter()
        .enumerate().map(|(i, b)| (b.label.as_str(), i)).collect();

    // Pre-compute predecessors for each block
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (j, bb) in func.basic_blocks.iter().enumerate() {
        match &bb.terminator {
            MirTerminator::Br(label) => {
                if let Some(&target) = label_to_idx.get(label.as_str()) {
                    preds[target].push(j);
                }
            }
            MirTerminator::CondBr { true_block, false_block, .. } => {
                if let Some(&target) = label_to_idx.get(true_block.as_str()) {
                    preds[target].push(j);
                }
                if let Some(&target) = label_to_idx.get(false_block.as_str()) {
                    preds[target].push(j);
                }
            }
            _ => {}
        }
    }

    // Initialize: entry dominates itself
    let mut dom: Vec<usize> = vec![usize::MAX; n];
    dom[0] = 0;

    // Iterative data-flow: each block's dom is the intersection of its predecessors' doms
    // Use a strict safety limit to prevent infinite loops with malformed CFGs.
    for _iter in 0..(n.max(4) * 4) {
        let mut changed = false;
        for i in 0..n {
            if preds[i].is_empty() { continue; }
            let valid_preds: Vec<usize> = preds[i].iter().copied()
                .filter(|&p| dom[p] != usize::MAX || p == 0).collect();
            if valid_preds.is_empty() { continue; }
            let first = valid_preds[0];
            let mut new_idom = first;
            for &p in &valid_preds[1..] {
                new_idom = intersect(&dom, new_idom, p);
            }
            if dom[i] != new_idom {
                dom[i] = new_idom;
                changed = true;
            }
        }
        if !changed { break; }
    }

    dom
}

/// Intersect two dominator chains: find the closest common dominator.
/// Standard algorithm from "A Simple, Fast Dominance Algorithm".
fn intersect(dom: &[usize], b1: usize, b2: usize) -> usize {
    if b1 >= dom.len() || b2 >= dom.len() { return 0; }
    let mut finger1 = b1;
    let mut finger2 = b2;
    let limit = dom.len() * 8;
    let mut safety = 0;
    while finger1 != finger2 && safety < limit {
        safety += 1;
        while finger1 > finger2 {
            finger1 = dom.get(finger1).copied().unwrap_or(0);
            safety += 1;
            if safety >= limit { break; }
        }
        while finger2 > finger1 {
            finger2 = dom.get(finger2).copied().unwrap_or(0);
            safety += 1;
            if safety >= limit { break; }
        }
    }
    if finger1 != finger2 { return 0; }
    finger1
}

/// Compute dominance frontier for each block.
fn compute_dominance_frontier(func: &MirFunction, dom: &[usize]) -> HashMap<usize, HashSet<usize>> {
    let n = func.basic_blocks.len();
    let mut df: HashMap<usize, HashSet<usize>> = HashMap::new();
    if n == 0 { return df; }

    for i in 0..n {
        let preds: Vec<usize> = (0..n)
            .filter(|&j| {
                let term = &func.basic_blocks[j].terminator;
                matches!(term, MirTerminator::Br(label) if label == &func.basic_blocks[i].label)
                    || matches!(term, MirTerminator::CondBr { true_block, false_block, .. }
                        if true_block == &func.basic_blocks[i].label || false_block == &func.basic_blocks[i].label)
            })
            .collect();

        if preds.len() >= 2 {
            for &p in &preds {
                if dom[i] >= dom.len() { continue; }
                let mut runner = p;
                let mut safety = 0;
                while runner != dom[i] && runner < dom.len() && safety < n * 4 {
                    safety += 1;
                    df.entry(runner).or_default().insert(i);
                    runner = dom.get(runner).copied().unwrap_or(0);
                }
            }
        }
    }

    df
}

/// Find predecessor blocks for a given block.
fn predecessors(func: &MirFunction, block_idx: usize) -> Vec<String> {
    let target_label = &func.basic_blocks[block_idx].label;
    let mut preds = Vec::new();
    for (_, bb) in func.basic_blocks.iter().enumerate() {
        match &bb.terminator {
            MirTerminator::Br(label) if label == target_label => preds.push(bb.label.clone()),
            MirTerminator::CondBr { true_block, false_block, .. } => {
                if true_block == target_label || false_block == target_label {
                    preds.push(bb.label.clone());
                }
            }
            _ => {}
        }
    }
    preds
}

/// Convert a MirValue to an SSA value ID, creating SSA values for constants.
/// This is the main helper for resolving values during SSA conversion.
fn resolve_value(
    value: &MirValue,
    ssa: &mut SsaFunction,
    stacks: &HashMap<usize, Vec<SsaValueId>>,
    param_ids: &[SsaValueId],
) -> SsaValueId {
    match value {
        MirValue::Constant(c) => {
            let const_type = match c {
                MirConstant::I32(_) => MirType::I32,
                MirConstant::I64(_) => MirType::I64,
                MirConstant::F64(_) => MirType::F64,
                MirConstant::Bool(_) => MirType::Bool,
                MirConstant::String(_) => MirType::Str,
                MirConstant::Void => MirType::Void,
                MirConstant::Null => MirType::Ptr(Box::new(MirType::Void)),
            };
            let id = ssa.values.len();
            ssa.values.push(SsaValue { type_: const_type, name: format!("_k{}", id) });
            ssa.const_values.insert(id, c.clone());
            id
        }
        MirValue::Local(id) => {
            // For non-promotable locals, create a dummy SsaValue so
            // SsaValueId is always valid. The codegen's resolve_mir!
            // falls back to loading from the actual alloca.
            stacks.get(id).and_then(|s| s.last().copied()).unwrap_or_else(|| {
                let vid = ssa.values.len();
                ssa.values.push(SsaValue { type_: MirType::I32, name: format!("_np{}", id) });
                vid
            })
        }
        MirValue::Param(id) => {
            param_ids.get(*id).copied().unwrap_or(*id)
        }
    }
}

/// Get the type of a MirValue.
fn value_type(value: &MirValue, func: &MirFunction, promotable: &HashSet<usize>, _ssa: &SsaFunction) -> MirType {
    match value {
        MirValue::Constant(c) => match c {
            MirConstant::I32(_) => MirType::I32,
            MirConstant::I64(_) => MirType::I64,
            MirConstant::F64(_) => MirType::F64,
            MirConstant::Bool(_) => MirType::Bool,
            MirConstant::String(_) => MirType::Str,
            MirConstant::Void => MirType::Void,
            MirConstant::Null => MirType::Ptr(Box::new(MirType::Void)),
        },
        MirValue::Local(id) => {
            if promotable.contains(id) {
                // Get type from alloca
                func.basic_blocks.iter()
                    .flat_map(|b| b.insts.iter())
                    .find_map(|inst| {
                        if let MirInst::Alloca { dest, type_, .. } = inst {
                            if *dest == *id { Some(type_.clone()) } else { None }
                        } else { None }
                    })
                    .unwrap_or(MirType::I32)
            } else {
                MirType::I32
            }
        }
        MirValue::Param(id) => {
            func.params.get(*id).cloned().unwrap_or(MirType::I32)
        }
    }
}

/// Get the result type of a binary operation.
fn binary_op_result_type(op: &MirBinaryOp, left: &MirValue, right: &MirValue, func: &MirFunction) -> MirType {
    // Comparison ops return bool
    match op {
        MirBinaryOp::Eq | MirBinaryOp::Neq | MirBinaryOp::Lt | MirBinaryOp::Gt | MirBinaryOp::Le | MirBinaryOp::Ge => {
            return MirType::Bool;
        }
        _ => {}
    }
    // Otherwise, the type depends on the operands
    MirType::I32 // default
}
