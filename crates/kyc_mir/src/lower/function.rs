use crate::mir::*;
use kyc_core::ast::*;
use super::*;

impl super::Lowerer {
    pub(crate) fn collect_class_fields(
        c: &ClassDecl,
        program: &Program,
        struct_defs: &std::collections::HashMap<String, Vec<(String, MirType)>>,
    ) -> Vec<(String, MirType)> {
        let mut fields = Vec::new();
        if let Some(ref parent_name) = c.parent {
            if let Some(parent_fields) = struct_defs.get(parent_name).filter(|f| !f.is_empty()) {
                fields.extend(parent_fields.clone());
            } else {
                for decl in &program.declarations {
                    if let Decl::Class(pc) = decl {
                        if &pc.name == parent_name {
                            fields.extend(Self::collect_class_fields(pc, program, struct_defs));
                            break;
                        }
                    }
                }
            }
        }
        for m in &c.members {
            if let ClassMember::Field(f) = m {
                fields.push((f.name.clone(), ast_type_to_mir(&f.type_, Some(struct_defs))));
            }
        }
        fields
    }

    /// Walk a class's inheritance chain to resolve a method call.  Returns the
    /// mangled `Class::method` name of the most-derived class that declares the
    /// method (or None if no ancestor declares it).  This is what gives us
    /// polymorphism: subclasses override inherited methods by simply shadowing
    /// the entry in `method_table`.
    pub(crate) fn lookup_method_in_chain(
        &self,
        class_name: &str,
        method_name: &str,
        method_table: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
        parent_map: &std::collections::HashMap<String, Option<String>>,
    ) -> Option<String> {
        let mut current = class_name.to_string();
        loop {
            if let Some(methods) = method_table.get(&current) {
                if let Some(mangled) = methods.get(method_name) {
                    return Some(mangled.clone());
                }
            }
            match parent_map.get(&current) {
                Some(Some(parent)) => current = parent.clone(),
                _ => break,
            }
        }
        None
    }

