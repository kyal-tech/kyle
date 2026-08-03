use kyc_core::ast::ParamMode;
/// Borrow/ownership analysis pass.
///
/// Tracks move/borrow state for each value:
/// - Copy types (primitives): passed by value, no tracking needed
/// - Move types (heap-allocated): single owner, freed when scope exits
///
/// Classification:
/// - Copy: I1, I8, I16, I32, I64, F32, F64, Bool, Char, Void, Ptr
/// - Move: Str, List, Struct, Dict, Array
///
/// This pass inserts `kl_free` calls for Move values at scope exit
/// (end of basic block) and when a Move local is overwritten.

use std::collections::{BTreeSet, HashMap, VecDeque};
use crate::mir::*;

#[derive(Clone, Copy, PartialEq)]
enum BorrowState {
    NotBorrowed,
    ImmBorrowed(u32),
    MutBorrowed,
}

pub struct BorrowAnalysis {
    errors: Vec<String>,
}

impl BorrowAnalysis {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn run(&mut self, module: &mut MirModule) {
        // Build a function param_modes lookup map for is_move_func.
        // Clone to avoid borrow conflicts with mutable iteration.
        let func_map: std::collections::HashMap<String, Vec<ParamMode>> = module.functions.iter()
            .map(|f| (f.name.clone(), f.param_modes.clone()))
            .collect();
        for func in &mut module.functions {
            self.process_function(func, &func_map);
        }
    }

    /// Compute predecessors for each basic block.
    fn build_preds(func: &MirFunction) -> Vec<Vec<usize>> {
        let n = func.basic_blocks.len();
        let mut preds = vec![vec![]; n];
        let mut succs: Vec<Vec<usize>> = vec![vec![]; n];

        // Build successor map from terminators
        for (i, block) in func.basic_blocks.iter().enumerate() {
            let succ = Self::terminator_successors(&block.terminator, &func.basic_blocks);
            for s in succ {
                succs[i].push(s);
            }
        }
        // Invert to get predecessors
        for (i, succ) in succs.iter().enumerate() {
            for &s in succ {
                if s < n {
                    preds[s].push(i);
                }
            }
        }
        preds
    }

    /// Get successor block indices from a terminator.
    fn terminator_successors(
        term: &MirTerminator,
        blocks: &[MirBasicBlock],
    ) -> Vec<usize> {
        match term {
            MirTerminator::Br(label) => {
                blocks.iter().position(|b| b.label == *label).map(|i| vec![i]).unwrap_or_default()
            }
            MirTerminator::CondBr { true_block, false_block, .. } => {
                let mut v = Vec::new();
                if let Some(i) = blocks.iter().position(|b| b.label == *true_block) {
                    v.push(i);
                }
                if let Some(i) = blocks.iter().position(|b| b.label == *false_block) {
                    v.push(i);
                }
                v
            }
            MirTerminator::Return(_) | MirTerminator::Unreachable => vec![],
        }
    }

