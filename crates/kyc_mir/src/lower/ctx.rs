use kyc_core::ast::*;
use crate::mir::*;

pub struct LowerCtx {
    pub(crate) next_local: usize,
    pub(crate) locals: std::collections::HashMap<String, usize>,
    pub(crate) current_block: MirBasicBlock,
    pub(crate) blocks: Vec<MirBasicBlock>,
    pub(crate) block_counter: usize,
    pub(crate) string_locals: Vec<usize>,
    pub(crate) local_types: std::collections::HashMap<usize, MirType>,
    pub(crate) break_targets: Vec<String>,
    pub(crate) continue_targets: Vec<String>,
    pub(crate) struct_defs: std::collections::HashMap<String, Vec<(String, MirType)>>,
    pub(crate) tail_if_as_return: bool,
    pub(crate) match_result_local: Option<usize>,
    pub(crate) is_fallible: bool,
    pub(crate) return_type: MirType,
    pub(crate) deferred_exprs: Vec<Box<Expr>>,
    pub(crate) break_flag_local: Option<usize>,
    pub(crate) if_result_alloca: Option<usize>,
    pub(crate) last_stmt_was_if: bool,
    /// Locals that are mutable borrow parameters (^&T) — reads/writes
    /// must go through a pointer (double-load / store-through-ptr).
    pub(crate) ref_param_locals: std::collections::HashSet<usize>,
}

impl LowerCtx {
    pub(crate) fn new() -> Self {
        Self {
            next_local: 0,
            locals: std::collections::HashMap::new(),
            current_block: MirBasicBlock::new("entry"),
            blocks: Vec::new(),
            block_counter: 0,
            string_locals: Vec::new(),
            local_types: std::collections::HashMap::new(),
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            struct_defs: std::collections::HashMap::new(),
            tail_if_as_return: false,
            match_result_local: None,
            is_fallible: false,
            return_type: MirType::Void,
            deferred_exprs: Vec::new(),
            break_flag_local: None,
            if_result_alloca: None,
            last_stmt_was_if: false,
            ref_param_locals: std::collections::HashSet::new(),
        }
    }

    pub(crate) fn alloc_local(&mut self, name: &str, type_: MirType) -> usize {
        let id = self.next_local;
        self.next_local += 1;
        self.current_block.insts.push(MirInst::Alloca {
            dest: id,
            type_: type_.clone(),
            name: name.to_string(),
        });
        self.local_types.insert(id, type_);
        id
    }

    pub(crate) fn fresh_block(&mut self) -> String {
        let label = format!("bb{}", self.block_counter);
        self.block_counter += 1;
        label
    }

    pub(crate) fn finish_block(&mut self, terminator: MirTerminator) {
        self.current_block.terminator = terminator;
        let label = self.fresh_block();
        let block = std::mem::replace(
            &mut self.current_block,
            MirBasicBlock::new(&label),
        );
        self.blocks.push(block);
    }

    /// Emit a return, wrapping the value in a success result struct if the
    /// function is fallible. Used by both explicit `return` statements and
    /// tail-expression returns.
    pub(crate) fn emit_return(&mut self, value: MirValue) {
        if self.is_fallible {
            if let MirValue::Local(id) = &value {
                if let Some(MirType::Struct(name, _)) = self.local_types.get(id) {
                    if name == "Result" {
                        self.finish_block(MirTerminator::Return(value));
                        return;
                    }
                }
            }
            let result_struct = self.return_type.clone();
            let result_local = self.alloc_local("_ret", result_struct.clone());
            // Set disc = 0 (success/ok)
            let disc_ptr = self.alloc_local("_rsp", MirType::I32);
            self.current_block.insts.push(MirInst::FieldPtr {
                dest: disc_ptr,
                ptr: result_local,
                field_index: 0,
                struct_type: Box::new(result_struct.clone()),
            });
            self.current_block.insts.push(MirInst::Store {
                dest: disc_ptr,
                value: MirValue::Constant(MirConstant::I32(0)),
            });
            // Set payload (widened to I64)
            let payload_ptr = self.alloc_local("_rpp", MirType::I64);
            self.current_block.insts.push(MirInst::FieldPtr {
                dest: payload_ptr,
                ptr: result_local,
                field_index: 1,
                struct_type: Box::new(result_struct.clone()),
            });
            let widened = if let MirValue::Local(id) = &value {
                if self.local_types.get(id).map(|t| *t != MirType::I64).unwrap_or(true) {
                    let cast = self.alloc_local("_rpv", MirType::I64);
                    self.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: value,
                        to_type: MirType::I64,
                    });
                    MirValue::Local(cast)
                } else {
                    value
                }
            } else {
                value
            };
            self.current_block.insts.push(MirInst::Store {
                dest: payload_ptr,
                value: widened,
            });
            self.finish_block(MirTerminator::Return(MirValue::Local(result_local)));
        } else {
            self.finish_block(MirTerminator::Return(value));
        }
    }
}
