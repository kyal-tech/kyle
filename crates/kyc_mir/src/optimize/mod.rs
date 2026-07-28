use std::collections::{HashSet, HashMap};
use crate::mir::*;

/// MIR optimization passes.
pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Self
    }

    /// Run all optimization passes on a module.
    pub fn optimize(&self, module: &mut MirModule) {
        // First, evaluate const fn calls (replaces them with constants)
        self.const_eval(module);
        self.inline_small_functions(module);
        for func in &mut module.functions {
            self.constant_fold(func);
            // Compute move_locals: locals with Move-type allocas
            let move_locals: HashSet<usize> = func.basic_blocks.iter()
                .flat_map(|b| b.insts.iter())
                .filter_map(|inst| {
                    if let MirInst::Alloca { dest, type_, .. } = inst {
                        if crate::mir::is_move_type(type_) { Some(*dest) } else { None }
                    } else { None }
                })
                .collect();
            self.dead_code_elim(func, &move_locals);
            self.remove_unreachable_blocks(func);
        }
    }



    /// Inline small functions (single basic block, ≤ 10 insts) at call sites.
    fn inline_small_functions(&self, module: &mut MirModule) {
        let funcs = module.functions.clone();
        for func_idx in 0..module.functions.len() {
            let caller_name = module.functions[func_idx].name.clone();
            let mut new_blocks: Vec<MirBasicBlock> = Vec::new();
            let mut block_map: HashMap<String, String> = HashMap::new();
            let mut local_offset: usize = 0;

            // First pass: find all call sites to inline candidates
            let mut inlines: Vec<(usize, usize, String)> = Vec::new(); // (block_idx, inst_idx, callee_name)
            for (bi, block) in module.functions[func_idx].basic_blocks.iter().enumerate() {
                for (ii, inst) in block.insts.iter().enumerate() {
                    if let MirInst::Call { name, .. } = inst {
                        // Check if callee is a candidate for inlining
                        if let Some(callee_idx) = funcs.iter().position(|f| f.name == *name) {
                            let callee = &funcs[callee_idx];
                            let is_small = callee.basic_blocks.len() == 1
                                && callee.basic_blocks[0].insts.iter()
                                    .filter(|inst| !matches!(inst, MirInst::Alloca { .. }))
                                    .count() <= 10
                                && callee.name != caller_name; // no recursion
                            if is_small {
                                inlines.push((bi, ii, name.clone()));
                            }
                        }
                    }
                }
            }

            if inlines.is_empty() {
                continue;
            }

            // Clone the function's blocks to modify them
            let mut func_blocks = module.functions[func_idx].basic_blocks.clone();
            // Process inlines in reverse order (so indices don't shift)
            for (bi, ii, callee_name) in inlines.into_iter().rev() {
                let callee_idx = funcs.iter().position(|f| f.name == callee_name).unwrap();
                let callee = &funcs[callee_idx];
                let dest_local = match &func_blocks[bi].insts[ii] {
                    MirInst::Call { dest: Some(d), .. } => Some(*d),
                    _ => None,
                };
                let call_args: Vec<MirValue> = match &func_blocks[bi].insts[ii] {
                    MirInst::Call { args, .. } => args.clone(),
                    _ => vec![],
                };

                // Compute local offset for inlined blocks
                let max_local = func_blocks.iter()
                    .flat_map(|b| b.insts.iter())
                    .filter_map(|inst| {
                        match inst {
                            MirInst::Alloca { dest, .. } => Some(*dest),
                            MirInst::Store { dest, .. } => Some(*dest),
                            MirInst::Load { dest, .. } => Some(*dest),
                            MirInst::BinaryOp { dest, .. } => Some(*dest),
                            MirInst::UnaryOp { dest, .. } => Some(*dest),
                            MirInst::Cast { dest, .. } => Some(*dest),
                            MirInst::Call { dest, .. } => *dest,
                            MirInst::PtrOffset { dest, .. } => Some(*dest),
                            MirInst::PtrStore { dest, .. } => Some(*dest),
                            MirInst::FieldPtr { dest, .. } => Some(*dest),
                            _ => None,
                        }
                    })
                    .max()
                    .unwrap_or(0) + 1;

                // Check that inlining only happens in blocks with simple terminators
                let term_is_br = matches!(&func_blocks[bi].terminator, MirTerminator::Br(_));
                if !term_is_br {
                    continue; // skip complex terminators for now
                }

                // Remove the Call instruction
                func_blocks[bi].insts.remove(ii);

                // Clone the callee's single block with remapped locals
                let callee_block = &callee.basic_blocks[0];
                let param_count = callee.params.len();
                let mut local_map: HashMap<usize, usize> = HashMap::new();

                // Map callee allocas to new locals
                for inst in &callee_block.insts {
                    if let MirInst::Alloca { dest, .. } = inst {
                        let new_local = max_local + local_map.len();
                        local_map.insert(*dest, new_local);
                        // Copy the alloca instruction
                        let mut new_alloca = inst.clone();
                        if let MirInst::Alloca { dest, .. } = &mut new_alloca {
                            *dest = new_local;
                        }
                        func_blocks[bi].insts.push(new_alloca);
                        // Store param values into param allocas
                        if *dest < param_count {
                            if let Some(arg) = call_args.get(*dest) {
                                func_blocks[bi].insts.push(MirInst::Store {
                                    dest: new_local,
                                    value: arg.clone(),
                                });
                            }
                        }
                    }
                }

                // Map and copy non-alloca instructions
                for inst in &callee_block.insts {
                    if matches!(inst, MirInst::Alloca { .. }) {
                        continue;
                    }
                    let mut new_inst = inst.clone();
                    Self::remap_inst_locals(&mut new_inst, &local_map);
                    func_blocks[bi].insts.push(new_inst);
                }
                // Handle return terminator: store result then continue to next block
                if let MirTerminator::Return(val) = &callee_block.terminator {
                    let mapped_val = Self::map_value(val, &local_map);
                    if let Some(dest) = dest_local {
                        func_blocks[bi].insts.push(MirInst::Store {
                            dest,
                            value: mapped_val,
                        });
                    }
                }
            }

            module.functions[func_idx].basic_blocks = func_blocks;
        }
    }

    fn map_value(val: &MirValue, map: &HashMap<usize, usize>) -> MirValue {
        match val {
            MirValue::Local(id) => {
                if let Some(new_id) = map.get(id) {
                    MirValue::Local(*new_id)
                } else {
                    val.clone()
                }
            }
            _ => val.clone(),
        }
    }

    fn remap_inst_locals(inst: &mut MirInst, map: &HashMap<usize, usize>) {
        match inst {
            MirInst::Store { dest, value } => {
                if let Some(new_dest) = map.get(dest) {
                    *dest = *new_dest;
                }
                *value = Self::map_value(value, map);
            }
            MirInst::Load { dest, src } => {
                if let Some(new_dest) = map.get(dest) {
                    *dest = *new_dest;
                }
                if let Some(new_src) = map.get(src) {
                    *src = *new_src;
                }
            }
            MirInst::BinaryOp { dest, left, right, .. } => {
                if let Some(new_dest) = map.get(dest) {
                    *dest = *new_dest;
                }
                *left = Self::map_value(left, map);
                *right = Self::map_value(right, map);
            }
            MirInst::UnaryOp { dest, operand, .. } => {
                if let Some(new_dest) = map.get(dest) {
                    *dest = *new_dest;
                }
                *operand = Self::map_value(operand, map);
            }
            MirInst::Call { dest, args, .. } => {
                if let Some(d) = dest {
                    if let Some(new_dest) = map.get(d) {
                        *dest = Some(*new_dest);
                    }
                }
                for arg in args.iter_mut() {
                    *arg = Self::map_value(arg, map);
                }
            }
            MirInst::Cast { dest, value, .. } => {
                if let Some(new_dest) = map.get(dest) {
                    *dest = *new_dest;
                }
                *value = Self::map_value(value, map);
            }
            MirInst::PtrOffset { dest, ptr, index, .. } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
                if let Some(new_ptr) = map.get(ptr) { *ptr = *new_ptr; }
                *index = Self::map_value(index, map);
            }
            MirInst::PtrStore { dest, ptr, index, value } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
                if let Some(new_ptr) = map.get(ptr) { *ptr = *new_ptr; }
                *index = Self::map_value(index, map);
                *value = Self::map_value(value, map);
            }
            MirInst::FieldPtr { dest, ptr, .. } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
                if let Some(new_ptr) = map.get(ptr) { *ptr = *new_ptr; }
            }
            MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                if let Some(new_d) = map.get(dest_ptr_local) { *dest_ptr_local = *new_d; }
                if let Some(new_s) = map.get(src_alloca_local) { *src_alloca_local = *new_s; }
            }
            MirInst::FnAddr { dest, .. } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
            }
            MirInst::AddressOf { dest, local_id } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
                if let Some(new_id) = map.get(local_id) { *local_id = *new_id; }
            }
            MirInst::CallIndirect { dest, fn_ptr, args, .. } => {
                if let Some(d) = dest {
                    if let Some(new_dest) = map.get(d) { *dest = Some(*new_dest); }
                }
                if let Some(new_fp) = map.get(fn_ptr) { *fn_ptr = *new_fp; }
                for arg in args.iter_mut() {
                    *arg = Self::map_value(arg, map);
                }
            }
            MirInst::AsyncSpawn { dest, arg, .. } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
                *arg = Self::map_value(arg, map);
            }
            MirInst::AsyncAwait { dest, handle, return_type: _ } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
                if let Some(new_h) = map.get(handle) { *handle = *new_h; }
            }
            MirInst::Alloca { dest, .. } => {
                if let Some(new_dest) = map.get(dest) { *dest = *new_dest; }
            }
            _ => {}
        }
    }

    /// Evaluate const fn calls where all arguments are constants.
    /// Replaces the call with a Store of the constant result.
    fn const_eval(&self, module: &mut MirModule) {
        // Build a map of const function name -> index in module
        let const_fns: HashMap<String, usize> = module.functions.iter()
            .enumerate()
            .filter(|(_, f)| f.is_const)
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        if const_fns.is_empty() {
            return;
        }

        let funcs = module.functions.clone(); // clone for borrow-free access
        let func_indices: HashMap<String, usize> = module.functions.iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        for func in &mut module.functions {
            for bb in &mut func.basic_blocks {
                let mut i = 0;
                while i < bb.insts.len() {
                    if let MirInst::Call { dest: Some(dest), name, args } = &bb.insts[i] {
                        if const_fns.contains_key(name) {
                            // Check if all args are constants
                            let const_args: Vec<MirConstant> = args.iter()
                                .filter_map(|a| match a {
                                    MirValue::Constant(c) => Some(c.clone()),
                                    _ => None,
                                })
                                .collect();
                            if const_args.len() == args.len() {
                                // Evaluate the const fn
                                if let Some(func_idx) = func_indices.get(name) {
                                    let callee = &funcs[*func_idx];
                                    let result = Self::eval_mir_function(callee, &const_args, &funcs);
                                    if let Some(result) = result {
                                        bb.insts[i] = MirInst::Store {
                                            dest: *dest,
                                            value: MirValue::Constant(result),
                                        };
                                    }
                                }
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    /// Evaluate a MIR function with constant arguments.
    /// Returns the constant result, or None if evaluation fails.
    fn eval_mir_function(func: &MirFunction, args: &[MirConstant], all_funcs: &[MirFunction]) -> Option<MirConstant> {
        use MirInst::*;

        // Map local ID -> constant value
        let mut locals: HashMap<usize, MirConstant> = HashMap::new();

        // Initialize params from args
        for (i, arg) in args.iter().enumerate() {
            locals.insert(i, arg.clone());
        }

        // Track current block
        let mut block_idx = 0;
        let mut visited = HashSet::new();
        let mut safety = 0;

        loop {
            safety += 1;
            if safety > 10000 {
                return None; // safety limit
            }
            if block_idx >= func.basic_blocks.len() || !visited.insert(block_idx) {
                return None; // loop detected or out of bounds
            }

            let block = &func.basic_blocks[block_idx];

            for inst in &block.insts {
                match inst {
                    Alloca { dest, .. } => {
                        locals.entry(*dest).or_insert(MirConstant::I32(0));
                    }
                    Store { dest, value } => {
                        if let Some(val) = Self::eval_mir_value(value, args, &locals) {
                            locals.insert(*dest, val);
                        } else {
                            return None;
                        }
                    }
                    Load { dest, src } => {
                        if let Some(val) = locals.get(src).cloned() {
                            locals.insert(*dest, val);
                        } else if src < &args.len() {
                            locals.insert(*dest, args[*src].clone());
                        } else {
                            return None;
                        }
                    }
                    BinaryOp { dest, op, left, right } => {
                        if let (Some(l), Some(r)) = (
                            Self::eval_mir_value(left, args, &locals),
                            Self::eval_mir_value(right, args, &locals),
                        ) {
                            locals.insert(*dest, Self::eval_const_binary(op, &l, &r));
                        } else {
                            return None;
                        }
                    }
                    UnaryOp { dest, op, operand } => {
                        if let Some(val) = Self::eval_mir_value(operand, args, &locals) {
                            locals.insert(*dest, match op {
                                MirUnaryOp::Neg => match val {
                                    MirConstant::I32(n) => MirConstant::I32(n.wrapping_neg()),
                                    MirConstant::I64(n) => MirConstant::I64(n.wrapping_neg()),
                                    _ => return None,
                                },
                                MirUnaryOp::Not => match val {
                                    MirConstant::Bool(b) => MirConstant::Bool(!b),
                                    MirConstant::I32(n) => MirConstant::I32(!n),
                                    MirConstant::I64(n) => MirConstant::I64(!n),
                                    _ => return None,
                                },
                                MirUnaryOp::BitNot => match val {
                                    MirConstant::I32(n) => MirConstant::I32(!n),
                                    MirConstant::I64(n) => MirConstant::I64(!n),
                                    _ => return None,
                                },
                            });
                        } else {
                            return None;
                        }
                    }
                    Cast { dest, value, to_type } => {
                        if let Some(val) = Self::eval_mir_value(value, args, &locals) {
                            let result = match (val.clone(), to_type) {
                                (MirConstant::I32(n), MirType::I64) => MirConstant::I64(n as i64),
                                (MirConstant::I32(n), MirType::Bool) => MirConstant::Bool(n != 0),
                                (MirConstant::I64(n), MirType::I32) => MirConstant::I32(n as i32),
                                (MirConstant::Bool(b), MirType::I32) => MirConstant::I32(if b { 1 } else { 0 }),
                                _ => val,
                            };
                            locals.insert(*dest, result);
                        } else {
                            return None;
                        }
                    }
                    Call { dest: Some(d), name, args: call_args } => {
                        // Recurse into const fn
                        let const_args: Vec<MirConstant> = call_args.iter()
                            .filter_map(|a| Self::eval_mir_value(a, args, &locals))
                            .collect();
                        if const_args.len() == call_args.len() {
                            if let Some(callee) = all_funcs.iter().find(|f| f.name == *name) {
                                if let Some(result) = Self::eval_mir_function(callee, &const_args, all_funcs) {
                                    locals.insert(*d, result);
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => {
                        // Unsupported instruction in const fn (FieldPtr, PtrOffset, etc.)
                        return None;
                    }
                }
            }

            // Handle terminator
            match &block.terminator {
                MirTerminator::Return(val) => {
                    return Self::eval_mir_value(val, args, &locals);
                }
                MirTerminator::Br(label) => {
                    block_idx = func.basic_blocks.iter().position(|b| &b.label == label)?;
                }
                MirTerminator::CondBr { cond, true_block, false_block } => {
                    let cond_val = Self::eval_mir_value(cond, args, &locals)?;
                    let take_true = match cond_val {
                        MirConstant::Bool(b) => b,
                        MirConstant::I32(n) => n != 0,
                        _ => return None,
                    };
                    let target = if take_true { true_block } else { false_block };
                    block_idx = func.basic_blocks.iter().position(|b| b.label == *target)?;
                }
                MirTerminator::Unreachable => {
                    return None;
                }
            }
        }
    }

    fn eval_mir_value(value: &MirValue, args: &[MirConstant], locals: &HashMap<usize, MirConstant>) -> Option<MirConstant> {
        match value {
            MirValue::Constant(c) => Some(c.clone()),
            MirValue::Local(id) => locals.get(id).cloned(),
            MirValue::Param(id) => args.get(*id).cloned(),
        }
    }

    fn eval_const_binary(op: &MirBinaryOp, left: &MirConstant, right: &MirConstant) -> MirConstant {
        match (left, right) {
            (MirConstant::I32(l), MirConstant::I32(r)) => {
                match op {
                    MirBinaryOp::Add => MirConstant::I32(l.wrapping_add(*r)),
                    MirBinaryOp::Sub => MirConstant::I32(l.wrapping_sub(*r)),
                    MirBinaryOp::Mul => MirConstant::I32(l.wrapping_mul(*r)),
                    MirBinaryOp::Div => if *r != 0 { MirConstant::I32(l.wrapping_div(*r)) } else { MirConstant::I32(0) },
                    MirBinaryOp::Rem => if *r != 0 { MirConstant::I32(l.wrapping_rem(*r)) } else { MirConstant::I32(0) },
                    MirBinaryOp::And => MirConstant::I32(l & r),
                    MirBinaryOp::Or => MirConstant::I32(l | r),
                    MirBinaryOp::Xor => MirConstant::I32(l ^ r),
                    MirBinaryOp::Shl => MirConstant::I32(l.wrapping_shl(*r as u32)),
                    MirBinaryOp::Shr => MirConstant::I32(l.wrapping_shr(*r as u32)),
                    MirBinaryOp::Eq => MirConstant::Bool(l == r),
                    MirBinaryOp::Neq => MirConstant::Bool(l != r),
                    MirBinaryOp::Lt => MirConstant::Bool(l < r),
                    MirBinaryOp::Gt => MirConstant::Bool(l > r),
                    MirBinaryOp::Le => MirConstant::Bool(l <= r),
                    MirBinaryOp::Ge => MirConstant::Bool(l >= r),
                }
            }
            (MirConstant::I64(l), MirConstant::I64(r)) => {
                match op {
                    MirBinaryOp::Add => MirConstant::I64(l.wrapping_add(*r)),
                    MirBinaryOp::Sub => MirConstant::I64(l.wrapping_sub(*r)),
                    MirBinaryOp::Mul => MirConstant::I64(l.wrapping_mul(*r)),
                    MirBinaryOp::Div => if *r != 0 { MirConstant::I64(l.wrapping_div(*r)) } else { MirConstant::I64(0) },
                    MirBinaryOp::Rem => if *r != 0 { MirConstant::I64(l.wrapping_rem(*r)) } else { MirConstant::I64(0) },
                    MirBinaryOp::And => MirConstant::I64(l & r),
                    MirBinaryOp::Or => MirConstant::I64(l | r),
                    MirBinaryOp::Xor => MirConstant::I64(l ^ r),
                    MirBinaryOp::Shl => MirConstant::I64(l.wrapping_shl(*r as u32)),
                    MirBinaryOp::Shr => MirConstant::I64(l.wrapping_shr(*r as u32)),
                    MirBinaryOp::Eq => MirConstant::Bool(l == r),
                    MirBinaryOp::Neq => MirConstant::Bool(l != r),
                    MirBinaryOp::Lt => MirConstant::Bool(l < r),
                    MirBinaryOp::Gt => MirConstant::Bool(l > r),
                    MirBinaryOp::Le => MirConstant::Bool(l <= r),
                    MirBinaryOp::Ge => MirConstant::Bool(l >= r),
                }
            }
            (MirConstant::Bool(l), MirConstant::Bool(r)) => {
                match op {
                    MirBinaryOp::And => MirConstant::Bool(*l && *r),
                    MirBinaryOp::Or => MirConstant::Bool(*l || *r),
                    MirBinaryOp::Eq => MirConstant::Bool(l == r),
                    MirBinaryOp::Neq => MirConstant::Bool(l != r),
                    MirBinaryOp::Lt => MirConstant::Bool(!l && *r),
                    MirBinaryOp::Gt => MirConstant::Bool(*l && !r),
                    MirBinaryOp::Le => MirConstant::Bool(*l <= *r),
                    MirBinaryOp::Ge => MirConstant::Bool(*l >= *r),
                    _ => MirConstant::Bool(false),
                }
            }
            _ => MirConstant::Void,
        }
    }

    /// Collect all local IDs referenced by a MirValue.
    fn collect_terminator_refs(term: &MirTerminator, set: &mut HashSet<usize>) {
        match term {
            MirTerminator::Return(v) => Self::collect_value_refs(v, set),
            MirTerminator::CondBr { cond, .. } => Self::collect_value_refs(cond, set),
            _ => {}
        }
    }

    fn collect_value_refs(value: &MirValue, set: &mut HashSet<usize>) {
        if let MirValue::Local(id) = value {
            set.insert(*id);
        }
    }

    /// Constant folding: evaluate constant expressions at compile time.
    fn constant_fold(&self, func: &mut MirFunction) {
        for bb in &mut func.basic_blocks {
            let mut i = 0;
            while i < bb.insts.len() {
                if let MirInst::BinaryOp { dest, op, left, right } = &bb.insts[i] {
                    if let (MirValue::Constant(lc), MirValue::Constant(rc)) = (left, right) {
                        let result = eval_const_binary_op(op, lc, rc);
                        bb.insts[i] = MirInst::Store {
                            dest: *dest,
                            value: MirValue::Constant(result),
                        };
                    }
                }
                i += 1;
            }
        }
    }

    /// Dead code elimination: remove stores whose value is never loaded.
    fn dead_code_elim(&self, func: &mut MirFunction, move_locals: &HashSet<usize>) {
        // Simple DCE: find locals that are stored but never used
        let mut stored = HashSet::new();
        let mut used = HashSet::new();

        for bb in &func.basic_blocks {
            // Track references in terminators (Return, CondBr)
            Self::collect_terminator_refs(&bb.terminator, &mut used);

            for inst in &bb.insts {
                match inst {
                    MirInst::Store { dest, value } => {
                        stored.insert(*dest);
                        Self::collect_value_refs(value, &mut used);
                    }
                    MirInst::Load { dest, src } => {
                        used.insert(*dest);
                        used.insert(*src);
                    }
                    MirInst::Alloca { .. } => {}
                    MirInst::BinaryOp { dest, left, right, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(left, &mut used);
                        Self::collect_value_refs(right, &mut used);
                    }
                    MirInst::UnaryOp { dest, operand, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(operand, &mut used);
                    }
                    MirInst::Call { dest, args, .. } => {
                        if let Some(d) = dest {
                            used.insert(*d);
                        }
                        for arg in args {
                            Self::collect_value_refs(arg, &mut used);
                        }
                    }
                    MirInst::PtrOffset { dest, ptr, index, .. } => {
                        used.insert(*dest);
                        used.insert(*ptr);
                        Self::collect_value_refs(index, &mut used);
                    }
                    MirInst::PtrStore { dest, ptr, index, value } => {
                        used.insert(*dest);
                        used.insert(*ptr);
                        Self::collect_value_refs(index, &mut used);
                        Self::collect_value_refs(value, &mut used);
                    }
                    MirInst::Cast { dest, value, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(value, &mut used);
                    }
                    MirInst::FieldPtr { dest, ptr, .. } => {
                        used.insert(*dest);
                        used.insert(*ptr);
                    }
                    MirInst::ArrayElemPtr { dest, ptr, index, .. } => {
                        used.insert(*dest);
                        used.insert(*ptr);
                        Self::collect_value_refs(index, &mut used);
                    }
                    MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                        used.insert(*dest_ptr_local);
                        used.insert(*src_alloca_local);
                    }
                    MirInst::FnAddr { dest, .. } => {
                        used.insert(*dest);
                    }
                    MirInst::AddressOf { dest, local_id } => {
                        used.insert(*dest);
                        used.insert(*local_id);
                    }
                    MirInst::CallIndirect { dest, fn_ptr, args, .. } => {
                        if let Some(d) = dest {
                            used.insert(*d);
                        }
                        used.insert(*fn_ptr);
                        for arg in args {
                            Self::collect_value_refs(arg, &mut used);
                        }
                    }
                    MirInst::AsyncSpawn { dest, arg, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(arg, &mut used);
                    }
                    MirInst::AsyncAwait { dest, handle, return_type: _ } => {
                        used.insert(*dest);
                        used.insert(*handle);
                    }
                    MirInst::SliceMake { dest, ptr, len, .. } => {
                        used.insert(*dest);
                        Self::collect_value_refs(ptr, &mut used);
                        Self::collect_value_refs(len, &mut used);
                    }
                }
            }
        }

        for bb in &mut func.basic_blocks {
            bb.insts.retain(|inst| {
                match inst {
                    MirInst::Store { dest, value: MirValue::Local(src) } => {
                        // Keep store if dest is used OR if transferring ownership of a Move type.
                        // Otherwise DCE could remove the store, preventing ownership transfer
                        // and causing double-free when both src and dest are freed.
                        used.contains(dest) || move_locals.contains(src)
                    }
                    MirInst::Store { value: MirValue::Param(_), .. } => true,
                    MirInst::Store { dest, .. } => used.contains(dest),
                    MirInst::Alloca { dest, .. } => used.contains(dest) || stored.contains(dest),
                    _ => true,
                }
            });
        }
    }

    /// Remove unreachable basic blocks.
    fn remove_unreachable_blocks(&self, func: &mut MirFunction) {
        if func.basic_blocks.is_empty() {
            return;
        }

        let mut reachable = HashSet::new();
        let mut worklist = vec![0usize];
        reachable.insert(0usize);

        while let Some(idx) = worklist.pop() {
            if let Some(bb) = func.basic_blocks.get(idx) {
                match &bb.terminator {
                    MirTerminator::Br(label) => {
                        if let Some(target) = func.basic_blocks.iter().position(|b| &b.label == label) {
                            if reachable.insert(target) {
                                worklist.push(target);
                            }
                        }
                    }
                    MirTerminator::CondBr { true_block, false_block, .. } => {
                        for label in [true_block, false_block] {
                            if let Some(target) = func.basic_blocks.iter().position(|b| &b.label == label) {
                                if reachable.insert(target) {
                                    worklist.push(target);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Map old indices to new order
        let mut new_blocks = Vec::new();
        let mut old_to_new = HashMap::new();
        for (old_idx, bb) in func.basic_blocks.iter().enumerate() {
            if reachable.contains(&old_idx) {
                old_to_new.insert(old_idx, new_blocks.len());
                new_blocks.push(bb.clone());
            }
        }

        // Fix terminator labels to point to new indices (they're string labels, so they're fine)

        func.basic_blocks = new_blocks;
    }
}

fn eval_const_binary_op(op: &MirBinaryOp, left: &MirConstant, right: &MirConstant) -> MirConstant {
    match (left, right) {
        (MirConstant::I32(l), MirConstant::I32(r)) => {
            match op {
                MirBinaryOp::Add => MirConstant::I32(l.wrapping_add(*r)),
                MirBinaryOp::Sub => MirConstant::I32(l.wrapping_sub(*r)),
                MirBinaryOp::Mul => MirConstant::I32(l.wrapping_mul(*r)),
                MirBinaryOp::Div => if *r != 0 { MirConstant::I32(l.wrapping_div(*r)) } else { MirConstant::I32(0) },
                MirBinaryOp::Rem => if *r != 0 { MirConstant::I32(l.wrapping_rem(*r)) } else { MirConstant::I32(0) },
                MirBinaryOp::And => MirConstant::I32(l & r),
                MirBinaryOp::Or => MirConstant::I32(l | r),
                MirBinaryOp::Xor => MirConstant::I32(l ^ r),
                MirBinaryOp::Eq => MirConstant::Bool(l == r),
                MirBinaryOp::Neq => MirConstant::Bool(l != r),
                MirBinaryOp::Lt => MirConstant::Bool(l < r),
                MirBinaryOp::Gt => MirConstant::Bool(l > r),
                MirBinaryOp::Le => MirConstant::Bool(l <= r),
                MirBinaryOp::Ge => MirConstant::Bool(l >= r),
                _ => MirConstant::I32(0),
            }
        }
        (MirConstant::Bool(l), MirConstant::Bool(r)) => {
            match op {
                MirBinaryOp::And => MirConstant::Bool(*l && *r),
                MirBinaryOp::Or => MirConstant::Bool(*l || *r),
                MirBinaryOp::Eq => MirConstant::Bool(l == r),
                MirBinaryOp::Neq => MirConstant::Bool(l != r),
                _ => MirConstant::Bool(false),
            }
        }
        _ => MirConstant::Void,
    }
}

/// Extract the destination local from an instruction (for GVN).
fn dest_of(inst: &MirInst) -> Option<usize> {
    match inst {
        MirInst::Store { dest, .. } => Some(*dest),
        MirInst::BinaryOp { dest, .. } => Some(*dest),
        MirInst::UnaryOp { dest, .. } => Some(*dest),
        MirInst::Cast { dest, .. } => Some(*dest),
        MirInst::Call { dest, .. } => *dest,
        MirInst::PtrOffset { dest, .. } => Some(*dest),
        MirInst::PtrStore { dest, .. } => Some(*dest),
        MirInst::FieldPtr { dest, .. } => Some(*dest),
        MirInst::FnAddr { dest, .. } => Some(*dest),
        MirInst::AsyncSpawn { dest, .. } => Some(*dest),
        MirInst::AsyncAwait { dest, .. } => Some(*dest),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_fold_add() {
        let mut func = MirFunction::new("test");
        let mut bb = MirBasicBlock::new("entry");
        bb.insts.push(MirInst::BinaryOp {
            dest: 0,
            op: MirBinaryOp::Add,
            left: MirValue::Constant(MirConstant::I32(2)),
            right: MirValue::Constant(MirConstant::I32(3)),
        });
        bb.terminator = MirTerminator::Return(MirValue::Constant(MirConstant::Void));
        func.basic_blocks.push(bb);

        let opt = Optimizer::new();
        opt.constant_fold(&mut func);

        if let MirInst::Store { value: MirValue::Constant(MirConstant::I32(n)), .. } = &func.basic_blocks[0].insts[0] {
            assert_eq!(*n, 5);
        } else {
            panic!("expected folded constant");
        }
    }

    #[test]
    fn test_remove_unreachable() {
        let mut func = MirFunction::new("test");
        func.basic_blocks.push(MirBasicBlock::new("entry"));
        func.basic_blocks[0].terminator = MirTerminator::Return(MirValue::Constant(MirConstant::Void));
        func.basic_blocks.push(MirBasicBlock::new("dead")); // unreachable

        assert_eq!(func.basic_blocks.len(), 2);
        let opt = Optimizer::new();
        opt.remove_unreachable_blocks(&mut func);
        assert_eq!(func.basic_blocks.len(), 1);
    }

    #[test]
    fn test_constant_fold_mul() {
        let mut func = MirFunction::new("test");
        let mut bb = MirBasicBlock::new("entry");
        bb.insts.push(MirInst::BinaryOp {
            dest: 0,
            op: MirBinaryOp::Mul,
            left: MirValue::Constant(MirConstant::I32(6)),
            right: MirValue::Constant(MirConstant::I32(7)),
        });
        bb.terminator = MirTerminator::Return(MirValue::Constant(MirConstant::Void));
        func.basic_blocks.push(bb);

        let opt = Optimizer::new();
        opt.constant_fold(&mut func);

        if let MirInst::Store { value: MirValue::Constant(MirConstant::I32(n)), .. } = &func.basic_blocks[0].insts[0] {
            assert_eq!(*n, 42);
        } else {
            panic!("expected folded constant 42");
        }
    }

    #[test]
    fn test_dce_preserves_conditional() {
        let mut func = MirFunction::new("test");
        let mut bb = MirBasicBlock::new("entry");
        // Store used by CondBr condition — must survive DCE
        bb.insts.push(MirInst::Alloca { dest: 0, type_: MirType::I32, name: "cond".into() });
        bb.insts.push(MirInst::Store { dest: 0, value: MirValue::Constant(MirConstant::I32(1)) });
        // Store never loaded — should be removed
        bb.insts.push(MirInst::Alloca { dest: 1, type_: MirType::I32, name: "dead".into() });
        bb.insts.push(MirInst::Store { dest: 1, value: MirValue::Constant(MirConstant::I32(99)) });
        bb.terminator = MirTerminator::CondBr {
            cond: MirValue::Local(0),
            true_block: "then".into(),
            false_block: "done".into(),
        };
        func.basic_blocks.push(bb);
        func.basic_blocks.push(MirBasicBlock::new("then"));
        func.basic_blocks.push(MirBasicBlock::new("done"));

        let opt = Optimizer::new();
        let empty: HashSet<usize> = HashSet::new();
        opt.dead_code_elim(&mut func, &empty);

        // Entry block should keep cond store but drop dead store
        let kept_stores: Vec<_> = func.basic_blocks[0].insts.iter()
            .filter(|i| matches!(i, MirInst::Store { .. }))
            .collect();
        assert_eq!(kept_stores.len(), 1, "DCE should remove the unused store");
        // The remaining store should be the cond variable (dest 0)
        if let MirInst::Store { dest, .. } = &kept_stores[0] {
            assert_eq!(*dest, 0, "the store used by CondBr must survive DCE");
        } else {
            panic!("expected a Store instruction");
        }
    }
}
