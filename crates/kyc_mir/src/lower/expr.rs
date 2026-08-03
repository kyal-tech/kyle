use crate::mir::*;
use kyc_core::ast::*;
use super::*;

impl super::Lowerer {
    pub(crate) fn lower_expr(&self, mut ctx: LowerCtx, expr: &Expr) -> LowerCtx {
        match expr {
            Expr::ArrayRepeat { value, count, .. } => {
                ctx = self.lower_expr(ctx, value);
                let elem_local = ctx.next_local - 1;
                let elem_type = ctx.local_types.get(&elem_local).cloned()
                    .unwrap_or(MirType::I32);
                // Evaluate count expression at compile time
                let size = match count.as_ref() {
                    Expr::Literal { value: Literal::Integer(n), .. } => *n as usize,
                    Expr::Identifier { name, .. } => {
                        // Look up constant value
                        if let Some(const_expr) = self.const_values.borrow().get(name) {
                            if let Expr::Literal { value: Literal::Integer(n), .. } = const_expr {
                                *n as usize
                            } else {
                                ctx = self.lower_expr(ctx, count);
                                0usize
                            }
                        } else {
                            ctx = self.lower_expr(ctx, count);
                            0usize
                        }
                    }
                    _ => {
                        ctx = self.lower_expr(ctx, count);
                        0usize
                    }
                };
                let arr_type = MirType::Array(Box::new(elem_type.clone()), size);
                let arr_local = ctx.alloc_local("_arr", arr_type.clone());
                // Fill each element
                let zero = MirConstant::I32(0);
                let one = MirConstant::I32(1);
                let idx_local = ctx.alloc_local("_ar_idx", MirType::I32);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: idx_local,
                    value: MirValue::Constant(zero),
                });
                let body_label = ctx.fresh_block();
                let check_label = ctx.fresh_block();
                let done_label = ctx.fresh_block();
                ctx.finish_block(MirTerminator::Br(check_label.clone()));
                ctx.current_block = MirBasicBlock::new(check_label.clone());
                let idx_loaded = ctx.alloc_local("_ar_ld", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: idx_loaded,
                    src: idx_local,
                });
                let cond = ctx.alloc_local("_ar_cond", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: cond,
                    op: MirBinaryOp::Lt,
                    left: MirValue::Local(idx_loaded),
                    right: MirValue::Constant(MirConstant::I32(size as i32)),
                });
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(cond),
                    true_block: body_label.clone(),
                    false_block: done_label.clone(),
                });
                ctx.current_block = MirBasicBlock::new(body_label.clone());
                let elem_ptr = ctx.alloc_local("_areptr", elem_type.clone());
                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                    dest: elem_ptr,
                    ptr: arr_local,
                    index: MirValue::Local(idx_loaded),
                    arr_type: Box::new(arr_type.clone()),
                    elem_type: Box::new(elem_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: elem_ptr,
                    value: MirValue::Local(elem_local),
                });
                let inc = ctx.alloc_local("_ar_inc", MirType::I32);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: inc,
                    op: MirBinaryOp::Add,
                    left: MirValue::Local(idx_loaded),
                    right: MirValue::Constant(one),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: idx_local,
                    value: MirValue::Local(inc),
                });
                ctx.finish_block(MirTerminator::Br(check_label.clone()));
                ctx.current_block = MirBasicBlock::new(done_label.clone());
                let arr_val = ctx.alloc_local("_arrv", arr_type.clone());
                ctx.current_block.insts.push(MirInst::Load {
                    dest: arr_val,
                    src: arr_local,
                });
                ctx
            }
            Expr::Array { elements, .. } => {
                if elements.is_empty() {
                    let arr_local = ctx.alloc_local("_arr", MirType::Array(Box::new(MirType::I32), 0));
                    let arr_val = ctx.alloc_local("_arrv", MirType::Array(Box::new(MirType::I32), 0));
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: arr_val,
                        src: arr_local,
                    });
                    return ctx;
                }
                let mut elem_locals = Vec::new();
                for elem in elements {
                    ctx = self.lower_expr(ctx, elem);
                    elem_locals.push(ctx.next_local - 1);
                }
                let elem_type = ctx.local_types.get(&elem_locals[0]).cloned()
                    .unwrap_or(MirType::I32);
                let size = elem_locals.len();
                let arr_type = MirType::Array(Box::new(elem_type.clone()), size);
                let arr_local = ctx.alloc_local("_arr", arr_type.clone());
                // Store each element into the array via computed GEP
                for (i, &elem_local) in elem_locals.iter().enumerate() {
                    let idx_val = MirValue::Constant(MirConstant::I32(i as i32));
                    let elem_ptr = ctx.alloc_local("_aeptr", elem_type.clone());
                    ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                        dest: elem_ptr,
                        ptr: arr_local,
                        index: idx_val,
                        arr_type: Box::new(arr_type.clone()),
                        elem_type: Box::new(elem_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: elem_ptr,
                        value: MirValue::Local(elem_local),
                    });
                }
                // Load the array value from alloca so assignment can store it
                let arr_val = ctx.alloc_local("_arrv", arr_type.clone());
                ctx.current_block.insts.push(MirInst::Load {
                    dest: arr_val,
                    src: arr_local,
                });
                ctx
            }
            Expr::Literal { value, .. } => {
                let (mir_type, constant) = match value {
                    Literal::Integer(n) => {
                        if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                            (MirType::I32, MirConstant::I32(*n as i32))
                        } else {
                            (MirType::I64, MirConstant::I64(*n))
                        }
                    }
                    Literal::Float(n) => (MirType::F64, MirConstant::F64(*n)),
                    Literal::String(s) => (MirType::Str, MirConstant::String(s.clone())),
                    Literal::Boolean(b) => (MirType::Bool, MirConstant::Bool(*b)),
                    Literal::Char(c) => (MirType::Char, MirConstant::I32(*c)),
                    Literal::None => (MirType::I32, MirConstant::Void),
                    Literal::Null => (MirType::Ptr(Box::new(MirType::Void)), MirConstant::Null),
                };
                let is_str = matches!(constant, MirConstant::String(_));
                if is_str {
                    // String literals must be heap-allocated so that move semantics
                    // can safely kl_free them. Store the constant in a temp, then
                    // call kl_clone_str to create an owned heap copy.
                    let const_local = ctx.alloc_local("_lit_const", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: const_local,
                        value: MirValue::Constant(constant),
                    });
                    let local = ctx.alloc_local("_lit", MirType::Str);
                    ctx.string_locals.push(local);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(local),
                        name: "ky_clone_str".to_string(),
                        args: vec![MirValue::Local(const_local)],
                    });
                    ctx
                } else {
                    let local = ctx.alloc_local("_lit", mir_type);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: local,
                        value: MirValue::Constant(constant),
                    });
                    ctx
                }
            }
            Expr::Identifier { name, .. } => {
                if let Some(local) = ctx.locals.get(name) {
                    let src = *local;
                    let load_type = ctx.local_types.get(&src).cloned().unwrap_or(MirType::I32);
                    // For ^&T (ref param) locals: use ky_ptr_read_i32 to read through the pointer
                    if ctx.ref_param_locals.contains(&src) {
                        let inner_type = if let MirType::Ptr(inner) = &load_type {
                            inner.as_ref().clone()
                        } else { MirType::I32 };
                        if inner_type == MirType::Str {
                            let dest = ctx.alloc_local("_refval", MirType::Str);
                            ctx.string_locals.push(dest);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest, src,
                            });
                            return ctx;
                        }
                        let ptr_load = ctx.alloc_local("_refptr", MirType::Ptr(Box::new(MirType::I8)));
                        ctx.current_block.insts.push(MirInst::Load {
                            dest: ptr_load, src,
                        });
                        let dest = ctx.alloc_local("_refval", inner_type.clone());
                        // Use ky_ptr_read_ptr for heap-allocated types (ptr-internal),
                        // ky_ptr_read_i32 for scalar types.
                        let is_heap = matches!(&inner_type, MirType::List(_) | MirType::Dict(_, _)
                            | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Ptr(_)
                            | MirType::Box(_));
                        if is_heap {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: "ky_ptr_read_ptr".to_string(),
                                args: vec![MirValue::Local(ptr_load)],
                            });
                        } else {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: "ky_ptr_read_i32".to_string(),
                                args: vec![MirValue::Local(ptr_load)],
                            });
                        }
                    } else {
                        let is_str_load = load_type == MirType::Str;
                        let dest = ctx.alloc_local("_load", load_type);
                        if is_str_load {
                            ctx.string_locals.push(dest);
                        }
                        ctx.current_block.insts.push(MirInst::Load {
                            dest, src,
                        });
                    }
                } else if let Some(const_expr) = self.const_values.borrow().get(name) {
                    // Inline module-level constant value
                    ctx = self.lower_expr(ctx, const_expr);
                } else if self.function_decls.borrow().contains_key(name) {
                    // Function name used as value (e.g. worker as ptr)
                    let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                    let dest = ctx.alloc_local("_fnaddr", ptr_type);
                    ctx.current_block.insts.push(MirInst::FnAddr {
                        dest,
                        name: name.clone(),
                    });
                }
                ctx
            }
            Expr::Binary { left, operator, right, .. } => {
                // Helper: map compound assignment operator to its base binary op
                let is_compound = matches!(operator,
                    BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign |
                    BinaryOp::DivAssign | BinaryOp::RemAssign
                );
                let compound_base_op = |op: &BinaryOp| -> Option<MirBinaryOp> {
                    match op {
                        BinaryOp::AddAssign => Some(MirBinaryOp::Add),
                        BinaryOp::SubAssign => Some(MirBinaryOp::Sub),
                        BinaryOp::MulAssign => Some(MirBinaryOp::Mul),
                        BinaryOp::DivAssign => Some(MirBinaryOp::Div),
                        BinaryOp::RemAssign => Some(MirBinaryOp::Rem),
                        _ => None,
                    }
                };

                // Handle compound assignment: x += expr, x -= expr, etc.
                if is_compound {
                    // Only handle simple identifier targets for now
                    if let Expr::Identifier { name, .. } = left.as_ref() {
                        if let Some(var_addr) = ctx.locals.get(name) {
                            let var_addr = *var_addr;
                            let var_type = ctx.local_types.get(&var_addr).cloned().unwrap_or(MirType::I32);
                            let left_is_str = var_type == MirType::Str;

                            // Load current value from variable
                            let loaded = ctx.alloc_local("_ca_load", var_type.clone());
                            if left_is_str {
                                ctx.string_locals.push(loaded);
                            }
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: loaded,
                                src: var_addr,
                            });

                            // Lower right side
                            ctx = self.lower_expr(ctx, right);
                            let right_local = ctx.next_local - 1;
                            let right_is_str = ctx.string_locals.contains(&right_local);

                            // String concatenation for +=
                            if matches!(operator, BinaryOp::AddAssign) && (left_is_str || right_is_str) {
                                let left_len = ctx.alloc_local("_strlen", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(left_len),
                                    name: "ky_strlen".to_string(),
                                    args: vec![MirValue::Local(loaded)],
                                });
                                let right_len = ctx.alloc_local("_strlen", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(right_len),
                                    name: "ky_strlen".to_string(),
                                    args: vec![MirValue::Local(right_local)],
                                });
                                let result = ctx.alloc_local("_bin", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(result),
                                    name: "ky_concat".to_string(),
                                    args: vec![
                                        MirValue::Local(loaded),
                                        MirValue::Local(left_len),
                                        MirValue::Local(right_local),
                                        MirValue::Local(right_len),
                                    ],
                                });
                                ctx.string_locals.push(result);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: var_addr,
                                    value: MirValue::Local(result),
                                });
                                return ctx;
                            }

                            // Determine base operator
                            let base_op = compound_base_op(&operator).unwrap_or(MirBinaryOp::Add);

                            // Coerce operands to same type
                            let left_type = ctx.local_types.get(&loaded).cloned().unwrap_or(MirType::I32);
                            let right_type = ctx.local_types.get(&right_local).cloned().unwrap_or(MirType::I32);
                            let wider = wider_type(&left_type, &right_type);
                            let is_float_op = matches!(wider, MirType::F32 | MirType::F64);
                            let left_operand = if left_type != wider && is_int_type(&left_type) && !is_float_op {
                                let cast = ctx.alloc_local("_widen", wider.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast,
                                    value: MirValue::Local(loaded),
                                    to_type: wider.clone(),
                                });
                                MirValue::Local(cast)
                            } else {
                                MirValue::Local(loaded)
                            };
                            let right_operand = if right_type != wider && is_int_type(&right_type) && !is_float_op {
                                let cast = ctx.alloc_local("_widen", wider.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast,
                                    value: MirValue::Local(right_local),
                                    to_type: wider.clone(),
                                });
                                MirValue::Local(cast)
                            } else {
                                MirValue::Local(right_local)
                            };

                            let result_type = if is_float_op { wider.clone() } else { var_type.clone() };
                            let result = ctx.alloc_local("_ca_result", result_type.clone());
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: result,
                                op: base_op,
                                left: left_operand,
                                right: right_operand,
                            });

                            // Store result back
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: var_addr,
                                value: MirValue::Local(result),
                            });
                            return ctx;
                        }
                    }
                }

                // Handle assignment: target = value
                if matches!(operator, BinaryOp::Assign) {
                    if let Expr::Index { target: index_target, index, .. } = left.as_ref() {
                        // For arrays, use variable alloca directly (skip whole-array Load)
                        let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = index_target.as_ref() {
                            if let Some(&local) = ctx.locals.get(name) {
                                let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                if matches!(t, MirType::Array(_, _)) {
                                    (local, local, Some(t))
                                } else {
                                    ctx = self.lower_expr(ctx, index_target);
                                    let tv = ctx.next_local - 1;
                                    (tv, tv, ctx.local_types.get(&tv).cloned())
                                }
                            } else {
                                ctx = self.lower_expr(ctx, index_target);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned())
                            }
                        } else {
                            // Nested array assignment like m[i][j] = val.
                            let indices = self.collect_array_indices(left);
                            if let Some((root_name, idx_exprs)) = indices {
                                if let Some(&root_local) = ctx.locals.get(&root_name) {
                                    ctx = self.lower_nested_array_geps(ctx, &idx_exprs, root_local);
                                    ctx = self.lower_expr(ctx, right);
                                    let val_local2 = ctx.next_local - 1;
                                    let gep_ptr = ctx.next_local - 2;
                                    ctx.current_block.insts.push(MirInst::Store {
                                        dest: gep_ptr,
                                        value: MirValue::Local(val_local2),
                                    });
                                    return ctx;
                                }
                            }
                            ctx = self.lower_expr(ctx, index_target);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned())
                        };
                        ctx = self.lower_expr(ctx, index);
                        let idx_val = ctx.next_local - 1;
                        // Array set: arr[i] = val → ArrayElemPtr + Store
                        if matches!(&target_type, Some(MirType::Array(_, _))) {
                            ctx = self.lower_expr(ctx, right);
                            let val_local2 = ctx.next_local - 1;
                            if let Some(MirType::Array(ref inner_box, _)) = target_type {
                                let et = inner_box.as_ref().clone();
                                let arr_ty_clone = target_type.clone().unwrap();
                                let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                                    dest: elem_ptr,
                                    ptr: arr_ptr,
                                    index: MirValue::Local(idx_val),
                                    arr_type: Box::new(arr_ty_clone),
                                    elem_type: Box::new(et),
                                });
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: elem_ptr,
                                    value: MirValue::Local(val_local2),
                                });
                            }
                            return ctx;
                        }
                        // Struct set: obj[i] = val → obj.op_index_set(i, val)
                        if let Some(MirType::Struct(class_name, _)) = &target_type {
                            let method_table = self.method_table.borrow();
                            if let Some(methods) = method_table.get(class_name) {
                                if let Some(mangled) = methods.get("op_index_set") {
                                    ctx = self.lower_expr(ctx, right);
                                    let val_local = ctx.next_local - 1;
                                    let ret_type = self.fn_returns.borrow()
                                        .get(mangled).cloned().unwrap_or(MirType::Void);
                                    let dest = ctx.alloc_local("_ois", ret_type.clone());
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(dest),
                                        name: mangled.clone(),
                                        args: vec![
                                            MirValue::Local(target_val),
                                            MirValue::Local(idx_val),
                                            MirValue::Local(val_local),
                                        ],
                                    });
                                    return ctx;
                                }
                            }
                        }
                        // Dict set: dict["key"] = val → kl_dict_set
                        if matches!(&target_type, Some(MirType::Dict(_, _))) {
                            ctx = self.lower_expr(ctx, right);
                            let val_local = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_dict_val", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64,
                                value: MirValue::Local(val_local),
                                to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None,
                                name: "ky_dict_set".to_string(),
                                args: vec![
                                    MirValue::Local(target_val),
                                    MirValue::Local(idx_val),
                                    MirValue::Local(val_i64),
                                ],
                            });
                            return ctx;
                        }
                        // Ptr/Str set: ptr[idx] = val → MirInst::PtrStore
                        if matches!(target_type, Some(MirType::Ptr(_))) || target_type == Some(MirType::Str) {
                            ctx = self.lower_expr(ctx, right);
                            let val_local = ctx.next_local - 1;
                            let result = ctx.alloc_local("_ps", MirType::I32);
                            ctx.current_block.insts.push(MirInst::PtrStore {
                                dest: result,
                                ptr: target_val,
                                index: MirValue::Local(idx_val),
                                value: MirValue::Local(val_local),
                            });
                            return ctx;
                        }
                        // List set: list[idx] = val → kl_list_set
                        let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: idx_i64,
                            value: MirValue::Local(idx_val),
                            to_type: MirType::I64,
                        });

                        ctx = self.lower_expr(ctx, right);
                        let val_local = ctx.next_local - 1;
                        let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: val_i64,
                            value: MirValue::Local(val_local),
                            to_type: MirType::I64,
                        });
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_list_set".to_string(),
                            args: vec![
                                MirValue::Local(target_val),
                                MirValue::Local(idx_i64),
                                MirValue::Local(val_i64),
                            ],
                        });

                        return ctx;
                    }
                    if let Expr::PropertyAccess { object, property, .. } = left.as_ref() {
                        // If field is List and value is empty Dict {}, create list instead
                        if let Expr::Identifier { name, .. } = object.as_ref() {
                            if let Some(&obj_local) = ctx.locals.get(name) {
                                let local_type = ctx.local_types.get(&obj_local).cloned();
                                let (struct_name_first, struct_fields_first): (Option<String>, Option<Vec<(String, MirType)>>) = match local_type {
                                    Some(MirType::Struct(n, fields)) => (Some(n), Some(fields)),
                                    Some(MirType::Ptr(inner)) => match *inner {
                                        MirType::Struct(n, fields) => (Some(n), Some(fields)),
                                        _ => (None, None),
                                    },
                                    _ => (None, None),
                                };
                                if let Some(ref fields) = struct_fields_first {
                                    let backing = format!("_{}", property);
                                    let field_idx = fields.iter().position(|(fname, _)| fname == property.as_str())
                                        .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                                    if let Some(fi) = field_idx {
                                        if let Some((_, MirType::List(inner))) = fields.get(fi) {
                                            if let Expr::Dictionary { entries, .. } = right.as_ref() {
                                                if entries.is_empty() {
                                                    let obj_ty = ctx.local_types.get(&obj_local).cloned().unwrap();
                                                    let handle = ctx.alloc_local("_listv", MirType::List(inner.clone()));
                                                    ctx.current_block.insts.push(MirInst::Call {
                                                        dest: Some(handle),
                                                        name: "ky_list_new".to_string(),
                                                        args: vec![],
                                                    });
                                                    let ft = ctx.alloc_local("_fptr", MirType::I64);
                                                    ctx.current_block.insts.push(MirInst::FieldPtr {
                                                        dest: ft,
                                                        ptr: obj_local,
                                                        field_index: fi,
                                                        struct_type: Box::new(obj_ty),
                                                    });
                                                    ctx.current_block.insts.push(MirInst::Store {
                                                        dest: ft,
                                                        value: MirValue::Local(handle),
                                                    });
                                                    return ctx;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        ctx = self.lower_expr(ctx, right);
                        let val_local = ctx.next_local - 1;
                        let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                            ctx.locals.get(name).copied()
                        } else {
                            None
                        };
                        if let Some(obj_ptr) = obj_ptr {
                            let local_type = ctx.local_types.get(&obj_ptr).cloned();
                            let (struct_name_opt, struct_fields): (Option<String>, Option<Vec<(String, MirType)>>) = match local_type {
                                Some(MirType::Struct(n, fields)) => (Some(n), Some(fields)),
                                Some(MirType::Ptr(inner)) => match *inner {
                                    MirType::Struct(n, fields) => (Some(n), Some(fields)),
                                    _ => (None, None),
                                },
                                _ => (None, None),
                            };
                            if let Some(fields) = struct_fields {
                                // Check if this field is actually a property with a setter
                                let class_name = if let Some(MirType::Struct(cname, _)) = ctx.local_types.get(&obj_ptr) {
                                    Some(cname.clone())
                                } else if let Some(MirType::Ptr(inner)) = ctx.local_types.get(&obj_ptr) {
                                    if let MirType::Struct(cname, _) = inner.as_ref() {
                                        Some(cname.clone())
                                    } else { None }
                                } else { None };
                                let setter_name = class_name.as_ref().and_then(|cn| {
                                    let methods = self.method_table.borrow();
                                    methods.get(cn.as_str()).and_then(|m| {
                                        let sn = format!("set_{}", property);
                                        m.get(&sn).cloned()
                                    })
                                });
                                if let Some(sn) = setter_name {
                                    // Generate call to property setter instead of field store
                                    let this_local = ctx.alloc_local("_this", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Store {
                                        dest: this_local,
                                        value: MirValue::Local(obj_ptr),
                                    });
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: None,
                                        name: sn,
                                        args: vec![MirValue::Local(this_local), MirValue::Local(val_local)],
                                    });
                                } else {
                                    let backing = format!("_{}", property);
                                    let field_idx = fields.iter().position(|(fname, _)| fname == property)
                                        .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                                    if let Some(field_idx) = field_idx {
                                        let ft = ctx.alloc_local("_fptr", MirType::I64);
                                        ctx.current_block.insts.push(MirInst::FieldPtr {
                                            dest: ft,
                                            ptr: obj_ptr,
                                            field_index: field_idx,
                                            struct_type: Box::new(MirType::Struct(struct_name_opt.clone().unwrap_or("_".to_string()), fields.clone())),
                                        });
                                        ctx.current_block.insts.push(MirInst::Store {
                                            dest: ft,
                                            value: MirValue::Local(val_local),
                                        });
                                    }
                                }
                            }
                        }
                        return ctx;
                    }
                }

                // Handle plain assignment: x = expr (from deferred expressions parsed as BinaryOp::Assign)
                if matches!(operator, BinaryOp::Assign) {
                    ctx = self.lower_expr(ctx, right);
                    let val_local = ctx.next_local - 1;
                    if let Expr::Index { target: list_expr, index, .. } = left.as_ref() {
                        // For arrays, use variable alloca directly (skip whole-array Load)
                        let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = list_expr.as_ref() {
                            if let Some(&local) = ctx.locals.get(name) {
                                let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                if matches!(t, MirType::Array(_, _)) {
                                    (local, local, t)
                                } else {
                                    ctx = self.lower_expr(ctx, list_expr);
                                    let tv = ctx.next_local - 1;
                                    (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                                }
                            } else {
                                ctx = self.lower_expr(ctx, list_expr);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                            }
                        } else {
                            ctx = self.lower_expr(ctx, list_expr);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        };
                        ctx = self.lower_expr(ctx, index);
                        let idx_val = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, right);
                        let val_local2 = ctx.next_local - 1;
                        if matches!(&target_type, MirType::Array(_, _)) {
                            let arr_ty_clone = target_type.clone();
                            let et = match &target_type {
                                MirType::Array(inner_box, _) => { inner_box.as_ref().clone() },
                                _ => MirType::I32,
                            };
                            let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                                dest: elem_ptr,
                                ptr: arr_ptr,
                                index: MirValue::Local(idx_val),
                                arr_type: Box::new(arr_ty_clone),
                                elem_type: Box::new(et),
                            });
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: elem_ptr,
                                value: MirValue::Local(val_local2),
                            });
                        } else if matches!(&target_type, MirType::Dict(_, _)) {
                            let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast { dest: val_i64, value: MirValue::Local(val_local2), to_type: MirType::I64 });
                            let key_arg = if ctx.local_types.get(&idx_val).map(|t| *t == MirType::Str).unwrap_or(false) {
                                MirValue::Local(idx_val)
                            } else {
                                MirValue::Local(idx_val)
                            };
                            ctx.current_block.insts.push(MirInst::Call { dest: None, name: "ky_dict_set".to_string(), args: vec![MirValue::Local(target_val), key_arg, MirValue::Local(val_i64)] });
                        } else {
                            let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast { dest: idx_i64, value: MirValue::Local(idx_val), to_type: MirType::I64 });
                            let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast { dest: val_i64, value: MirValue::Local(val_local2), to_type: MirType::I64 });
                            ctx.current_block.insts.push(MirInst::Call { dest: None, name: "ky_list_set".to_string(), args: vec![MirValue::Local(target_val), MirValue::Local(idx_i64), MirValue::Local(val_i64)] });
                        }
                    } else if let Expr::Identifier { name, .. } = left.as_ref() {
                        if let Some(local) = ctx.locals.get(name) {
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: *local,
                                value: MirValue::Local(val_local),
                            });
                        }
                    }
                    return ctx;
                }

                ctx = self.lower_expr(ctx, left);
                let left_local = ctx.next_local - 1;
                let left_is_str = ctx.string_locals.contains(&left_local)
                    || ctx.local_types.get(&left_local).map_or(false, |t| *t == MirType::Str);
                ctx = self.lower_expr(ctx, right);
                let right_local = ctx.next_local - 1;
                let right_is_str = ctx.string_locals.contains(&right_local)
                    || ctx.local_types.get(&right_local).map_or(false, |t| *t == MirType::Str);
                // Range expression outside brackets — no-op
                if matches!(operator, BinaryOp::Range) {
                    return ctx;
                }

                // `is` type test — compare left expression type with right type name
                if matches!(operator, BinaryOp::Is) {
                    let left_local = ctx.next_local - 1;
                    let left_type = ctx.local_types.get(&left_local);
                    let right_mir_type = if let Expr::Identifier { name, .. } = right.as_ref() {
                        match name.as_str() {
                            "i8" => Some(MirType::I8),
                            "i16" => Some(MirType::I16),
                            "i32" => Some(MirType::I32),
                            "i64" => Some(MirType::I64),
                            "f32" => Some(MirType::F32),
                            "f64" => Some(MirType::F64),
                            "bool" => Some(MirType::Bool),
                            "char" => Some(MirType::Char),
                            "str" => Some(MirType::Str),
                            _ => None,
                        }
                    } else { None };
                    let matches = left_type.and_then(|lt| right_mir_type.map(|rt| *lt == rt)).unwrap_or(false);
                    let dest = ctx.alloc_local("_is", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest,
                        value: if matches {
                            MirValue::Constant(MirConstant::I32(1))
                        } else {
                            MirValue::Constant(MirConstant::I32(0))
                        },
                    });
                    return ctx;
                }

                // `as` cast: left as TypeName
                if matches!(operator, BinaryOp::As) {
                    let left_local = ctx.next_local - 1;
                    let to_type = if let Expr::Identifier { name, .. } = right.as_ref() {
                        match name.as_str() {
                            "i8" => MirType::I8,
                            "u8" => MirType::U8,
                            "i16" => MirType::I16,
                            "u16" => MirType::U16,
                            "i32" => MirType::I32,
                            "u32" => MirType::U32,
                            "i64" => MirType::I64,
                            "u64" => MirType::U64,
                            "f32" => MirType::F32,
                            "f64" => MirType::F64,
                            "bool" => MirType::Bool,
                            "char" => MirType::Char,
                            "str" => MirType::Str,
                            "ptr" => MirType::Ptr(Box::new(MirType::Void)),
                            name => {
                                if let Some(defs) = ctx.struct_defs.get(name) {
                                    MirType::Struct(name.to_string(), defs.clone())
                                } else {
                                    MirType::I32
                                }
                            }
                        }
                    } else {
                        MirType::I32
                    };
                    let dest = ctx.alloc_local("_cast", to_type.clone());
                    // str as ptr and &str as ptr: handled by no-op Cast
                    // (both are ptr in LLVM). &str dereference is done in prelude.
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest,
                        value: MirValue::Local(left_local),
                        to_type,
                    });
                    return ctx;
                }

                // String concatenation if either operand is a string
                if matches!(operator, BinaryOp::Add) && (left_is_str || right_is_str) {
                    // Get ptr and len for each operand
                    // Left: use the existing local (pointer)
                    // Need length for left if it's a string
                    let left_len = if left_is_str {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(d),
                            name: "ky_strlen".to_string(),
                            args: vec![MirValue::Local(left_local)],
                        });
                        d
                    } else {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: d,
                            value: MirValue::Constant(MirConstant::I32(0)),
                        });
                        d
                    };
                    let right_len = if right_is_str {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(d),
                            name: "ky_strlen".to_string(),
                            args: vec![MirValue::Local(right_local)],
                        });
                        d
                    } else {
                        let d = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: d,
                            value: MirValue::Constant(MirConstant::I32(0)),
                        });
                        d
                    };
                    let result = ctx.alloc_local("_bin", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_concat".to_string(),
                        args: vec![
                            MirValue::Local(left_local),
                            MirValue::Local(left_len),
                            MirValue::Local(right_local),
                            MirValue::Local(right_len),
                        ],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // String equality comparison: use kl_eq_str instead of pointer comparison
                if (matches!(operator, BinaryOp::Eq | BinaryOp::Neq)) && (left_is_str || right_is_str) {
                    let result = ctx.alloc_local("_bin", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_eq_str".to_string(),
                        args: vec![MirValue::Local(left_local), MirValue::Local(right_local)],
                    });
                    if matches!(operator, BinaryOp::Neq) {
                        let one = ctx.alloc_local("_one", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: one,
                            value: MirValue::Constant(MirConstant::I32(1)),
                        });
                        let neq = ctx.alloc_local("_bin", MirType::I32);
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: neq,
                            op: MirBinaryOp::Xor,
                            left: MirValue::Local(result),
                            right: MirValue::Local(one),
                        });
                        return ctx;
                    }
                    return ctx;
                }

                // String comparison (Lt/Gt/Le/Ge): use ky_str_cmp
                if (matches!(operator, BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge))
                    && (left_is_str || right_is_str) {
                    let cmp = ctx.alloc_local("_cmp", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(cmp),
                        name: "ky_str_cmp".to_string(),
                        args: vec![MirValue::Local(left_local), MirValue::Local(right_local)],
                    });
                    let zero = ctx.alloc_local("_zero", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: zero, value: MirValue::Constant(MirConstant::I32(0)),
                    });
                    let result = ctx.alloc_local("_cmpres", MirType::I32);
                    let (lt, gt) = match operator {
                        BinaryOp::Lt => (true, false),
                        BinaryOp::Gt => (false, true),
                        BinaryOp::Le => (true, true),
                        BinaryOp::Ge => (true, true),
                        _ => (false, false),
                    };
                    if lt && gt {
                        // Le/Ge: cmp <= 0 or cmp >= 0
                        let cond = if matches!(operator, BinaryOp::Le) { MirBinaryOp::Le } else { MirBinaryOp::Ge };
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: result, op: cond,
                            left: MirValue::Local(cmp), right: MirValue::Local(zero),
                        });
                    } else {
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: result, op: if lt { MirBinaryOp::Lt } else { MirBinaryOp::Gt },
                            left: MirValue::Local(cmp), right: MirValue::Local(zero),
                        });
                    }
                    return ctx;
                }

                // Operator overloading: dispatch to op_X method for struct types
                let overload_op_name = match operator {
                    BinaryOp::Add => Some("op_add"),
                    BinaryOp::Sub => Some("op_sub"),
                    BinaryOp::Mul => Some("op_mul"),
                    BinaryOp::Div => Some("op_div"),
                    BinaryOp::Rem => Some("op_mod"),
                    BinaryOp::Eq => Some("op_eq"),
                    BinaryOp::Neq => Some("op_ne"),
                    BinaryOp::Lt => Some("op_lt"),
                    BinaryOp::Gt => Some("op_gt"),
                    BinaryOp::Le => Some("op_le"),
                    BinaryOp::Ge => Some("op_ge"),
                    _ => None,
                };
                if let Some(op_name) = overload_op_name {
                    let left_type = ctx.local_types.get(&left_local).cloned().unwrap_or(MirType::I32);
                    if let MirType::Struct(class_name, _) = &left_type {
                        let method_table = self.method_table.borrow();
                        if let Some(methods) = method_table.get(class_name) {
                            if let Some(mangled) = methods.get(op_name) {
                                let ret_type = self.fn_returns.borrow()
                                    .get(mangled).cloned().unwrap_or(MirType::I64);
                                let dest = ctx.alloc_local("_op", ret_type);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: mangled.clone(),
                                    args: vec![MirValue::Local(left_local), MirValue::Local(right_local)],
                                });
                                return ctx;
                            }
                        }
                    }
                }

                // Coerce operands to the same type for binary operations.
                // Get the actual MIR types of each operand.
                let left_type = ctx.local_types.get(&left_local).cloned().unwrap_or(MirType::I32);
                let right_type = ctx.local_types.get(&right_local).cloned().unwrap_or(MirType::I32);
                let wider = wider_type(&left_type, &right_type);
                let is_float_op = matches!(wider, MirType::F32 | MirType::F64);
                let left_operand = if left_type != wider && is_int_type(&left_type) && !is_float_op {
                    let cast = ctx.alloc_local("_widen", wider.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(left_local),
                        to_type: wider.clone(),
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Local(left_local)
                };
                let right_operand = if right_type != wider && is_int_type(&right_type) && !is_float_op {
                    let cast = ctx.alloc_local("_widen", wider.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(right_local),
                        to_type: wider.clone(),
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Local(right_local)
                };

                // Comparison ops always produce I32. Arithmetic ops produce the wider type.
                let is_cmp = matches!(operator,
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt |
                    BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge);
                let result_type = if is_cmp { MirType::I32 } else { wider.clone() };
                let dest = ctx.alloc_local("_bin", result_type);

                let op = match operator {
                    BinaryOp::Add => MirBinaryOp::Add,
                    BinaryOp::Sub => MirBinaryOp::Sub,
                    BinaryOp::Mul => MirBinaryOp::Mul,
                    BinaryOp::Div => MirBinaryOp::Div,
                    BinaryOp::Rem => MirBinaryOp::Rem,
                    BinaryOp::And => MirBinaryOp::And,
                    BinaryOp::Or => MirBinaryOp::Or,
                    BinaryOp::BitAnd => MirBinaryOp::And,
                    BinaryOp::BitOr => MirBinaryOp::Or,
                    BinaryOp::BitXor => MirBinaryOp::Xor,
                    BinaryOp::Shl => MirBinaryOp::Shl,
                    BinaryOp::Shr => MirBinaryOp::Shr,
                    BinaryOp::Eq => MirBinaryOp::Eq,
                    BinaryOp::Neq => MirBinaryOp::Neq,
                    BinaryOp::Lt => MirBinaryOp::Lt,
                    BinaryOp::Gt => MirBinaryOp::Gt,
                    BinaryOp::Le => MirBinaryOp::Le,
                    BinaryOp::Ge => MirBinaryOp::Ge,
                    BinaryOp::AddPercent | BinaryOp::SubPercent | BinaryOp::MulPercent => {
                        let fn_name = match operator {
                            BinaryOp::AddPercent => "ky_add_pct",
                            BinaryOp::SubPercent => "ky_sub_pct",
                            BinaryOp::MulPercent => "ky_mul_pct",
                            _ => unreachable!(),
                        };
                        let pct_left = ctx.alloc_local("_pct_l", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pct_left,
                            value: left_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pct_right = ctx.alloc_local("_pct_r", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pct_right,
                            value: right_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pct_dest = ctx.alloc_local("_pct", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(pct_dest),
                            name: fn_name.to_string(),
                            args: vec![
                                MirValue::Local(pct_left),
                                MirValue::Local(pct_right),
                            ],
                        });
                        return ctx;
                    }
                    BinaryOp::Pow => {
                        // Power: generate call to ky_pow(i64, i64) -> i64
                        // Cast operands to i64 first
                        let pow_left = ctx.alloc_local("_pow_l", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pow_left,
                            value: left_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pow_right = ctx.alloc_local("_pow_r", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: pow_right,
                            value: right_operand.clone(),
                            to_type: MirType::I64,
                        });
                        let pow_dest = ctx.alloc_local("_pow", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(pow_dest),
                            name: "ky_pow".to_string(),
                            args: vec![
                                MirValue::Local(pow_left),
                                MirValue::Local(pow_right),
                            ],
                        });
                        return ctx;
                    }
                    _ => MirBinaryOp::Add,
                };
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest,
                    op,
                    left: left_operand,
                    right: right_operand,
                });
                ctx
            }
            Expr::Unary { operator, operand, .. } => {
                ctx = self.lower_expr(ctx, operand);
                let operand_type = ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32);
                // Operator overloading for struct types
                let overload_method_name = match operator {
                    UnaryOp::Neg => Some("op_neg"),
                    UnaryOp::Not => Some("op_not"),
                    _ => None,
                };
                if let Some(method_name) = overload_method_name {
                    if let MirType::Struct(class_name, _) = &operand_type {
                        let method_table = self.method_table.borrow();
                        if let Some(methods) = method_table.get(class_name) {
                            if let Some(mangled) = methods.get(method_name) {
                                let ret_type = self.fn_returns.borrow()
                                    .get(mangled).cloned().unwrap_or(MirType::I64);
                                let dest = ctx.alloc_local("_un", ret_type);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: mangled.clone(),
                                    args: vec![MirValue::Local(ctx.next_local - 2)],
                                });
                                return ctx;
                            }
                        }
                    }
                }
                let dest = ctx.alloc_local("_un", MirType::I32);
                let op = match operator {
                    UnaryOp::Neg => MirUnaryOp::Neg,
                    UnaryOp::Not => MirUnaryOp::Not,
                    UnaryOp::BitNot => MirUnaryOp::BitNot,
                };
                ctx.current_block.insts.push(MirInst::UnaryOp {
                    dest,
                    op,
                    operand: MirValue::Local(ctx.next_local - 2),
                });
                ctx
            }
            Expr::FunctionCall { target, arguments, type_args, .. } => {
                // Method dispatch: obj.method(args) → Call ClassName::method(obj, args...)
                // Checked BEFORE the list-specific shortcuts so real class methods win
                // over the generic list.add/list.pop fallbacks.
                if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Check for enum variant construction: Option.Some(value)
                    // This must come BEFORE method dispatch because enum type names
                    // are not variables and cannot be lowered as expressions.
                    if let Expr::Identifier { name: enum_name, .. } = object.as_ref() {
                        let ev_map = self.enum_variants.borrow();
                        if let Some(variants) = ev_map.get(enum_name) {
                            if let Some(&variant_idx) = variants.get(property) {
                                // Determine inner type name for Option-like enums
                                let _inner_name = if arguments.len() == 1 {
                                    match &arguments[0] {
                                        Expr::Identifier { name, .. } => {
                                            if let Some(&local) = ctx.locals.get(name) {
                                                if let Some(MirType::Struct(inner, _)) = ctx.local_types.get(&local) {
                                                    Some(inner.clone())
                    } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        }
                                        Expr::Literal { value: Literal::Integer(_), .. } => Some("i32".to_string()),
                                        Expr::Literal { value: Literal::String(_), .. } => Some("str".to_string()),
                                        Expr::Literal { value: Literal::Boolean(_), .. } => Some("bool".to_string()),
                                        _ => None,
                                    }
                                } else {
                                    None
                                };
                                // All enum variant types share the same tagged-union layout
                                // { disc: i32, payload: i64 } regardless of type parameters.
                                // Use the base enum name to match ast_type_to_mir resolution
                                // and function return types.
                                let concrete_name = enum_name.clone();
                                let struct_type = MirType::Struct(concrete_name.clone(), vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ]);

                                // Register concrete struct type name in struct_defs
                                ctx.struct_defs.entry(concrete_name).or_insert_with(|| vec![
                                    ("disc".to_string(), MirType::I32),
                                    ("payload".to_string(), MirType::I64),
                                ]);

                                // Allocate temps FIRST, struct LAST so the result local is the struct
                                let disc_ptr = ctx.alloc_local("_edp", MirType::I64);
                                let disc_val = MirValue::Constant(MirConstant::I32(variant_idx as i32));

                                let (pay_ptr, arg_val) = if let Some(arg) = arguments.first() {
                                    ctx = self.lower_expr(ctx, arg);
                                    let av = ctx.next_local - 1;
                                    let pp = ctx.alloc_local("_epp", MirType::I64);
                                    let ai = ctx.alloc_local("_epv", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: ai,
                                        value: MirValue::Local(av),
                                        to_type: MirType::I64,
                                    });
                                    (Some(pp), Some(ai))
                                } else {
                                    (None, None)
                                };

                                // Allocate struct LAST so the caller sees the correct result local
                                let dest = ctx.alloc_local("_enum", struct_type.clone());

                                // Store discriminant
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: disc_ptr,
                                    ptr: dest,
                                    field_index: 0,
                                    struct_type: Box::new(struct_type.clone()),
                                });
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: disc_ptr,
                                    value: disc_val,
                                });

                                // Store payload
                                if let (Some(pp), Some(av)) = (pay_ptr, arg_val) {
                                    ctx.current_block.insts.push(MirInst::FieldPtr {
                                        dest: pp,
                                        ptr: dest,
                                        field_index: 1,
                                        struct_type: Box::new(struct_type),
                                    });
                                    ctx.current_block.insts.push(MirInst::Store {
                                        dest: pp,
                                        value: MirValue::Local(av),
                                    });
                                }

                                return ctx;
                            }
                        }
                    }
                    // Static method dispatch: ClassName.method(args) — no receiver needed
                    if let Expr::Identifier { name: class_name, .. } = object.as_ref() {
                        let method_table = self.method_table.borrow();
                        if let Some(methods) = method_table.get(class_name) {
                            if let Some(mangled) = methods.get(property) {
                                if self.static_methods.borrow().contains(mangled) {
                                    let mut call_args = Vec::new();
                                    for arg in arguments {
                                        ctx = self.lower_expr(ctx, arg);
                                        call_args.push(MirValue::Local(ctx.next_local - 1));
                                    }
                                    let call_type = self.fn_returns.borrow()
                                        .get(mangled).cloned().unwrap_or(MirType::I64);
                                    let dest = ctx.alloc_local("_modcall", call_type.clone());
                                    if call_type == MirType::Str {
                                        ctx.string_locals.push(dest);
                                    }
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(dest),
                                        name: mangled.clone(),
                                        args: call_args,
                                    });
                                    return ctx;
                                }
                            }
                        }
                    }

                    // Check for module-qualified function call: module.func(args)
                    // where `module` is not a local variable and not an enum name.
                    if let Expr::Identifier { name: mod_name, .. } = object.as_ref() {
                        if !ctx.locals.contains_key(mod_name) {
                            let ev_map = self.enum_variants.borrow();
                            if !ev_map.contains_key(mod_name) {
                                // Map namespaced API to flat function name
                                let resolve_namespace = |ns: &str, func: &str| -> Option<String> {
                                    match (ns, func) {
                                        ("parallel", "for") => Some("ky_parallel_for".into()),
                                        ("thread", "spawn") => Some("ky_spawn_thread".into()),
                                        ("thread", "join") => Some("ky_join_thread".into()),
                                        ("thread", "sleep") => Some("ky_sleep".into()),
                                        ("thread", "yield") => Some("ky_yield".into()),
                                        ("assert", "is_true") => Some("assert".into()),
                                        ("assert", "eq") => Some("assert_eq".into()),
                                        ("assert", "ne") => Some("assert_ne".into()),
                                        ("assert", "str_eq") => Some("assert_str".into()),
                                        ("math", "pow") => Some("ky_pow".into()),
                                        ("math", "ceil") => Some("ceil".into()),
                                        ("math", "floor") => Some("floor".into()),
                                        ("math", "round") => Some("round".into()),
                                        ("json", "parse") => Some("json_parse".into()),
                                        ("json", "stringify") => Some("json_stringify".into()),
                                        ("json", "serialize") => Some("serialize".into()),
                                        ("json", "deserialize") => Some("deserialize".into()),
                                        ("crypto", "sha1") => Some("ky_sha1".into()),
                                        ("crypto", "base64_encode") => Some("ky_base64_encode".into()),
                                        ("process", "env") => Some("ky_getenv".into()),
                                        ("tcp", "listen") => Some("ky_tcp_listen".into()),
                                        ("tcp", "accept") => Some("ky_tcp_accept".into()),
                                        ("tcp", "read") => Some("ky_tcp_read".into()),
                                        ("tcp", "write") => Some("ky_tcp_write".into()),
                                        ("tcp", "close") => Some("ky_tcp_close".into()),
                                        ("dict", "contains") => Some("ky_dict_contains".into()),
                                        ("dict", "remove") => Some("ky_dict_remove".into()),
                                        ("str_builder", "new") => Some("ky_str_builder_new".into()),
                                        ("str_builder", "append") => Some("ky_str_builder_append".into()),
                                        ("str_builder", "to_str") => Some("ky_str_builder_to_str".into()),
                                        ("str_builder", "free") => Some("ky_str_builder_free".into()),
                                        ("fs", "exists") => Some("ky_fs_exists".into()),
                                        ("fs", "is_dir") => Some("ky_fs_is_dir".into()),
                                        ("fs", "is_file") => Some("ky_fs_is_file".into()),
                                        ("fs", "size") => Some("ky_fs_size".into()),
                                        ("fs", "copy") => Some("ky_fs_copy".into()),
                                        ("fs", "remove") => Some("ky_fs_remove".into()),
                                        ("fs", "create_dir") => Some("ky_fs_create_dir".into()),
                                        ("fs", "remove_dir") => Some("ky_fs_remove_dir".into()),
                                        ("fs", "rename") => Some("ky_fs_rename".into()),
                                        ("fs", "read_to_string") => Some("ky_fs_read_to_string".into()),
                                        ("fs", "write_string") => Some("ky_fs_write_string".into()),
                                        ("fs", "list_dir") => Some("ky_fs_list_dir".into()),
                                        ("set", "new") => Some("ky_set_new".into()),
                                        ("set", "free") => Some("ky_set_free".into()),
                                        ("set", "add") => Some("ky_set_add".into()),
                                        ("set", "contains") => Some("ky_set_contains".into()),
                                        ("set", "remove") => Some("ky_set_remove".into()),
                                        ("set", "len") => Some("ky_set_len".into()),
                                        ("file", "open") => Some("ky_open".into()),
                                        ("file", "read") => Some("ky_read_str".into()),
                                        ("file", "write") => Some("ky_write_str".into()),
                                        ("file", "close") => Some("ky_close".into()),
                                        ("time", "now_ms") => Some("ky_time_now_ms".into()),
                                        ("time", "now_us") => Some("ky_time_now_us".into()),
                                        _ => None,
                                    }
                                };
                                let mut fn_name = resolve_namespace(mod_name, property)
                                    .unwrap_or_else(|| property.clone());
                                let mut args = Vec::new();
                                for arg in arguments {
                                    if let Expr::Identifier { name, .. } = arg {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            if let Some(t) = ctx.local_types.get(&local) {
                                                if matches!(t, MirType::Struct(_, _)) || is_move_type(t) {
                                                    args.push(MirValue::Local(local));
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    ctx = self.lower_expr(ctx, arg);
                                    args.push(MirValue::Local(ctx.next_local - 1));
                                }
                                // Special handling for json.deserialize<T>(str)
                                if fn_name == "deserialize" && args.len() == 1 {
                                    if let Some(first_type_arg) = type_args.first() {
                                        let struct_defs = ctx.struct_defs.clone();
                                        let mir_type = ast_type_to_mir(first_type_arg, Some(&struct_defs));
                                        if let MirType::Struct(_, fields) = &mir_type {
                                            let descriptor = build_descriptor(fields);
                                            let json_arg = args.remove(0);
                                            let out_local = ctx.alloc_local("_dout", mir_type.clone());
                                            args.push(json_arg);
                                            args.push(MirValue::Constant(MirConstant::String(descriptor)));
                                            args.push(MirValue::Local(out_local));
                                            let ret_local = ctx.alloc_local("_dret", MirType::I32);
                                            ctx.current_block.insts.push(MirInst::Call {
                                                dest: Some(ret_local),
                                                name: "ky_json_to_struct".to_string(),
                                                args,
                                            });
                                            let load = ctx.alloc_local("_dval", mir_type.clone());
                                            ctx.current_block.insts.push(MirInst::Load {
                                                dest: load,
                                                src: out_local,
                                            });
                                            return ctx;
                                        }
                                    }
                                    // Without type args, fall back to json_parse (returns Dict<str,i64>)
                                    fn_name = "json_parse".to_string();
                                }
                                let call_type = builtin_return_type(&fn_name)
                                    .or_else(|| self.fn_returns.borrow().get(&fn_name).cloned())
                                    .unwrap_or(MirType::Void);
                                let dest = ctx.alloc_local("_modcall", call_type.clone());
                                if call_type == MirType::Str {
                                    ctx.string_locals.push(dest);
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: fn_name,
                                    args,
                                });
                                return ctx;
                            }
                        }
                    }

                    // Lower the receiver first, but for struct identifiers pass by reference (no copy).
                    // ^&T ref params are excluded: they must go through lower_expr to dereference.
                    let obj_local = if let Expr::Identifier { name, .. } = object.as_ref() {
                        if let Some(&local) = ctx.locals.get(name) {
                            let lt = ctx.local_types.get(&local);
                            if matches!(lt, Some(MirType::Struct(_, _)) | Some(MirType::Ptr(_)) | Some(MirType::Slice(_)))
                                && !ctx.ref_param_locals.contains(&local)
                            {
                                local
                            } else {
                                ctx = self.lower_expr(ctx, object);
                                ctx.next_local - 1
                            }
                        } else {
                            ctx = self.lower_expr(ctx, object);
                            ctx.next_local - 1
                        }
                    } else {
                        ctx = self.lower_expr(ctx, object);
                        ctx.next_local - 1
                    };
                    let obj_type = ctx.local_types.get(&obj_local).cloned();

                    // If the receiver is a class instance (MirType::Struct) and the class
                    // declares a method named `property`, emit a real method call.
                    // Also unwrap Ptr(Struct(...)) for closure-inferred types.
                    let struct_type = obj_type.as_ref().and_then(|t| {
                        if let MirType::Struct(name, _) = t { Some(name.clone()) }
                        else if let MirType::Ptr(inner) = t {
                            if let MirType::Struct(name, _) = inner.as_ref() { Some(name.clone()) }
                            else { None }
                        } else { None }
                    });
                    if let Some(ref class_name) = struct_type {
                        // For concrete generic types (e.g. "Box__i32"), ensure methods are monomorphized
                        if class_name.contains("__") {
                            self.ensure_generic_class_methods(class_name);
                        }
                        let method_table = self.method_table.borrow();
                        let parent_map = self.class_parent_map.borrow();
                        if let Some(mangled) = self.lookup_method_in_chain(class_name, property, &method_table, &parent_map) {
                                // Static methods don't get `this` as first arg
                                let is_static = self.static_methods.borrow().contains(&mangled);
                                let mut call_args = if is_static {
                                    Vec::new()
                                } else {
                                    // Class methods expect `this` as a POINTER to the struct.
                                    // If obj_local is a struct value, pass its address instead.
                                    if matches!(ctx.local_types.get(&obj_local), Some(MirType::Struct(_, _))) {
                                        let ptr = ctx.alloc_local("_thisptr", MirType::Ptr(Box::new(MirType::Void)));
                                        ctx.current_block.insts.push(MirInst::AddressOf {
                                            dest: ptr,
                                            local_id: obj_local,
                                        });
                                        vec![MirValue::Local(ptr)]
                                    } else {
                                        vec![MirValue::Local(obj_local)]
                                    }
                                };
                                for arg in arguments {
                                    // Struct-typed or Move-typed identifiers: pass original local
                                    if let Expr::Identifier { name, .. } = arg {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            if let Some(t) = ctx.local_types.get(&local) {
                                                if matches!(t, MirType::Struct(_, _)) || is_move_type(t) {
                                                    call_args.push(MirValue::Local(local));
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                    ctx = self.lower_expr(ctx, arg);
                                    call_args.push(MirValue::Local(ctx.next_local - 1));
                                }
                                let call_type = self.fn_returns.borrow()
                                    .get(&mangled).cloned().unwrap_or(MirType::Void);
                                let dest = ctx.alloc_local("_mcall", call_type.clone());
                                if call_type == MirType::Str {
                                    ctx.string_locals.push(dest);
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(dest),
                                    name: mangled.clone(),
                                    args: call_args,
                                });
                                return ctx;
                            }
                    }

                    // Built-in type method dispatch (str, list, array, dict, char)
                    let is_str = obj_type.as_ref().map(|t| *t == MirType::Str).unwrap_or(false);
                    let is_list = obj_type.as_ref().map(|t| matches!(t, MirType::List(_))).unwrap_or(false);
                    let list_elem_type = if let Some(MirType::List(inner)) = obj_type.as_ref() { inner.as_ref().clone() } else { MirType::I64 };
                    let is_array = obj_type.as_ref().map(|t| matches!(t, MirType::Array(_, _))).unwrap_or(false);
                    let is_char = obj_type.as_ref().map(|t| *t == MirType::Char).unwrap_or(false);
                    let is_slice = obj_type.as_ref().map(|t| matches!(t, MirType::Slice(_))).unwrap_or(false);

                    // === UNIVERSAL METHOD: .type() on any value ===
                    if property == "type" && arguments.is_empty() {
                        if let Some(ref obj_t) = obj_type {
                            build_typeinfo_struct(obj_t, &mut ctx);
                            return ctx;
                        }
                    }

                    // === ARRAY METHODS ===
                    if is_array && property == "len" && arguments.is_empty() {
                        // Array length is compile-time constant (N in [T, N])
                        // Return as I32 to match loop variable type in for-range
                        if let MirType::Array(_, size) = obj_type.as_ref().unwrap() {
                            let result = ctx.alloc_local("_arrlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result,
                                value: MirValue::Constant(MirConstant::I32(*size as i32)),
                            });
                            return ctx;
                        }
                    }

                    // === SLICE METHODS ===
                    if is_slice && property == "len" && arguments.is_empty() {
                        // Slice length is stored in field 1 of the slice struct (i64)
                        if let Some(MirType::Slice(inner)) = obj_type.as_ref() {
                            let slice_type = MirType::Slice(inner.clone());
                            let len_ptr = ctx.alloc_local("_slenp", MirType::I64);
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: len_ptr,
                                ptr: obj_local,
                                field_index: 1,
                                struct_type: Box::new(slice_type.clone()),
                            });
                            let len_i64 = ctx.alloc_local("_slen64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: len_i64,
                                src: len_ptr,
                            });
                            let result = ctx.alloc_local("_slen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: result,
                                value: MirValue::Local(len_i64),
                                to_type: MirType::I32,
                            });
                            return ctx;
                        }
                    }

                    // === STRING METHODS ===
                    if is_str && property == "len" && arguments.is_empty() {
                        let result = ctx.alloc_local("_strlen", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_strlen".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && (property == "to_upper" || property == "upper") && arguments.is_empty() {
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_to_upper".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && (property == "to_lower" || property == "lower") && arguments.is_empty() {
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_to_lower".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && property == "trim" && arguments.is_empty() {
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_trim".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_str && property == "contains" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let arg = ctx.next_local - 1;
                        let result = ctx.alloc_local("_c", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_contains".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(arg)],
                        });
                        return ctx;
                    }
                    if is_str && property == "starts_with" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let arg = ctx.next_local - 1;
                        let result = ctx.alloc_local("_sw", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_starts_with".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(arg)],
                        });
                        return ctx;
                    }
                    if is_str && property == "ends_with" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let arg = ctx.next_local - 1;
                        let result = ctx.alloc_local("_ew", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_ends_with".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(arg)],
                        });
                        return ctx;
                    }
                    if is_str && property == "replace" && arguments.len() == 2 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let from = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, &arguments[1]);
                        let to = ctx.next_local - 1;
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_replace".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(from), MirValue::Local(to)],
                        });
                        return ctx;
                    }
                    if is_str && property == "substr" && arguments.len() == 2 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let start = ctx.next_local - 1;
                        ctx = self.lower_expr(ctx, &arguments[1]);
                        let count = ctx.next_local - 1;
                        let result = ctx.alloc_local("_s", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_substr".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(start), MirValue::Local(count)],
                        });
                        return ctx;
                    }
                    if is_str && property == "char_at" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let idx = ctx.next_local - 1;
                        let result = ctx.alloc_local("_c", MirType::Char);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_char_at".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(idx)],
                        });
                        return ctx;
                    }
                    if is_str && (property == "index_of" || property == "find") && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let substr = ctx.next_local - 1;
                        let result = ctx.alloc_local("_sio", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_index_of".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(substr)],
                        });
                        return ctx;
                    }
                    if is_str && property == "split" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let delim = ctx.next_local - 1;
                        let result = ctx.alloc_local("_ssp", MirType::List(Box::new(MirType::Str)));
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_split".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(delim)],
                        });
                        return ctx;
                    }
                    if is_str && property == "to_list" && arguments.is_empty() {
                        let result = ctx.alloc_local("_stl", MirType::List(Box::new(MirType::Str)));
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_str_to_list".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    // String is_* methods: get first char, then call the char function
                    if is_str && arguments.is_empty() {
                        let is_fn = match property.as_str() {
                            "is_digit" => Some("ky_is_digit"),
                            "is_alpha" => Some("ky_is_alpha"),
                            "is_alnum" => Some("ky_is_alnum"),
                            "is_whitespace" => Some("ky_is_whitespace"),
                            "is_upper" => Some("ky_is_upper"),
                            "is_lower" => Some("ky_is_lower"),
                            _ => None,
                        };
                        if let Some(fn_name) = is_fn {
                            // Get first character code
                            let zero = ctx.alloc_local("_iz", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: zero, value: MirValue::Constant(MirConstant::I32(0)),
                            });
                            let first_char = ctx.alloc_local("_ifc", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(first_char), name: "ky_char_at".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(zero)],
                            });
                            let result = ctx.alloc_local("_ir", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: fn_name.to_string(),
                                args: vec![MirValue::Local(first_char)],
                            });
                            return ctx;
                        }
                    }

                    // === CHAR METHODS ===
                    if is_char && property == "ord" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ord", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_ord".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_digit" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cd", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_digit".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_alpha" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ca", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_alpha".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_alnum" && arguments.is_empty() {
                        let result = ctx.alloc_local("_can", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_alnum".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_whitespace" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cw", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_whitespace".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_upper" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cu", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_upper".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    if is_char && property == "is_lower" && arguments.is_empty() {
                        let result = ctx.alloc_local("_cl", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_is_lower".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }

                    // === UNIVERSAL CONVERSION METHODS (.to_str, .to_int, etc.) ===
                    if property == "to_str" && arguments.is_empty() {
                        let id_type = ctx.local_types.get(&obj_local).cloned().unwrap_or(MirType::I32);
                        if id_type == MirType::Str {
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_clone_str".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        } else if id_type == MirType::Char {
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_char_to_str".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        } else if matches!(id_type, MirType::F32 | MirType::F64) {
                            // F32/F64 → F64 → ky_f64_to_str(str)
                            let f64_val = if id_type == MirType::F32 {
                                let c = ctx.alloc_local("_f2f", MirType::F64);
                                ctx.current_block.insts.push(MirInst::Cast { dest: c, value: MirValue::Local(obj_local), to_type: MirType::F64 });
                                MirValue::Local(c)
                            } else { MirValue::Local(obj_local) };
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_f64_to_str".to_string(),
                                args: vec![f64_val],
                            });
                            return ctx;
                        } else if let MirType::Struct(ref struct_name, _) = id_type {
                            if struct_name.starts_with("Option__") {
                                let inner = extract_inner_type(struct_name);
                                let payload_ptr = ctx.alloc_local("_opp", MirType::Ptr(Box::new(MirType::I64)));
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: payload_ptr, ptr: obj_local, field_index: 1,
                                    struct_type: Box::new(id_type),
                                });
                                let payload = ctx.alloc_local("_op", MirType::I64);
                                ctx.current_block.insts.push(MirInst::Load { dest: payload, src: payload_ptr });
                                if inner == MirType::Str {
                                    let result = ctx.alloc_local("_ts", MirType::Str);
                                    ctx.string_locals.push(result);
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(result), name: "ky_clone_str".to_string(),
                                        args: vec![MirValue::Local(payload)],
                                    });
                                } else if matches!(inner, MirType::F32 | MirType::F64) {
                                    let f64_val = ctx.alloc_local("_opf", MirType::F64);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: f64_val, value: MirValue::Local(payload), to_type: MirType::F64,
                                    });
                                    let result = ctx.alloc_local("_ts", MirType::Str);
                                    ctx.string_locals.push(result);
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(result), name: "ky_f64_to_str".to_string(),
                                        args: vec![MirValue::Local(f64_val)],
                                    });
                                } else if inner == MirType::Char {
                                    let c_val = ctx.alloc_local("_opc", MirType::Char);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: c_val, value: MirValue::Local(payload), to_type: MirType::Char,
                                    });
                                    let result = ctx.alloc_local("_ts", MirType::Str);
                                    ctx.string_locals.push(result);
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(result), name: "ky_char_to_str".to_string(),
                                        args: vec![MirValue::Local(c_val)],
                                    });
                                } else {
                                    let result = ctx.alloc_local("_ts", MirType::Str);
                                    ctx.string_locals.push(result);
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(result), name: "ky_i64_to_str".to_string(),
                                        args: vec![MirValue::Local(payload)],
                                    });
                                }
                                return ctx;
                            }
                            let i64_val = MirValue::Local(obj_local);
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_i64_to_str".to_string(),
                                args: vec![i64_val],
                            });
                            return ctx;
                        } else {
                            let i64_val = if id_type == MirType::I32 {
                                let ext = ctx.alloc_local("_i64v", MirType::I64);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: ext,
                                    value: MirValue::Local(obj_local),
                                    to_type: MirType::I64,
                                });
                                MirValue::Local(ext)
                            } else {
                                MirValue::Local(obj_local)
                            };
                            let result = ctx.alloc_local("_ts", MirType::Str);
                            ctx.string_locals.push(result);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_i64_to_str".to_string(),
                                args: vec![i64_val],
                            });
                            return ctx;
                        }
                    }
                    // Type-specific conversion methods
                    if property == "to_i32" && arguments.is_empty() {
                        let id_type = ctx.local_types.get(&obj_local).cloned().unwrap_or(MirType::I32);
                        let result = ctx.alloc_local("_ti32", MirType::I32);
                        if id_type == MirType::Str {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_str_to_i32".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                        } else {
                            ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I32 });
                        }
                        return ctx;
                    }
                    if property == "to_i64" && arguments.is_empty() {
                        let id_type = ctx.local_types.get(&obj_local).cloned().unwrap_or(MirType::I64);
                        let result = ctx.alloc_local("_ti64", MirType::I64);
                        if id_type == MirType::Str {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_str_to_i64".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                        } else {
                            ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I64 });
                        }
                        return ctx;
                    }
                    if property == "to_i16" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ti16", MirType::I16);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I16 });
                        return ctx;
                    }
                    if property == "to_i8" && arguments.is_empty() {
                        let result = ctx.alloc_local("_ti8", MirType::I8);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I8 });
                        return ctx;
                    }
                    if property == "to_u32" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu32", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I32 });
                        return ctx;
                    }
                    if property == "to_u64" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I64 });
                        return ctx;
                    }
                    if property == "to_u16" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu16", MirType::I16);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I16 });
                        return ctx;
                    }
                    if property == "to_u8" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tu8", MirType::I8);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::I8 });
                        return ctx;
                    }
                    if property == "to_f64" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tf64", MirType::F64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::F64 });
                        return ctx;
                    }
                    if property == "to_f32" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tf32", MirType::F32);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::F32 });
                        return ctx;
                    }
                    if property == "to_char" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tch", MirType::Char);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::Char });
                        return ctx;
                    }
                    // Option<T> methods: is_some, is_none, unwrap
                    if (property == "is_some" || property == "is_none") && arguments.is_empty() {
                        // Option is stored as {disc: i32, payload: i64}
                        let struct_type = MirType::Struct("__option".to_string(), vec![
                            ("disc".to_string(), MirType::I32),
                            ("payload".to_string(), MirType::I64),
                        ]);
                        let disc_ptr = ctx.alloc_local("_odp", MirType::Ptr(Box::new(MirType::I32)));
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: disc_ptr, ptr: obj_local, field_index: 0,
                            struct_type: Box::new(struct_type),
                        });
                        let disc = ctx.alloc_local("_od", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Load { dest: disc, src: disc_ptr });
                        let zero = ctx.alloc_local("_oz", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Store { dest: zero, value: MirValue::Constant(MirConstant::I32(0)) });
                        let eq = ctx.alloc_local("_oe", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: eq, op: if property == "is_some" { MirBinaryOp::Neq } else { MirBinaryOp::Eq },
                            left: MirValue::Local(disc), right: MirValue::Local(zero),
                        });
                        return ctx;
                    }
                    if property == "unwrap" && arguments.is_empty() {
                        let obj_type = ctx.local_types.get(&obj_local).cloned().unwrap_or(MirType::I64);
                        let struct_type = if let MirType::Struct(_, _) = &obj_type { obj_type.clone() } else {
                            MirType::Struct("__option".to_string(), vec![
                                ("disc".to_string(), MirType::I32),
                                ("payload".to_string(), MirType::I64),
                            ])
                        };
                        let payload_ptr = ctx.alloc_local("_opp", MirType::Ptr(Box::new(MirType::I64)));
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: payload_ptr, ptr: obj_local, field_index: 1,
                            struct_type: Box::new(struct_type.clone()),
                        });
                        let payload = ctx.alloc_local("_op", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Load { dest: payload, src: payload_ptr });
                        // Cast payload from I64 to inner type if needed
                        let inner_type = if let MirType::Struct(name, _) = &struct_type {
                            extract_inner_type(name)
                        } else { MirType::I64 };
                        if inner_type != MirType::I64 {
                            let casted = ctx.alloc_local("_opcast", inner_type.clone());
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: casted, value: MirValue::Local(payload), to_type: inner_type,
                            });
                        }
                        return ctx;
                    }
                    if property == "unwrap_or" && arguments.len() == 1 {
                        let obj_type = ctx.local_types.get(&obj_local).cloned().unwrap_or(MirType::I64);
                        let struct_type = if let MirType::Struct(_, _) = &obj_type { obj_type.clone() } else {
                            MirType::Struct("__option".to_string(), vec![
                                ("disc".to_string(), MirType::I32),
                                ("payload".to_string(), MirType::I64),
                            ])
                        };
                        let inner_type = if let MirType::Struct(name, _) = &struct_type {
                            extract_inner_type(name)
                        } else { MirType::I64 };
                        let inner_type2 = inner_type.clone();
                        let result_local = ctx.alloc_local("_uores", inner_type.clone());
                        let disc_ptr = ctx.alloc_local("_uodp", MirType::I32);
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: disc_ptr, ptr: obj_local, field_index: 0,
                            struct_type: Box::new(struct_type.clone()),
                        });
                        let disc_val = ctx.alloc_local("_uodv", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Load { dest: disc_val, src: disc_ptr });
                        let some_block = ctx.fresh_block();
                        let none_block = ctx.fresh_block();
                        let merge_block = ctx.fresh_block();
                        let is_some = ctx.alloc_local("_uois", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: is_some, op: MirBinaryOp::Neq,
                            left: MirValue::Local(disc_val), right: MirValue::Constant(MirConstant::I32(0)),
                        });
                        ctx.finish_block(MirTerminator::CondBr {
                            cond: MirValue::Local(is_some),
                            true_block: some_block.clone(), false_block: none_block.clone(),
                        });
                        // Some block: load payload, cast to inner type
                        ctx.current_block = MirBasicBlock::new(some_block);
                        let payload_ptr = ctx.alloc_local("_uopp", MirType::I64);
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: payload_ptr, ptr: obj_local, field_index: 1,
                            struct_type: Box::new(struct_type),
                        });
                        let payload_val = ctx.alloc_local("_uopv", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Load { dest: payload_val, src: payload_ptr });
                        let some_val = if inner_type2 != MirType::I64 {
                            let casted = ctx.alloc_local("_uoc", inner_type2.clone());
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: casted, value: MirValue::Local(payload_val), to_type: inner_type2.clone(),
                            });
                            casted
                        } else { payload_val };
                        ctx.current_block.insts.push(MirInst::Store { dest: result_local, value: MirValue::Local(some_val) });
                        ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                        // None block: evaluate default argument
                        ctx.current_block = MirBasicBlock::new(none_block);
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let default_val = ctx.next_local - 1;
                        let default_casted = if inner_type2 != MirType::I64 {
                            let casted = ctx.alloc_local("_uodc", inner_type.clone());
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: casted, value: MirValue::Local(default_val), to_type: inner_type,
                            });
                            casted
                        } else { default_val };
                        ctx.current_block.insts.push(MirInst::Store { dest: result_local, value: MirValue::Local(default_casted) });
                        ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                        // Merge block
                        ctx.current_block = MirBasicBlock::new(merge_block);
                        return ctx;
                    }
                    if property == "to_bool" && arguments.is_empty() {
                        let result = ctx.alloc_local("_tb", MirType::Bool);
                        ctx.current_block.insts.push(MirInst::Cast { dest: result, value: MirValue::Local(obj_local), to_type: MirType::Bool });
                        return ctx;
                    }
                    if property == "to_decimal" && arguments.is_empty() {
                        let i64_val = ctx.alloc_local("_td", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast { dest: i64_val, value: MirValue::Local(obj_local), to_type: MirType::I64 });
                        let result = ctx.alloc_local("_tds", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_decimal_to_str".to_string(),
                            args: vec![MirValue::Local(i64_val)],
                        });
                        return ctx;
                    }
                    if property == "stringify" && arguments.is_empty() {
                        // Convert to JSON: calls json_stringify on the dict/struct
                        let result = ctx.alloc_local("_sj", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_json_stringify".to_string(),
                            args: vec![MirValue::Local(obj_local)],
                        });
                        return ctx;
                    }
                    // === STR GET (char_at via ky_substr) ===
                    if is_str && property == "get" && arguments.len() == 1 {
                        ctx = self.lower_expr(ctx, &arguments[0]);
                        let idx = ctx.next_local - 1;
                        let idx_i64 = ctx.alloc_local("_sg64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                        });
                        let one_i64 = ctx.alloc_local("_one64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: one_i64, value: MirValue::Constant(MirConstant::I64(1)),
                        });
                        let result = ctx.alloc_local("_sg", MirType::Str);
                        ctx.string_locals.push(result);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result), name: "ky_substr".to_string(),
                            args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64), MirValue::Local(one_i64)],
                        });
                        return ctx;
                    }
                    // === LIST METHODS ===
                    if is_list {
                        if property == "pop" {
                            let result = ctx.alloc_local("_pop", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_list_pop".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            if list_elem_type == MirType::Str {
                                let str_res = ctx.alloc_local("_popstr", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: str_res, value: MirValue::Local(result), to_type: MirType::Str,
                                });
                                ctx.string_locals.push(str_res);
                            } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                                let struct_res = ctx.alloc_local("_popst", list_elem_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: struct_res, value: MirValue::Local(result), to_type: list_elem_type,
                                });
                            }
                            return ctx;
                        }
                        if property == "reserve" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let cap_val = ctx.next_local - 1;
                            let cap_i64 = ctx.alloc_local("_cap64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: cap_i64,
                                value: MirValue::Local(cap_val),
                                to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None,
                                name: "ky_list_reserve".to_string(),
                                args: vec![
                                    MirValue::Local(obj_local),
                                    MirValue::Local(cap_i64),
                                ],
                            });
                            return ctx;
                        }
                        if property == "push" || property == "add" {
                            for arg in arguments {
                                ctx = self.lower_expr(ctx, arg);
                                let arg_val = ctx.next_local - 1;
                                let arg_type = ctx.local_types.get(&arg_val).cloned().unwrap_or(MirType::I32);
                                let arg_i64 = ctx.alloc_local("_arg64", MirType::I64);
                                if matches!(arg_type, MirType::Struct(_, _)) {
                                    // When adding a struct to a list whose elem type is I64 (inferred),
                                    // update the list's ELEMENT type so that list[i] returns the struct.
                                    // We need to update the alloca's type, not the loaded value's type.
                                    if let Expr::Identifier { name, .. } = object.as_ref() {
                                        if let Some(var_local) = ctx.locals.get(name) {
                                            if let Some(MirType::List(elem)) = ctx.local_types.get(var_local) {
                                                if matches!(elem.as_ref(), MirType::I64) {
                                                    let struct_type = arg_type.clone();
                                                    ctx.local_types.insert(*var_local, MirType::List(Box::new(struct_type)));
                                                }
                                            }
                                        }
                                    }
                                    // Allocate heap memory for struct, store value there, push pointer
                                    let heap_ptr = ctx.alloc_local("_heapp", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Call {
                                        dest: Some(heap_ptr),
                                        name: "ky_alloc".to_string(),
                                        args: vec![MirValue::Constant(MirConstant::I64(64))],
                                    });
                                    // Copy struct value to heap memory
                                    ctx.current_block.insts.push(MirInst::Memcpy {
                                        dest_ptr_local: heap_ptr,
                                        src_alloca_local: arg_val,
                                        struct_type: Box::new(arg_type.clone()),
                                    });
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: arg_i64,
                                        value: MirValue::Local(heap_ptr),
                                        to_type: MirType::I64,
                                    });
                                } else {
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: arg_i64,
                                        value: MirValue::Local(arg_val),
                                        to_type: MirType::I64,
                                    });
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: None,
                                    name: "ky_list_push".to_string(),
                                    args: vec![MirValue::Local(obj_local), MirValue::Local(arg_i64)],
                                });
                            }
                            return ctx;
                        }
                        if is_list && property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_listlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "pop_first" && arguments.is_empty() {
                            let result = ctx.alloc_local("_popf", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_pop_first".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            if list_elem_type == MirType::Str {
                                let str_res = ctx.alloc_local("_popfstr", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: str_res, value: MirValue::Local(result), to_type: MirType::Str,
                                });
                                ctx.string_locals.push(str_res);
                            } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                                let struct_res = ctx.alloc_local("_popfst", list_elem_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: struct_res, value: MirValue::Local(result), to_type: list_elem_type,
                                });
                            }
                            return ctx;
                        }
                        if is_list && property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "contains" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let arg = ctx.next_local - 1;
                            let arg_i64 = ctx.alloc_local("_c64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: arg_i64, value: MirValue::Local(arg), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_cres", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_contains".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(arg_i64)],
                            });
                            return ctx;
                        }
                        if is_list && property == "insert" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_i64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_v64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_insert".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if is_list && property == "remove_at" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_i64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_rvat", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_remove_at".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64)],
                            });
                            return ctx;
                        }
                        // Remove by VALUE (find first occurrence, remove it, return 0/1)
                        if is_list && property == "remove" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_rv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_rres", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_remove_value".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        // === GET / SET (direct element access) ===
                        if is_list && property == "get" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_gi64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_get", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_get".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64)],
                            });
                            if list_elem_type == MirType::Str {
                                let str_res = ctx.alloc_local("_getstr", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: str_res, value: MirValue::Local(result), to_type: MirType::Str,
                                });
                                ctx.string_locals.push(str_res);
                            } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                                let struct_res = ctx.alloc_local("_getst", list_elem_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: struct_res, value: MirValue::Local(result), to_type: list_elem_type,
                                });
                            }
                            return ctx;
                        }
                        if is_list && property == "first" && arguments.is_empty() {
                            let zero = ctx.alloc_local("_fz", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: zero, value: MirValue::Constant(MirConstant::I64(0)),
                            });
                            let result = ctx.alloc_local("_first", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_get".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(zero)],
                            });
                            if list_elem_type == MirType::Str {
                                let str_res = ctx.alloc_local("_firststr", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: str_res, value: MirValue::Local(result), to_type: MirType::Str,
                                });
                                ctx.string_locals.push(str_res);
                            } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                                let struct_res = ctx.alloc_local("_firstst", list_elem_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: struct_res, value: MirValue::Local(result), to_type: list_elem_type,
                                });
                            }
                            return ctx;
                        }
                        if is_list && property == "last" && arguments.is_empty() {
                            let len = ctx.alloc_local("_ll", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(len), name: "ky_list_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            let one = ctx.alloc_local("_lo", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: one, value: MirValue::Constant(MirConstant::I64(1)),
                            });
                            let idx = ctx.alloc_local("_li", MirType::I64);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: idx, op: MirBinaryOp::Sub, left: MirValue::Local(len), right: MirValue::Local(one),
                            });
                            let result = ctx.alloc_local("_last", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_get".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx)],
                            });
                            if list_elem_type == MirType::Str {
                                let str_res = ctx.alloc_local("_laststr", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: str_res, value: MirValue::Local(result), to_type: MirType::Str,
                                });
                                ctx.string_locals.push(str_res);
                            } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                                let struct_res = ctx.alloc_local("_lastst", list_elem_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: struct_res, value: MirValue::Local(result), to_type: list_elem_type,
                                });
                            }
                            return ctx;
                        }
                        if is_list && property == "set" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let idx = ctx.next_local - 1;
                            let idx_i64 = ctx.alloc_local("_si64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: idx_i64, value: MirValue::Local(idx), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_sv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_set".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(idx_i64), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        // === AGGREGATE METHODS (sum, product, max, min) ===
                        if is_list && property == "sum" && arguments.is_empty() {
                            let result = ctx.alloc_local("_sum", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_sum".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "product" && arguments.is_empty() {
                            let result = ctx.alloc_local("_prod", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_product".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "max" && arguments.is_empty() {
                            let result = ctx.alloc_local("_max", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_max".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "min" && arguments.is_empty() {
                            let result = ctx.alloc_local("_min", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_min".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "reverse" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_reverse".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "index_of" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_io64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_io", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_index_of".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if is_list && property == "sort" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_list_sort".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if is_list && property == "chunk" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let size = ctx.next_local - 1;
                            let size_i64 = ctx.alloc_local("_ch64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: size_i64, value: MirValue::Local(size), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_chunk", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_chunk".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(size_i64)],
                            });
                            return ctx;
                        }
                        // === LAZY ITERATOR ===
                        if is_list && property == "iter" && arguments.is_empty() {
                            let list_i64 = ctx.alloc_local("_li64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: list_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_iter", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_new".to_string(),
                                args: vec![MirValue::Local(list_i64)],
                            });
                            return ctx;
                        }
                        // === HIGHER-ORDER METHODS (map, filter, fold, reduce) ===
                        if is_list && property == "map" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_mapres", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_map".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if is_list && property == "filter" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_filres", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_filter".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if is_list && property == "fold" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let init_val = ctx.next_local - 1;
                            let init_i64 = ctx.alloc_local("_fldi", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: init_i64, value: MirValue::Local(init_val), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_fldres", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_fold".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(init_i64), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if is_list && property == "reduce" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_rdcres", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_list_reduce".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                    }

                    // === ITERATOR METHODS (lazy iteration API) ===
                    let is_iter = obj_type.as_ref().map(|t| matches!(t, MirType::Ptr(_))).unwrap_or(false);
                    if is_iter {
                        if property == "next" && arguments.is_empty() {
                            let iter_i64 = ctx.alloc_local("_it64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_next", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_next".to_string(),
                                args: vec![MirValue::Local(iter_i64)],
                            });
                            return ctx;
                        }
                        if property == "map" && arguments.len() == 1 {
                            let iter_i64 = ctx.alloc_local("_im64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_itermap", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_map".to_string(),
                                args: vec![MirValue::Local(iter_i64), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if property == "filter" && arguments.len() == 1 {
                            let iter_i64 = ctx.alloc_local("_if64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let fn_ptr = ctx.next_local - 1;
                            let result = ctx.alloc_local("_iterfil", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_filter".to_string(),
                                args: vec![MirValue::Local(iter_i64), MirValue::Local(fn_ptr)],
                            });
                            return ctx;
                        }
                        if property == "collect" && arguments.is_empty() {
                            let iter_i64 = ctx.alloc_local("_ic64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: iter_i64, value: MirValue::Local(obj_local), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_icol", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_iter_collect".to_string(),
                                args: vec![MirValue::Local(iter_i64)],
                            });
                            return ctx;
                        }
                    }

                    // Dict method shortcuts (len, set, get, has, clear, remove)
                    let is_set = obj_type.as_ref().map(|t| matches!(t, MirType::Set(_))).unwrap_or(false);
                    let is_queue = obj_type.as_ref().map(|t| matches!(t, MirType::Queue(_))).unwrap_or(false);
                    let is_stack = obj_type.as_ref().map(|t| matches!(t, MirType::Stack(_))).unwrap_or(false);
                    let is_deque = obj_type.as_ref().map(|t| matches!(t, MirType::Deque(_))).unwrap_or(false);
                    let is_llist = obj_type.as_ref().map(|t| matches!(t, MirType::LinkedList(_))).unwrap_or(false);
                    if is_set {
                        if property == "add" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_sv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_set_add".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "contains" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_sv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            let result = ctx.alloc_local("_scontains", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_contains".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "remove" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_sv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_set_remove".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_setlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_set_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "union" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let other = ctx.next_local - 1;
                            let result = ctx.alloc_local("_sun", MirType::Set(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_union".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(other)],
                            });
                            return ctx;
                        }
                        if property == "intersection" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let other = ctx.next_local - 1;
                            let result = ctx.alloc_local("_sint", MirType::Set(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_intersection".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(other)],
                            });
                            return ctx;
                        }
                        if property == "difference" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let other = ctx.next_local - 1;
                            let result = ctx.alloc_local("_sdiff", MirType::Set(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_difference".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(other)],
                            });
                            return ctx;
                        }
                        if property == "symmetric_difference" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let other = ctx.next_local - 1;
                            let result = ctx.alloc_local("_ssdiff", MirType::Set(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_symmetric_difference".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(other)],
                            });
                            return ctx;
                        }
                        if property == "is_subset" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let other = ctx.next_local - 1;
                            let result = ctx.alloc_local("_ssub", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_is_subset".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(other)],
                            });
                            return ctx;
                        }
                        if property == "to_list" && arguments.is_empty() {
                            let result = ctx.alloc_local("_stl", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_set_to_list".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                    }
                    if is_queue {
                        if property == "push" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_qv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_queue_push".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "pop" && arguments.is_empty() {
                            let result = ctx.alloc_local("_qpop", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_queue_pop".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "peek" && arguments.is_empty() {
                            let result = ctx.alloc_local("_qpeek", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_queue_peek".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_qlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_queue_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_queue_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "to_list" && arguments.is_empty() {
                            let result = ctx.alloc_local("_qtl", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_queue_to_list".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "to_stack" && arguments.is_empty() {
                            let result = ctx.alloc_local("_qts", MirType::Stack(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_queue_to_stack".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                    }
                    if is_stack {
                        if property == "push" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_sv64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_stack_push".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "pop" && arguments.is_empty() {
                            let result = ctx.alloc_local("_spop", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_stack_pop".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "peek" && arguments.is_empty() {
                            let result = ctx.alloc_local("_speek", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_stack_peek".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_slen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_stack_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_stack_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "to_list" && arguments.is_empty() {
                            let result = ctx.alloc_local("_stl", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_stack_to_list".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "to_queue" && arguments.is_empty() {
                            let result = ctx.alloc_local("_stq", MirType::Queue(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_stack_to_queue".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                    }
                    // === DEQUE METHODS ===
                    if is_deque {
                        if property == "push_back" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_db64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_deque_push_back".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "push_front" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_df64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_deque_push_front".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "pop_back" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dpb", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_deque_pop_back".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "pop_front" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dpf", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_deque_pop_front".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "peek_back" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dpkb", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_deque_peek_back".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "peek_front" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dpkf", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_deque_peek_front".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_deque_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_deque_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                    }
                    // === LINKED LIST METHODS ===
                    if is_llist {
                        if property == "push_back" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_lb64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_linked_list_push_back".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "push_front" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_lf64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_linked_list_push_front".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "pop_back" && arguments.is_empty() {
                            let result = ctx.alloc_local("_lpb", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_linked_list_pop_back".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "pop_front" && arguments.is_empty() {
                            let result = ctx.alloc_local("_lpf", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_linked_list_pop_front".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "peek_back" && arguments.is_empty() {
                            let result = ctx.alloc_local("_lpbk", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_linked_list_peek_back".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "peek_front" && arguments.is_empty() {
                            let result = ctx.alloc_local("_lpfk", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_linked_list_peek_front".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "len" && arguments.is_empty() {
                            let result = ctx.alloc_local("_llen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_linked_list_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_linked_list_clear".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                    }
                    let is_dict = obj_type.as_ref().map(|t| matches!(t, MirType::Dict(_, _))).unwrap_or(false);
                    if is_dict {
                        if property == "len" {
                            let result = ctx.alloc_local("_dictlen", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_dict_len".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "keys" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dkeys", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_dict_keys".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "has" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let key_val = ctx.next_local - 1;
                            let result = ctx.alloc_local("_dicthas", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_dict_contains".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(key_val)],
                            });
                            return ctx;
                        }
                        if property == "remove" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let key_val = ctx.next_local - 1;
                            let result = ctx.alloc_local("_dictrm", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_dict_remove".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(key_val)],
                            });
                            return ctx;
                        }
                        if property == "get" && arguments.len() == 1 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let key_val = ctx.next_local - 1;
                            let result = ctx.alloc_local("_dictget", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result),
                                name: "ky_dict_get".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(key_val)],
                            });
                            // Cast to the dict's value type if not I64
                            if let Some(MirType::Dict(_, v)) = obj_type {
                                let elem_type = v.as_ref();
                                if *elem_type == MirType::Str {
                                    let str_res = ctx.alloc_local("_dictget_str", MirType::Str);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: str_res,
                                        value: MirValue::Local(result),
                                        to_type: MirType::Str,
                                    });
                                    ctx.string_locals.push(str_res);
                                } else if *elem_type != MirType::I64 {
                                    let casted = ctx.alloc_local("_dictget_cast", elem_type.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: casted,
                                        value: MirValue::Local(result),
                                        to_type: elem_type.clone(),
                                    });
                                }
                            }
                            return ctx;
                        }
                        if property == "set" && arguments.len() == 2 {
                            ctx = self.lower_expr(ctx, &arguments[0]);
                            let key_val = ctx.next_local - 1;
                            ctx = self.lower_expr(ctx, &arguments[1]);
                            let val_val = ctx.next_local - 1;
                            let val_i64 = ctx.alloc_local("_dval64", MirType::I64);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i64, value: MirValue::Local(val_val), to_type: MirType::I64,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_dict_set".to_string(),
                                args: vec![MirValue::Local(obj_local), MirValue::Local(key_val), MirValue::Local(val_i64)],
                            });
                            return ctx;
                        }
                        if property == "values" && arguments.is_empty() {
                            let result = ctx.alloc_local("_dv", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_dict_values".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "items" && arguments.is_empty() {
                            let result = ctx.alloc_local("_di", MirType::List(Box::new(MirType::I64)));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(result), name: "ky_dict_items".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            return ctx;
                        }
                        if property == "clear" && arguments.is_empty() {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None, name: "ky_dict_free".to_string(),
                                args: vec![MirValue::Local(obj_local)],
                            });
                            let new_dict = ctx.alloc_local("_newdict", obj_type.clone().unwrap_or(MirType::Dict(Box::new(MirType::Str), Box::new(MirType::I64))));
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(new_dict), name: "ky_dict_new".to_string(),
                                args: vec![],
                            });
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: obj_local, value: MirValue::Local(new_dict),
                            });
                            return ctx;
                        }
                    }

                    // Clone shortcut for Move types (str, list, dict)
                    if property == "clone" && arguments.is_empty() {
                        if let Some(obj_type) = &obj_type {
                            let fn_name = match obj_type {
                                MirType::Str => Some("ky_clone_str"),
                                MirType::List(_) => Some("ky_clone_list"),
                                MirType::Dict(_, _) => Some("ky_clone_dict"),
                                _ => None,
                            };
                            if let Some(fn_name) = fn_name {
                                let result = ctx.alloc_local("_clone", obj_type.clone());
                                if matches!(obj_type, MirType::Str) {
                                    ctx.string_locals.push(result);
                                }
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(result),
                                    name: fn_name.to_string(),
                                    args: vec![MirValue::Local(obj_local)],
                                });
                                return ctx;
                            }
                        }
                    }

                    // === MUTABLE PRIMITIVE GET/SET ===
                    if property == "get" && arguments.is_empty() {
                        if let Some(t) = &obj_type {
                            if matches!(t, MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64
                                        | MirType::U8 | MirType::U16 | MirType::U32 | MirType::U64
                                        | MirType::F32 | MirType::F64 | MirType::Bool | MirType::Char) {
                                return ctx;
                            }
                        }
                    }
                    if property == "set" && arguments.len() == 1 {
                        if let Some(t) = &obj_type {
                            if matches!(t, MirType::I1 | MirType::I8 | MirType::I16 | MirType::I32 | MirType::I64
                                        | MirType::U8 | MirType::U16 | MirType::U32 | MirType::U64
                                        | MirType::F32 | MirType::F64 | MirType::Bool | MirType::Char) {
                                ctx = self.lower_expr(ctx, &arguments[0]);
                                let sv = ctx.next_local - 1;
                                let st = ctx.local_types.get(&sv).cloned().unwrap_or(t.clone());
                                let sv2 = if st != *t {
                                    let cv = ctx.alloc_local("_msv", t.clone());
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: cv, value: MirValue::Local(sv), to_type: t.clone(),
                                    });
                                    MirValue::Local(cv)
                                } else { MirValue::Local(sv) };
                                if let Expr::Identifier { name, .. } = object.as_ref() {
                                    if let Some(&alloca) = ctx.locals.get(name) {
                                        ctx.current_block.insts.push(MirInst::Store {
                                            dest: alloca,
                                            value: sv2,
                                        });
                                    }
                                }
                                return ctx;
                            }
                        }
                    }
                }

                // Handle calling through an expression (e.g. handlers[i](args)):
                // lower the target to get the function pointer, then emit CallIndirect.
                if !matches!(target.as_ref(), Expr::Identifier { .. } | Expr::PropertyAccess { .. }) {
                    ctx = self.lower_expr(ctx, target);
                    let fn_ptr_local = ctx.next_local - 1;
                    let mut args = Vec::new();
                    for arg in arguments {
                        // For struct-typed arguments, pass by pointer (not by value)
                        if let Expr::Identifier { name, .. } = arg {
                            if let Some(&local) = ctx.locals.get(name) {
                                if matches!(ctx.local_types.get(&local), Some(MirType::Struct(_, _))) {
                                    args.push(MirValue::Local(local));
                                    continue;
                                }
                            }
                        }
                        ctx = self.lower_expr(ctx, arg);
                        args.push(MirValue::Local(ctx.next_local - 1));
                    }
                    let param_types: Vec<MirType> = args.iter().map(|a| {
                        match a {
                            MirValue::Local(id) => {
                                let t = ctx.local_types.get(id).cloned().unwrap_or(MirType::I32);
                                // Struct types are passed as pointers in closures
                                if matches!(t, MirType::Struct(_, _)) { MirType::Ptr(Box::new(MirType::Void)) }
                                else { t }
                            }
                            _ => MirType::I32,
                        }
                    }).collect();
                    let dest = ctx.alloc_local("_ccall", MirType::I32);
                    ctx.current_block.insts.push(MirInst::CallIndirect {
                        dest: Some(dest), fn_ptr: fn_ptr_local,
                        ret_type: MirType::I32, param_types, args,
                    });
                    return ctx;
                }

                let name = if let Expr::Identifier { name, .. } = target.as_ref() {
                    name.clone()
                } else {
                    "_call".to_string()
                };

                // Check for closure call: if `name` refers to a local or parameter
                // (not a function declaration), emit an indirect call through the function pointer.
                if let Some(&local) = ctx.locals.get(&name) {
                    // Lower arguments, passing structs by pointer (not by value)
                    let mut args = Vec::new();
                    for arg in arguments {
                        if let Expr::Identifier { name, .. } = arg {
                            if let Some(&arg_local) = ctx.locals.get(name) {
                                if matches!(ctx.local_types.get(&arg_local), Some(MirType::Struct(_, _))) {
                                    args.push(MirValue::Local(arg_local));
                                    continue;
                                }
                            }
                        }
                        ctx = self.lower_expr(ctx, arg);
                        args.push(MirValue::Local(ctx.next_local - 1));
                    }
                    // Infer param_types: struct types become Ptr(Void) for by-ref ABI
                    let param_types: Vec<MirType> = args.iter().map(|a| {
                        match a {
                            MirValue::Local(id) => {
                                let t = ctx.local_types.get(id).cloned().unwrap_or(MirType::I32);
                                if matches!(t, MirType::Struct(_, _)) { MirType::Ptr(Box::new(MirType::Void)) }
                                else { t }
                            }
                            _ => MirType::I32,
                        }
                    }).collect();
                    let ret_type = MirType::I32;
                    let dest = ctx.alloc_local("_ccall", ret_type.clone());
                    ctx.current_block.insts.push(MirInst::CallIndirect {
                        dest: Some(dest),
                        fn_ptr: local,
                        ret_type,
                        param_types,
                        args,
                    });
                    return ctx;
                }

                // Special case: range(n) — create a list [0, 1, ..., n-1]
                if name == "range" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let count_local = ctx.next_local - 1;
                    let count_i64 = ctx.alloc_local("_cnt64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: count_i64,
                        value: MirValue::Local(count_local),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_range", MirType::List(Box::new(MirType::I64)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_range".to_string(),
                        args: vec![MirValue::Local(count_i64)],
                    });
                    return ctx;
                }
                // Special case: range(start, end) — create a list [start, start+1, ..., end-1]
                if name == "range" && arguments.len() == 2 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let start_local = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, &arguments[1]);
                    let end_local = ctx.next_local - 1;
                    let start_i64 = ctx.alloc_local("_rs64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: start_i64,
                        value: MirValue::Local(start_local),
                        to_type: MirType::I64,
                    });
                    let end_i64 = ctx.alloc_local("_re64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: end_i64,
                        value: MirValue::Local(end_local),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_range2", MirType::List(Box::new(MirType::I64)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_range_two".to_string(),
                        args: vec![MirValue::Local(start_i64), MirValue::Local(end_i64)],
                    });
                    return ctx;
                }

                // Special case: len() built-in — return string, list, or dict length
                if name == "len" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    let t = ctx.local_types.get(&arg_local);
                    let is_list = t.map(|t| matches!(t, MirType::List(_))).unwrap_or(false);
                    let is_dict = t.map(|t| matches!(t, MirType::Dict(_, _))).unwrap_or(false);
                    if is_dict {
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_dict_len".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    } else if is_list {
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_list_len".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    } else {
                        let result = ctx.alloc_local("_len", MirType::I32);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_strlen".to_string(),
                            args: vec![MirValue::Local(arg_local)],
                        });
                    }
                    return ctx;
                }

                // Special case: input() / input(prompt) built-in — read line from stdin
                if name == "input" {
                    if arguments.is_empty() {
                        let result = ctx.alloc_local("_inp", MirType::Str);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_input".to_string(),
                            args: vec![],
                        });
                        ctx.string_locals.push(result);
                        return ctx;
                    }
                }
                if name == "input" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let prompt_local = ctx.next_local - 1;
                    let prompt_len = ctx.alloc_local("_plen", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(prompt_len),
                        name: "ky_strlen".to_string(),
                        args: vec![MirValue::Local(prompt_local)],
                    });
                    let result = ctx.alloc_local("_inp", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_input_with_prompt".to_string(),
                        args: vec![MirValue::Local(prompt_local), MirValue::Local(prompt_len)],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // Special case: str() built-in — convert number to string
                // Note: str(v) standalone conversion removed — use v.to_str() method instead

                // Special case: substr(str, start, count) — substring extraction
                if name == "substr" && arguments.len() == 3 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let str_local = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, &arguments[1]);
                    let start_local = ctx.next_local - 1;
                    let start_i64 = ctx.alloc_local("_s64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: start_i64,
                        value: MirValue::Local(start_local),
                        to_type: MirType::I64,
                    });
                    ctx = self.lower_expr(ctx, &arguments[2]);
                    let count_local = ctx.next_local - 1;
                    let count_i64 = ctx.alloc_local("_c64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: count_i64,
                        value: MirValue::Local(count_local),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_substr", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_substr".to_string(),
                        args: vec![
                            MirValue::Local(str_local),
                            MirValue::Local(start_i64),
                            MirValue::Local(count_i64),
                        ],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }

                // Special case: print/println with string literal argument
                if (name == "print" || name == "println") && arguments.len() == 1 {
                    if let Expr::Literal { value: Literal::String(s), .. } = &arguments[0] {
                        let dest = ctx.alloc_local("_call", MirType::Void);
                        let call_name = if name == "println" { "ky_println" } else { "ky_print" };
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(dest),
                            name: call_name.to_string(),
                            args: vec![
                                MirValue::Constant(MirConstant::String(s.clone())),
                                MirValue::Constant(MirConstant::I32(s.len() as i32)),
                            ],
                        });
                        return ctx;
                    }
                }

                // Remap class constructor calls: Point(10, 20) → Point::new
                let mut resolved_name = self.class_constructor_map.borrow()
                    .get(&name).cloned().unwrap_or_else(|| {
                        // For protected names (_name → name stripped), use the table entry
                        if name.starts_with('_') {
                            let stripped = &name[1..];
                            if self.function_decls.borrow().contains_key(stripped) {
                                stripped.to_string()
                            } else { name.clone() }
                        } else { name.clone() }
                    });

                let mut args = Vec::new();
                // Check for default parameter values
                let decl_opt = self.function_decls.borrow().get(&resolved_name).cloned();
                if let Some(decl) = decl_opt {
                    let supplied = arguments.len();
                    for (i, param) in decl.params.iter().enumerate() {
                        if param.variadic {
                            let arg = &arguments[i];
                            // &T (MutableBorrow) params: pass address of local directly
                            if param.mode == ParamMode::MutableBorrow {
                                if let Expr::BorrowRef { expression, .. } = arg {
                                    if let Expr::Identifier { name, .. } = expression.as_ref() {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            let ptr = ctx.alloc_local("_mbptr", MirType::Ptr(Box::new(MirType::I8)));
                                            ctx.current_block.insts.push(MirInst::AddressOf {
                                                dest: ptr,
                                                local_id: local,
                                            });
                                            args.push(MirValue::Local(ptr));
                                            continue;
                                        }
                                    }
                                }
                            }
                            // Struct-typed identifiers: pass original local by reference
                            if let Expr::Identifier { name, .. } = arg {
                                if let Some(&local) = ctx.locals.get(name) {
                                    let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                    if matches!(&t, MirType::Struct(_, _)) || is_move_type(&t) {
                                        args.push(MirValue::Local(local));
                                        continue;
                                    }
                                }
                            }
                            ctx = self.lower_expr(ctx, arg);
                            args.push(MirValue::Local(ctx.next_local - 1));
                        } else if i < supplied {
                            let arg = &arguments[i];
                            // &T (MutableBorrow) params: pass address of local directly
                            if param.mode == ParamMode::MutableBorrow {
                                if let Expr::BorrowRef { expression, .. } = arg {
                                    if let Expr::Identifier { name, .. } = expression.as_ref() {
                                        if let Some(&local) = ctx.locals.get(name) {
                                            let ptr = ctx.alloc_local("_mbptr3", MirType::Ptr(Box::new(MirType::I8)));
                                            ctx.current_block.insts.push(MirInst::AddressOf {
                                                dest: ptr,
                                                local_id: local,
                                            });
                                            args.push(MirValue::Local(ptr));
                                            continue;
                                        }
                                    }
                                }
                            }
                            // Struct-typed identifiers: pass original local by reference
                            if let Expr::Identifier { name, .. } = arg {
                                if let Some(&local) = ctx.locals.get(name) {
                                    let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                    if matches!(&t, MirType::Struct(_, _)) || is_move_type(&t) {
                                        args.push(MirValue::Local(local));
                                        continue;
                                    }
                                }
                            }
                            ctx = self.lower_expr(ctx, arg);
                            args.push(MirValue::Local(ctx.next_local - 1));
                        } else if let Some(default_expr) = &param.default {
                            // Missing argument with default — lower the default expression
                            ctx = self.lower_expr(ctx, default_expr);
                            args.push(MirValue::Local(ctx.next_local - 1));
                        } else {
                            // Missing argument without default — should be caught by type checker
                            break;
                        }
                    }
                } else {
                    for arg in arguments {
                        // &x: pass ADDRESS of local (mutable borrow)
                        if let Expr::BorrowRef { expression, .. } = arg {
                            if let Expr::Identifier { name, .. } = expression.as_ref() {
                                if let Some(&local) = ctx.locals.get(name) {
                                    if let Some(MirType::Array(_, _)) = ctx.local_types.get(&local) {
                                        ctx = self.lower_expr(ctx, arg);
                                        args.push(MirValue::Local(ctx.next_local - 1));
                                        continue;
                                    }
                                    let ptr = ctx.alloc_local("_mbptr2", MirType::Ptr(Box::new(MirType::I8)));
                                    ctx.current_block.insts.push(MirInst::AddressOf {
                                        dest: ptr,
                                        local_id: local,
                                    });
                                    args.push(MirValue::Local(ptr));
                                    continue;
                                }
                            }
                        }
                        if let Expr::Identifier { name, .. } = arg {
                            if let Some(&local) = ctx.locals.get(name) {
                                let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                                if matches!(&t, MirType::Struct(_, _)) || is_move_type(&t) {
                                    args.push(MirValue::Local(local));
                                    continue;
                                }
                            }
                        }
                        ctx = self.lower_expr(ctx, arg);
                        args.push(MirValue::Local(ctx.next_local - 1));
                    }
                }

                // Check if this is a call to a generic function — monomorphize lazily
                {
                    let generic_templates = self.generic_function_templates.borrow();
                    let template_opt = generic_templates.get(&resolved_name).cloned();
                    drop(generic_templates);
                    if let Some(template) = template_opt {
                        // Get argument MIR types from the already-lowered args
                        let arg_types: Vec<MirType> = args.iter().map(|a| {
                            match a {
                                MirValue::Local(id) => ctx.local_types.get(id).cloned().unwrap_or(MirType::I32),
                                _ => MirType::I32,
                            }
                        }).collect();
                        // Infer type params from concrete argument types.
                        let type_subst = infer_function_type_params(&template, &arg_types);
                        let type_args: Vec<MirType> = template.type_params.iter()
                            .map(|tp| type_subst.get(&tp.name).cloned().unwrap_or(MirType::I32))
                            .collect();
                        let specialized_name = make_concrete_name(&resolved_name, &type_args);

                        // Check if already specialized
                        if !self.fn_returns.borrow().contains_key(&specialized_name) {
                            // Clone and specialize the function AST
                            let mut specialized_decl = clone_and_specialize_function(&template, &type_subst);
                            specialized_decl.name = specialized_name.clone();

                            // Pre-register any generic struct types in the function signature
                            // so that lower_function can resolve them in struct_defs.
                            {
                                let generic_struct_tpls = self.generic_struct_templates.borrow();
                                let mut struct_defs = self.struct_defs.borrow_mut();
                                if let Some(rt) = &template.return_type {
                                    pre_register_generic_type(rt, &type_subst, &generic_struct_tpls, &mut struct_defs);
                                }
                                for p in &template.params {
                                    pre_register_generic_type(&p.type_, &type_subst, &generic_struct_tpls, &mut struct_defs);
                                }
                            }

                            // Compute specialized return type (struct_defs now has concrete types)
                            let struct_defs = self.struct_defs.borrow();
                            let ret_type = template.return_type.as_ref()
                                .map(|rt| ast_type_to_mir_with_subst(rt, Some(&struct_defs), &type_subst))
                                .unwrap_or(MirType::Void);
                            drop(struct_defs);

                            // Register return type for the specialized function
                            self.fn_returns.borrow_mut().insert(specialized_name.clone(), ret_type.clone());

                            // Lower the specialized function
                            if let Some(mir_func) = self.lower_function(&specialized_decl) {
                                self.specialized_mir_functions.borrow_mut().push(mir_func);
                            }
                        }

                        // Redirect call to the specialized function
                        resolved_name = specialized_name;
                    }
                }

                // Handle serialize(val) and deserialize<T>(str) before call_type setup
                {
                    // type<T>() — return TypeInfo for a type
                    if resolved_name == "type" && type_args.len() == 1 {
                        if let Some(first_type_arg) = type_args.first() {
                            let struct_defs = ctx.struct_defs.clone();
                            let mir_type = ast_type_to_mir(first_type_arg, Some(&struct_defs));
                            let _typeinfo = build_typeinfo_struct(&mir_type, &mut ctx);
                            return ctx;
                        }
                    }
                    // serialize(val) — class to JSON
                    if resolved_name == "serialize" && args.len() == 1 {
                        if let MirValue::Local(id) = &args[0] {
                            if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(id).cloned() {
                                let descriptor = build_descriptor(&fields);
                                args.push(MirValue::Constant(MirConstant::String(descriptor)));
                                resolved_name = "ky_struct_to_json".to_string();
                            }
                        }
                    // deserialize<T>(str) — JSON to class T
                    } else if resolved_name == "deserialize" && args.len() == 1 {
                        if let Some(first_type_arg) = type_args.first() {
                            let struct_defs = ctx.struct_defs.clone();
                            let mir_type = ast_type_to_mir(first_type_arg, Some(&struct_defs));
                            if let MirType::Struct(_, fields) = &mir_type {
                                let descriptor = build_descriptor(fields);
                                let json_arg = args.remove(0);
                                let out_local = ctx.alloc_local("_dout", mir_type.clone());
                                args.push(json_arg);
                                args.push(MirValue::Constant(MirConstant::String(descriptor)));
                                args.push(MirValue::Local(out_local));
                                // Call ky_json_to_struct (returns i32, writes to out_local)
                                let ret_local = ctx.alloc_local("_dret", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(ret_local),
                                    name: "ky_json_to_struct".to_string(),
                                    args,
                                });
                                // Load result from output struct (which ky_json_to_struct wrote to)
                                let load = ctx.alloc_local("_dval", mir_type.clone());
                                ctx.current_block.insts.push(MirInst::Load {
                                    dest: load,
                                    src: out_local,
                                });
                                return ctx;
                            }
                        }
                    }
                }

                let call_type = builtin_return_type(&resolved_name).unwrap_or_else(|| {
                    self.fn_returns.borrow().get(&resolved_name).cloned().unwrap_or(MirType::I32)
                });
                let dest = ctx.alloc_local("_call", call_type.clone());
                if call_type == MirType::Str {
                    ctx.string_locals.push(dest);
                }

                // Special case: println() with no arguments — print a newline
                if name == "println" && args.is_empty() {
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(dest),
                        name: "ky_println".to_string(),
                        args: vec![
                            MirValue::Constant(MirConstant::String("\n".to_string())),
                            MirValue::Constant(MirConstant::I32(1)),
                        ],
                    });
                    return ctx;
                }

                // For print/println with dynamic arguments
                if (name == "print" || name == "println") && !args.is_empty() {
                    let first_arg = &args[0];
                    if let MirValue::Local(id) = first_arg {
                        let is_str = ctx.string_locals.contains(id)
                            || ctx.local_types.get(id).map_or(false, |t| *t == MirType::Str);
                        if is_str {
                            // Use the str value directly — no extra Load needed
                            // (the expression result IS the pointer)
                            let print_arg = *id;
                            let len_dest = ctx.alloc_local("_strlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(len_dest),
                                name: "ky_strlen".to_string(),
                                args: vec![MirValue::Local(print_arg)],
                            });
                            let call_name = if name == "println" { "ky_println" } else { "ky_print" };
                            let pret = ctx.alloc_local("_pret", call_type.clone());
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(pret),
                                name: call_name.to_string(),
                                args: vec![
                                    MirValue::Local(print_arg),
                                    MirValue::Local(len_dest),
                                ],
                            });
                            return ctx;
                        } else {
                            // Non-string argument — convert to string, then print
                            let id_type = if *id < ctx.local_types.len() {
                                ctx.local_types.get(id).cloned().unwrap_or(MirType::I64)
                            } else { MirType::I64 };
                            let str_val = if id_type == MirType::Char {
                                // Convert char to string via ky_char_to_str
                                let s = ctx.alloc_local("_csv", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(s),
                                    name: "ky_char_to_str".to_string(),
                                    args: vec![MirValue::Local(*id)],
                                });
                                ctx.string_locals.push(s);
                                s
                            } else if id_type == MirType::I64 || id_type == MirType::I32 {
                                // Convert integer to string via kl_i64_to_str
                                let i64_val = if id_type == MirType::I32 {
                                    let ext = ctx.alloc_local("_i64v", MirType::I64);
                                    ctx.current_block.insts.push(MirInst::Cast {
                                        dest: ext,
                                        value: MirValue::Local(*id),
                                        to_type: MirType::I64,
                                    });
                                    MirValue::Local(ext)
                                } else { MirValue::Local(*id) };
                                let s = ctx.alloc_local("_strv", MirType::Str);
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: Some(s),
                                    name: "ky_i64_to_str".to_string(),
                                    args: vec![i64_val],
                                });
                                ctx.string_locals.push(s);
                                s
                            } else {
                                *id
                            };
                            let len_dest = ctx.alloc_local("_strlen", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(len_dest),
                                name: "ky_strlen".to_string(),
                                args: vec![MirValue::Local(str_val)],
                            });
                            let print_arg = str_val;
                            let call_name = if name == "println" { "ky_println" } else { "ky_print" };
                            let pret = ctx.alloc_local("_pret", call_type.clone());
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(pret),
                                name: call_name.to_string(),
                                args: vec![MirValue::Local(print_arg), MirValue::Local(len_dest)],
                            });
                            return ctx;
                        }
                    }
                }

                // Special case: ok(val) — construct success result struct
                if name == "ok" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    let disc_ptr = ctx.alloc_local("_odp", MirType::I32);
                    let payload_val = ctx.alloc_local("_opv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: payload_val,
                        value: MirValue::Local(arg_local),
                        to_type: MirType::I64,
                    });
                    let payload_ptr = ctx.alloc_local("_opp", MirType::I64);
                    // Allocate struct LAST
                    let result_local = ctx.alloc_local("_okres", call_type.clone());
                    // disc = 0 (success)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr,
                        ptr: result_local,
                        field_index: 0,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr,
                        value: MirValue::Constant(MirConstant::I32(0)),
                    });
                    // payload = value (extended to i64)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: payload_ptr,
                        ptr: result_local,
                        field_index: 1,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: payload_ptr,
                        value: MirValue::Local(payload_val),
                    });
                    // Load the struct from alloca to return the value (not the pointer)
                    let result_load = ctx.alloc_local("_okres_v", call_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result_load,
                        src: result_local,
                    });
                    return ctx;
                }

                // Special case: error(msg) — construct error result struct
                if name == "error" && arguments.len() == 1 {
                    ctx = self.lower_expr(ctx, &arguments[0]);
                    let arg_local = ctx.next_local - 1;
                    // Allocate temps FIRST, struct LAST
                    let disc_ptr = ctx.alloc_local("_edp", MirType::I32);
                    let payload_val = ctx.alloc_local("_epv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: payload_val,
                        value: MirValue::Local(arg_local),
                        to_type: MirType::I64,
                    });
                    let payload_ptr = ctx.alloc_local("_epp", MirType::I64);
                    let result_local = ctx.alloc_local("_erres", call_type.clone());
                    // disc = 1 (error)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr,
                        ptr: result_local,
                        field_index: 0,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr,
                        value: MirValue::Constant(MirConstant::I32(1)),
                    });
                    // payload = argument (cast to i64 if possible)
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: payload_ptr,
                        ptr: result_local,
                        field_index: 1,
                        struct_type: Box::new(call_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: payload_ptr,
                        value: MirValue::Local(payload_val),
                    });
                    // Load the struct from alloca to return the value (not the pointer)
                    let result_load = ctx.alloc_local("_erres_v", call_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result_load,
                        src: result_local,
                    });
                    return ctx;
                }

                // Special case: some(val) — construct Some option variant
                if name == "some" && arguments.len() == 1 {
                    let arg_val = &args[0];
                    let arg_type = args.first().and_then(|id| {
                        if let MirValue::Local(id) = id {
                            ctx.local_types.get(id).cloned()
                        } else { None }
                    }).unwrap_or(MirType::I32);
                    let concrete_name = make_concrete_name("Option", &[arg_type.clone()]);
                    let opt_type = MirType::Struct(concrete_name.clone(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]);
                    ctx.struct_defs.entry(concrete_name).or_insert_with(|| vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]);
                    let disc_ptr = ctx.alloc_local("_sdp", MirType::I32);
                    let payload_val = ctx.alloc_local("_spv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: payload_val, value: arg_val.clone(), to_type: MirType::I64,
                    });
                    let payload_ptr = ctx.alloc_local("_spp", MirType::I64);
                    let result_local = ctx.alloc_local("_someres", opt_type.clone());
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: disc_ptr, ptr: result_local, field_index: 0,
                        struct_type: Box::new(opt_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: disc_ptr, value: MirValue::Constant(MirConstant::I32(1)),
                    });
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: payload_ptr, ptr: result_local, field_index: 1,
                        struct_type: Box::new(opt_type.clone()),
                    });
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: payload_ptr, value: MirValue::Local(payload_val),
                    });
                    let result_load = ctx.alloc_local("_someres_v", opt_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result_load, src: result_local,
                    });
                    return ctx;
                }

                // Special case: box(val) — heap-allocate T and store val
                if name == "box" && arguments.len() == 1 {
                    let arg_val = &args[0];
                    // Get the inner type from the argument
                    let arg_type = args.first().and_then(|id| {
                        if let MirValue::Local(id) = id {
                            ctx.local_types.get(id).cloned()
                        } else { None }
                    }).unwrap_or(MirType::I32);
                    let size = mir_type_to_size(&arg_type) as i64;
                    let ptr = ctx.alloc_local("_boxptr", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(ptr),
                        name: "ky_alloc".to_string(),
                        args: vec![MirValue::Constant(MirConstant::I64(size))],
                    });
                    // Store the value at the allocated pointer
                    let elem_type = arg_type;
                    let pointee_type = MirType::Ptr(Box::new(elem_type.clone()));
                    // Store through pointer by casting to i64 first (like PtrStore)
                    let val_i64 = ctx.alloc_local("_boxv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: arg_val.clone(),
                        to_type: MirType::I64,
                    });
                    // For now, just return the pointer as the result
                    // The user can dereference it later
                    let result = ctx.alloc_local("_box", MirType::Box(Box::new(elem_type)));
                    ctx.current_block.insts.push(MirInst::Store {
                        dest: result,
                        value: MirValue::Local(ptr),
                    });
                    return ctx;
                }

                // Auto-JSON for client.post(url, data) where data is a class
                if name == "post" && args.len() == 2 {
                    if let MirValue::Local(id) = &args[1] {
                        if let Some(MirType::Struct(_, fields)) = ctx.local_types.get(id).cloned() {
                            let mut desc_parts: Vec<String> = Vec::new();
                            for (fname, ftype) in &fields {
                                let tn = match ftype { MirType::Str => "str", MirType::I32 => "i32", MirType::I64 => "i64", MirType::Bool => "bool", MirType::F64 => "f64", _ => "i32" };
                                desc_parts.push(format!("{}:{}", fname, tn));
                            }
                            let desc = desc_parts.join(",");
                            let data_arg = args.remove(1);
                            let ser_dest = ctx.alloc_local("_ser", MirType::Str);
                            ctx.string_locals.push(ser_dest);
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(ser_dest),
                                name: "ky_struct_to_json".to_string(),
                                args: vec![data_arg, MirValue::Constant(MirConstant::String(desc))],
                            });
                            args.push(MirValue::Local(ser_dest));
                        }
                    }
                }

                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(dest),
                    name: resolved_name.clone(),
                    args,
                });
                if matches!(resolved_name.as_str(), "to_upper" | "to_lower" | "upper" | "lower" | "trim" | "replace" | "input" | "input_with_prompt" | "read_str" | "substr" | "json_stringify" | "serialize" | "ky_struct_to_json") {
                    ctx.string_locals.push(dest);
                }
                ctx
            }
            Expr::Assignment { target, value, .. } => {

                // Handle list[index] = value → kl_list_set
                // Handle dict[key] = value → kl_dict_set
                if let Expr::Index { target: list_expr, index, .. } = target.as_ref() {
                    // For arrays, use variable alloca directly (skip whole-array Load)
                    let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = list_expr.as_ref() {
                        if let Some(&local) = ctx.locals.get(name) {
                            let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                            if matches!(t, MirType::Array(_, _)) {
                                (local, local, t)
                            } else {
                                ctx = self.lower_expr(ctx, list_expr);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                            }
                        } else {
                            ctx = self.lower_expr(ctx, list_expr);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        }
                    } else {
                        // Nested array assignment like m[i][j] = val.
                        // Walk the index chain to emit GEPs into the root array.
                        let indices = self.collect_array_indices(target);
                        if let Some((root_name, idx_exprs)) = indices {
                            if let Some(&root_local) = ctx.locals.get(&root_name) {
                                ctx = self.lower_nested_array_geps(ctx, &idx_exprs, root_local);
                                // Emit Store directly — GEP chain already computed
                                ctx = self.lower_expr(ctx, value);
                                let val_local = ctx.next_local - 1;
                                let gep_ptr = ctx.next_local - 2;
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: gep_ptr,
                                    value: MirValue::Local(val_local),
                                });
                                return ctx;
                            } else {
                                ctx = self.lower_expr(ctx, list_expr);
                                let tv = ctx.next_local - 1;
                                (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                            }
                        } else {
                            ctx = self.lower_expr(ctx, list_expr);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        }
                    };
                    ctx = self.lower_expr(ctx, index);
                    let idx_val = ctx.next_local - 1;
                    ctx = self.lower_expr(ctx, value);
                    let val_local = ctx.next_local - 1;
                    let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });

                    if matches!(&target_type, MirType::Array(_, _)) {
                        let arr_ty_clone = target_type.clone();
                        let et = match &target_type {
                            MirType::Array(inner_box, _) => {
                                match inner_box.as_ref() {
                                    t => t.clone(),
                                }
                            },
                            _ => MirType::I32,
                        };
                        let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                        ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                            dest: elem_ptr,
                            ptr: arr_ptr,
                            index: MirValue::Local(idx_val),
                            arr_type: Box::new(arr_ty_clone),
                            elem_type: Box::new(et),
                        });
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: elem_ptr,
                            value: MirValue::Local(val_local),
                        });
                    } else if matches!(&target_type, MirType::Dict(_, _)) {
                        let key_arg = if let MirValue::Local(id) = MirValue::Local(idx_val) {
                            if ctx.local_types.get(&id).map(|t| *t == MirType::Str).unwrap_or(false) {
                                MirValue::Local(id)
                            } else {
                                MirValue::Local(idx_val)
                            }
                        } else {
                            MirValue::Local(idx_val)
                        };
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_dict_set".to_string(),
                            args: vec![
                                MirValue::Local(target_val),
                                key_arg,
                                MirValue::Local(val_i64),
                            ],
                        });
                    } else {
                        let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: idx_i64,
                            value: MirValue::Local(idx_val),
                            to_type: MirType::I64,
                        });
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_list_set".to_string(),
                            args: vec![
                                MirValue::Local(target_val),
                                MirValue::Local(idx_i64),
                                MirValue::Local(val_i64),
                            ],
                        });
                    }
                    return ctx;
                }

                // For struct field assignment with empty Dict {} but field is List, create list instead
                if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    if let Expr::Identifier { name, .. } = object.as_ref() {
                        if let Some(&obj_local) = ctx.locals.get(name) {
                            let obj_type = ctx.local_types.get(&obj_local).cloned();
                            if let Some(MirType::Struct(_, fields)) = &obj_type {
                                let backing = format!("_{}", property);
                                let field_idx = fields.iter().position(|(fname, _)| fname == property.as_str())
                                    .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                                if let Some(fi) = field_idx {
                                    if let Some((_, MirType::List(inner))) = fields.get(fi) {
                                        let obj_ty = obj_type.clone().unwrap();
                                        let handle = ctx.alloc_local("_listv", MirType::List(inner.clone()));
                                        ctx.current_block.insts.push(MirInst::Call {
                                            dest: Some(handle),
                                            name: "ky_list_new".to_string(),
                                            args: vec![],
                                        });
                                        let field_ptr = ctx.alloc_local("_fieldptr", MirType::I64);
                                        ctx.current_block.insts.push(MirInst::FieldPtr {
                                            dest: field_ptr,
                                            ptr: obj_local,
                                            field_index: fi,
                                            struct_type: Box::new(obj_ty),
                                        });
                                        ctx.current_block.insts.push(MirInst::Store {
                                            dest: field_ptr,
                                            value: MirValue::Local(handle),
                                        });
                                        return ctx;
                                    }
                                }
                            }
                        }
                    }
                }
                let adjusted_value = value;

                ctx = self.lower_expr(ctx, adjusted_value);
                let val_local = ctx.next_local - 1;
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    let local = if let Some(existing) = ctx.locals.get(name) {
                        *existing
                    } else {
                        // Implicit variable declaration: infer type from value
                        let var_type = ctx.local_types.get(&val_local).cloned().unwrap_or_else(|| {
                            if ctx.string_locals.contains(&val_local) { MirType::Str } else { MirType::I32 }
                        });
                        let new_local = ctx.alloc_local(name, var_type);
                        ctx.locals.insert(name.clone(), new_local);
                        new_local
                    };
                    // For ^&T (ref param) locals: load pointer + write through it via runtime call
                    if ctx.ref_param_locals.contains(&local) {
                        let ptr_tmp = ctx.alloc_local("_rfptr", MirType::Ptr(Box::new(MirType::I8)));
                        ctx.current_block.insts.push(MirInst::Load {
                            dest: ptr_tmp, src: local,
                        });
                        // Determine value type: use ky_ptr_write_ptr for heap types, ky_ptr_write_i32 for scalars
                        let val_type = ctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                        let is_heap = matches!(&val_type, MirType::Str | MirType::List(_) | MirType::Dict(_, _)
                            | MirType::Set(_) | MirType::Queue(_) | MirType::Stack(_) | MirType::Ptr(_)
                            | MirType::Box(_));
                        if is_heap {
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None,
                                name: "ky_ptr_write_ptr".to_string(),
                                args: vec![MirValue::Local(ptr_tmp), MirValue::Local(val_local)],
                            });
                        } else {
                            let val_i32 = ctx.alloc_local("_rfv32", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: val_i32,
                                value: MirValue::Local(val_local),
                                to_type: MirType::I32,
                            });
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: None,
                                name: "ky_ptr_write_i32".to_string(),
                                args: vec![MirValue::Local(ptr_tmp), MirValue::Local(val_i32)],
                            });
                        }
                    } else {
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: local,
                            value: MirValue::Local(val_local),
                        });
                    }
                } else if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Check for property setter: obj.prop = val → Class::set_prop(obj, val)
                    let obj_id = if let Expr::Identifier { name, .. } = object.as_ref() {
                        ctx.locals.get(name).copied()
                    } else {
                        ctx = self.lower_expr(ctx, object);
                        Some(ctx.next_local - 1)
                    };
                    if let Some(obj_l) = obj_id {
                        if let Some(MirType::Struct(class_name, _)) = ctx.local_types.get(&obj_l) {
                            let method_table = self.method_table.borrow();
                            let parent_map = self.class_parent_map.borrow();
                            let setter_name = format!("set_{}", property);
                            if let Some(mangled) = self.lookup_method_in_chain(class_name, &setter_name, &method_table, &parent_map) {
                                ctx.current_block.insts.push(MirInst::Call {
                                    dest: None,
                                    name: mangled.clone(),
                                    args: vec![MirValue::Local(obj_l), MirValue::Local(val_local)],
                                });
                                return ctx;
                            }
                        }
                    }
                    // Struct field assignment: p.x = val
                    let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                        ctx.locals.get(name).copied()
                    } else {
                        None
                    };
                    if let Some(obj_ptr) = obj_ptr {
                        let obj_type = ctx.local_types.get(&obj_ptr).cloned();
                        if let Some(MirType::Struct(_, fields)) = &obj_type {
                            let backing = format!("_{}", property);
                            let field_idx = fields.iter().position(|(fname, _)| fname == property)
                                .or_else(|| fields.iter().position(|(fname, _)| fname == &backing));
                            if let Some(field_idx) = field_idx {
                                let field_ptr = ctx.alloc_local("_fieldptr", MirType::I64);
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: field_ptr,
                                    ptr: obj_ptr,
                                    field_index: field_idx,
                                    struct_type: Box::new(obj_type.unwrap()),
                                });
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: field_ptr,
                                    value: MirValue::Local(val_local),
                                });
                            }
                        }
                    }
                } else if let Expr::Tuple { elements: target_elems, .. } = target.as_ref() {
                    // Destructuring: (x, y) = (a, b) or (x, y) = func()
                    if let Expr::Tuple { elements: value_elems, .. } = value.as_ref() {
                        for (target_elem, value_elem) in target_elems.iter().zip(value_elems.iter()) {
                            ctx = self.lower_expr(ctx, value_elem);
                            let elem_val = ctx.next_local - 1;
                            if let Expr::Identifier { name, .. } = target_elem {
                                let var_type = ctx.local_types.get(&elem_val).cloned().unwrap_or(MirType::I32);
                                let local = ctx.alloc_local(name, var_type);
                                ctx.locals.insert(name.clone(), local);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: local,
                                    value: MirValue::Local(elem_val),
                                });
                            }
                        }
                    } else {
                        // (x, y) = func() — lower func call, extract tuple elements
                        ctx = self.lower_expr(ctx, value);
                        let tuple_local = ctx.next_local - 1;
                        let tuple_type = ctx.local_types.get(&tuple_local).cloned().unwrap_or(MirType::I32);
                         if let MirType::Struct(ref _sname, ref fields) = tuple_type {
                             for (i, target_elem) in target_elems.iter().enumerate() {
                                 if let Expr::Identifier { name, .. } = target_elem {
                                     if i < fields.len() {
                                         let field_type = fields[i].1.clone();
                                         let fptr = ctx.alloc_local("_tfptr", MirType::I64);
                                         ctx.current_block.insts.push(MirInst::FieldPtr {
                                             dest: fptr, ptr: tuple_local, field_index: i,
                                             struct_type: Box::new(tuple_type.clone()),
                                         });
                                         let val = ctx.alloc_local("_tval", field_type.clone());
                                         ctx.current_block.insts.push(MirInst::Load { dest: val, src: fptr });
                                         let local = ctx.alloc_local(name, field_type);
                                         ctx.locals.insert(name.clone(), local);
                                         ctx.current_block.insts.push(MirInst::Store { dest: local, value: MirValue::Local(val) });
                                     }
                                 }
                             }
                         }
                         if let MirType::List(ref elem_type) = tuple_type {
                             for (i, target_elem) in target_elems.iter().enumerate() {
                                 if let Expr::Identifier { name, .. } = target_elem {
                                     let idx_i64 = ctx.alloc_local("_ldi", MirType::I64);
                                     ctx.current_block.insts.push(MirInst::Store {
                                         dest: idx_i64, value: MirValue::Constant(MirConstant::I64(i as i64)),
                                     });
                                     let raw = ctx.alloc_local("_ldr", MirType::I64);
                                     ctx.current_block.insts.push(MirInst::Call {
                                         dest: Some(raw), name: "ky_list_get".to_string(),
                                         args: vec![MirValue::Local(tuple_local), MirValue::Local(idx_i64)],
                                     });
                                     let field_type = elem_type.as_ref().clone();
                                     let val = if field_type != MirType::I64 {
                                         let cast = ctx.alloc_local("_ldc", field_type.clone());
                                         ctx.current_block.insts.push(MirInst::Cast { dest: cast, value: MirValue::Local(raw), to_type: field_type.clone() });
                                         cast
                                     } else { raw };
                                     let local = ctx.alloc_local(name, field_type.clone());
                                     ctx.locals.insert(name.clone(), local);
                                     ctx.current_block.insts.push(MirInst::Store { dest: local, value: MirValue::Local(val) });
                                 }
                             }
                         }
                    }
                }
                ctx
            }
            Expr::PropertyAccess { object, property, .. } => {
                // Check for enum variant without payload: Option.None
                if let Expr::Identifier { name: enum_name, .. } = object.as_ref() {
                    let ev_map = self.enum_variants.borrow();
                    if let Some(variants) = ev_map.get(enum_name) {
                        if let Some(&variant_idx) = variants.get(property) {
                            let struct_type = MirType::Struct(enum_name.clone(), vec![
                                ("disc".to_string(), MirType::I32),
                                ("payload".to_string(), MirType::I64),
                            ]);
                            let disc_ptr = ctx.alloc_local("_edp", MirType::I64);
                            let dest = ctx.alloc_local("_enum", struct_type.clone());
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: disc_ptr,
                                ptr: dest,
                                field_index: 0,
                                struct_type: Box::new(struct_type),
                            });
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: disc_ptr,
                                value: MirValue::Constant(MirConstant::I32(variant_idx as i32)),
                            });
                            return ctx;
                        }
                    }
                }
                // If accessing `len` on a list-typed variable, call kl_list_len.
                // Must check type before lowering to avoid treating struct fields
                // named `len` (e.g., Parser.len) as list length calls.
                if property == "len" {
                    let is_list = match object.as_ref() {
                        Expr::Identifier { name, .. } => {
                            ctx.locals.get(name)
                                .and_then(|l| ctx.local_types.get(l))
                                .map(|t| matches!(t, MirType::List(_)))
                                .unwrap_or(false)
                        }
                        _ => false,
                    };
                    if is_list {
                        ctx = self.lower_expr(ctx, object);
                        let obj_val = ctx.next_local - 1;
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_list_len".to_string(),
                            args: vec![MirValue::Local(obj_val)],
                        });
                        return ctx;
                    }
                    let is_dict = match object.as_ref() {
                        Expr::Identifier { name, .. } => {
                            ctx.locals.get(name)
                                .and_then(|l| ctx.local_types.get(l))
                                .map(|t| matches!(t, MirType::Dict(_, _)))
                                .unwrap_or(false)
                        }
                        _ => false,
                    };
                    if is_dict {
                        ctx = self.lower_expr(ctx, object);
                        let obj_val = ctx.next_local - 1;
                        let result = ctx.alloc_local("_len", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(result),
                            name: "ky_dict_len".to_string(),
                            args: vec![MirValue::Local(obj_val)],
                        });
                        return ctx;
                    }
                }
                // Check for property getter: obj.prop → Class::get_prop(obj)
                let getter_obj = if let Expr::Identifier { name, .. } = object.as_ref() {
                    ctx.locals.get(name).copied()
                } else { None };
                if let Some(go) = getter_obj {
                    if let Some(MirType::Struct(sname, _)) = ctx.local_types.get(&go) {
                        let method_table = self.method_table.borrow();
                        let parent_map = self.class_parent_map.borrow();
                        if let Some(mangled) = self.lookup_method_in_chain(sname, &format!("get_{}", property), &method_table, &parent_map) {
                            let call_type = self.fn_returns.borrow()
                                .get(&mangled).cloned().unwrap_or(MirType::Void);
                            let dest = ctx.alloc_local("_getter", call_type.clone());
                            if call_type == MirType::Str {
                                ctx.string_locals.push(dest);
                            }
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: mangled.clone(),
                                args: vec![MirValue::Local(go)],
                            });
                            return ctx;
                        }
                    }
                }
                // Struct field access: use the variable's alloca pointer directly
                let obj_ptr = if let Expr::Identifier { name, .. } = object.as_ref() {
                    ctx.locals.get(name).copied()
                } else {
                    ctx = self.lower_expr(ctx, object);
                    let obj_val = ctx.next_local - 1;
                    // Fallback: assume the loaded value is a struct pointer
                    Some(obj_val)
                };
                if let Some(obj_ptr) = obj_ptr {
                    let obj_type = ctx.local_types.get(&obj_ptr).cloned();
                    // Unwrap Ptr(Struct(...)) for closure-inferred types
                    let (struct_name, struct_fields, _is_ptr) = match &obj_type {
                        Some(MirType::Struct(sname, fields)) => (Some(sname.clone()), Some(fields.clone()), false),
                        Some(MirType::Ptr(inner)) => match inner.as_ref() {
                            MirType::Struct(sname, fields) => (Some(sname.clone()), Some(fields.clone()), true),
                            _ => (None, None, false),
                        },
                        _ => (None, None, false),
                    };
                    // Resolve the struct name and fields, searching all struct_defs
                    // if the type is a generic pointer (e.g., closure-inferred Ptr(I8)).
                    let (sname, resolved_fields) = if let (Some(sn), Some(fields)) = (struct_name, struct_fields) {
                        let rf = if fields.is_empty() {
                            ctx.struct_defs.get(&sn).cloned().unwrap_or_default()
                        } else {
                            fields
                        };
                        (sn, rf)
                    } else {
                        // Fallback: scan all struct_defs for matching property
                        let mut found = None;
                        for (sn, sfields) in &ctx.struct_defs {
                            if sfields.iter().any(|(fn_, _)| fn_ == property || fn_ == &format!("_{}", property)) {
                                found = Some((sn.clone(), sfields.clone()));
                                break;
                            }
                        }
                        if let Some(result) = found { result }
                        else { return ctx; }
                    };
                    let backing = format!("_{}", property);
                    let field_idx = resolved_fields.iter().position(|(fname, _)| fname == property)
                        .or_else(|| resolved_fields.iter().position(|(fname, _)| fname == &backing));
                    if let Some(field_idx) = field_idx {
                        let field_type = resolved_fields[field_idx].1.clone();
                        let field_ptr = ctx.alloc_local("_fieldptr", MirType::I64);
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: field_ptr,
                            ptr: obj_ptr,
                            field_index: field_idx,
                            struct_type: Box::new(MirType::Struct(sname.clone(), resolved_fields)),
                        });
                        let result = ctx.alloc_local("_field", field_type.clone());
                        ctx.current_block.insts.push(MirInst::Load {
                            dest: result,
                            src: field_ptr,
                        });
                        if field_type == MirType::Str {
                            ctx.string_locals.push(result);
                        }
                        return ctx;
                    }
                }
                ctx
            }
            Expr::OptionalChain { target, property, .. } => {
                ctx = self.lower_expr(ctx, target);
                let target_local = ctx.next_local - 1;
                let target_type = ctx.local_types.get(&target_local).cloned()
                    .unwrap_or(MirType::Struct("Result".to_string(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]));

                let result_struct = ctx.alloc_local("_ochain", target_type.clone());

                // Get discriminant
                let disc_ptr = ctx.alloc_local("_odp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr,
                    ptr: target_local,
                    field_index: 0,
                    struct_type: Box::new(target_type.clone()),
                });
                let disc = ctx.alloc_local("_odv", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc,
                    src: disc_ptr,
                });

                let some_block = ctx.fresh_block();
                let none_block = ctx.fresh_block();
                let merge_block = ctx.fresh_block();

                let is_none = ctx.alloc_local("_oneq", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: is_none,
                    op: MirBinaryOp::Eq,
                    left: MirValue::Local(disc),
                    right: MirValue::Constant(MirConstant::I32(0)),
                });
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(is_none),
                    true_block: none_block.clone(),
                    false_block: some_block.clone(),
                });

                // None block: set disc=0, branch to merge
                ctx.current_block = MirBasicBlock::new(none_block);
                let rn_disc_ptr = ctx.alloc_local("_rndp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: rn_disc_ptr,
                    ptr: result_struct,
                    field_index: 0,
                    struct_type: Box::new(target_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: rn_disc_ptr,
                    value: MirValue::Constant(MirConstant::I32(0)),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                // Some block: extract payload, access property, wrap in Some
                ctx.current_block = MirBasicBlock::new(some_block);
                let src_payload_ptr = ctx.alloc_local("_spp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: src_payload_ptr,
                    ptr: target_local,
                    field_index: 1,
                    struct_type: Box::new(target_type.clone()),
                });
                let payload_val = ctx.alloc_local("_spv", MirType::I64);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: payload_val,
                    src: src_payload_ptr,
                });

                // Access property on the inner struct
                // Strategy: look for the inner struct either by name (mangled) or by field search
                let result_payload = if let MirType::Struct(struct_name, _) = &target_type {
                    // Try mangled name first: "Option__Person" → "Person"
                    let inner_by_name = struct_name.split("__").nth(1)
                        .and_then(|n| ctx.struct_defs.get(n))
                        .and_then(|fields| fields.iter().position(|(fn_, _)| fn_ == property))
                        .map(|field_idx| (struct_name.split("__").nth(1).unwrap().to_string(), field_idx));

                    // Try search-based approach: find any struct that has this field
                    let inner_by_search = ctx.struct_defs.iter()
                        .find(|(_, fields)| fields.iter().any(|(fn_, _)| fn_ == property))
                        .and_then(|(name, fields)| {
                            fields.iter().position(|(fn_, _)| fn_ == property)
                                .map(|field_idx| (name.clone(), field_idx))
                        });

                    if let Some((inner_name, field_idx)) = inner_by_name.or(inner_by_search) {
                        if let Some(inner_fields) = ctx.struct_defs.get(&inner_name) {
                            let field_type = inner_fields[field_idx].1.clone();
                            let inner_mir = MirType::Struct(inner_name, inner_fields.clone());
                            let struct_val = ctx.alloc_local("_och_s", inner_mir.clone());
                            ctx.current_block.insts.push(MirInst::Cast {
                                dest: struct_val,
                                value: MirValue::Local(payload_val),
                                to_type: inner_mir.clone(),
                            });
                            let field_ptr = ctx.alloc_local("_och_fp", field_type.clone());
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: field_ptr,
                                ptr: struct_val,
                                field_index: field_idx,
                                struct_type: Box::new(inner_mir),
                            });
                            let field_val = ctx.alloc_local("_och_fv", field_type.clone());
                            ctx.current_block.insts.push(MirInst::Load {
                                dest: field_val,
                                src: field_ptr,
                            });
                            if field_type == MirType::Str {
                                ctx.string_locals.push(field_val);
                            }
                            if field_type != MirType::I64 {
                                let casted = ctx.alloc_local("_och_c", MirType::I64);
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: casted,
                                    value: MirValue::Local(field_val),
                                    to_type: MirType::I64,
                                });
                                MirValue::Local(casted)
                            } else {
                                MirValue::Local(field_val)
                            }
                        } else {
                            MirValue::Local(payload_val)
                        }
                    } else {
                        MirValue::Local(payload_val)
                    }
                } else {
                    MirValue::Local(payload_val)
                };

                // Set disc=1 and store payload
                let r_disc_ptr = ctx.alloc_local("_rdp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: r_disc_ptr,
                    ptr: result_struct,
                    field_index: 0,
                    struct_type: Box::new(target_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: r_disc_ptr,
                    value: MirValue::Constant(MirConstant::I32(1)),
                });
                let r_payload_ptr = ctx.alloc_local("_rpp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: r_payload_ptr,
                    ptr: result_struct,
                    field_index: 1,
                    struct_type: Box::new(target_type.clone()),
                });
                ctx.current_block.insts.push(MirInst::Store {
                    dest: r_payload_ptr,
                    value: result_payload,
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                ctx.current_block = MirBasicBlock::new(merge_block);
                // Load result struct as the expression's final value
                let result_type = ctx.local_types.get(&result_struct).cloned()
                    .unwrap_or(target_type.clone());
                let result_val = ctx.alloc_local("_och_res", result_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result_val,
                    src: result_struct,
                });
                // Ensure string tracking for Option<T> where T=Str

                ctx
            }
            Expr::ErrorProp { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                let result_local = ctx.next_local - 1;
                let result_type = ctx.local_types.get(&result_local).cloned()
                    .unwrap_or(MirType::Struct("Result".to_string(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]));

                // Get discriminant (field 0)
                let disc_ptr = ctx.alloc_local("_edp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr,
                    ptr: result_local,
                    field_index: 0,
                    struct_type: Box::new(result_type.clone()),
                });
                let disc = ctx.alloc_local("_edv", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc,
                    src: disc_ptr,
                });

                // Check if disc == 0 (ok/success) — ok() sets disc=0, error() sets disc=1
                let is_ok = ctx.alloc_local("_eeq", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: is_ok,
                    op: MirBinaryOp::Eq,
                    left: MirValue::Local(disc),
                    right: MirValue::Constant(MirConstant::I32(0)),
                });

                let error_block = ctx.fresh_block();
                let continue_block = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(is_ok),
                    true_block: continue_block.clone(),
                    false_block: error_block.clone(),
                });

                // Error block: early-return the error if fallible
                ctx.current_block = MirBasicBlock::new(error_block);
                if ctx.is_fallible {
                    ctx.emit_return(MirValue::Local(result_local));
                } else {
                    ctx.finish_block(MirTerminator::Unreachable);
                }

                // Continue block: extract payload (field 1)
                ctx.current_block = MirBasicBlock::new(continue_block);
                let payload_ptr = ctx.alloc_local("_epp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: payload_ptr,
                    ptr: result_local,
                    field_index: 1,
                    struct_type: Box::new(result_type),
                });
                let payload = ctx.alloc_local("_epv", MirType::I64);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: payload,
                    src: payload_ptr,
                });
                ctx
            }
            Expr::StringInterp { parts, .. } => {
                let mut str_locals: Vec<usize> = Vec::new();
                for part in parts {
                    ctx = self.lower_expr(ctx, part);
                    let val_local = ctx.next_local - 1;
                    let is_str = ctx.local_types.get(&val_local).map_or(false, |t| *t == MirType::Str);
                    if is_str {
                        str_locals.push(val_local);
                    } else {
                        let cast_local = ctx.alloc_local("_cast64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: cast_local,
                            value: MirValue::Local(val_local),
                            to_type: MirType::I64,
                        });
                        let str_local = ctx.alloc_local("_strptr", MirType::Str);
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: Some(str_local),
                            name: "ky_i64_to_str".to_string(),
                            args: vec![MirValue::Local(cast_local)],
                        });
                        ctx.string_locals.push(str_local);
                        str_locals.push(str_local);
                    }
                }
                let mut result = str_locals[0];
                for i in 1..str_locals.len() {
                    let left = result;
                    let right = str_locals[i];
                    let left_len = ctx.alloc_local("_strlen", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(left_len),
                        name: "ky_strlen".to_string(),
                        args: vec![MirValue::Local(left)],
                    });
                    let right_len = ctx.alloc_local("_strlen", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(right_len),
                        name: "ky_strlen".to_string(),
                        args: vec![MirValue::Local(right)],
                    });
                    let new_result = ctx.alloc_local("_sinterp", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(new_result),
                        name: "ky_concat".to_string(),
                        args: vec![
                            MirValue::Local(left),
                            MirValue::Local(left_len),
                            MirValue::Local(right),
                            MirValue::Local(right_len),
                        ],
                    });
                    ctx.string_locals.push(new_result);
                    result = new_result;
                }
                ctx
            }
            Expr::SetLiteral { collection_name, elements, .. } => {
                let elem_type = elements.iter().find_map(|e| {
                    if let Expr::Literal { value: Literal::String(_), .. } = e { Some(MirType::Str) }
                    else { None }
                }).unwrap_or(MirType::I64);

                let (initializer_name, add_name, mir_type) = match collection_name.as_str() {
                    "set" => ("ky_set_new", "ky_set_add", MirType::Set(Box::new(elem_type.clone()))),
                    "queue" => ("ky_queue_new", "ky_queue_push", MirType::Queue(Box::new(elem_type.clone()))),
                    "stack" => ("ky_stack_new", "ky_stack_push", MirType::Stack(Box::new(elem_type.clone()))),
                    "deque" => ("ky_deque_new", "ky_deque_push_back", MirType::Deque(Box::new(elem_type.clone()))),
                    "linked_list" => ("ky_linked_list_new", "ky_linked_list_push_back", MirType::LinkedList(Box::new(elem_type.clone()))),
                    _ => ("ky_set_new", "ky_set_add", MirType::Set(Box::new(elem_type.clone()))),
                };

                let handle = ctx.alloc_local(&format!("_{}", collection_name), mir_type.clone());
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(handle),
                    name: initializer_name.to_string(),
                    args: vec![],
                });
                for elem in elements {
                    ctx = self.lower_expr(ctx, elem);
                    let val = ctx.next_local - 1;
                    let val_i64 = ctx.alloc_local("_sval64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val),
                        to_type: MirType::I64,
                    });
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: None,
                        name: add_name.to_string(),
                        args: vec![MirValue::Local(handle), MirValue::Local(val_i64)],
                    });
                }
                let result = ctx.alloc_local(&format!("_{}v", collection_name), mir_type);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result,
                    value: MirValue::Local(handle),
                });
                ctx
            }
            Expr::List { elements, .. } => {
                let elem_type = elements.iter().find_map(|e| {
                    if let Expr::Literal { value: Literal::String(_), .. } = e { Some(MirType::Str) }
                    else { None }
                }).unwrap_or(MirType::I64);
                let handle = ctx.alloc_local("_list", MirType::List(Box::new(elem_type.clone())));
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(handle),
                    name: "ky_list_new".to_string(),
                    args: vec![],
                });
                for elem in elements {
                    // Handle spread operator: [...list, new_elem]
                    if let Expr::Spread { expression, .. } = elem {
                        ctx = self.lower_expr(ctx, expression);
                        let spread_val = ctx.next_local - 1;
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_list_extend".to_string(),
                            args: vec![MirValue::Local(handle), MirValue::Local(spread_val)],
                        });
                    } else {
                        ctx = self.lower_expr(ctx, elem);
                        let val = ctx.next_local - 1;
                        let val_i64 = ctx.alloc_local("_val64", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: val_i64,
                            value: MirValue::Local(val),
                            to_type: MirType::I64,
                        });
                        ctx.current_block.insts.push(MirInst::Call {
                            dest: None,
                            name: "ky_list_push".to_string(),
                            args: vec![MirValue::Local(handle), MirValue::Local(val_i64)],
                        });
                    }
                }
                let result = ctx.alloc_local("_listv", MirType::List(Box::new(elem_type)));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result,
                    value: MirValue::Local(handle),
                });
                ctx
            }
            Expr::Index { target, index, .. } => {
                // For arrays, avoid lowering the target (which generates unnecessary whole-array Load)
                let (target_val, arr_ptr, target_type) = if let Expr::Identifier { name, .. } = target.as_ref() {
                    if let Some(&local) = ctx.locals.get(name) {
                        let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                        // Only skip Load for arrays and slices (zero-copy GEP). For other types, use normal expression lowering.
                        if matches!(t, MirType::Array(_, _) | MirType::Slice(_)) {
                            // Use variable's alloca directly (no Load)
                            (local, local, t)
                        } else {
                            ctx = self.lower_expr(ctx, target);
                            let tv = ctx.next_local - 1;
                            (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                        }
                    } else {
                        ctx = self.lower_expr(ctx, target);
                        let tv = ctx.next_local - 1;
                        (tv, tv, ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32))
                    }
                } else {
                    ctx = self.lower_expr(ctx, target);
                    let tv = ctx.next_local - 1;
                    let t = ctx.local_types.get(&tv).cloned().unwrap_or(MirType::I32);
                    // For non-identity array targets (e.g., nested mat[i][j]), the value
                    // needs a temp alloca so ArrayElemPtr can GEP into it
                    if matches!(t, MirType::Array(_, _)) {
                        let arr_tmp = ctx.alloc_local("_arrtmp", t.clone());
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: arr_tmp,
                            value: MirValue::Local(tv),
                        });
                        (tv, arr_tmp, t)
                    } else {
                        (tv, tv, t)
                    }
                };
                ctx = self.lower_expr(ctx, index);
                let index_val = ctx.next_local - 1;
                let target_type = ctx.local_types.get(&target_val).cloned().unwrap_or(MirType::I32);
                if target_type == MirType::Str {
                    // String indexing: source[i] -> substr(source, i, 1) -> returns str
                    let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: idx_i64,
                        value: MirValue::Local(index_val),
                        to_type: MirType::I64,
                    });
                    let result = ctx.alloc_local("_substr", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_substr".to_string(),
                        args: vec![
                            MirValue::Local(target_val),
                            MirValue::Local(idx_i64),
                            MirValue::Constant(MirConstant::I64(1)),
                        ],
                    });
                    ctx.string_locals.push(result);
                    return ctx;
                }
                if matches!(&target_type, MirType::Array(_, _)) {
                    let et = match &target_type { MirType::Array(e, _) => *(e.clone()), _ => MirType::I32 };
                    let elem_ptr = ctx.alloc_local("_aelem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                        dest: elem_ptr,
                        ptr: arr_ptr,
                        index: MirValue::Local(index_val),
                        arr_type: Box::new(target_type.clone()),
                        elem_type: Box::new(et.clone()),
                    });
                    let loaded = ctx.alloc_local("_aelem_val", et);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: loaded,
                        src: elem_ptr,
                    });
                    return ctx;
                }
                // === SLICE INDEXING ===
                if let MirType::Slice(inner) = &target_type {
                    let et = *inner.clone();
                    let slice_type = MirType::Slice(inner.clone());
                    // Get ptr field (field 0) from slice struct
                    let ptr_field = ctx.alloc_local("_sptrf", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::FieldPtr {
                        dest: ptr_field,
                        ptr: target_val,
                        field_index: 0,
                        struct_type: Box::new(slice_type.clone()),
                    });
                    let base_ptr = ctx.alloc_local("_sbase", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: base_ptr,
                        src: ptr_field,
                    });
                    // GEP: base_ptr + index
                    let index_i64 = ctx.alloc_local("_si64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: index_i64,
                        value: MirValue::Local(index_val),
                        to_type: MirType::I64,
                    });
                    let elem_ptr = ctx.alloc_local("_selem_ptr", MirType::Ptr(Box::new(MirType::I8)));
                    let et_clone = et.clone();
                    ctx.current_block.insts.push(MirInst::PtrOffset {
                        dest: elem_ptr,
                        ptr: base_ptr,
                        index: MirValue::Local(index_i64),
                        elem_type: Box::new(et_clone),
                    });
                    let loaded = ctx.alloc_local("_selem_val", et);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: loaded,
                        src: elem_ptr,
                    });
                    return ctx;
                }
                if let MirType::Dict(_, v) = &target_type {
                    let key_arg = if matches!(ctx.local_types.get(&index_val), Some(MirType::Str)) {
                        MirValue::Local(index_val)
                    } else {
                        // Index must be a string for dict access
                        return ctx;
                    };
                    let result = ctx.alloc_local("_dict_idx", MirType::I64);

                    ctx.current_block.insts.push(MirInst::Call {
                        dest: Some(result),
                        name: "ky_dict_get".to_string(),
                        args: vec![MirValue::Local(target_val), key_arg],
                    });
                    let elem_type = v.as_ref().clone();
                    if elem_type == MirType::Str {
                        let str_res = ctx.alloc_local("_dict_idx_str", MirType::Str);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: str_res,
                            value: MirValue::Local(result),
                            to_type: MirType::Str,
                        });
                        ctx.string_locals.push(str_res);
                    } else if elem_type != MirType::I64 {
                        let casted = ctx.alloc_local("_dict_idx_cast", elem_type.clone());
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: casted,
                            value: MirValue::Local(result),
                            to_type: elem_type,

                        });
                    }
                    return ctx;
                }
                // Ptr type indexing: ptr[index] → PtrOffset + Load
                if matches!(target_type, MirType::Ptr(_)) {
                    let ptr_type = ctx.local_types.get(&target_val).cloned().unwrap_or(MirType::I64);
                    let elem_type = if target_type == MirType::I64 { MirType::I8 } else { MirType::I8 };
                    let elem_type2 = elem_type.clone();
                    let offset = ctx.alloc_local("_ptroff", ptr_type.clone());
                    ctx.current_block.insts.push(MirInst::PtrOffset {
                        dest: offset,
                        ptr: target_val,
                        index: MirValue::Local(index_val),
                        elem_type: Box::new(elem_type2),
                    });
                    let result = ctx.alloc_local("_ptrload", elem_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result,
                        src: offset,
                    });
                    return ctx;
                }
                // Struct index: obj[i] → obj.op_index(i)
                if let MirType::Struct(class_name, _) = &target_type {
                    let method_table = self.method_table.borrow();
                    if let Some(methods) = method_table.get(class_name) {
                        if let Some(mangled) = methods.get("op_index") {
                            let ret_type = self.fn_returns.borrow()
                                .get(mangled).cloned().unwrap_or(MirType::I64);
                            let dest = ctx.alloc_local("_opidx", ret_type.clone());
                            if ret_type == MirType::Str {
                                ctx.string_locals.push(dest);
                            }
                            ctx.current_block.insts.push(MirInst::Call {
                                dest: Some(dest),
                                name: mangled.clone(),
                                args: vec![MirValue::Local(target_val), MirValue::Local(index_val)],
                            });
                            return ctx;
                        }
                    }
                }
                let idx_i64 = ctx.alloc_local("_idx64", MirType::I64);
                ctx.current_block.insts.push(MirInst::Cast {
                    dest: idx_i64,
                    value: MirValue::Local(index_val),
                    to_type: MirType::I64,
                });
                let list_elem_type = match &target_type {
                    MirType::List(inner) => inner.as_ref().clone(),
                    _ => MirType::I64,
                };
                // kl_list_get always returns i64 (pointers for strings, values for ints)
                let result = ctx.alloc_local("_idx", MirType::I64);
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(result),
                    name: "ky_list_get".to_string(),
                    args: vec![MirValue::Local(target_val), MirValue::Local(idx_i64)],
                });
                if list_elem_type == MirType::Str {
                    let str_result = ctx.alloc_local("_idxstr", MirType::Str);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: str_result,
                        value: MirValue::Local(result),
                        to_type: MirType::Str,
                    });
                    ctx.string_locals.push(str_result);
                } else if matches!(list_elem_type, MirType::Struct(_, _)) {
                    // Struct pointer from list: inttoptr + load
                    let struct_result = ctx.alloc_local("_struct", list_elem_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: struct_result,
                        value: MirValue::Local(result),
                        to_type: list_elem_type,
                    });
                }
                ctx
            }
            Expr::Dictionary { entries, .. } => {
                // Determine value type from entries
                let val_type = entries.first()
                    .and_then(|(_, v)| {
                        if let Expr::Literal { value: Literal::String(_), .. } = v { Some(MirType::Str) }
                        else if let Expr::Literal { value: Literal::Integer(_), .. } = v { Some(MirType::I64) }
                        else { None }
                    })
                    .unwrap_or(MirType::I64);
                let dict_type = MirType::Dict(Box::new(MirType::Str), Box::new(val_type));
                // Allocate dict via runtime call (into ptr-typed temp, then store to typed alloca)
                let handle = ctx.alloc_local("_dict_raw", MirType::Ptr(Box::new(MirType::Void)));
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(handle),
                    name: "ky_dict_new".to_string(),
                    args: vec![],
                });
                // Insert each entry
                for (key_str, val_expr) in entries {
                    ctx = self.lower_expr(ctx, val_expr);
                    let val_local = ctx.next_local - 1;

                    let val_i64 = ctx.alloc_local("_dv", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: val_i64,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    ctx.current_block.insts.push(MirInst::Call {
                        dest: None,
                        name: "ky_dict_set".to_string(),
                        args: vec![
                            MirValue::Local(handle),
                            MirValue::Constant(MirConstant::String(key_str.clone())),

                            MirValue::Local(val_i64),
                        ],
                    });
                }
                // Allocate a correctly-typed local as the final result
                let result = ctx.alloc_local("_dict", dict_type);
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result,
                    value: MirValue::Local(handle),

                });
                ctx
            }
            Expr::StructLiteral { struct_name, type_args: _, fields, .. } => {
                let struct_defs = ctx.struct_defs.clone();
                // Check if this is a generic struct template
                let generic_struct: Option<StructDecl> = (!struct_defs.contains_key(struct_name.as_str()))
                    .then(|| {
                        let templates = self.generic_struct_templates.borrow();
                        templates.get(struct_name.as_str()).cloned()
                    })
                    .flatten();

                if let Some(tpl) = generic_struct {
                    // --- Generic struct: monomorphize on the fly ---
                    // First, lower all field expressions to get their concrete MIR types
                    let mut field_val_ids: Vec<usize> = Vec::new();
                    for (_, field_expr) in fields {
                        ctx = self.lower_expr(ctx, field_expr);
                        field_val_ids.push(ctx.next_local - 1);
                    }
                    // Infer type params from concrete field value types
                    let mut type_subst: std::collections::HashMap<String, MirType> = std::collections::HashMap::new();
                    for ((field_name, _), val_id) in fields.iter().zip(&field_val_ids) {
                        if let Some(tf) = tpl.fields.iter().find(|f| f.name == *field_name) {
                            if let Some(concrete_type) = ctx.local_types.get(val_id) {
                                for tp in &tpl.type_params {
                                    if is_type_ref(&tf.type_, &tp.name) {
                                        type_subst.entry(tp.name.clone()).or_insert_with(|| concrete_type.clone());
                                    }
                                }
                            }
                        }
                    }
                    // Build ordered type args matching template's type_params order
                    let type_args: Vec<MirType> = tpl.type_params.iter()
                        .map(|tp| type_subst.get(&tp.name).cloned().unwrap_or(MirType::I32))
                        .collect();
                    let concrete_name = make_concrete_name(struct_name, &type_args);
                    // Create concrete field types with substitution and register in struct_defs
                    let struct_defs = ctx.struct_defs.clone(); // Use struct_defs from ctx for field resolution
                    let concrete_fields: Vec<(String, MirType)> = tpl.fields.iter()
                        .map(|f| (f.name.clone(), ast_type_to_mir_with_subst(&f.type_, Some(&struct_defs), &type_subst)))
                        .collect();
                    // Register in Lowerer's struct_defs
                    self.struct_defs.borrow_mut().insert(concrete_name.clone(), concrete_fields.clone());
                    // Now create the struct alloca and store fields
                    let struct_type = MirType::Struct(concrete_name.clone(), concrete_fields.clone());
                    let struct_local = ctx.alloc_local("_slit", struct_type.clone());
                    for ((field_name, _), val_local) in fields.iter().zip(&field_val_ids) {
                        if let Some(field_idx) = concrete_fields.iter().position(|(fn_, _)| *fn_ == *field_name) {
                            let fptr = ctx.alloc_local("_sfptr", MirType::Void);
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: fptr,
                                ptr: struct_local,
                                field_index: field_idx,
                                struct_type: Box::new(MirType::Struct(concrete_name.clone(), concrete_fields.clone())),
                            });
                            let field_type = concrete_fields[field_idx].1.clone();
                            let cast_val = if field_type != ctx.local_types.get(val_local).cloned().unwrap_or(MirType::I64) {
                                let cast_local = ctx.alloc_local("_scast", field_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast_local,
                                    value: MirValue::Local(*val_local),
                                    to_type: field_type,
                                });
                                MirValue::Local(cast_local)
                            } else {
                                MirValue::Local(*val_local)
                            };
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: fptr,
                                value: cast_val,
                            });
                        }
                    }
                    let result = ctx.alloc_local("_sval", struct_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result,
                        src: struct_local,
                    });
                    result
                } else if let Some(def_fields) = struct_defs.get(struct_name.as_str()) {
                    // --- Non-generic struct ---
                    let def_fields = def_fields.clone();
                    let struct_type = MirType::Struct(struct_name.clone(), def_fields.clone());
                    let struct_local = ctx.alloc_local("_slit", struct_type.clone());
                    let mut provided_fields: std::collections::HashSet<String> = std::collections::HashSet::new();
                    for (field_name, field_expr) in fields {
                        ctx = self.lower_expr(ctx, field_expr);
                        let val_local = ctx.next_local - 1;
                        if let Some(field_idx) = def_fields.iter()
                            .position(|(fname, _)| fname == field_name)
                        {
                            provided_fields.insert(field_name.clone());
                            let fptr = ctx.alloc_local("_sfptr", MirType::Void);
                            ctx.current_block.insts.push(MirInst::FieldPtr {
                                dest: fptr,
                                ptr: struct_local,
                                field_index: field_idx,
                                struct_type: Box::new(MirType::Struct(struct_name.clone(), def_fields.clone())),
                            });
                            let field_type = def_fields[field_idx].1.clone();
                            let cast_val = if field_type != ctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I64) {
                                let cast_local = ctx.alloc_local("_scast", field_type.clone());
                                ctx.current_block.insts.push(MirInst::Cast {
                                    dest: cast_local,
                                    value: MirValue::Local(val_local),
                                    to_type: field_type,
                                });
                                MirValue::Local(cast_local)
                            } else {
                                MirValue::Local(val_local)
                            };
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: fptr,
                                value: cast_val,
                            });
                        }
                    }
                    // Apply field defaults for any missing fields
                    {
                        let fds = self.field_defaults.borrow();
                        if let Some(field_defaults) = fds.get(struct_name.as_str()) {
                            for (field_idx, (fn_name, _)) in def_fields.iter().enumerate() {
                                if !provided_fields.contains(fn_name) {
                                    if let Some(default_expr) = field_defaults.get(fn_name) {
                                        ctx = self.lower_expr(ctx, default_expr);
                                        let val_local = ctx.next_local - 1;
                                        let fptr = ctx.alloc_local("_sfptr", MirType::Void);
                                        ctx.current_block.insts.push(MirInst::FieldPtr {
                                            dest: fptr,
                                            ptr: struct_local,
                                            field_index: field_idx,
                                            struct_type: Box::new(MirType::Struct(struct_name.clone(), def_fields.clone())),
                                        });
                                        let field_type = def_fields[field_idx].1.clone();
                                        let cast_val = if field_type != ctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I64) {
                                            let cast_local = ctx.alloc_local("_scast", field_type.clone());
                                            ctx.current_block.insts.push(MirInst::Cast {
                                                dest: cast_local,
                                                value: MirValue::Local(val_local),
                                                to_type: field_type,
                                            });
                                            MirValue::Local(cast_local)
                                        } else {
                                            MirValue::Local(val_local)
                                        };
                                        ctx.current_block.insts.push(MirInst::Store {
                                            dest: fptr,
                                            value: cast_val,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    let result = ctx.alloc_local("_sval", struct_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: result,
                        src: struct_local,
                    });
                    result
                } else {
                    ctx.alloc_local("_slit_err", MirType::Void)
                };
                ctx
            }
            Expr::Tuple { elements, .. } => {
                if elements.len() <= 1 {
                    // Single-element tuple is just the element
                    if elements.is_empty() {
                        ctx
                    } else {
                        ctx = self.lower_expr(ctx, &elements[0]);
                        ctx
                    }
                } else {
                    // Multi-element tuple: build a struct
                    let mut elem_ids = Vec::new();
                    for elem in elements {
                        ctx = self.lower_expr(ctx, elem);
                        elem_ids.push(ctx.next_local - 1);
                    }
                    // Build struct type from element types
                    let field_types: Vec<(String, MirType)> = elem_ids.iter().enumerate()
                        .map(|(i, id)| (format!("_{}", i), ctx.local_types.get(id).cloned().unwrap_or(MirType::I32)))
                        .collect();
                    let type_suffix: String = field_types.iter()
                        .map(|(_, t)| match t {
                            MirType::I32 => "i32",
                            MirType::I64 => "i64",
                            MirType::Str => "str",
                            MirType::Bool => "bool",
                            MirType::F64 => "f64",
                            _ => "x",
                        })
                        .collect();
                    let struct_type = MirType::Struct(format!("_tuple_{}_{}", elements.len(), type_suffix), field_types.clone());
                    let struct_local = ctx.alloc_local("_tup", struct_type.clone());
                    for (i, elem_id) in elem_ids.iter().enumerate() {
                        let fptr = ctx.alloc_local("_tfptr", MirType::I64);
                        ctx.current_block.insts.push(MirInst::FieldPtr {
                            dest: fptr,
                            ptr: struct_local,
                            field_index: i,
                            struct_type: Box::new(struct_type.clone()),
                        });
                        ctx.current_block.insts.push(MirInst::Store {
                            dest: fptr,
                            value: MirValue::Local(*elem_id),
                        });
                    }
                    // Load the struct as the expression result (last local)
                    let load_local = ctx.alloc_local("_tload", struct_type.clone());
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: load_local,
                        src: struct_local,
                    });
                    ctx
                }
            }
            Expr::Closure { params, body, .. } => {
                let mut counter = self.closure_counter.borrow_mut();
                let fn_name = format!("_closure_{}", *counter);
                *counter += 1;
                drop(counter);

                let mut mir_func = MirFunction::new(&fn_name);
                // Use type annotations from AST, fall back to inference
                let param_types: Vec<MirType> = params.iter()
                    .map(|(p, t)| param_type_from_annotation(t)
                        .unwrap_or_else(|| infer_closure_param_type(p, body)))
                    .collect();
                mir_func.params = param_types.clone();
                mir_func.return_type = MirType::I32; // default, will be inferred

                let mut cctx = LowerCtx::new();
                cctx.struct_defs = ctx.struct_defs.clone();
                // Bind params to locals with inferred types
                for (i, (pname, _)) in params.iter().enumerate() {
                    let pt = param_types[i].clone();
                    let local = cctx.alloc_local(pname, pt.clone());
                    cctx.current_block.insts.push(MirInst::Store {
                        dest: local,
                        value: MirValue::Param(i),
                    });
                    cctx.locals.insert(pname.clone(), local);
                    // Record type for use in body lowering
                    cctx.local_types.insert(local, pt.clone());
                }
                // Lower body expression
                cctx = self.lower_expr(cctx, body);
                let val_local = cctx.next_local - 1;
                // Infer return type if needed
                if mir_func.return_type == MirType::I32 {
                    if let Some(actual_type) = cctx.local_types.get(&val_local) {
                        mir_func.return_type = actual_type.clone();
                    }
                }
                cctx.emit_return(MirValue::Local(val_local));
                mir_func.local_count = cctx.next_local;
                mir_func.basic_blocks = cctx.blocks;

                // Store the closure function
                self.closure_functions.borrow_mut().push(mir_func);

                // Emit FnAddr to get the function pointer
                let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                let dest = ctx.alloc_local("_closure", ptr_type);
                ctx.current_block.insts.push(MirInst::FnAddr {
                    dest,
                    name: fn_name,
                });
                ctx
            }
            Expr::Await { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                let handle_local = ctx.next_local - 1;
                // Determine the actual return type (what the async fn declared)
                let return_type: MirType = match expression.as_ref() {
                    Expr::FunctionCall { target, .. } => {
                        match target.as_ref() {
                            Expr::Identifier { name, .. } => {
                                self.async_fn_returns.borrow().get(name.as_str())
                                    .cloned()
                                    .or_else(|| self.fn_returns.borrow().get(name.as_str()).cloned())
                                    .unwrap_or(MirType::I64)
                            }
                            _ => MirType::I64,
                        }
                    }
                    _ => MirType::I64,
                };
                // Allocate result with the declared return type (handles are i64,
                // but await extracts the actual value)
                let result = ctx.alloc_local("_await", return_type.clone());
                ctx.current_block.insts.push(MirInst::AsyncAwait {
                    dest: result,
                    handle: handle_local,
                    return_type,
                });
                ctx
            }
            Expr::Async { expression, .. } => {
                let mut counter = self.async_counter.borrow_mut();
                let fn_name = format!("_async_{}", *counter);
                *counter += 1;
                drop(counter);

                let mut mir_func = MirFunction::new(&fn_name);
                mir_func.params = vec![];
                mir_func.return_type = MirType::I64;

                let mut cctx = LowerCtx::new();
                cctx.struct_defs = self.struct_defs.borrow().clone();
                cctx = self.lower_expr(cctx, expression);
                let val_local = cctx.next_local - 1;
                // Widen to i64
                let val_type = cctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                let ret_local = if val_type != MirType::I64 {
                    let widened = cctx.alloc_local("_widen", MirType::I64);
                    cctx.current_block.insts.push(MirInst::Cast {
                        dest: widened,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    widened
                } else {
                    val_local
                };
                cctx.emit_return(MirValue::Local(ret_local));
                mir_func.local_count = cctx.next_local;
                mir_func.basic_blocks = cctx.blocks;

                self.async_functions.borrow_mut().push(mir_func);

                let dest = ctx.alloc_local("_async_h", MirType::I64);
                ctx.current_block.insts.push(MirInst::AsyncSpawn {
                    dest,
                    function_name: fn_name,
                    arg: MirValue::Constant(MirConstant::I64(0)),
                });
                ctx
            }
            Expr::AsyncBlock { body, .. } => {
                // async: block — generate a zero-param function and spawn it
                let mut counter = self.async_counter.borrow_mut();
                let fn_name = format!("_async_block_{}", *counter);
                *counter += 1;
                drop(counter);

                let mut mir_func = MirFunction::new(&fn_name);
                mir_func.params = vec![];
                mir_func.return_type = MirType::I64;

                let mut cctx = LowerCtx::new();
                cctx.struct_defs = self.struct_defs.borrow().clone();

                let last_is_tail = match body.statements.last() {
                    Some(Stmt::Expression(e)) => !matches!(e, Expr::Assignment { .. }),
                    _ => false,
                };
                let stmt_count = body.statements.len();
                for (i, stmt) in body.statements.iter().enumerate() {
                    if i + 1 == stmt_count {
                        if let Stmt::If(_) = stmt {
                            cctx.tail_if_as_return = true;
                        }
                    }
                    cctx = self.lower_stmt(cctx, stmt);
                }

                if cctx.current_block.terminator == MirTerminator::Unreachable {
                    if last_is_tail {
                        let val_local = cctx.next_local.checked_sub(1);
                        if let Some(vl) = val_local {
                            let vt = cctx.local_types.get(&vl).cloned().unwrap_or(MirType::I32);
                            if vt != MirType::I64 {
                                let w = cctx.alloc_local("_bw", MirType::I64);
                                cctx.current_block.insts.push(MirInst::Cast {
                                    dest: w, value: MirValue::Local(vl), to_type: MirType::I64,
                                });
                                cctx.emit_return(MirValue::Local(w));
                            } else {
                                cctx.emit_return(MirValue::Local(vl));
                            }
                        } else {
                            cctx.emit_return(MirValue::Constant(MirConstant::I64(0)));
                        }
                    } else {
                        cctx.emit_return(MirValue::Constant(MirConstant::I64(0)));
                    }
                }
                mir_func.local_count = cctx.next_local;
                mir_func.basic_blocks = cctx.blocks;
                self.async_functions.borrow_mut().push(mir_func);

                let dest = ctx.alloc_local("_async_h", MirType::I64);
                ctx.current_block.insts.push(MirInst::AsyncSpawn {
                    dest, function_name: fn_name,
                    arg: MirValue::Constant(MirConstant::I64(0)),
                });
                ctx
            }
            Expr::Spread { expression, .. } => {
                ctx = self.lower_expr(ctx, expression);
                ctx
            }
            Expr::RangeSlice { target, start, end, .. } => {
                // For arrays, don't lower expression (use alloca directly)
                let (target_val, target_type) = if let Expr::Identifier { name, .. } = target.as_ref() {
                    if let Some(&local) = ctx.locals.get(name) {
                        let t = ctx.local_types.get(&local).cloned().unwrap_or(MirType::I32);
                        if matches!(t, MirType::Array(_, _)) {
                            (local, t)
                        } else {
                            ctx = self.lower_expr(ctx, target);
                            (ctx.next_local - 1, ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32))
                        }
                    } else {
                        ctx = self.lower_expr(ctx, target);
                        (ctx.next_local - 1, ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32))
                    }
                } else {
                    ctx = self.lower_expr(ctx, target);
                    (ctx.next_local - 1, ctx.local_types.get(&(ctx.next_local - 1)).cloned().unwrap_or(MirType::I32))
                };
                let start_i64 = if let Some(s) = start {
                    ctx = self.lower_expr(ctx, s);
                    let val = ctx.next_local - 1;
                    let cast = ctx.alloc_local("_sli64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(val),
                        to_type: MirType::I64,
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Constant(MirConstant::I64(0))
                };
                let end_i64 = if let Some(e) = end {
                    ctx = self.lower_expr(ctx, e);
                    let val = ctx.next_local - 1;
                    let cast = ctx.alloc_local("_eli64", MirType::I64);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: cast,
                        value: MirValue::Local(val),
                        to_type: MirType::I64,
                    });
                    MirValue::Local(cast)
                } else {
                    MirValue::Constant(MirConstant::I64(-1))
                };
                // Handle array ranges: produce &[T] slice
                if let MirType::Array(inner, _) = &target_type {
                    // Get element pointer via ArrayElemPtr
                    let start_i32 = ctx.alloc_local("_sli32", MirType::I32);
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: start_i32,
                        value: start_i64.clone(),
                        to_type: MirType::I32,
                    });
                    let elem_ptr = ctx.alloc_local("_sptr", MirType::Ptr(Box::new(MirType::I8)));
                    ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                        dest: elem_ptr,
                        ptr: target_val,
                        index: MirValue::Local(start_i32),
                        arr_type: Box::new(target_type.clone()),
                        elem_type: inner.clone(),
                    });
                    // Compute length: end - start (or arr_len for full slice)
                    let len = ctx.alloc_local("_slen", MirType::I64);
                    let len_i64 = if let Some(_) = end {
                        ctx.current_block.insts.push(MirInst::BinaryOp {
                            dest: len,
                            op: MirBinaryOp::Sub,
                            left: end_i64,
                            right: start_i64.clone(),
                        });
                        MirValue::Local(len)
                    } else if let MirType::Array(_, size) = &target_type {
                        MirValue::Constant(MirConstant::I64(*size as i64))
                    } else {
                        MirValue::Constant(MirConstant::I64(-1))
                    };
                    let result = ctx.alloc_local("_slice", MirType::Slice(inner.clone()));
                    ctx.current_block.insts.push(MirInst::SliceMake {
                        dest: result,
                        ptr: MirValue::Local(elem_ptr),
                        len: len_i64,
                        elem_type: inner.clone(),
                    });
                    return ctx;
                }
                let result = ctx.alloc_local("_slice", MirType::List(Box::new(MirType::I64)));
                ctx.current_block.insts.push(MirInst::Call {
                    dest: Some(result),
                    name: "ky_list_slice".to_string(),
                    args: vec![
                        MirValue::Local(target_val),
                        start_i64,
                        end_i64,
                    ],
                });
                ctx
            }
            Expr::Loop { body, .. } => {
                let loop_label = ctx.fresh_block();
                let end_label = ctx.fresh_block();
                let loop_label2 = loop_label.clone();
                ctx.finish_block(MirTerminator::Br(loop_label2.clone()));
                ctx.current_block = MirBasicBlock::new(loop_label);
                ctx.break_targets.push(end_label.clone());
                ctx.continue_targets.push(loop_label2.clone());
                for stmt in &body.statements {
                    ctx = self.lower_stmt(ctx, stmt);
                }
                ctx.break_targets.pop();
                ctx.continue_targets.pop();
                ctx.finish_block(MirTerminator::Br(loop_label2.clone()));
                ctx.current_block = MirBasicBlock::new(end_label);
                ctx
            }
            Expr::IfExpr { .. } => unreachable!("desugared in HIR"),
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                let result_local = ctx.alloc_local("_tern_res", MirType::I64);
                ctx = self.lower_expr(ctx, cond);
                let cond_val = MirValue::Local(ctx.next_local - 1);
                let then_block = ctx.fresh_block();
                let else_block = ctx.fresh_block();
                let merge_block = ctx.fresh_block();
                ctx.finish_block(MirTerminator::CondBr {
                    cond: cond_val,
                    true_block: then_block.clone(),
                    false_block: else_block.clone(),
                });
                // Then block
                ctx.current_block = MirBasicBlock::new(then_block.clone());
                ctx = self.lower_expr(ctx, then_expr);
                let then_val = ctx.next_local - 1;
                let then_type = ctx.local_types.get(&then_val).cloned().unwrap_or(MirType::I64);
                ctx.local_types.insert(result_local, then_type.clone());
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(then_val),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                // Else block
                ctx.current_block = MirBasicBlock::new(else_block.clone());
                ctx = self.lower_expr(ctx, else_expr);
                let else_val = ctx.next_local - 1;
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(else_val),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                // Merge block — load result as the expression's return value
                ctx.current_block = MirBasicBlock::new(merge_block);
                let result = ctx.alloc_local("_tern_val", then_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result,
                    src: result_local,
                });
                ctx
            }
            Expr::MatchExpr { expression, arms, .. } => {
                // Lower the matched expression
                ctx = self.lower_expr(ctx, expression);
                let match_val = ctx.next_local - 1;
                let result_local = ctx.alloc_local("_match_res", MirType::I64);
                let mut arm_types: Vec<MirType> = Vec::new();
                let merge_block = ctx.fresh_block();
                let arm_count = arms.len();
                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = ctx.fresh_block();
                    let is_last = i == arm_count - 1;
                    let next_target = if is_last {
                        merge_block.clone()
                    } else {
                        ctx.fresh_block()
                    };
                    match &arm.pattern {
                        Pattern::Tuple { .. } | Pattern::Range { .. } | Pattern::IsType { .. } | Pattern::Wildcard { .. } | Pattern::Identifier { .. } => {
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
                            // Bind identifier pattern
                            if let Pattern::Identifier { name, .. } = &arm.pattern {
                                let local = ctx.alloc_local(name, MirType::I64);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: local,
                                    value: MirValue::Local(match_val),
                                });
                                ctx.locals.insert(name.clone(), local);
                            }
                            // Lower body, store last expression to result
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type.clone());
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            // After Wildcard, no more arms
                            ctx.current_block = MirBasicBlock::new(merge_block.clone());
                            let result_type = last_type;
                            ctx.local_types.insert(result_local, result_type);
                            break;
                        }
                        Pattern::Literal { value, .. } => {
                            let (vt, lc) = literal_to_mir(value);
                            let lit = ctx.alloc_local("_lit", vt);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: lit, value: MirValue::Constant(lc),
                            });
                            let eq = ctx.alloc_local("_eq", MirType::Bool);
                            ctx.current_block.insts.push(MirInst::BinaryOp {
                                dest: eq, op: MirBinaryOp::Eq,
                                left: MirValue::Local(match_val),
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
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::Or { .. } => {
                            ctx.finish_block(MirTerminator::Br(arm_label.clone()));
                            ctx.current_block = MirBasicBlock::new(arm_label);
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        Pattern::EnumVariant { enum_name, variant, args, .. } => {
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
                            let mv_type = ctx.local_types.get(&match_val).cloned().unwrap_or(MirType::I64);
                            let struct_ptr = if matches!(mv_type, MirType::Struct(_, _)) {
                                let alloca = ctx.alloc_local("_mvtmp", mv_type);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: alloca,
                                    value: MirValue::Local(match_val),
                                });
                                alloca
                            } else {
                                match_val
                            };
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
                            let idx_local = ctx.alloc_local("_vidx", MirType::I32);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: idx_local,
                                value: MirValue::Constant(MirConstant::I32(variant_idx as i32)),
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
                            if !args.is_empty() {
                                let payload_ptr = ctx.alloc_local("_pay_ptr", MirType::I64);
                                ctx.current_block.insts.push(MirInst::FieldPtr {
                                    dest: payload_ptr,
                                    ptr: struct_ptr,
                                    field_index: 1,
                                    struct_type: Box::new(struct_type),
                                });
                                for arg in args.iter() {
                                    if let Pattern::Identifier { name, .. } = arg {
                                        let val = ctx.alloc_local(name, MirType::I64);
                                        ctx.current_block.insts.push(MirInst::Load {
                                            dest: val,
                                            src: payload_ptr,
                                        });
                                        ctx.locals.insert(name.clone(), val);
                                    }
                                }
                            }
                            for stmt in &arm.body.statements {
                                ctx = self.lower_stmt(ctx, stmt);
                            }
                            let last_val = ctx.next_local - 1;
                            let last_type = ctx.local_types.get(&last_val).cloned().unwrap_or(MirType::I64);
                            arm_types.push(last_type.clone());
                            ctx.local_types.insert(result_local, last_type);
                            ctx.current_block.insts.push(MirInst::Store {
                                dest: result_local,
                                value: MirValue::Local(last_val),
                            });
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            if !is_last {
                                ctx.current_block = MirBasicBlock::new(next_target);
                            }
                        }
                        _ => {
                            // Unsupported pattern — fall through
                            ctx.finish_block(MirTerminator::Br(merge_block.clone()));
                            ctx.current_block = MirBasicBlock::new(merge_block.clone());
                            break;
                        }
                    }
                }
                // Set current block to merge
                ctx.current_block = MirBasicBlock::new(merge_block);
                let res_type = arm_types.first().cloned().unwrap_or(MirType::I64);
                let result = ctx.alloc_local("_match_val", res_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result,
                    src: result_local,
                });
                ctx
            }
            Expr::BorrowRef { expression, .. } => {
                if let Expr::Identifier { name, .. } = expression.as_ref() {
                    // Check if this is a reference to a module-level function
                    if self.fn_returns.borrow().contains_key(name) {
                        let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                        let dest = ctx.alloc_local("_fnaddr", ptr_type);
                        ctx.current_block.insts.push(MirInst::FnAddr {
                            dest,
                            name: name.clone(),
                        });
                        return ctx;
                    }
                    if let Some(&local_id) = ctx.locals.get(name) {
                        // For ^T (ref param) locals: load the stored pointer and pass it through
                        if ctx.ref_param_locals.contains(&local_id) {
                            let dest = ctx.alloc_local("_addr", MirType::Ptr(Box::new(MirType::I8)));
                            ctx.current_block.insts.push(MirInst::Load {
                                dest,
                                src: local_id,
                            });
                            return ctx;
                        }
                        let local_ty = ctx.local_types.get(&local_id).cloned();
                        if let Some(t) = local_ty {
                            let is_heap_type = matches!(t,
                                MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_)
                            );
                            if is_heap_type {
                                let dest = ctx.alloc_local("_addr", t.clone());
                                ctx.current_block.insts.push(MirInst::Load {
                                    dest, src: local_id,
                                });
                                return ctx;
                            }
                            if let MirType::Array(elem_type, size) = t {
                                let zero_i32 = ctx.alloc_local("_zero", MirType::I32);
                                ctx.current_block.insts.push(MirInst::Store {
                                    dest: zero_i32,
                                    value: MirValue::Constant(MirConstant::I32(0)),
                                });
                                let elem_ptr = ctx.alloc_local("_arr0", MirType::Ptr(Box::new(MirType::I8)));
                                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                                    dest: elem_ptr,
                                    ptr: local_id,
                                    index: MirValue::Local(zero_i32),
                                    arr_type: Box::new(MirType::Array(elem_type.clone(), size)),
                                    elem_type: elem_type.clone(),
                                });
                                let slice_local = ctx.alloc_local("_arr_slice", MirType::Slice(elem_type.clone()));
                                ctx.current_block.insts.push(MirInst::SliceMake {
                                    dest: slice_local,
                                    ptr: MirValue::Local(elem_ptr),
                                    len: MirValue::Constant(MirConstant::I64(size as i64)),
                                    elem_type: elem_type.clone(),
                                });
                                return ctx;
                            }
                        }
                        let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                        let dest = ctx.alloc_local("_addr", ptr_type);
                        ctx.current_block.insts.push(MirInst::AddressOf {
                            dest,
                            local_id,
                        });
                        return ctx;
                    }
                }
                ctx = self.lower_expr(ctx, expression);
                let inner_local = ctx.next_local - 1;
                // For str, list, dict, set: borrow passes value directly
                if let Some(t) = ctx.local_types.get(&inner_local) {
                    let is_heap_type = matches!(t,
                        MirType::Str | MirType::List(_) | MirType::Dict(_, _) | MirType::Set(_)
                    );
                    if is_heap_type {
                        return ctx;
                    }
                }
                let ptr_type = MirType::Ptr(Box::new(MirType::I8));
                let dest = ctx.alloc_local("_addr", ptr_type);
                ctx.current_block.insts.push(MirInst::AddressOf {
                    dest,
                    local_id: inner_local,
                });
                ctx
            }
            Expr::NullCoalesce { left, right, .. } => {
                // Lower left expression (Option<T>)
                ctx = self.lower_expr(ctx, left);
                let left_local = ctx.next_local - 1;
                let left_type = ctx.local_types.get(&left_local).cloned()
                    .unwrap_or(MirType::Struct("Result".to_string(), vec![
                        ("disc".to_string(), MirType::I32),
                        ("payload".to_string(), MirType::I64),
                    ]));

                // Determine inner type T from Option__T struct name
                let inner_type = if let MirType::Struct(name, _) = &left_type {
                    extract_inner_type(name)
                } else {
                    MirType::I32
                };

                // Allocate result local
                let result_local = ctx.alloc_local("_ncres", inner_type.clone());

                // Allocate discriminant pointer
                let disc_ptr = ctx.alloc_local("_ncdp", MirType::I32);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: disc_ptr,
                    ptr: left_local,
                    field_index: 0,
                    struct_type: Box::new(left_type.clone()),
                });
                let disc_val = ctx.alloc_local("_ncdv", MirType::I32);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: disc_val,
                    src: disc_ptr,
                });

                let some_block = ctx.fresh_block();
                let none_block = ctx.fresh_block();
                let merge_block = ctx.fresh_block();

                // Check if disc != 0 (Some)
                let is_some = ctx.alloc_local("_ncis", MirType::Bool);
                ctx.current_block.insts.push(MirInst::BinaryOp {
                    dest: is_some,
                    op: MirBinaryOp::Neq,
                    left: MirValue::Local(disc_val),
                    right: MirValue::Constant(MirConstant::I32(0)),
                });
                ctx.finish_block(MirTerminator::CondBr {
                    cond: MirValue::Local(is_some),
                    true_block: some_block.clone(),
                    false_block: none_block.clone(),
                });

                // Some block: extract payload, cast to inner type, store
                ctx.current_block = MirBasicBlock::new(some_block);
                let payload_ptr = ctx.alloc_local("_ncpp", MirType::I64);
                ctx.current_block.insts.push(MirInst::FieldPtr {
                    dest: payload_ptr,
                    ptr: left_local,
                    field_index: 1,
                    struct_type: Box::new(left_type.clone()),
                });
                let payload_val = ctx.alloc_local("_ncpv", MirType::I64);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: payload_val,
                    src: payload_ptr,
                });
                // Cast payload I64 → inner_type if needed
                let some_result = if inner_type != MirType::I64 {
                    let casted = ctx.alloc_local("_ncc", inner_type.clone());
                    ctx.current_block.insts.push(MirInst::Cast {
                        dest: casted,
                        value: MirValue::Local(payload_val),
                        to_type: inner_type.clone(),
                    });
                    casted
                } else {
                    payload_val
                };
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(some_result),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                // None block: evaluate right expression, store
                ctx.current_block = MirBasicBlock::new(none_block);
                ctx = self.lower_expr(ctx, right);
                let right_val = ctx.next_local - 1;
                ctx.current_block.insts.push(MirInst::Store {
                    dest: result_local,
                    value: MirValue::Local(right_val),
                });
                ctx.finish_block(MirTerminator::Br(merge_block.clone()));

                // Merge block: load result
                ctx.current_block = MirBasicBlock::new(merge_block);
                let result_val = ctx.alloc_local("_ncresv", inner_type);
                ctx.current_block.insts.push(MirInst::Load {
                    dest: result_val,
                    src: result_local,
                });
                ctx
            }
        }
    }

    /// Collect root identifier and all index expressions from a nested Index chain.
    /// e.g., m[i][j] → Some(("m", [i_expr, j_expr]))
    pub(crate) fn collect_array_indices<'a>(&self, expr: &'a Expr) -> Option<(String, Vec<&'a Expr>)> {
        match expr {
            Expr::Index { target, index, .. } => {
                let mut result = self.collect_array_indices(target)?;
                result.1.push(index);
                Some(result)
            }
            Expr::Identifier { name, .. } => Some((name.clone(), vec![])),
            _ => None,
        }
    }

    /// Given a root array local and a list of index expressions (innermost first),
    /// compute the final element pointer via nested GEP into the root array.
    /// Uses the original Expr::Index lowering logic which correctly handles array
    /// identifiers by using their alloca directly.
    pub(crate) fn lower_nested_array_geps(&self, mut ctx: LowerCtx, idx_exprs: &[&Expr], root_local: usize) -> LowerCtx {
        let mut ptr = root_local;
        let mut cur_type = ctx.local_types.get(&root_local).cloned().unwrap_or(MirType::I32);
        for idx_expr in idx_exprs {
            ctx = self.lower_expr(ctx, idx_expr);
            let idx_val = ctx.next_local - 1;
            if let MirType::Array(ref inner, _) = cur_type {
                let elem_ptr = ctx.alloc_local("_nested_aep", MirType::Ptr(Box::new(MirType::I8)));
                ctx.current_block.insts.push(MirInst::ArrayElemPtr {
                    dest: elem_ptr,
                    ptr: ptr,
                    index: MirValue::Local(idx_val),
                    arr_type: Box::new(cur_type.clone()),
                    elem_type: Box::new(inner.as_ref().clone()),
                });
                ptr = elem_ptr;
                cur_type = inner.as_ref().clone();
            }
        }
        ctx
    }
}
