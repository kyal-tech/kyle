use crate::mir::*;
use kyc_core::ast::*;
use super::*;

impl super::Lowerer {
    pub(crate) fn lower_stmt(&self, mut ctx: LowerCtx, stmt: &Stmt) -> LowerCtx {
        match stmt {
            Stmt::Expression(expr) => {
                ctx = self.lower_expr(ctx, expr);
                ctx
            }
            Stmt::Variable(v) => {
                // Pre-register Option struct fields
                if let Some(ref type_) = v.type_ {
                    register_option_type(type_, &mut ctx.struct_defs);
                }
                // Special case: array variable initialized from list literal
                // e.g. arr: [i32, 3] = [10, 20, 30]
                // Instead of creating a heap list, store elements inline
                // (uses same ArrayElemPtr + Store pattern as ArrayRepeat lowering)
                if let Some(AstType::Array { inner, size, .. }) = &v.type_ {
                    if let Expr::List { elements, .. } = v.value.as_ref() {
                        let var_type = ast_type_to_mir(v.type_.as_ref().unwrap(), Some(&ctx.struct_defs));
                        let local = ctx.alloc_local(&v.name, var_type.clone());
                        let elem_type = ast_type_to_mir(inner, Some(&ctx.struct_defs));
                        // Lower all element expressions first
                        let mut elem_locals: Vec<usize> = Vec::new();
                        for elem in elements.iter() {
                            ctx = self.lower_expr(ctx, elem);
                            elem_locals.push(ctx.next_local - 1);
                        }
                        // Then create ArrayElemPtr + Store for each element
                        let max_i = elements.len().min(*size);
                        for i in 0..max_i {
                            let elem_local = elem_locals[i];
                            let elem_ptr = ctx.alloc_local("_ael", MirType::I64);
                            ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                                dest: elem_ptr,
                                ptr: local,
                                index: MirValue::Constant(MirConstant::I32(i as i32)),
                                arr_type: Box::new(var_type.clone()),
                                elem_type: Box::new(elem_type.clone()),
                            });
                            let elem_val = if let Some(et) = ctx.local_types.get(&elem_local) {
                                if *et != elem_type {
                                    let cast = ctx.alloc_local("_aec", elem_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast, value: MirValue::Local(elem_local), to_type: elem_type.clone(),
                                    });
                                    MirValue::Local(cast)
                                } else {
                                    MirValue::Local(elem_local)
                                }
                            } else {
                                MirValue::Local(elem_local)
                            };
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: elem_ptr,
                                value: elem_val,
                            });
                        }
                        ctx.locals.insert(v.name.clone(), local);
                        ctx.local_types.insert(local, var_type.clone());
                        return ctx;
                    }
                }
                let has_init = !matches!(v.value.as_ref(), Expr::Literal { value: Literal::None, .. });
                let mut is_list = false;
                let mut is_set = false;
                if !has_init {
                    if let Some(AstType::Generic { name, .. }) = &v.type_ {
                        if name == "list" {
                            is_list = true;
                        } else if name == "set" {
                            is_set = true;
                        }
                    }
                }
                let val_local = if has_init {
                    ctx = self.lower_expr(ctx, &v.value);
                    Some(ctx.next_local - 1)
                } else if is_set {
                    let set_ptr = ctx.alloc_local("_setv", ast_type_to_mir(v.type_.as_ref().unwrap(), Some(&ctx.struct_defs)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(set_ptr),
                        name: "ky_set_new".to_string(),
                        args: vec![],
                    });
                    Some(set_ptr)
                } else if is_list {
                    let list_ptr = ctx.alloc_local("_listv", ast_type_to_mir(v.type_.as_ref().unwrap(), Some(&ctx.struct_defs)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(list_ptr),
                        name: "ky_list_new".to_string(),
                        args: vec![],
                    });
                    Some(list_ptr)
                } else {
                    None
                };
                let var_type = v.type_.as_ref()
                    .map(|t| ast_type_to_mir(t, Some(&ctx.struct_defs)))
                    .unwrap_or_else(|| {
                        if let Some(vl) = val_local {
                            let inferred = ctx.local_types.get(&vl).cloned();
                            if let Some(t) = inferred {
                                // Preserve the RHS type when it's pointer-backed or a
                                // wider scalar. Inferring I32 would truncate pointers
                                // (e.g. bytes → i32 loses the upper 32 bits).
                                if matches!(t, MirType::List(_) | MirType::Struct(_, _) | MirType::Array(_, _)
                                    | MirType::Dict(_, _) | MirType::Set(_) | MirType::Queue(_)
                                    | MirType::Stack(_) | MirType::Deque(_) | MirType::LinkedList(_)
                                    | MirType::Chan(_) | MirType::Slice(_) | MirType::Box(_)
                                    | MirType::Ptr(_) | MirType::Bytes) {
                                    t
                                } else if matches!(t, MirType::Str | MirType::I64 | MirType::F64 | MirType::Char | MirType::U64) {
                                    t
                                } else {
                                    MirType::I32
                                }
                            } else {
                                MirType::I32
                            }
                        } else {
                            MirType::I32
                        }
                    });
                let is_option_none = val_local.is_none()
                    && matches!(&var_type, MirType::Struct(n, _) if n.starts_with("Option__"));
                let local = ctx.alloc_local(&v.name, var_type.clone());
                if let Some(vl) = val_local {
                    let val_type = ctx.local_types.get(&vl).cloned().unwrap_or(MirType::I32);
                    let store_val = if val_type != var_type {
                        let cast = ctx.alloc_local("_tcast", var_type.clone());
                        ctx.current_block.insts.push(MirInst::Cast { dest: cast, value: MirValue::Local(vl), to_type: var_type.clone() });
                        MirValue::Local(cast)
                    } else {
                        MirValue::Local(vl)
                    };
                    // Skip store for empty array literal to avoid type mismatch in LLVM
                    let is_empty_array = matches!(v.value.as_ref(), Expr::Array { elements, .. } if elements.is_empty());
                    if !is_empty_array || !matches!(val_type, MirType::Array(_, _)) {
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: local,
                            value: store_val,
                        });
                    }
                }
                if is_option_none {
                    // Auto-initialize Option<T> with None (disc = 0)
                    let disc_ptr = ctx.alloc_local("_disc", MirType::I32);
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr,
                        ptr: local,
                        field_index: 0,
                        struct_type: Box::new(ctx.local_types[&local].clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr,
                        value: MirValue::Constant(MirConstant::I32(0)),
                    });
                }
                ctx.locals.insert(v.name.clone(), local);
                ctx
            }
            Stmt::Return(ret_val) => {
                // Emit deferred calls in reverse LIFO order before returning
                let deferred = std::mem::take(&mut ctx.deferred_exprs);
                for expr in deferred.iter().rev() {
                    ctx = self.lower_expr(ctx, expr);
                }
                if let Some(expr) = ret_val {
                    let mut val_ctx = self.lower_expr(ctx, expr);
                    // If the expression already finished the block (e.g., ok/error handler),
                    // return directly without adding another return
                    if val_ctx.current_block.insts.is_empty()
                        && val_ctx.current_block.terminator != MirTerminator::Unreachable
                    {
                        return val_ctx;
                    }
                    let val = if let Some(last) = val_ctx.current_block.insts.last() {
                        match last {
                            MirInst::Call { dest: Some(d), .. } => MirValue::Local(*d),
                            _ => MirValue::Local(val_ctx.next_local - 1),
                        }
                    } else {
                        MirValue::Constant(MirConstant::Void)
                    };
                    val_ctx.emit_return(val);
                    val_ctx
                } else {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                    ctx
                }
            }
            Stmt::If(s) => {
                let is_tail = ctx.tail_if_as_return;
                ctx.tail_if_as_return = false;
                ctx.last_stmt_was_if = false;
                let else_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let then_label = ctx.fresh_block();
                // When tail, allocate shared result alloca so branches with nested
                // if/else store to the same slot and the merge block returns it.
                let result_alloca = if is_tail {
                    let ra = ctx.alloc_local("_if_res", MirType::I64);
                    ctx.if_result_alloca = Some(ra);
                    Some(ra)
                } else {
                    None
                };
                let cond_ctx = self.lower_expr(ctx, &s.condition);
                ctx = cond_ctx;
                let cond_val = if let Some(last) = ctx.current_block.insts.last() {
                    match last {
                        MirInst::Call { dest: Some(d), .. } => MirValue::Local(*d),
                        MirInst::BinaryOp { dest, .. } => MirValue::Local(*dest),
                        MirInst::UnaryOp { dest, .. } => MirValue::Local(*dest),
                        MirInst::Load { dest, .. } => MirValue::Local(*dest),
                        _ => MirValue::Local(ctx.next_local - 1),
                    }
                } else {
                    MirValue::Constant(MirConstant::Bool(false))
                };
                // If the condition is an Option<T>, check discriminant (field 0 != 0)
                let cond_val = if let MirValue::Local(cond_local) = cond_val {
                    if let Some(MirType::Struct(struct_name, _)) = ctx.local_types.get(&cond_local) {
                        if struct_name.starts_with("Option__") {
                            // Store loaded struct to a temp alloca for field access
                            let struct_type = ctx.local_types.get(&cond_local).cloned().unwrap();
                            let temp = ctx.alloc_local("_ocond", struct_type.clone());
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: temp, value: MirValue::Local(cond_local),
                            });
                            let disc_ptr = ctx.alloc_local("_odp", MirType::Ptr(Box::new(MirType::I32)));
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: disc_ptr, ptr: temp, field_index: 0,
                                struct_type: Box::new(struct_type),
                            });
                            let disc = ctx.alloc_local("_od", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Load { dest: disc, src: disc_ptr });
                            let zero = ctx.alloc_local("_oz", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: zero, value: MirValue::Constant(MirConstant::I32(0)),
                            });
                            let is_some = ctx.alloc_local("_os", MirType::I32);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: is_some, op: MirBinaryOp::Neq,
                                left: MirValue::Local(disc), right: MirValue::Local(zero),
                            });
                            MirValue::Local(is_some)
                        } else {
                            cond_val
                        }
                    } else {
                        cond_val
                    }
                } else {
                    cond_val
                };
                let elif_cond_labels: Vec<String> = (0..s.elif_branches.len())
                    .map(|_| ctx.fresh_block())
                    .collect();
                if !s.elif_branches.is_empty() {
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: cond_val,
                        true_block: then_label.clone(),
                        false_block: elif_cond_labels[0].clone(),
                    });
                } else if s.else_branch.is_some() {
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: cond_val,
                        true_block: then_label.clone(),
                        false_block: else_label.clone(),
                    });
                } else {
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: cond_val,
                        true_block: then_label.clone(),
                        false_block: end_label.clone(),
                    });
                }
                // Then block
                ctx.current_block = MirBasicBlock::new(then_label);
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                if is_tail {
                    if let Some(ra) = result_alloca {
                        if !ctx.last_stmt_was_if {
                            let last = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                            ctx.local_types.insert(ra, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: ra,
                                value: MirValue::Local(last),
                            });
                        }
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    } else {
                        ctx.emit_return(MirValue::Local(ctx.next_local - 1));
                    }
                } else if let Some(ra) = ctx.if_result_alloca {
                    // Parent if is a tail expression: store last expr to parent's result alloca
                    let last = ctx.next_local - 1;
                    let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                    ctx.local_types.insert(ra, last_type);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: ra,
                        value: MirValue::Local(last),
                    });
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                } else {
                    ctx.finish_block(MirTerminator::Br(end_label.clone()));
                }

                // Handle elif branches
                for (i, elif) in s.elif_branches.iter().enumerate() {
                    ctx.current_block = MirBasicBlock::new(elif_cond_labels[i].clone());
                    let elif_cond_ctx = self.lower_expr(ctx, &elif.condition);
                    ctx = elif_cond_ctx;
                    let elif_cond = MirValue::Local(ctx.next_local - 1);
                    // If elif condition is Option<T>, check discriminant
                    let elif_cond = if let MirValue::Local(cond_local) = elif_cond {
                        if let Some(MirType::Struct(struct_name, _)) = ctx.local_types.get(&cond_local) {
                            if struct_name.starts_with("Option__") {
                                let struct_type = ctx.local_types.get(&cond_local).cloned().unwrap();
                                let temp = ctx.alloc_local("_econd", struct_type.clone());
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: temp, value: MirValue::Local(cond_local),
                                });
                                let disc_ptr = ctx.alloc_local("_edp", MirType::Ptr(Box::new(MirType::I32)));
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: disc_ptr, ptr: temp, field_index: 0,
                                    struct_type: Box::new(struct_type),
                                });
                                let disc = ctx.alloc_local("_ed", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Load { dest: disc, src: disc_ptr });
                                let zero = ctx.alloc_local("_ez", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: zero, value: MirValue::Constant(MirConstant::I32(0)),
                                });
                                let is_some = ctx.alloc_local("_es", MirType::I32);
                                ctx.current_block.insts.push(MirInst::BinaryOp {
                                    dest: is_some, op: MirBinaryOp::Neq,
                                    left: MirValue::Local(disc), right: MirValue::Local(zero),
                                });
                                MirValue::Local(is_some)
                            } else { elif_cond }
                        } else { elif_cond }
                    } else { elif_cond };
                    let elif_then_label = ctx.fresh_block();
                    let elif_false_target = if i + 1 < s.elif_branches.len() {
                        elif_cond_labels[i + 1].clone()
                    } else if s.else_branch.is_some() {
                        else_label.clone()
                    } else {
                        end_label.clone()
                    };
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: elif_cond,
                        true_block: elif_then_label.clone(),
                        false_block: elif_false_target,
                    });
                    // elif then
                    ctx.current_block = MirBasicBlock::new(elif_then_label);
                    for stmt in &elif.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    if is_tail {
                        if let Some(ra) = result_alloca {
                            if !ctx.last_stmt_was_if {
                                let last = ctx.next_local - 1;
                                let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                                ctx.local_types.insert(ra, last_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: ra,
                                    value: MirValue::Local(last),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                        } else {
                            ctx.emit_return(MirValue::Local(ctx.next_local - 1));
                        }
                    } else if let Some(ra) = ctx.if_result_alloca {
                        let last = ctx.next_local - 1;
                        let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                        ctx.local_types.insert(ra, last_type);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: ra,
                            value: MirValue::Local(last),
                        });
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                }

                // Else block
                if let Some(el) = &s.else_branch {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    for stmt in &el.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    if is_tail {
                        if let Some(ra) = result_alloca {
                            if !ctx.last_stmt_was_if {
                                let last = ctx.next_local - 1;
                                let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                                ctx.local_types.insert(ra, last_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: ra,
                                    value: MirValue::Local(last),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                        } else {
                            ctx.emit_return(MirValue::Local(ctx.next_local - 1));
                        }
                    } else if let Some(ra) = ctx.if_result_alloca {
                        let last = ctx.next_local - 1;
                        let last_type = ctx.local_types.get(&last).cloned().unwrap_or(MirType::I64);
                        ctx.local_types.insert(ra, last_type);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: ra,
                            value: MirValue::Local(last),
                        });
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                } else if !s.elif_branches.is_empty() {
                    ctx.current_block = MirBasicBlock::new(else_label);
                    if is_tail {
                        ctx.emit_return(MirValue::Constant(MirConstant::Void));
                    } else {
                        ctx.finish_block(MirTerminator::Br(end_label.clone()));
                    }
                }

                if is_tail {
                    ctx.if_result_alloca = None;
                    // Merge block: return result_alloca value (or void if no else)
                    ctx.current_block = MirBasicBlock::new(end_label);
                    if let Some(ra) = result_alloca {
                        let result_type = ctx.local_types.get(&ra).cloned().unwrap_or(MirType::I64);
                        let load = ctx.alloc_local("_if_res_val", result_type);
                        ctx.current_block.insts.push(MirInst::Load {
                            dest: load,
                            src: ra,
                        });
                        ctx.emit_return(MirValue::Local(load));
                    } else {
                        // Only reachable if is_tail && no allocation — impossible
                        ctx.emit_return(MirValue::Constant(MirConstant::Void));
                    }
                } else {
                    ctx.current_block = MirBasicBlock::new(end_label);
                }
                ctx.last_stmt_was_if = true;
                ctx
            }
            Stmt::While(s) => {
                let has_else = s.else_branch.is_some();
                let cond_label = ctx.fresh_block();
                let body_label = ctx.fresh_block();
                let exit_label = ctx.fresh_block();
                let else_label = ctx.fresh_block();
                let skip_else_label = ctx.fresh_block();
                let merge_label = ctx.fresh_block();

                let flag_local = if has_else {
                    let f = ctx.alloc_local("_loop_break", MirType::Bool);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: f,
                        value: MirValue::Constant(MirConstant::Bool(false)),
                    });
                    Some(f)
                } else {
                    None
                };

                ctx.finish_block(MirTerminator::Br(cond_label.clone()));
                ctx.current_block = MirBasicBlock::new(cond_label.clone());
                let cond_ctx = self.lower_expr(ctx, &s.condition);
                ctx = cond_ctx;
                let cond_val = MirValue::Local(ctx.next_local - 1);

                let false_target = if has_else { else_label.clone() } else { exit_label.clone() };
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: body_label.clone(),
                    false_block: false_target,
                });

                // Body block
                ctx.current_block = MirBasicBlock::new(body_label);
                let break_target = if has_else { else_label.clone() } else { exit_label.clone() };
                ctx.break_targets.push(break_target);
                ctx.continue_targets.push(cond_label.clone());
                if has_else { ctx.break_flag_local = flag_local; }
                for stmt in &s.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.break_flag_local = None;
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                if has_else {
                    // else_label: check flag → execute else or skip
                    ctx.current_block = MirBasicBlock::new(else_label);
                    let f_load = ctx.alloc_local("_flag_v", MirType::Bool);
                    ctx.current_block.insts.push(MirInst::Load { dest: f_load, src: flag_local.unwrap() });
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: MirValue::Local(f_load),
                        true_block: skip_else_label.clone(),
                        false_block: exit_label.clone(),
                    });

                    // exit_label: else body
                    ctx.current_block = MirBasicBlock::new(exit_label);
                    if let Some(eb) = &s.else_branch {
                        for stmt in &eb.statements {
                            ctx = self.lower_stmt(ctx, stmt);
                        }
                    }
                    ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                    // skip_else_label
                    ctx.current_block = MirBasicBlock::new(skip_else_label);
                    ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                    ctx.current_block = MirBasicBlock::new(merge_label);
                } else {
                    ctx.current_block = MirBasicBlock::new(exit_label);
                }
                ctx
            }
            Stmt::For(s) => {
                // Check if iterable is a range expression (for i in 0..10)
                let range_op = if let Expr::Binary { operator: op @ (BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::RangeExclusive), .. } = &*s.iterable {
                    Some(*op)
                } else { None };
                if let Some(range_op) = range_op {
                    let (left, right) = if let Expr::Binary { left, right, .. } = &*s.iterable {
                        (left, right)
                    } else { unreachable!() };
                    // === RANGE-BASED FOR LOOP ===
                    let cond_label = ctx.fresh_block();
                    let body_label = ctx.fresh_block();
                    let inc_label = ctx.fresh_block();
                    let check_label = ctx.fresh_block();
                    let else_label = ctx.fresh_block();
                    let skip_else_label = ctx.fresh_block();
                    let merge_label = ctx.fresh_block();

                    let has_else = s.else_branch.is_some();

                    let flag_local = if has_else {
                        let f = ctx.alloc_local("_loop_break", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: f,
                            value: MirValue::Constant(MirConstant::Bool(false)),
                        });
                        Some(f)
                    } else {
                        None
                    };

                    // 1. Lower start and end expressions
                    ctx = self.lower_expr(ctx, left);
                    let start_val = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, right);
                    let end_val = ctx.next_local - 1;

                    // Allocate loop variable as I32
                    let var_local = ctx.alloc_local(&s.variable, MirType::I32);
                    ctx.locals.insert(s.variable.clone(), var_local);

                    // Store start value in loop variable
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: var_local,
                        value: MirValue::Local(start_val),
                    });

                    // Store end value in alloca for cross-block access
                    let end_alloca = ctx.alloc_local("_for_end", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: end_alloca,
                        value: MirValue::Local(end_val),
                    });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    // 2. Cond block: load i, compare i < end
                    ctx.current_block = MirBasicBlock::new(cond_label.clone());

                    let i_val = ctx.alloc_local("_for_i", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: i_val, src: var_local });
                    let end_loaded = ctx.alloc_local("_for_el", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: end_loaded, src: end_alloca });

                    let cmp = ctx.alloc_local("_for_cmp", MirType::Bool);
                    let cmp_op = match range_op {
                        BinaryOp::RangeInclusive => MirBinaryOp::Le,
                        _ => MirBinaryOp::Lt,
                    };
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: cmp, op: cmp_op,
                        left: MirValue::Local(i_val), right: MirValue::Local(end_loaded),
                    });

                    let false_target = if has_else { else_label.clone() } else { check_label.clone() };
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: MirValue::Local(cmp),
                        true_block: body_label.clone(),
                        false_block: false_target.clone(),
                    });

                    // 3. Body block
                    ctx.current_block = MirBasicBlock::new(body_label.clone());
                    ctx.break_targets.push(false_target);
                    ctx.continue_targets.push(inc_label.clone());
                    if has_else { ctx.break_flag_local = flag_local; }

                    // Lower body statements
                    for stmt in &s.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    ctx.break_flag_local = None;
                    ctx.break_targets.pop();
                    ctx.continue_targets.pop();

                    // Fall through to inc block
                    ctx.finish_block(MirTerminator::Br(inc_label.clone()));

                    // 4. Increment block: i += 1, then goto cond
                    ctx.current_block = MirBasicBlock::new(inc_label.clone());
                    let iv2 = ctx.alloc_local("_for_iv2", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: iv2, src: var_local });
                    let iv3 = ctx.alloc_local("_for_iv3", MirType::I32);
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: iv3, op: MirBinaryOp::Add,
                        left: MirValue::Local(iv2), right: MirValue::Constant(MirConstant::I32(1)),
                    });
                    ctx.current_block.insts.push(MirInst::Store { dest: var_local, value: MirValue::Local(iv3) });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    if has_else {
                        // 5a. Else-label: check flag → execute else or skip
                        ctx.current_block = MirBasicBlock::new(else_label.clone());
                        let f_load = ctx.alloc_local("_flag_v", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Load { dest: f_load, src: flag_local.unwrap() });
                        ctx.finish_block(MirTerminator::CondBr {
                            cond: MirValue::Local(f_load),
                            true_block: skip_else_label.clone(),
                            false_block: check_label.clone(),
                        });

                        // 5b. Check-label: execute else body
                        ctx.current_block = MirBasicBlock::new(check_label.clone());
                        if let Some(eb) = &s.else_branch {
                            for stmt in &eb.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                        }
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        // 5c. Skip else
                        ctx.current_block = MirBasicBlock::new(skip_else_label);
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        ctx.current_block = MirBasicBlock::new(merge_label);
                    } else {
                        // 5. End block
                        ctx.current_block = MirBasicBlock::new(check_label);
                    }
                    ctx
                } else {
                    // === LIST-BASED FOR LOOP ===
                    let cond_label = ctx.fresh_block();
                    let body_label = ctx.fresh_block();
                    let inc_label = ctx.fresh_block();
                    let check_label = ctx.fresh_block();
                    let else_label = ctx.fresh_block();
                    let skip_else_label = ctx.fresh_block();
                    let merge_label = ctx.fresh_block();

                    let has_else = s.else_branch.is_some();

                    let flag_local = if has_else {
                        let f = ctx.alloc_local("_loop_break", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: f,
                            value: MirValue::Constant(MirConstant::Bool(false)),
                        });
                        Some(f)
                    } else {
                        None
                    };

                    // 1. Lower the iterable expression
                    // When iterating over a list variable by name, use the original alloca
                    // directly (no Store) to avoid triggering borrow analysis move.
                    let (iter_val, elem_type, iter_alloca) = if let Expr::BorrowRef { expression, .. } = &*s.iterable {
                        let var_name = match &*s.iterable {
                            Expr::BorrowRef { expression, .. } => {
                                if let Expr::Identifier { name, .. } = expression.as_ref() { name.clone() }
                                else { String::new() }
                            }
                            _ => String::new(),
                        };
                        let var_local = ctx.locals.get(&var_name).copied().unwrap_or(0);
                        let var_type = ctx.local_types.get(&var_local).cloned().unwrap_or(MirType::I64);
                        if matches!(&var_type, MirType::Dict(_, _)) {
                            // Dict iteration
                            let keys_list = ctx.alloc_local("_dkeys", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(keys_list),
                                name: "ky_dict_keys".to_string(),
                                args: vec![MirValue::Local(var_local)],
                            });
                            (var_local, MirType::Str, None)
                        } else if let MirType::List(inner) = &var_type {
                            // List borrow: use original list variable directly (no Store)
                            (var_local, inner.as_ref().clone(), Some(var_local))
                        } else {
                            ctx = self.lower_expr(ctx, &s.iterable);
                            (ctx.next_local - 1, MirType::I64, None)
                        }
                    } else {
                        // Non-borrow: for list identifiers, use original local to avoid move
                        let list_ident_orig = match &*s.iterable {
                            Expr::Identifier { name, .. } => ctx.locals.get(name).copied().and_then(|local| {
                                let t = ctx.local_types.get(&local)?;
                                if matches!(t, MirType::List(_)) { Some(local) } else { None }
                            }),
                            _ => None,
                        };
                        let (iv, et, ia) = if let Some(orig_local) = list_ident_orig {
                            let et = match ctx.local_types.get(&orig_local) {
                                Some(MirType::List(inner)) => inner.as_ref().clone(),
                                _ => MirType::I64,
                            };
                            (orig_local, et, Some(orig_local))
                        } else {
                            ctx = self.lower_expr(ctx, &s.iterable);
                            let iv = ctx.next_local - 1;
                            let et = match ctx.local_types.get(&iv) {
                                Some(MirType::List(inner)) => inner.as_ref().clone(),
                                _ => MirType::I64,
                            };
                            (iv, et, None)
                        };
                        (iv, et, ia)
                    };

                    // Allocate loop variable with proper type
                    let var_local = ctx.alloc_local(&s.variable, elem_type.clone());
                    ctx.locals.insert(s.variable.clone(), var_local);
                    if elem_type == MirType::Str {
                        ctx.string_locals.push(var_local);
                    }

                    // Determine list source: use original alloca if available, otherwise _for_list
                    let list_source = if let Some(orig) = iter_alloca {
                        orig
                    } else {
                        let list_alloca = ctx.alloc_local("_for_list", MirType::List(Box::new(elem_type.clone())));
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: list_alloca,
                            value: MirValue::Local(iter_val),
                        });
                        list_alloca
                    };

                    // Allocate and init index to 0
                    let idx_local = ctx.alloc_local("_for_idx", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: idx_local,
                        value: MirValue::Constant(MirConstant::I32(0)),
                    });

                    // Load list ptr and call kl_list_len
                    let list_tmp = ctx.alloc_local("_for_lt", MirType::List(Box::new(elem_type.clone())));
                    ctx.current_block.insts.push(MirInst::Load { dest: list_tmp, src: list_source });
                    let len_local = ctx.alloc_local("_for_len", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(len_local),
                        name: "ky_list_len".to_string(),
                        args: vec![MirValue::Local(list_tmp)],
                    });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    // 2. Cond block: load index, compare with len
                    ctx.current_block = MirBasicBlock::new(cond_label.clone());

                    // Load index
                    let idx = ctx.alloc_local("_for_i", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: idx, src: idx_local });

                    // Load and cast len i64→i32
                    let len_val = ctx.alloc_local("_for_lv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Load { dest: len_val, src: len_local });
                    let len32 = ctx.alloc_local("_for_len32", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: len32, value: MirValue::Local(len_val), to_type: MirType::I32,
                    });

                    // Compare idx < len
                    let cmp = ctx.alloc_local("_for_cmp", MirType::Bool);
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: cmp, op: MirBinaryOp::Lt,
                        left: MirValue::Local(idx), right: MirValue::Local(len32),
                    });

                    let false_target = if has_else { else_label.clone() } else { check_label.clone() };
                    ctx.finish_block(MirTerminator::CondBr {
                        cond: MirValue::Local(cmp),
                        true_block: body_label.clone(),
                        false_block: false_target.clone(),
                    });

                    // 3. Body block
                    ctx.current_block = MirBasicBlock::new(body_label.clone());
                    ctx.break_targets.push(false_target);
                    ctx.continue_targets.push(inc_label.clone());
                    if has_else { ctx.break_flag_local = flag_local; }

                    // Load current index
                    let idx2 = ctx.alloc_local("_for_i2", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: idx2, src: idx_local });

                    // Cast index to i64 for kl_list_get
                    let idx2_64 = ctx.alloc_local("_for_i64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: idx2_64, value: MirValue::Local(idx2), to_type: MirType::I64,
                    });

                    // Load list pointer
                    let list_tmp2 = ctx.alloc_local("_for_lt2", MirType::List(Box::new(elem_type.clone())));
                    ctx.current_block.insts.push(MirInst::Load { dest: list_tmp2, src: list_source });

                    // Call kl_list_get
                    let elem_raw = ctx.alloc_local("_for_elem", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(elem_raw),
                        name: "ky_list_get".to_string(),
                        args: vec![MirValue::Local(list_tmp2), MirValue::Local(idx2_64)],
                    });

                    // Store element into loop variable with correct type
                    if elem_type == MirType::Str {
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: var_local, value: MirValue::Local(elem_raw), to_type: MirType::Str,
                        });
                    } else if elem_type != MirType::I64 {
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: var_local, value: MirValue::Local(elem_raw), to_type: elem_type.clone(),
                        });
                    } else {
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: var_local, value: MirValue::Local(elem_raw),
                        });
                    }

                    // Lower body statements
                    for stmt in &s.body.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                    ctx.break_flag_local = None;
                    ctx.break_targets.pop();
                    ctx.continue_targets.pop();

                    ctx.finish_block(MirTerminator::Br(inc_label.clone()));

                    // 4. Increment block: idx += 1, then goto cond
                    ctx.current_block = MirBasicBlock::new(inc_label.clone());
                    let idx3 = ctx.alloc_local("_for_i3", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Load { dest: idx3, src: idx_local });
                    let idx4 = ctx.alloc_local("_for_i4", MirType::I32);
                    ctx.current_block.insts.push(MirInst::BinaryOp {
                        dest: idx4, op: MirBinaryOp::Add,
                        left: MirValue::Local(idx3), right: MirValue::Constant(MirConstant::I32(1)),
                    });
                    ctx.current_block.insts.push(MirInst::Store { dest: idx_local, value: MirValue::Local(idx4) });

                    ctx.finish_block(MirTerminator::Br(cond_label.clone()));

                    if has_else {
                        // 5a. Else-label: check flag → execute else or skip
                        ctx.current_block = MirBasicBlock::new(else_label.clone());
                        let f_load = ctx.alloc_local("_flag_v", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Load { dest: f_load, src: flag_local.unwrap() });
                        ctx.finish_block(MirTerminator::CondBr {
                            cond: MirValue::Local(f_load),
                            true_block: skip_else_label.clone(),
                            false_block: check_label.clone(),
                        });

                        // 5b. Check-label: execute else body
                        ctx.current_block = MirBasicBlock::new(check_label.clone());
                        if let Some(eb) = &s.else_branch {
                            for stmt in &eb.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                        }
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        // 5c. Skip else
                        ctx.current_block = MirBasicBlock::new(skip_else_label);
                        ctx.finish_block(MirTerminator::Br(merge_label.clone()));

                        ctx.current_block = MirBasicBlock::new(merge_label);
                    } else {
                        // 5. End block
                        ctx.current_block = MirBasicBlock::new(check_label);
                    }
                    ctx
                }
            }
            Stmt::Match(s) => {
                let end_label = ctx.fresh_block();
                let needs_val = s.arms.iter().any(|a| {
                    matches!(a.pattern, Pattern::Literal { .. } | Pattern::EnumVariant { .. } | Pattern::Tuple { .. } | Pattern::Range { .. })
                        || matches!(&a.pattern, Pattern::Or { patterns, .. } if patterns.iter().any(|p| matches!(p, Pattern::Literal { .. } | Pattern::EnumVariant { .. } | Pattern::Tuple { .. } | Pattern::Range { .. })))
                });
                if needs_val {
                    ctx = self.lower_expr(ctx, &s.expression);
                }
                let match_val = if needs_val { Some(ctx.next_local - 1) } else { None };

                // Lower each arm with condition checks (forward order).
                // Each literal/variant arm: check cond → arm_body, fallthrough → next check.
                // Wildcard/identifier: always execute (no check).
                let arm_count = s.arms.len();
                for (i, arm) in s.arms.iter().enumerate() {
                    let arm_label = ctx.fresh_block();
                    let is_last = i == arm_count - 1;
                    let next_target = if is_last {
                        end_label.clone()
                    } else {
                        ctx.fresh_block()
                    };
                    match &arm.pattern {
                        Pattern::Literal { value, .. } => {
                            let (vt, lc) = literal_to_mir(value);
                            let lit = ctx.alloc_local("_lit", vt);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: lit, value: MirValue::Constant(lc),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(match_val.unwrap()),
                                right: MirValue::Local(lit),
                            });
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::EnumVariant { enum_name, variant, args, .. } => {
                            // Look up variant index from enum_variants map
                            let ev_map = self.enum_variants.borrow();
                            let variant_idx = ev_map.get(enum_name)
                                .and_then(|m| m.get(variant))
                                .copied()
                                .unwrap_or(0);

                            let struct_type = MirType::Struct(
                                if enum_name == "Result" { "Result".to_string() } else { enum_name.clone() },
                                vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ],
                            );

                            // FieldPtr needs a pointer, but match_val is a loaded value.
                            // Store it to a temp alloca to get an addressable pointer.
                            let mv = match_val.unwrap();
                            let mv_type = ctx.local_types.get(&mv).cloned().unwrap_or(MirType::I64);
                            let struct_ptr = if matches!(mv_type, MirType::Struct(_, _)) {
                                let alloca = ctx.alloc_local("_mvtmp", mv_type.clone());
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: alloca,
                                    value: MirValue::Local(mv),
                                });
                                alloca
                            } else {
                                mv
                            };

                            // Load discriminant from match value
                            let disc_ptr = ctx.alloc_local("_disc_ptr", MirType::Ptr(Box::new(MirType::I32)));
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: disc_ptr,
                                ptr: struct_ptr,
                                field_index: 0,
                                struct_type: Box::new(struct_type.clone()),
                            });
                            let disc_val = ctx.alloc_local("_disc", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: disc_val,
                                src: disc_ptr,
                            });

                            // Compare discriminant with variant index
                            // Option.Some has disc=1 (not variant index 0), Option.None has disc=0
                            let expected_disc = if enum_name == "Option" && variant == "Some" {
                                1_i32
                            } else if enum_name == "Option" && variant == "None" {
                                0_i32
                            } else {
                                variant_idx as i32
                            };
                            let idx_local = ctx.alloc_local("_vidx", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: idx_local,
                                value: MirValue::Constant(MirConstant::I32(expected_disc)),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(disc_val),
                                right: MirValue::Local(idx_local),
                            });

                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(eq),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }

                            // Bind payload values to pattern variables
                            if !args.is_empty() {
                                let payload_ptr = ctx.alloc_local("_pay_ptr", MirType::I64);
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: payload_ptr,
                                    ptr: struct_ptr,
                                    field_index: 1,
                                    struct_type: Box::new(struct_type),
                                });
                                // Determine inner type for Option/Result
                                let inner_type = match mv_type.clone() {
                                    MirType::Struct(ref name, _) => {
                                        if let Some(inner) = name.strip_prefix("Option__") {
                                            match inner {
                                                "str" => MirType::Str, "i32" => MirType::I32,
                                                "i64" => MirType::I64, "f64" => MirType::F64,
                                                "bool" => MirType::Bool, _ => MirType::I64,
                                            }
                                        } else if name == "Result" { MirType::I64 }
                                        else { MirType::I64 }
                                    }
                                    _ => MirType::I64,
                                };
                                for (_pi, arg) in args.iter().enumerate() {
                                    match arg {
                                        Pattern::Identifier { name, .. } => {
                                            let pay_val = ctx.alloc_local("_pay", inner_type.clone());
                                            if inner_type == MirType::I64 {
                                                ctx.current_block.insts.push(MirInst::Load {
                                                    dest: pay_val, src: payload_ptr,
                                                });
                                            } else {
                                                let pay_i64 = ctx.alloc_local("_pay_i64", MirType::I64);
                                                ctx.current_block.insts.push(MirInst::Load {
                                                    dest: pay_i64, src: payload_ptr,
                                                });
                                                ctx.current_block.insts.push(MirInst::Cast {
                                                    dest: pay_val, value: MirValue::Local(pay_i64),
                                                    to_type: inner_type.clone(),
                                                });
                                            }
                                            let local = ctx.alloc_local(name, inner_type.clone());
                                            ctx.current_block.insts.push(MirInst::Store {
                                                dest: local,
                                                value: MirValue::Local(pay_val),
                                            });
                                            ctx.locals.insert(name.clone(), local);
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));

                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::Range { start, end, inclusive, .. } => {
                            let (st, sv) = literal_to_mir(start);
                            let (et, ev) = literal_to_mir(end);
                            let start_lit = ctx.alloc_local("_rs", st);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: start_lit, value: MirValue::Constant(sv),
                            });
                            let end_lit = ctx.alloc_local("_re", et);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: end_lit, value: MirValue::Constant(ev),
                            });
                            let ge = ctx.alloc_local("_ge", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: ge, op: MirBinaryOp::Ge,
                                left: MirValue::Local(match_val.unwrap()),
                                right: MirValue::Local(start_lit),
                            });
                            let le_op = if *inclusive { MirBinaryOp::Le } else { MirBinaryOp::Lt };
                            let le = ctx.alloc_local("_le", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: le, op: le_op,
                                left: MirValue::Local(match_val.unwrap()),
                                right: MirValue::Local(end_lit),
                            });
                            let cond = ctx.alloc_local("_rng", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: cond, op: MirBinaryOp::And,
                                left: MirValue::Local(ge),
                                right: MirValue::Local(le),
                            });
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(cond),
                                    true_block: guard_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::CondBr {
                                    cond: MirValue::Local(cond),
                                    true_block: arm_label.clone(),
                                    false_block: next_target.clone(),
                                });
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            // Execute arm body (handled below in shared code)
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::Tuple { .. } => {
                            // Tuple: treat as wildcard for now (always matches)
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label.clone());
                        }
                        Pattern::IsType { .. } | Pattern::Wildcard { .. } | Pattern::Identifier { .. } => {
                            if let Some(guard) = &arm.guard {
                                let guard_label = ctx.fresh_block();
                                ctx.finish_block(MirTerminator::Br(guard_label.clone()));
                                ctx.current_block = MirBasicBlock::new(guard_label);
                                ctx = self.lower_match_guard(ctx, guard, &arm_label, &next_target);
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            } else {
                                ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                                ctx.current_block = MirBasicBlock::new(arm_label);
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if arm.guard.is_some() {
                                // Guard may fail; continue to next arm (next_target is the false branch)
                                if !is_last {
                                    ctx.current_block = MirBasicBlock::new(next_target);
                                } else {
                                    ctx.current_block = MirBasicBlock::new(end_label.clone());
                                }
                            } else {
                                // No guard: this arm always matches, skip remaining arms
                                ctx.current_block = MirBasicBlock::new(end_label.clone());
                                return ctx;
                            }
                        }
                        Pattern::Or { .. } => {
                            // Or pattern: always matches (wildcard behavior)
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label.clone());
                            // Create the arm body block (shared for all alternatives)
                            ctx.current_block = MirBasicBlock::new(arm_label.clone());
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            if let Some(result_local) = ctx.match_result_local {
                                let last_val = ctx.next_local - 1;
                                let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                                let val_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I32);
                                let store_val = if result_type != val_type {
                                    let cast = ctx.alloc_local("_tc", result_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cast,
                                        value: MirValue::Local(last_val),
                                        to_type: result_type,
                                    });
                                    cast
                                } else {
                                    last_val
                                };
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: result_local,
                                    value: MirValue::Local(store_val),
                                });
                            }
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        _ => {
                            // Unknown pattern: skip
                            ctx.finish_block(MirTerminator::Br(end_label.clone()));
                            ctx.current_block = MirBasicBlock::new(end_label);
                            return ctx;
                        }
                    }
                }
                // If all arms were literal (no wildcard), the current block is already end_label
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Stmt::Guard(g) => {
                ctx = self.lower_expr(ctx, &g.condition);
                let cond_val = MirValue::Local(ctx.next_local - 1);
                let then_label = ctx.fresh_block();
                let else_label = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: then_label.clone(),
                    false_block: else_label.clone(),
                });
                // then: continue
                ctx.current_block = MirBasicBlock::new(then_label);
                let after = ctx.fresh_block();
                ctx.finish_block(MirTerminator::Br(after.clone()));
                // else: run body (should contain return/break)
                ctx.current_block = MirBasicBlock::new(else_label);
                for stmt in &g.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.current_block = MirBasicBlock::new(after);
                ctx
            }
            Stmt::Unsafe(u) => {
                for stmt in &u.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx
            }
            Stmt::Defer(d) => {
                ctx.deferred_exprs.push(d.call.clone());
                ctx
            }
            Stmt::BindingIf(b) => {
                // Lower the optional expression
                ctx = self.lower_expr(ctx, &b.value);
                let opt_val = ctx.next_local - 1;
                let opt_type = ctx.local_types.get(&opt_val).cloned().unwrap_or(MirType::I32);
                // Determine inner type from Option type name
                let inner_type = if let MirType::Struct(name, _) = &opt_type {
                    if let Some(inner) = name.strip_prefix("Option__") {
                        match inner {
                            "str" => MirType::Str, "i32" => MirType::I32,
                            "i64" => MirType::I64, "f64" => MirType::F64,
                            "bool" => MirType::Bool, _ => MirType::I64,
                        }
                    } else { MirType::I64 }
                } else { MirType::I64 };
                // Read disc field (disc == 1 means Some)
                let disc_ptr = ctx.alloc_local("_disc_ptr", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr, ptr: opt_val, field_index: 0,
                    struct_type: Box::new(opt_type.clone()),
                });
                let disc_val = ctx.alloc_local("_disc_val", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc_val, src: disc_ptr,
                });
                // Compare disc == 1 (Some)
                let cmp = ctx.alloc_local("_bif_cmp", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: cmp, op: MirBinaryOp::Eq,
                    left: MirValue::Local(disc_val),
                    right: MirValue::Constant(MirConstant::I32(1)),
                });
                let then_label = ctx.fresh_block();
                let else_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(cmp),
                    true_block: then_label.clone(),
                    false_block: else_label.clone(),
                });
                // Then block: extract payload, bind variable, execute body
                ctx.current_block = MirBasicBlock::new(then_label);
                let pay_ptr = ctx.alloc_local("_pay_ptr", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: pay_ptr, ptr: opt_val, field_index: 1,
                    struct_type: Box::new(opt_type.clone()),
                });
                let pay_val = ctx.alloc_local("_pay_val", inner_type.clone());
                if inner_type == MirType::I64 {
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: pay_val, src: pay_ptr,
                    });
                } else {
                    let pay_i64 = ctx.alloc_local("_pay_i64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: pay_i64, src: pay_ptr,
                    });
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: pay_val, value: MirValue::Local(pay_i64),
                        to_type: inner_type.clone(),
                    });
                }
                let var_local = ctx.alloc_local(&b.name, inner_type.clone());
                ctx.locals.insert(b.name.clone(), var_local);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: var_local, value: MirValue::Local(pay_val),
                });
                for stmt in &b.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.finish_block(MirTerminator::Br(end_label.clone()));
                // Else block
                ctx.current_block = MirBasicBlock::new(else_label);
                if let Some(el) = &b.else_branch {
                    for stmt in &el.statements {
                        ctx = self.lower_stmt(ctx, stmt);
                    }
                }
                ctx.finish_block(MirTerminator::Br(end_label.clone()));
                // Merge
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Stmt::Break(_, _) => {
                if let Some(flag) = ctx.break_flag_local {
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: flag,
                        value: MirValue::Constant(MirConstant::Bool(true)),
                    });
                }
                if let Some(target) = ctx.break_targets.last().cloned() {
                    ctx.finish_block(MirTerminator::Br(target));
                } else {
                    ctx.finish_block(MirTerminator::Unreachable);
                }
                ctx
            }
            Stmt::Continue(_) => {
                if let Some(target) = ctx.continue_targets.last().cloned() {
                    ctx.finish_block(MirTerminator::Br(target));
                } else {
                    ctx.finish_block(MirTerminator::Unreachable);
                }
                ctx
            }
            Stmt::WhileBind(w) => {
                for stmt in &w.body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx
            }
            Stmt::Constant(c) => {
                ctx = self.lower_expr(ctx, &c.value);
                ctx
            }
            Stmt::TypedVariable(v) => {
                ctx = self.lower_expr(ctx, &v.value);
                let val_local = ctx.next_local - 1;
                let val_type = ctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                let var_type = v.type_.as_ref()
                    .map(|t| ast_type_to_mir(t, Some(&ctx.struct_defs)))
                    .unwrap_or(val_type.clone());
                let store_val = if val_type != var_type {
                    let cast = ctx.alloc_local("_tcast", var_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast { dest: cast, value: MirValue::Local(val_local), to_type: var_type.clone() });
                    MirValue::Local(cast)
                } else {
                    MirValue::Local(val_local)
                };
                let local = ctx.alloc_local(&v.name, var_type.clone());
                ctx.locals.insert(v.name.clone(), local);
                ctx.current_block.insts.push(MirInst::Store { dest: local, value: store_val });
                ctx
            }
        }
    }

    pub(crate) fn lower_match_guard(&self, mut ctx: LowerCtx, guard: &Expr, true_block: &str, false_block: &str) -> LowerCtx {
        let prev_next = ctx.next_local;
        ctx = self.lower_expr(ctx, guard);
        let guard_val = if ctx.next_local > prev_next {
            ctx.next_local - 1
        } else {
            // No expression result (Void) — treat as false
            return ctx;
        };
        let guard_type = ctx.local_types.get(&guard_val).cloned().unwrap_or(MirType::Bool);
        // If guard is not bool, compare with 0 for truthiness
        let cond = if guard_type == MirType::Bool {
            MirValue::Local(guard_val)
        } else {
            let zero = ctx.alloc_local("_zero", guard_type.clone());
            ctx.current_block.insts.push(MirInst::Store {
                dest: zero,
                value: MirValue::Constant(MirConstant::I32(0)),
            });
            let cmp = ctx.alloc_local("_gcmp", MirType::Bool);
            ctx.current_block.insts.push(MirInst::BinaryOp {
                dest: cmp, op: MirBinaryOp::Neq,
                left: MirValue::Local(guard_val),
                right: MirValue::Local(zero),
            });
            MirValue::Local(cmp)
        };
        ctx.finish_block(MirTerminator::CondBr {
            cond,
            true_block: true_block.to_string(),
            false_block: false_block.to_string(),
        });
        ctx
    }
}