    /// Lower an `async fn` declaration. Generates two MIR functions:
    ///   1. `_async_body_{name}` — the actual body (returns i64)
    ///   2. `{name}` — the wrapper that spawns and returns the task handle
    pub(crate) fn lower_async_fn(&self, f: &FunctionDecl, body: &Block) -> Option<MirFunction> {
        let struct_defs = self.struct_defs.borrow().clone();
        let body_fn_name = format!("_async_body_{}", f.name);
        let real_params: Vec<&Parameter> = f.params.iter()
            .filter(|p| !(p.name == "this" || p.name == "self"))
            .collect();
        let param_count = real_params.len();

        // === 1. Generate the async body function ===
        let mut body_func = MirFunction::new(&body_fn_name);
        body_func.params = vec![MirType::I64]; // receives args_ptr as i64
        body_func.return_type = MirType::I64;

        let mut cctx = LowerCtx::new();
        cctx.struct_defs = struct_defs.clone();

        if param_count == 0 {
            // No params — ignore the arg
        } else if param_count == 1 {
            // Single param: pass value directly as i64
            let ptype = ast_type_to_mir(&real_params[0].type_, Some(&struct_defs));
            let raw_val = cctx.alloc_local("_arg0", ptype.clone());
            cctx.current_block.insts.push(MirInst::Store {
                dest: raw_val,
                value: MirValue::Param(0),
            });
            if ptype != MirType::I64 {
                let cast_local = cctx.alloc_local(&real_params[0].name, ptype.clone());
                cctx.current_block.insts.push(MirInst::Cast {
                    dest: cast_local,
                    value: MirValue::Local(raw_val),
                    to_type: ptype,
                });
                cctx.locals.insert(real_params[0].name.clone(), cast_local);
            } else {
                cctx.locals.insert(real_params[0].name.clone(), raw_val);
            }
        }

        // Lower the body statements
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

        // Emit return — widen tail value to i64
        if cctx.current_block.terminator == MirTerminator::Unreachable {
            let tail_local = if last_is_tail {
                cctx.next_local.checked_sub(1)
            } else { None };
            if let Some(val_local) = tail_local {
                let val_type = cctx.local_types.get(&val_local).cloned().unwrap_or(MirType::I32);
                if val_type != MirType::I64 {
                    let widened = cctx.alloc_local("_aw", MirType::I64);
                    cctx.current_block.insts.push(MirInst::Cast {
                        dest: widened,
                        value: MirValue::Local(val_local),
                        to_type: MirType::I64,
                    });
                    cctx.emit_return(MirValue::Local(widened));
                } else {
                    cctx.emit_return(MirValue::Local(val_local));
                }
            } else {
                cctx.emit_return(MirValue::Constant(MirConstant::I64(0)));
            }
        }
        body_func.local_count = cctx.next_local;
        body_func.basic_blocks = cctx.blocks;
        self.async_functions.borrow_mut().push(body_func);

        // === 2. Generate the wrapper function (returns i64 task handle) ===
        let mut wrapper = MirFunction::new(&f.name);
        wrapper.params = f.params.iter().map(|p| {
            ast_type_to_mir(&p.type_, Some(&struct_defs))
        }).collect();
        wrapper.return_type = MirType::I64;

        let mut ctx = LowerCtx::new();
        ctx.struct_defs = struct_defs;

        // Bind wrapper params to locals (same as regular function lowering)
        for (i, param) in f.params.iter().enumerate() {
            let ptype = ast_type_to_mir(&param.type_, Some(&ctx.struct_defs));
            let local = ctx.alloc_local(&param.name, ptype);
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(i),
            });
            ctx.locals.insert(param.name.clone(), local);
        }

        let dest = ctx.alloc_local("_async_h", MirType::I64);
        if param_count <= 1 {
            // Single param or no param: pass value directly (or 0)
            let arg_val = if param_count == 1 {
                let pname = &real_params[0].name;
                if let Some(&local) = ctx.locals.get(pname) {
                    let ptype = ast_type_to_mir(&real_params[0].type_, Some(&ctx.struct_defs));
                    if ptype != MirType::I64 {
                        let widened = ctx.alloc_local("_pw", MirType::I64);
                        ctx.current_block.insts.push(MirInst::Cast {
                            dest: widened,
                            value: MirValue::Local(local),
                            to_type: MirType::I64,
                        });
                        MirValue::Local(widened)
                    } else {
                        MirValue::Local(local)
                    }
                } else {
                    MirValue::Constant(MirConstant::I64(0))
                }
            } else {
                MirValue::Constant(MirConstant::I64(0))
            };
            ctx.current_block.insts.push(MirInst::AsyncSpawn {
                dest,
                function_name: body_fn_name,
                arg: arg_val,
            });
        } else {
            // Multiple params: pack into heap-allocated array
            // which is TODO for multi-param case
            // For now just pass 0 (params will be lost)
            ctx.current_block.insts.push(MirInst::AsyncSpawn {
                dest,
                function_name: body_fn_name,
                arg: MirValue::Constant(MirConstant::I64(0)),
            });
        }
        ctx.emit_return(MirValue::Local(dest));
        wrapper.local_count = ctx.next_local;
        wrapper.basic_blocks = ctx.blocks;

        Some(wrapper)
    }

    pub(crate) fn lower_function(&self, f: &FunctionDecl) -> Option<MirFunction> {
        let body = f.body.as_ref()?;
        // Handle async fn: generate body + wrapper that spawns on thread pool
        if f.is_async {
            return self.lower_async_fn(f, body);
        }
        // Pre-register Option types in struct_defs for all params
        {
            let mut struct_defs = self.struct_defs.borrow_mut();
            for p in &f.params {
                register_option_type(&p.type_, &mut struct_defs);
            }
            if let Some(rt) = &f.return_type {
                register_option_type(rt, &mut struct_defs);
            }
        }
        let struct_defs = self.struct_defs.borrow().clone();
        let mut mir_func = MirFunction::new(&f.name);
        mir_func.params = f.params.iter()
            .map(|p| {
                let base = if p.variadic {
                    MirType::List(Box::new(ast_type_to_mir(&p.type_, Some(&struct_defs))))
                } else {
                    ast_type_to_mir(&p.type_, Some(&struct_defs))
                };
                if p.mode == ParamMode::MutableBorrow {
                    MirType::Ptr(Box::new(base))
                } else {
                    base
                }
            })
            .collect();
        mir_func.param_modes = f.params.iter().map(|p| p.mode).collect();
        mir_func.return_type = f.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
            .unwrap_or(MirType::Void);
        let is_fallible = f.return_type.as_ref().map_or(false, |rt| {
            matches!(rt, AstType::Error { .. })
                || matches!(rt, AstType::Generic { name, .. } if name == "Result")
        });
        mir_func.is_fallible = is_fallible;
        mir_func.is_const = f.is_const;

        let mut ctx = LowerCtx::new();
        ctx.struct_defs = self.struct_defs.borrow().clone();
        ctx.is_fallible = is_fallible;
        ctx.return_type = mir_func.return_type.clone();

        // Allocate and store params
        for (i, param) in f.params.iter().enumerate() {
            let ptype = if param.variadic {
                MirType::List(Box::new(ast_type_to_mir(&param.type_, Some(&ctx.struct_defs))))
            } else {
                ast_type_to_mir(&param.type_, Some(&ctx.struct_defs))
            };
            let is_ref = param.mode == ParamMode::MutableBorrow;
            let local_type = if is_ref {
                MirType::Ptr(Box::new(ptype.clone()))
            } else {
                ptype
            };
            let local = ctx.alloc_local(&param.name, local_type);
            if is_ref {
                ctx.ref_param_locals.insert(local);
            }
            ctx.current_block.insts.push(MirInst::Store {
                dest: local,
                value: MirValue::Param(i),
            });
            ctx.locals.insert(param.name.clone(), local);
        }

        // Lower body statements
        let last_is_tail = match body.statements.last() {
            Some(Stmt::Expression(e)) => !matches!(e, Expr::Assignment { .. }),
            _ => false,
        };
        let last_is_if_tail = matches!(body.statements.last(), Some(Stmt::If(_)));
        let last_is_match_tail = matches!(body.statements.last(), Some(Stmt::Match(_)));
        let stmt_count = body.statements.len();
        if last_is_match_tail {
            if f.return_type.is_some() {
                let result_local = ctx.alloc_local("_match_res", MirType::I64);
                ctx.match_result_local = Some(result_local);
            }
        }
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count && last_is_if_tail {
                ctx.tail_if_as_return = true;
            }
            ctx = self.lower_stmt(ctx, stmt);
        }

        // Default terminator: return void or tail expression value
        if ctx.current_block.terminator == MirTerminator::Unreachable {
            // Save tail value BEFORE deferred expressions (deferred may add more locals)
            let tail_local = if last_is_tail {
                // The tail expression's result is the last local before deferred
                let t = ctx.next_local.checked_sub(1);
                if let Some(l) = t { Some(l) } else { None }
            } else { None };
            // Emit deferred calls in reverse LIFO order before implicit return
            let deferred = std::mem::take(&mut ctx.deferred_exprs);
            for expr in deferred.iter().rev() {
                ctx = self.lower_expr(ctx, expr);
            }
            if let Some(val_local) = tail_local {
                // If the tail expression is a void call (e.g., print()), return Void
                let is_void = ctx.local_types.get(&val_local).map_or(false, |t| *t == MirType::Void);
                if is_void {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                } else {
                    ctx.emit_return(MirValue::Local(val_local));
                    // Infer return type from the tail expression when no explicit type given
                    if mir_func.return_type == MirType::Void {
                        if let Some(actual_type) = ctx.local_types.get(&val_local) {
                            mir_func.return_type = actual_type.clone();
                        }
                    }
                }
            } else if last_is_match_tail {
                if let Some(result_local) = ctx.match_result_local {
                    let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                    let load = ctx.alloc_local("_match_res_val", result_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: load,
                        src: result_local,
                    });
                    ctx.emit_return(MirValue::Local(load));
                } else {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                }
            } else if !last_is_if_tail {
                ctx.emit_return(MirValue::Constant(MirConstant::Void));
            }
        }

        for b in &ctx.blocks {
        }
        mir_func.basic_blocks = ctx.blocks;
        mir_func.local_count = ctx.next_local;
        Some(mir_func)
    }

    /// Ensure all methods of a concrete generic class are monomorphized and registered.
    /// `concrete_name` is like "Box__i32" — extract base, look up generic templates,
    /// monomorphize each method, and register in method_table.
    pub(crate) fn ensure_generic_class_methods(&self, concrete_name: &str) {
        // Extract base name: "Box__i32" → "Box"
        let base_name = match concrete_name.split("__").next() {
            Some(n) if n != concrete_name => n.to_string(),
            _ => return,
        };
        // Check if already monomorphized (method_table already has concrete_name)
        if self.method_table.borrow().contains_key(concrete_name) {
            return;
        }
        // Look up class methods from generic_class_methods
        let class_methods;
        let generic_templates;
        let struct_tpl;
        {
            let cm = self.generic_class_methods.borrow();
            class_methods = cm.get(&base_name).cloned().unwrap_or_default();
            generic_templates = self.generic_function_templates.borrow();
            let st = self.generic_struct_templates.borrow();
            struct_tpl = st.get(&base_name).cloned();
        }
        if class_methods.is_empty() || struct_tpl.is_none() {
            return;
        }
        let tpl = struct_tpl.unwrap();
        // Reconstruct concrete fields from struct_defs
        let concrete_fields = self.struct_defs.borrow().get(concrete_name).cloned().unwrap_or_default();
        if concrete_fields.is_empty() {
            return;
        }
        // Infer type substitution from field types
        let mut type_subst: std::collections::HashMap<String, MirType> = std::collections::HashMap::new();
        for tf in &tpl.fields {
            if let Some(concrete_field) = concrete_fields.iter().find(|(n, _)| *n == tf.name) {
                // Check if template field type matches a type param name
                if let AstType::User { name, .. } = &tf.type_ {
                    if tpl.type_params.iter().any(|tp| tp.name == *name) {
                        type_subst.entry(name.clone()).or_insert_with(|| concrete_field.1.clone());
                    }
                }
            }
        }
        // For each method, monomorphize and register
        for method in &class_methods {
            let template_name = format!("{}::{}", base_name, method.name);
            let template = match generic_templates.get(&template_name) {
                Some(t) => t,
                None => continue,
            };
            let specialized_name = format!("{}::{}", concrete_name, method.name);
            let mut full_type_subst = type_subst.clone();
            full_type_subst.insert(base_name.clone(), MirType::Struct(concrete_name.to_string(), concrete_fields.clone()));
            let mut specialized_decl = clone_and_specialize_function(template, &full_type_subst);
            specialized_decl.name = method.name.clone(); // lower_method prepends class_name::
            // Pre-register generic struct types in signature
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
            // Compute and register return type
            let struct_defs = self.struct_defs.borrow();
            let ret_type = template.return_type.as_ref()
                .map(|rt| ast_type_to_mir_with_subst(rt, Some(&struct_defs), &type_subst))
                .unwrap_or(MirType::Void);
            drop(struct_defs);
            self.fn_returns.borrow_mut().insert(specialized_name.clone(), ret_type);
            // Lower the specialized method
            if let Some(mir_func) = self.lower_method(&specialized_decl, concrete_name) {
                self.specialized_mir_functions.borrow_mut().push(mir_func);
            }
            // Register in method_table
            self.method_table.borrow_mut()
                .entry(concrete_name.to_string())
                .or_default()
                .insert(method.name.clone(), specialized_name);
        }
    }

    /// Lower a class method. Like `lower_function`, but the method's MIR
    /// signature prepends an implicit `this` parameter of type
    /// `MirType::Struct(class_name, fields)` so the body can reference `this`
    /// and the method can be called as `ClassName::method(obj, args...)`.
    pub(crate) fn lower_method(&self, m: &FunctionDecl, class_name: &str) -> Option<MirFunction> {
        let body = m.body.as_ref()?;
        let struct_defs = self.struct_defs.borrow().clone();
        let mut mir_func = MirFunction::new(&format!("{}::{}", class_name, m.name));
        let is_static = m.is_static;
        let this_type = struct_defs.get(class_name)
            .map(|fields| MirType::Struct(class_name.to_string(), fields.clone()))
            .unwrap_or(MirType::Struct(class_name.to_string(), vec![]));
        // Static methods don't get a `this` parameter; instance methods do.
        let mut ctx = LowerCtx::new();
        ctx.struct_defs = struct_defs.clone();
        if is_static {
            // Static method: no `this`, just explicit params
            let mut params: Vec<MirType> = Vec::new();
            for p in &m.params {
                params.push(ast_type_to_mir(&p.type_, Some(&struct_defs)));
            }
            mir_func.params = params;
            let param_modes: Vec<ParamMode> = m.params.iter().map(|p| p.mode).collect();
            mir_func.param_modes = param_modes;
        } else {
            let mut params: Vec<MirType> = vec![this_type.clone()];
            for (i, p) in m.params.iter().enumerate() {
                if i == 0 && (p.name == "this" || p.name == "self") {
                    continue;
                }
                params.push(ast_type_to_mir(&p.type_, Some(&struct_defs)));
            }
            mir_func.params = params;
            let mut param_modes = vec![ParamMode::MutableBorrow];
            param_modes.extend(m.params.iter().enumerate().filter(|(i, p)| !(*i == 0 && (p.name == "this" || p.name == "self"))).map(|(_, p)| p.mode));
            mir_func.param_modes = param_modes;
        }
        mir_func.return_type = m.return_type.as_ref()
            .map(|rt| ast_type_to_mir(rt, Some(&struct_defs)))
            .unwrap_or(MirType::Void);
        let is_fallible = m.return_type.as_ref().map_or(false, |rt| {
            matches!(rt, AstType::Error { .. })
                || matches!(rt, AstType::Generic { name, .. } if name == "Result")
        });
        mir_func.is_fallible = is_fallible;

        ctx.struct_defs = struct_defs;
        ctx.is_fallible = is_fallible;
        ctx.return_type = mir_func.return_type.clone();

        if is_static {
            // Static method: bind explicit params directly (no `this`)
            for (i, param) in m.params.iter().enumerate() {
                let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_, Some(&ctx.struct_defs)));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: local,
                    value: MirValue::Param(i),
                });
                ctx.locals.insert(param.name.clone(), local);
            }
        } else {
            // Bind `this` (param 0) into a local so the body's `Expr::PropertyAccess`
            // on `this` resolves to a struct field.
            // Store as Ptr(Struct) so the codegen handles it as a by-reference parameter
            let this_local = ctx.alloc_local("this", MirType::Ptr(Box::new(this_type.clone())));
            ctx.current_block.insts.push(MirInst::Store {
                dest: this_local,
                value: MirValue::Param(0),
            });
            ctx.locals.insert("this".to_string(), this_local);
            ctx.locals.insert("self".to_string(), this_local);

            // Bind the explicit params (offset by 1 because of implicit `this`).
            let mut param_offset = 1usize;
            for (i, param) in m.params.iter().enumerate() {
                if i == 0 && (param.name == "this" || param.name == "self") {
                    continue;
                }
                let local = ctx.alloc_local(&param.name, ast_type_to_mir(&param.type_, Some(&ctx.struct_defs)));
                ctx.current_block.insts.push(MirInst::Store {
                    dest: local,
                    value: MirValue::Param(param_offset),
                });
                ctx.locals.insert(param.name.clone(), local);
                param_offset += 1;
            }
        }

        // Lower body statements
        let last_is_tail = match body.statements.last() {
            Some(Stmt::Expression(e)) => !matches!(e, Expr::Assignment { .. }),
            _ => false,
        };
        let last_is_if_tail = matches!(body.statements.last(), Some(Stmt::If(_)));
        let last_is_match_tail = matches!(body.statements.last(), Some(Stmt::Match(_)));
        let stmt_count = body.statements.len();
        if last_is_match_tail {
            if m.return_type.is_some() {
                let result_local = ctx.alloc_local("_match_res", MirType::I64);
                ctx.match_result_local = Some(result_local);
            }
        }
        for (i, stmt) in body.statements.iter().enumerate() {
            if i + 1 == stmt_count && last_is_if_tail {
                ctx.tail_if_as_return = true;
            }
            ctx = self.lower_stmt(ctx, stmt);
        }

        if ctx.current_block.terminator == MirTerminator::Unreachable {
            if last_is_tail {
                let val_local = ctx.next_local - 1;
                ctx.emit_return(MirValue::Local(val_local));
                // Infer return type from tail expression when no explicit type
                if mir_func.return_type == MirType::Void {
                    if let Some(actual_type) = ctx.local_types.get(&val_local) {
                        mir_func.return_type = actual_type.clone();
                    }
                }
            } else if last_is_match_tail {
                if let Some(result_local) = ctx.match_result_local {
                    let result_type = ctx.local_types.get(&result_local).cloned().unwrap_or(MirType::I32);
                    let load = ctx.alloc_local("_match_res_val", result_type);
                    ctx.current_block.insts.push(MirInst::Load {
                        dest: load,
                        src: result_local,
                    });
                    ctx.emit_return(MirValue::Local(load));
                } else {
                    ctx.emit_return(MirValue::Constant(MirConstant::Void));
                }
            } else if !last_is_if_tail {
                ctx.emit_return(MirValue::Constant(MirConstant::Void));
            }
        }

        mir_func.basic_blocks = ctx.blocks;
        mir_func.local_count = ctx.next_local;
        Some(mir_func)
    }



}