    /// Forward dataflow: compute alive_in for each block using intersection at joins.
    /// A local is alive entering a block only if it's alive at the exit of ALL predecessors.
    /// Compute alive_in for each block using forward dataflow with intersection at joins.
    /// Uses a pure processing function that does NOT check aliveness (no error reporting).
    fn compute_alive_in<'a>(
        &self,
        func: &MirFunction,
        move_locals: &BTreeSet<usize>,
        local_types: &HashMap<usize, MirType>,
        param_locals: &BTreeSet<usize>,
        func_map: &std::collections::HashMap<String, Vec<ParamMode>>,
        load_map: &HashMap<usize, usize>,
    ) -> Vec<BTreeSet<usize>> {
        let n = func.basic_blocks.len();
        let preds = Self::build_preds(func);

        let mut alive_in: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
        // Initialize alive_out to ALL move-type locals (TOP of lattice).
        // With intersection-at-joins semantics, TOP ensures that a local which
        // is alive on at least one predecessor survives through the merge.
        // Without this, loop back-edges cause false use-after-move errors:
        // the body's alive_out starts empty, so the header intersection drops
        // all locals from the entry block, making them dead inside the loop.
        let mut alive_out: Vec<BTreeSet<usize>> = (0..n).map(|_| move_locals.clone()).collect();
        let mut worklist: VecDeque<usize> = (0..n).collect();

        while let Some(b) = worklist.pop_front() {
            let block = &func.basic_blocks[b];

            // Compute alive_in[b] from predecessors
            if b == 0 {
                let mut entry_alive = BTreeSet::new();
                for &p in param_locals {
                    if local_types.get(&p).map_or(false, |t| is_move_type(t)) {
                        entry_alive.insert(p);
                    }
                }
                alive_in[b] = entry_alive;
            } else if let Some(pred_list) = preds.get(b) {
                if pred_list.is_empty() {
                    alive_in[b].clear();
                } else {
                    let mut intersection: Option<BTreeSet<usize>> = None;
                    for &p in pred_list {
                        if let Some(out) = alive_out.get(p) {
                            match &intersection {
                                None => intersection = Some(out.clone()),
                                Some(current) => {
                                    intersection = Some(current.intersection(out).cloned().collect());
                                }
                            }
                        }
                    }
                    alive_in[b] = intersection.unwrap_or_default();
                }
            }

            // Compute alive_out by processing instructions (no error checking)
            let mut alive = alive_in[b].clone();
            Self::compute_alive_out(block, &mut alive, move_locals, func_map, load_map);
            let new_out = alive;

            if new_out != alive_out[b] {
                alive_out[b] = new_out;
                let succs = Self::terminator_successors(&block.terminator, &func.basic_blocks);
                for s in succs {
                    if !worklist.contains(&s) {
                        worklist.push_back(s);
                    }
                }
            }
        }

        alive_in
    }

    /// Compute alive_out for a block by processing instructions and terminator.
    /// Does NOT check aliveness (no error reporting) — only tracks alive/dead state.
    fn compute_alive_out(
        block: &MirBasicBlock,
        alive: &mut BTreeSet<usize>,
        move_locals: &BTreeSet<usize>,
        func_map: &std::collections::HashMap<String, Vec<ParamMode>>,
        load_map: &HashMap<usize, usize>,
    ) {
        for inst in &block.insts {
            match inst {
                MirInst::Alloca { dest, type_, .. } => {
                    if is_move_type(type_) {
                        alive.insert(*dest);
                    }
                }
                MirInst::Load { dest, .. } => {
                    if move_locals.contains(dest) {
                        alive.insert(*dest);
                    }
                }
                MirInst::Store { dest, value } => {
                    if let MirValue::Local(src) = value {
                        alive.remove(src);
                        // Also mark the original source of a Load alias as consumed, if still alive
                        if let Some(&orig) = load_map.get(src) {
                            if alive.contains(&orig) {
                                alive.remove(&orig);
                            }
                        }
                    }
                    if move_locals.contains(dest) {
                        alive.insert(*dest);
                    }
                }
                MirInst::Call { dest, name, args } => {
                    // MOVE-BY-DEFAULT: args are consumed unless the param is Borrow/MutableBorrow
                    // Functions NOT in func_map (runtime externs like ky_list_push) NEVER consume
                    // their arguments — they only borrow. Must match process_inst logic below.
                    let modes = func_map.get(name);
                    for (i, arg) in args.iter().enumerate() {
                        if let MirValue::Local(l) = arg {
                            let is_borrow = modes.map_or(false, |m| i < m.len() && (m[i] == ParamMode::Borrow || m[i] == ParamMode::MutableBorrow));
                            if modes.is_some() && !is_borrow {
                                alive.remove(l);
                                // Also mark the original source of a Load alias as consumed, if still alive
                                if let Some(&orig) = load_map.get(l) {
                                    if alive.contains(&orig) {
                                        alive.remove(&orig);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(d) = dest {
                        if move_locals.contains(d) {
                            alive.insert(*d);
                        }
                    }
                }
                _ => {}
            }
        }
        // Terminator: return removes the returned local (caller takes ownership)
        if let MirTerminator::Return(MirValue::Local(l)) = &block.terminator {
            alive.remove(l);
        }
    }

    fn process_function(&mut self, func: &mut MirFunction, func_map: &std::collections::HashMap<String, Vec<ParamMode>>) {
        let move_locals = self.find_move_locals(func);
        let local_names = self.build_local_names(func);
        let local_types = self.build_local_types(func);

        if move_locals.is_empty() {
            return;
        }

        // Identify parameter locals: locals that receive a Param value via Store
        let param_locals: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Store { dest, value: MirValue::Param(_) } = inst {
                    Some(*dest)
                } else { None }
            })
            .collect();

        // Identify locals initialized from string constants (literals).
        // These point to static data (.rodata) and must NOT be freed.
        let string_literal_locals: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                match inst {
                    MirInst::Store { dest, value } => {
                        match value {
                            MirValue::Constant(MirConstant::String(_)) => Some(*dest),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            })
            .collect();

        // Track Load-only destinations (aliases): these are temps that hold the
        // same pointer as their source. They should NOT be freed independently,
        // otherwise the original and the alias cause a double-free.
        let load_only: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Load { dest, .. } = inst {
                    Some(*dest)
                } else { None }
            })
            .collect();

        // Track locals that hold borrowed references from list/dict get operations.
        // ky_list_get returns a pointer INTO the list's internal buffer — NOT an owned
        // value. Freeing these would corrupt the list and cause use-after-free.
        let get_results_raw: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Call { dest, name, .. } = inst {
                    if name == "ky_list_get" || name == "ky_dict_get" {
                        return *dest;
                    }
                }
                None
            })
            .collect();
        let list_get_results: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Cast { dest, value, .. } = inst {
                    if let MirValue::Local(src) = value {
                        if get_results_raw.contains(src) {
                            return Some(*dest);
                        }
                    }
                }
                if let MirInst::Call { dest, name, .. } = inst {
                    if name == "ky_list_get" || name == "ky_dict_get" {
                        return *dest;
                    }
                }
                None
            })
            .collect();

        // Track ky_clone_str results: these are heap-allocated copies that need
        // ky_free, but the SSA/codegen has issues freeing them across multiple
        // return blocks. Exclude for now to avoid runtime crashes.
        let clone_results: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Call { dest, name, .. } = inst {
                    if name == "ky_clone_str" { return *dest; }
                }
                None
            })
            .collect();

        // Track ^&T ref param dereference results: these are borrowed values,
        // NOT owned — they must NOT be freed at function exit.
        let ref_deref_results: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Call { dest, name, .. } = inst {
                    if name == "ky_ptr_read_ptr" || name == "ky_ptr_read_i32" {
                        return *dest;
                    }
                }
                None
            })
            .collect();

        // Track Load source locals: when a Load creates an alias for a move-type
        // (like str), the source holds the same pointer as the alias. We must not
        // free the source if an alias still holds its pointer.
        let mut load_sources: BTreeSet<usize> = BTreeSet::new();
        let mut load_map: HashMap<usize, usize> = HashMap::new();
        let mut field_loaded: BTreeSet<usize> = BTreeSet::new();
        // First pass: find all FieldPtr targets
        let field_ptrs: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::FieldPtr { dest, .. } = inst { Some(*dest) } else { None }
            })
            .collect();
        for block in &func.basic_blocks {
            for inst in &block.insts {
                if let MirInst::Load { dest, src } = inst {
                    if local_types.get(dest).map_or(false, |t| is_move_type(t)) {
                        load_sources.insert(*src);
                        load_map.insert(*dest, *src);
                        // Values loaded from struct fields should NOT be freed (field still owns them)
                        if field_ptrs.contains(src) {
                            field_loaded.insert(*dest);
                        }
                    }
                }
            }
        }

        let alive_in = self.compute_alive_in(func, &move_locals, &local_types, &param_locals, func_map, &load_map);

        let mut to_insert: Vec<(usize, usize)> = Vec::new();

        let mut borrow_states: HashMap<usize, BorrowState> = HashMap::new();

        for (block_idx, block) in func.basic_blocks.iter().enumerate() {
            let mut alive = if block_idx < alive_in.len() {
                alive_in[block_idx].clone()
            } else {
                BTreeSet::new()
            };
            // Reset borrow states at block boundary (conservative: borrows don't cross blocks)
            if block_idx > 0 {
                borrow_states.clear();
            }

            for inst in &block.insts {
                self.process_inst(inst, &mut alive, &move_locals, &local_names, &local_types, func_map, &load_map, &mut borrow_states);
            }

            // Handle terminator
            match &block.terminator {
                MirTerminator::Return(value) => {
                    if let MirValue::Local(l) = value {
                        // Return transfers ownership to the caller. The returned
                        // local doesn't need to be "alive" — it could be a newly
                        // allocated value (e.g., constructor returning `this`).
                        alive.remove(l);
                    }
                    // Free all remaining alive locals at function exit (except params, Load aliases,
                    // load sources (their pointer is held by the alias), and string literals)
                    for l in &alive {
                        if !param_locals.contains(l)
                            && !load_only.contains(l)
                            && !load_sources.contains(l)
                            && !string_literal_locals.contains(l)
                            && !field_loaded.contains(l)
                            && !list_get_results.contains(l)
                            && !clone_results.contains(l)
                            && !ref_deref_results.contains(l)
                        {
                            to_insert.push((block_idx, *l));
                        }
                    }
                }
                MirTerminator::CondBr { cond, .. } => {
                    if let MirValue::Local(l) = cond {
                        self.check_alive(*l, &alive, &move_locals, &local_names, &local_types, "read");
                    }
                }
                _ => {}
            }
        }

        // Insert kl_free instructions at return points
        to_insert.sort();
        let mut per_block: Vec<Vec<usize>> = Vec::new();
        for (block_idx, local) in to_insert {
            while per_block.len() <= block_idx {
                per_block.push(Vec::new());
            }
            per_block[block_idx].push(local);
        }
        for (block_idx, locals) in per_block.iter().enumerate() {
            if block_idx >= func.basic_blocks.len() {
                continue;
            }
            let block = &mut func.basic_blocks[block_idx];
            for local in locals {
                let free_name = match local_types.get(local) {
                    Some(MirType::List(_)) => "ky_list_free",
                    Some(MirType::Dict(_, _)) => "ky_dict_free",
                    Some(MirType::Set(_)) => "ky_set_free",
                    Some(MirType::Queue(_)) => "ky_queue_free",
                    Some(MirType::Stack(_)) => "ky_stack_free",
                    Some(MirType::Deque(_)) => "ky_deque_free",
                    Some(MirType::LinkedList(_)) => "ky_linked_list_free",
                    Some(MirType::Chan(_)) => "ky_channel_free",
                    _ => "ky_free",
                };
                block.insts.push(MirInst::Call {
                    dest: None,
                    name: free_name.to_string(),
                    args: vec![MirValue::Local(*local)],
                });
            }
        }
    }

    /// Process a single MIR instruction, updating the alive set.
    fn process_inst(
        &mut self,
        inst: &MirInst,
        alive: &mut BTreeSet<usize>,
        move_locals: &BTreeSet<usize>,
        local_names: &HashMap<usize, String>,
        local_types: &HashMap<usize, MirType>,
        func_map: &std::collections::HashMap<String, Vec<ParamMode>>,
        load_map: &HashMap<usize, usize>,
        borrow_states: &mut HashMap<usize, BorrowState>,
    ) {
        match inst {
            MirInst::Alloca { dest, type_, .. } => {
                if is_move_type(type_) {
                    alive.insert(*dest);
                }
            }
            MirInst::Load { dest, src } => {
                // Check that the source is alive (use-after-move detection)
                self.check_alive(*src, alive, move_locals, local_names, local_types, "read");
                if move_locals.contains(dest) {
                    alive.insert(*dest);
                }
            }
            MirInst::BinaryOp { left, right, .. } => {
                if let MirValue::Local(l) = left {
                    self.check_alive(*l, alive, move_locals, local_names, local_types, "read");
                }
                if let MirValue::Local(r) = right {
                    self.check_alive(*r, alive, move_locals, local_names, local_types, "read");
                }
            }
            MirInst::UnaryOp { operand, .. } => {
                if let MirValue::Local(l) = operand {
                    self.check_alive(*l, alive, move_locals, local_names, local_types, "read");
                }
            }
            MirInst::Cast { dest, value, .. } => {
                // Cast between move types (e.g., list<T> → list<U>) transfers ownership:
                // the source is consumed (moved) and the dest owns the value.
                if let MirValue::Local(src) = value {
                    let src_is_move = local_types.get(src).map_or(false, |t| is_move_type(t));
                    let dst_is_move = local_types.get(dest).map_or(false, |t| is_move_type(t));
                    if src_is_move && dst_is_move {
                        self.check_alive(*src, alive, move_locals, local_names, local_types, "move");
                        if alive.contains(src) {
                            alive.remove(src);
                        }
                        let orig = *load_map.get(src).unwrap_or(src);
                        if alive.contains(&orig) && *src != orig {
                            self.check_alive(orig, alive, move_locals, local_names, local_types, "move");
                            alive.remove(&orig);
                        }
                    } else {
                        self.check_alive(*src, alive, move_locals, local_names, local_types, "read");
                    }
                }
                if move_locals.contains(dest) {
                    alive.insert(*dest);
                }
            }
            MirInst::Store { dest, value } => {
                if let MirValue::Local(src) = value {
                    self.check_alive(*src, alive, move_locals, local_names, local_types, "move");
                    if alive.contains(src) {
                        alive.remove(src);
                    }
                    // Also mark the original source of a Load alias as consumed, if still alive
                    if let Some(&orig) = load_map.get(src) {
                        if alive.contains(&orig) {
                            self.check_alive(orig, alive, move_locals, local_names, local_types, "move");
                            alive.remove(&orig);
                        }
                    }
                }
                if move_locals.contains(dest) {
                    alive.insert(*dest);
                }
            }
            MirInst::Call { dest, name, args } => {
                // MOVE-BY-DEFAULT: args are consumed unless the param is Borrow/MutableBorrow
                let modes = func_map.get(name);
                for (i, arg) in args.iter().enumerate() {
                    if let MirValue::Local(l) = arg {
                        let mode = modes.and_then(|m| m.get(i).copied());
                        match mode {
                            Some(ParamMode::Borrow) => {
                                self.check_alive(*l, alive, move_locals, local_names, local_types, "borrow");
                                let orig = *load_map.get(l).unwrap_or(l);
                                let state = borrow_states.get(&orig).copied().unwrap_or(BorrowState::NotBorrowed);
                                match state {
                                    BorrowState::MutBorrowed => {
                                        self.errors.push(format!(
                                            "cannot borrow `{}` immutably — it is already mutably borrowed",
                                            local_names.get(&orig).cloned().unwrap_or_default()
                                        ));
                                    }
                                    _ => {
                                        let count = if let BorrowState::ImmBorrowed(c) = state { c + 1 } else { 1 };
                                        borrow_states.insert(orig, BorrowState::ImmBorrowed(count));
                                    }
                                }
                            }
                            Some(ParamMode::MutableBorrow) => {
                                self.check_alive(*l, alive, move_locals, local_names, local_types, "mut borrow");
                                let orig = *load_map.get(l).unwrap_or(l);
                                let state = borrow_states.get(&orig).copied().unwrap_or(BorrowState::NotBorrowed);
                                match state {
                                    BorrowState::NotBorrowed => {
                                        borrow_states.insert(orig, BorrowState::MutBorrowed);
                                    }
                                    BorrowState::ImmBorrowed(c) => {
                                        self.errors.push(format!(
                                            "cannot mutably borrow `{}` — it is already borrowed immutably ({} borrows active)",
                                            local_names.get(&orig).cloned().unwrap_or_default(), c
                                        ));
                                    }
                                    BorrowState::MutBorrowed => {
                                        self.errors.push(format!(
                                            "cannot mutably borrow `{}` — it is already mutably borrowed",
                                            local_names.get(&orig).cloned().unwrap_or_default()
                                        ));
                                    }
                                }
                            }
                            _ => {
                                // Builtins NOT in func_map: assume they DON'T consume Move args
                                if modes.is_some() {
                                    // MOVE (default for user-defined functions)
                                    self.check_alive(*l, alive, move_locals, local_names, local_types, "move");
                                    if alive.contains(l) {
                                        alive.remove(l);
                                    }
                                    let orig = *load_map.get(l).unwrap_or(l);
                                    if alive.contains(&orig) && *l != orig {
                                        self.check_alive(orig, alive, move_locals, local_names, local_types, "move");
                                        alive.remove(&orig);
                                    }
                                    // A move releases all borrows on the source
                                    borrow_states.remove(&orig);
                                } else if (name == "ky_ptr_write_ptr" || name == "ky_ptr_write_i32") && i == 1 {
                                    // ky_ptr_write_ptr/i32 transfers ownership of the value arg (index 1)
                                    // through the pointer — the caller is no longer responsible for it.
                                    self.check_alive(*l, alive, move_locals, local_names, local_types, "move");
                                    if alive.contains(l) {
                                        alive.remove(l);
                                    }
                                    let orig = *load_map.get(l).unwrap_or(l);
                                    if alive.contains(&orig) && *l != orig {
                                        self.check_alive(orig, alive, move_locals, local_names, local_types, "move");
                                        alive.remove(&orig);
                                    }
                                    borrow_states.remove(&orig);
                                }
                            }
                        }
                    }
                }
                // Release borrows after call (borrows are released when call returns)
                for (i, arg) in args.iter().enumerate() {
                    if let MirValue::Local(l) = arg {
                        let mode = modes.and_then(|m| m.get(i).copied());
                        if matches!(mode, Some(ParamMode::Borrow | ParamMode::MutableBorrow)) {
                            let orig = *load_map.get(l).unwrap_or(l);
                            borrow_states.remove(&orig);
                        }
                    }
                }
                if let Some(d) = dest {
                    if move_locals.contains(d) {
                        alive.insert(*d);
                    }
                }
            }
            MirInst::PtrOffset { ptr, .. } => {
                self.check_alive(*ptr, alive, move_locals, local_names, local_types, "read");
            }
            MirInst::PtrStore { ptr, value, .. } => {
                self.check_alive(*ptr, alive, move_locals, local_names, local_types, "read");
                if let MirValue::Local(id) = value {
                    self.check_alive(*id, alive, move_locals, local_names, local_types, "read");
                }
            }
            MirInst::Memcpy { dest_ptr_local, src_alloca_local, .. } => {
                self.check_alive(*dest_ptr_local, alive, move_locals, local_names, local_types, "read");
                self.check_alive(*src_alloca_local, alive, move_locals, local_names, local_types, "read");
            }
            MirInst::FieldPtr { ptr, .. } => {
                self.check_alive(*ptr, alive, move_locals, local_names, local_types, "read");
            }
            MirInst::AsyncAwait { handle, .. } => {
                self.check_alive(*handle, alive, move_locals, local_names, local_types, "read");
            }
            _ => {}
        }
    }

fn check_alive(
    &mut self,
    local: usize,
    alive: &BTreeSet<usize>,
    move_locals: &BTreeSet<usize>,
    local_names: &HashMap<usize, String>,
    local_types: &HashMap<usize, MirType>,
    context: &str,
) {
    if !move_locals.contains(&local) {
        return;
    }
    if !alive.contains(&local) {
        let name = local_names
            .get(&local)
            .cloned()
            .unwrap_or_else(|| format!("%{}", local));
        self.errors.push(format!(
            "use-after-move: cannot {} `{}` (local #{}) — value has been moved",
            context, name, local
        ));
    }
}

    fn is_move_local(&self, local: usize, local_types: &HashMap<usize, MirType>) -> bool {
        local_types.get(&local).map_or(false, |t| is_move_type(t))
    }

    fn build_local_types(&self, func: &MirFunction) -> HashMap<usize, MirType> {
        let mut types = HashMap::new();
        for block in &func.basic_blocks {
            for inst in &block.insts {
                if let MirInst::Alloca { dest, type_, .. } = inst {
                    types.insert(*dest, type_.clone());
                }
            }
        }
        types
    }

    fn build_local_names(&self, func: &MirFunction) -> HashMap<usize, String> {
        let mut names = HashMap::new();
        for block in &func.basic_blocks {
            for inst in &block.insts {
                if let MirInst::Alloca { dest, name, .. } = inst {
                    names.insert(*dest, name.clone());
                }
            }
        }
        names
    }

    fn find_move_locals(&self, func: &MirFunction) -> BTreeSet<usize> {
        let mut move_locals = BTreeSet::new();
        // Collect list_get/dict_get result locals — these are borrowed references
        // into the list/dict internal buffer, NOT owned values. Never free them.
        let get_results: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Call { dest, name, .. } = inst {
                    if name == "ky_list_get" || name == "ky_dict_get" {
                        return *dest;
                    }
                }
                None
            })
            .collect();
        // Also track Cast destinations from list_get results — these are str/struct
        // temps that hold the (casted) borrowed reference. Must NOT be freed.
        let cast_from_get: BTreeSet<usize> = func.basic_blocks.iter()
            .flat_map(|b| b.insts.iter())
            .filter_map(|inst| {
                if let MirInst::Cast { dest, value, .. } = inst {
                    if let MirValue::Local(src) = value {
                        if get_results.contains(src) {
                            return Some(*dest);
                        }
                    }
                }
                None
            })
            .collect();
        let exclude: BTreeSet<usize> = get_results.union(&cast_from_get).cloned().collect();
        for block in &func.basic_blocks {
            for inst in &block.insts {
                if let MirInst::Alloca { dest, name, type_ } = inst {
                    // Skip string constants (global string refs — not heap-allocated)
                    if name.starts_with("_lit_const") {
                        continue;
                    }
                    // Skip list_get/dict_get results and their casts (borrowed, not owned)
                    if exclude.contains(dest) {
                        continue;
                    }
                    if is_move_type(type_) {
                        move_locals.insert(*dest);
                    }
                }
            }
        }
        move_locals
    }
}

/// Returns true if this function CONSUMES its arguments (ownership transfer).
/// By default (borrow-by-default), ALL functions BORROW their arguments.
/// Only explicit `^` params on user-defined functions consume.
/// Checks the function's param_modes for any ParamMode::Move.
fn is_move_func(name: &str, _funcs: &[crate::mir::MirFunction]) -> bool {
    if let Some(func) = _funcs.iter().find(|f| f.name == name) {
        func.param_modes.iter().any(|m| *m == ParamMode::Move)
    } else {
        false
    }
}

/// Lookup a function's param_modes from a pre-built map.
/// Returns true if the function has any `ParamMode::Move` params.
fn is_move_func_from_map(name: &str, func_map: &std::collections::HashMap<String, Vec<ParamMode>>) -> bool {
    func_map.get(name).map_or(false, |modes| modes.iter().any(|m| *m == ParamMode::Move))
}

/// Check if a function has a `MutableBorrow` param at the given index.
fn is_mut_borrow_param(name: &str, idx: usize, func_map: &std::collections::HashMap<String, Vec<ParamMode>>) -> bool {
    func_map.get(name).map_or(false, |modes| idx < modes.len() && modes[idx] == ParamMode::MutableBorrow)
}

/// Returns true if this runtime function creates a heap-allocated value.
/// Used by the codegen backend to determine which call results to manage.
#[allow(dead_code)]
pub fn is_alloc_func(name: &str) -> bool {
    matches!(
        name,
        "ky_alloc"
            | "ky_concat"
            | "ky_list_new"
            | "ky_list_copy"
            | "ky_dict_new"
            | "ky_dict_copy"
            | "ky_array_new"
            | "ky_array_copy"
            | "ky_string_add"
            | "ky_str_repeat"
            | "ky_str_replace"
            | "ky_substr"
            | "ky_i64_to_str"
            | "ky_str_to_upper"
            | "ky_str_to_lower"
            | "ky_str_trim"
            | "ky_clone_str"
            | "ky_clone_list"
            | "ky_clone_dict"
            | "ky_list_map"
            | "ky_list_filter"
            | "ky_iter_new"
            | "ky_iter_map"
            | "ky_iter_filter"
            | "ky_iter_collect"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::is_move_type;

    #[test]
    fn test_move_type_classification() {
        assert!(!is_move_type(&MirType::I32));
        assert!(!is_move_type(&MirType::F64));
        assert!(!is_move_type(&MirType::Bool));
        assert!(!is_move_type(&MirType::Ptr(Box::new(MirType::I32))));
        assert!(is_move_type(&MirType::Str));
        assert!(is_move_type(&MirType::List(Box::new(MirType::I32))));
        assert!(!is_move_type(&MirType::Struct("Point".to_string(), vec![])));
        assert!(!is_move_type(&MirType::Array(Box::new(MirType::I32), 0)));
    }

    #[test]
    fn test_alloc_func_detection() {
        assert!(is_alloc_func("ky_concat"));
        assert!(is_alloc_func("ky_alloc"));
        assert!(is_alloc_func("ky_list_new"));
        assert!(!is_alloc_func("ky_print"));
        assert!(!is_alloc_func("ky_strlen"));
    }

    #[test]
    fn test_empty_module() {
        let mut module = MirModule {
            functions: vec![],
            globals: vec![],
            links: vec![],
        };
        let mut analysis = BorrowAnalysis::new();
        analysis.run(&mut module); // should not panic
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_no_move_locals() {
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![MirType::I32],
            return_type: MirType::I32,
            is_fallible: false,
            is_const: false,
            basic_blocks: vec![MirBasicBlock {
                label: "entry".to_string(),
                insts: vec![
                    MirInst::Alloca { dest: 0, type_: MirType::I32, name: "x".to_string() },
                    MirInst::Store { dest: 0, value: MirValue::Constant(MirConstant::I32(42)) },
                    MirInst::Load { dest: 1, src: 0 },
                ],
                terminator: MirTerminator::Return(MirValue::Local(1)),
            }],
            local_count: 2,
            param_modes: vec![],
        };
        let mut analysis = BorrowAnalysis::new();
        let original_len = func.basic_blocks[0].insts.len();
        let mut module = MirModule { functions: vec![func], globals: vec![], links: vec![] };
        analysis.run(&mut module);
        // No instructions should be added (no Move types)
        assert_eq!(module.functions[0].basic_blocks[0].insts.len(), original_len);
    }

    #[test]
    fn test_borrow_by_default_in_mir() {
        // By default, function calls BORROW their args (not consume).
        // So using the value after a call is OK.
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MirType::Void,
            is_fallible: false,
            is_const: false,
            basic_blocks: vec![MirBasicBlock {
                label: "entry".to_string(),
                insts: vec![
                    MirInst::Alloca { dest: 0, type_: MirType::Str, name: "s".to_string() },
                    MirInst::Call {
                        dest: None,
                        name: "ky_print".to_string(),
                        args: vec![MirValue::Local(0)],
                    },
                    // Store after call should NOT be a use-after-move (borrow-by-default)
                    MirInst::Store { dest: 1, value: MirValue::Local(0) },
                ],
                terminator: MirTerminator::Return(MirValue::Local(1)),
            }],
            local_count: 2,
            param_modes: vec![],
        };
        let mut analysis = BorrowAnalysis::new();
        let mut module = MirModule { functions: vec![func], globals: vec![], links: vec![] };
        analysis.run(&mut module);
        // Borrow-by-default: no errors expected
        assert!(analysis.errors().is_empty(), "Expected no errors, got: {:?}", analysis.errors());
    }

    #[test]
    fn test_no_false_positive_on_copy_types() {
        let func = MirFunction {
            name: "test".to_string(),
            params: vec![],
            return_type: MirType::Void,
            is_fallible: false,
            is_const: false,
            basic_blocks: vec![MirBasicBlock {
                label: "entry".to_string(),
                insts: vec![
                    MirInst::Alloca { dest: 0, type_: MirType::I32, name: "x".to_string() },
                    MirInst::Store { dest: 0, value: MirValue::Constant(MirConstant::I32(42)) },
                    MirInst::Load { dest: 1, src: 0 },
                    MirInst::BinaryOp {
                        dest: 2,
                        op: MirBinaryOp::Add,
                        left: MirValue::Local(0),
                        right: MirValue::Local(1),
                    },
                ],
                terminator: MirTerminator::Return(MirValue::Local(2)),
            }],
            local_count: 3,
            param_modes: vec![],
        };
        let mut analysis = BorrowAnalysis::new();
        let mut module = MirModule { functions: vec![func], globals: vec![], links: vec![] };
        analysis.run(&mut module);
        // I32 is Copy — no use-after-move errors expected
        assert_eq!(analysis.errors().len(), 0, "Expected no errors, got: {:?}", analysis.errors());
    }

    #[test]
    fn test_return_alive_not_error() {
        let func = MirFunction {
            name: "make_str".to_string(),
            params: vec![],
            return_type: MirType::Str,
            is_fallible: false,
            is_const: false,
            basic_blocks: vec![MirBasicBlock {
                label: "entry".to_string(),
                insts: vec![
                    MirInst::Alloca { dest: 0, type_: MirType::Str, name: "s".to_string() },
                    MirInst::Call {
                        dest: Some(0),
                        name: "ky_concat".to_string(),
                        args: vec![
                            MirValue::Constant(MirConstant::String("hello".to_string())),
                            MirValue::Constant(MirConstant::String("world".to_string())),
                        ],
                    },
                ],
                terminator: MirTerminator::Return(MirValue::Local(0)),
            }],
            local_count: 1,
            param_modes: vec![],
        };
        let mut analysis = BorrowAnalysis::new();
        let mut module = MirModule { functions: vec![func], globals: vec![], links: vec![] };
        analysis.run(&mut module);
        // Returning a Str transfers ownership to caller — no error
        assert_eq!(analysis.errors().len(), 0, "Expected no errors, got: {:?}", analysis.errors());
    }
}
