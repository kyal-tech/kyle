use std::path::PathBuf;
use std::collections::HashMap;
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::*;
use kyc_core::ast::*;
use kyc_core::span::Span;
use kyc_core::types::Type;
use kyc_frontend::token::TokenKind;
use kyc_semantic::symbol_table::{Symbol, SymKind, SymbolTable};
use kyc_semantic::module_resolver::ModuleResolver;
use crate::package::{find_project_root, LockFile, cache::package_src_dir};

/// KL Language Server — provides diagnostics, completion, hover, and
/// go-to-definition via the Language Server Protocol.
pub struct LanguageServer {
    connection: Connection,
    sources: HashMap<String, String>,
}

impl LanguageServer {
    pub fn new() -> Result<Self, String> {
        let (connection, io_threads) = Connection::stdio();
        let server = Self {
            connection,
            sources: HashMap::new(),
        };
        std::mem::forget(io_threads);
        Ok(server)
    }

    pub fn run(&mut self) -> Result<(), String> {
        let caps = self.server_capabilities();
        let init_params = serde_json::to_value(&caps).unwrap();
        let _init = self.connection.initialize(init_params)
            .map_err(|e| format!("initialize error: {}", e))?;

        loop {
            let msg = self.connection.receiver.recv()
                .map_err(|e| format!("recv error: {}", e))?;
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req).unwrap_or(false) {
                        return Ok(());
                    }
                    self.handle_request(req);
                }
                Message::Notification(not) => {
                    self.handle_notification(not);
                }
                Message::Response(_) => {}
            }
        }
    }

    fn server_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                ..Default::default()
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                ..Default::default()
            }),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: Default::default(),
            })),
            code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(false) }),
            inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
                InlayHintOptions { resolve_provider: Some(false), work_done_progress_options: Default::default() }
            ))),
            semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                SemanticTokensOptions {
                    legend: SemanticTokensLegend {
                        token_types: vec![
                            SemanticTokenType::VARIABLE,
                            SemanticTokenType::TYPE,
                            SemanticTokenType::CLASS,
                            SemanticTokenType::STRUCT,
                            SemanticTokenType::ENUM,
                            SemanticTokenType::FUNCTION,
                            SemanticTokenType::METHOD,
                            SemanticTokenType::PARAMETER,
                            SemanticTokenType::PROPERTY,
                        ],
                        token_modifiers: vec![
                            SemanticTokenModifier::MODIFICATION,
                            SemanticTokenModifier::DECLARATION,
                            SemanticTokenModifier::READONLY,
                        ],
                    },
                    range: None,
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    work_done_progress_options: Default::default(),
                },
            )),
            ..Default::default()
        }
    }

    fn handle_request(&mut self, req: Request) {
        match req.method.as_str() {
            "textDocument/completion" => self.handle_completion(req),
            "textDocument/hover" => self.handle_hover(req),
            "textDocument/definition" => self.handle_definition(req),
            "textDocument/references" => self.handle_references(req),
            "textDocument/documentSymbol" => self.handle_document_symbol(req),
            "workspace/symbol" => self.handle_workspace_symbol(req),
            "textDocument/signatureHelp" => self.handle_signature_help(req),
            "textDocument/codeAction" => self.handle_code_action(req),
            "textDocument/formatting" => self.handle_formatting(req),
            "textDocument/rename" => self.handle_rename(req),
            "textDocument/prepareRename" => self.handle_prepare_rename(req),
            "textDocument/codeLens" => self.handle_code_lens(req),
            "textDocument/inlayHint" => self.handle_inlay_hint(req),
            "textDocument/semanticTokens/full" => self.handle_semantic_tokens_full(req),
            _ => {
                let resp = Response::new_err(
                    req.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("unknown request: {}", req.method),
                );
                let _ = self.connection.sender.send(Message::Response(resp));
            }
        }
    }

    fn handle_notification(&mut self, not: Notification) {
        match not.method.as_str() {
            "textDocument/didOpen" => self.handle_did_open(not),
            "textDocument/didChange" => self.handle_did_change(not),
            "textDocument/didSave" => self.handle_did_save(not),
            "textDocument/didClose" => self.handle_did_close(not),
            _ => {}
        }
    }

    fn handle_did_open(&mut self, not: Notification) {
        let params: DidOpenTextDocumentParams = serde_json::from_value(not.params).unwrap();
        let uri = params.text_document.uri.to_string();
        self.sources.insert(uri.clone(), params.text_document.text);
        self.publish_diagnostics(&uri);
    }

    fn handle_did_change(&mut self, not: Notification) {
        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params).unwrap();
        let uri = params.text_document.uri.to_string();
        if let Some(change) = params.content_changes.into_iter().last() {
            if let Some(range) = change.range {
                // Incremental: apply range-based change
                if let Some(source) = self.sources.get(&uri) {
                    let new_source = apply_range_change(source, &range, &change.text);
                    self.sources.insert(uri.clone(), new_source);
                }
            } else {
                // Full replacement (first open or sync failure fallback)
                self.sources.insert(uri.clone(), change.text);
            }
        }
        self.publish_diagnostics(&uri);
    }

    fn handle_did_close(&mut self, not: Notification) {
        let params: DidCloseTextDocumentParams = serde_json::from_value(not.params).unwrap();
        let uri = params.text_document.uri.to_string();
        self.sources.remove(&uri);
    }

    fn handle_did_save(&mut self, not: Notification) {
        let params: DidSaveTextDocumentParams = serde_json::from_value(not.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = match self.sources.get(&uri) {
            Some(s) => s.clone(),
            None => return,
        };

        // Quick check — only compile if it looks like a KL file
        if !uri.ends_with(".ky") {
            return;
        }

        // Build in a background thread so we don't block the LSP
        let uri_clone = uri.clone();
        let source_clone = source.clone();
        std::thread::spawn(move || {
            let mut lexer = kyc_frontend::lexer::Lexer::new(&source_clone);
            let tokens = lexer.tokenize();
            let mut parser = kyc_frontend::parser::Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    let file_name = uri_clone.trim_start_matches("file://");
                    let mut source_map = kyc_core::source_map::SourceMap::new();
                    let _file_id = source_map.add(file_name.to_string(), source_clone);
                    let mut analyzer = kyc_semantic::analyzer::SemanticAnalyzer::new()
                        .with_source(source_map, file_name.to_string());
                    analyzer.analyze(&program);
                    if analyzer.has_errors() {
                        // Diagnostics are already published by didChange
                        return;
                    }
                    // Type-check passed
                }
                Err(_) => {}
            }
        });
    }

    /// Parse a file, resolve its imports, and run semantic analysis.
    /// Returns the resolved program, analyzer, and the file name.
    fn resolve_and_analyze(&self, uri: &str, source: &str) -> Option<(Program, kyc_semantic::analyzer::SemanticAnalyzer, String)> {
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let mut program = parser.parse().ok()?;
        let file_name = uri.trim_start_matches("file://");

        let base_dir = PathBuf::from(file_name)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut resolver = ModuleResolver::new();
        resolver.set_source_dir(base_dir.clone());
        resolver.add_search_path(base_dir.clone());
        if let Some(project_root) = find_project_root(&base_dir) {
            let src_dir = project_root.join("src");
            if src_dir.exists() && src_dir != base_dir {
                resolver.add_search_path(src_dir);
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            let std_path = cwd.join("std");
            if std_path.exists() { resolver.add_search_path(std_path); }
            let pkg_path = cwd.join("packages");
            if pkg_path.exists() { resolver.add_search_path(pkg_path); }
        }
        let local_std = base_dir.join("std");
        if local_std.exists() { resolver.add_search_path(local_std); }
        let local_pkg = base_dir.join("packages");
        if local_pkg.exists() { resolver.add_search_path(local_pkg); }
        if let Some(project_root) = find_project_root(&base_dir) {
            let lock_path = project_root.join("ky.lock");
            if let Ok(lock) = LockFile::read(&lock_path) {
                for entry in &lock.packages {
                    let src_dir = package_src_dir(&entry.name, &entry.version);
                    if src_dir.exists() { resolver.add_search_path(src_dir); }
                }
            }
            if let Ok(entries) = std::fs::read_dir(crate::package::cache::cache_root()) {
                for entry in entries.flatten() {
                    let src_dir = entry.path().join("src");
                    if src_dir.exists() { resolver.add_search_path(src_dir); }
                }
            }
        }

        // Resolve all imports
        let mut import_decls: Vec<(usize, Vec<Decl>)> = Vec::new();
        let mut seen_modules: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (i, decl) in program.declarations.iter().enumerate() {
            match decl {
                Decl::Import(imp) => {
                    if seen_modules.insert(imp.module_name.clone()) {
                        if let Ok(module) = resolver.resolve_import(&imp.module_name, imp.relative) {
                            import_decls.push((i, module.program.declarations.clone()));
                        }
                    }
                }
                Decl::FromImport(fi) => {
                    if seen_modules.insert(fi.module_name.clone()) {
                        if let Ok(module) = resolver.resolve_import(&fi.module_name, fi.relative) {
                            // Import ALL declarations from the module (not just requested names)
                            // so that types like enums referenced by class fields are in scope.
                            import_decls.push((i, module.program.declarations.clone()));
                        }
                    }
                }
                _ => {}
            }
        }
        for (idx, decls) in import_decls.into_iter().rev() {
            if !decls.is_empty() {
                let mut rest = program.declarations.split_off(idx + 1);
                program.declarations.extend(decls);
                program.declarations.append(&mut rest);
            }
        }

        // Pull contracts from cached modules that are referenced by imported classes
        let class_contracts: std::collections::HashSet<String> = program.declarations.iter()
            .filter_map(|d| {
                if let Decl::Class(c) = d {
                    if c.contracts.is_empty() { None } else { Some(c.contracts.clone()) }
                } else { None }
            })
            .fold(std::collections::HashSet::new(), |mut acc, v| { acc.extend(v); acc });
        if !class_contracts.is_empty() {
            for cached in resolver.cache.values() {
                for cd in &cached.program.declarations {
                    if let Decl::Contract(ct) = cd {
                        if class_contracts.contains(&ct.name) {
                            program.declarations.push(cd.clone());
                        }
                    }
                }
            }
        }

        let mut source_map = kyc_core::source_map::SourceMap::new();
        let _file_id = source_map.add(file_name.to_string(), source.to_string());
        let mut analyzer = kyc_semantic::analyzer::SemanticAnalyzer::new()
            .with_source(source_map, file_name.to_string());
        analyzer.analyze(&program);
        Some((program, analyzer, file_name.to_string()))
    }

    /// Collect all search paths where .ky modules/packages can be found.
    fn collect_search_paths(&self, uri: &str) -> Vec<PathBuf> {
        let file_name = uri.trim_start_matches("file://");
        let base_dir = PathBuf::from(file_name)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut paths = Vec::new();
        paths.push(base_dir.clone());
        if let Some(project_root) = find_project_root(&base_dir) {
            let src_dir = project_root.join("src");
            if src_dir.exists() { paths.push(src_dir); }
        }
        if let Ok(cwd) = std::env::current_dir() {
            let std_path = cwd.join("std");
            if std_path.exists() { paths.push(std_path); }
            let pkg_path = cwd.join("packages");
            if pkg_path.exists() { paths.push(pkg_path); }
        }
        let local_std = base_dir.join("std");
        if local_std.exists() { paths.push(local_std); }
        let local_pkg = base_dir.join("packages");
        if local_pkg.exists() { paths.push(local_pkg); }
        if let Some(project_root) = find_project_root(&base_dir) {
            let lock_path = project_root.join("ky.lock");
            if let Ok(lock) = LockFile::read(&lock_path) {
                for entry in &lock.packages {
                    let src_dir = package_src_dir(&entry.name, &entry.version);
                    if src_dir.exists() { paths.push(src_dir); }
                }
            }
            if let Ok(entries) = std::fs::read_dir(crate::package::cache::cache_root()) {
                for entry in entries.flatten() {
                    let src_dir = entry.path().join("src");
                    if src_dir.exists() { paths.push(src_dir); }
                }
            }
        }
        // Also add search paths from the compiler binary
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let dev_pkg = exe_dir.join("../packages");
                if dev_pkg.exists() { paths.push(dev_pkg); }
            }
        }
        paths
    }

    /// Resolve a dotted module path (e.g. "http.server") to its parsed Program.
    fn resolve_module(&self, mod_path: &str) -> Option<Program> {
        let uri = self.sources.keys().next()?;
        let search_dirs = self.collect_search_paths(uri);
        let file_name = mod_path.replace('.', std::path::MAIN_SEPARATOR_STR);
        for dir in &search_dirs {
            // Try <dir>/<path>.ky and <dir>/<path>/lib.ky
            let ky_file = dir.join(&file_name).with_extension("ky");
            let lib_file = dir.join(&file_name).join("lib.ky");
            let path = if ky_file.exists() { ky_file } else if lib_file.exists() { lib_file } else { continue; };
            if let Ok(source) = std::fs::read_to_string(&path) {
                let mut lexer = kyc_frontend::lexer::Lexer::new(&source);
                let tokens = lexer.tokenize();
                let mut parser = kyc_frontend::parser::Parser::new(tokens);
                if let Ok(program) = parser.parse() {
                    return Some(program);
                }
            }
        }
        None
    }

    fn publish_diagnostics(&self, uri: &str) {
        let source = match self.sources.get(uri) {
            Some(s) => s.clone(),
            None => return,
        };

        if uri.ends_with("ky.toml") {
            self.publish_manifest_diagnostics(uri, &source);
            return;
        }

        let mut lsp_diags = Vec::new();

        match self.resolve_and_analyze(uri, &source) {
            Some((_program, analyzer, _file_name)) => {

                for diag in analyzer.reporter().diagnostics() {
                    let range = diag.span
                        .as_ref()
                        .map(span_to_range)
                        .unwrap_or(Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        });
                    let severity = match diag.severity {
                        kyc_core::diagnostic::Severity::Error => Some(DiagnosticSeverity::ERROR),
                        kyc_core::diagnostic::Severity::Warning => Some(DiagnosticSeverity::WARNING),
                        kyc_core::diagnostic::Severity::Note => Some(DiagnosticSeverity::INFORMATION),
                        kyc_core::diagnostic::Severity::Ice => Some(DiagnosticSeverity::ERROR),
                    };
                    lsp_diags.push(Diagnostic {
                        range,
                        severity,
                        source: Some("ky".to_string()),
                        message: format!("[{}] {}", diag.code, diag.message),
                        ..Default::default()
                    });
                }
            }
            None => {
                lsp_diags.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("ky".to_string()),
                    message: "Parse error".to_string(),
                    ..Default::default()
                });
            }
        }

        let params = PublishDiagnosticsParams {
            uri: lsp_types::Url::parse(uri).unwrap(),
            diagnostics: lsp_diags,
            version: None,
        };
        let not = Notification::new(
            "textDocument/publishDiagnostics".to_string(),
            serde_json::to_value(params).unwrap(),
        );
        let _ = self.connection.sender.send(Message::Notification(not));
    }

    fn publish_manifest_diagnostics(&self, uri: &str, source: &str) {
        let mut lsp_diags = Vec::new();

        // Try parsing as TOML manifest
        match crate::package::Manifest::from_str(source) {
            Ok(manifest) => {
                // Parse OK — run validation
                if let Err(errors) = manifest.validate() {
                    for msg in errors {
                        lsp_diags.push(Diagnostic {
                            range: Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: 0 },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            source: Some("ky".to_string()),
                            message: format!("[manifest] {}", msg),
                            ..Default::default()
                        });
                    }
                }
            }
            Err(toml_err) => {
                // TOML parse error — extract position from byte span
                let msg = format!("TOML parse error: {}", toml_err);
                let range = toml_err.span()
                    .and_then(|span| byte_span_to_range(source, &span))
                    .unwrap_or(Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    });
                lsp_diags.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("ky".to_string()),
                    message: format!("[manifest] {}", msg),
                    ..Default::default()
                });
            }
        }

        let params = PublishDiagnosticsParams {
            uri: lsp_types::Url::parse(uri).unwrap(),
            diagnostics: lsp_diags,
            version: None,
        };
        let not = Notification::new(
            "textDocument/publishDiagnostics".to_string(),
            serde_json::to_value(params).unwrap(),
        );
        let _ = self.connection.sender.send(Message::Notification(not));
    }

    fn handle_inlay_hint(&mut self, req: Request) {
        let params: InlayHintParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = match self.sources.get(&uri) {
            Some(s) => s.clone(),
            None => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<InlayHint>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };
        let file_name = uri.trim_start_matches("file://");

        let mut lexer = kyc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(_) => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<InlayHint>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };

        let mut source_map = kyc_core::source_map::SourceMap::new();
        let _file_id = source_map.add(file_name.to_string(), source.clone());
        let mut analyzer = kyc_semantic::analyzer::SemanticAnalyzer::new()
            .with_source(source_map, file_name.to_string());
        analyzer.analyze(&program);

        let symbols = analyzer.type_checker.symbols();
        let fn_return_types = &analyzer.type_checker.fn_return_types;
        let mut hints: Vec<InlayHint> = Vec::new();

        for decl in &program.declarations {
            match decl {
                Decl::Variable(v) => {
                    if v.type_.is_none() {
                        if let Some(sym) = symbols.lookup(&v.name) {
                            if let SymKind::Variable { type_: Some(t), .. } = &sym.kind {
                                hints.push(InlayHint {
                                    position: Position {
                                        line: v.span.start.line.saturating_sub(1) as u32,
                                        character: v.span.start.column.saturating_sub(1) as u32 + v.name.len() as u32,
                                    },
                                    label: InlayHintLabel::String(format!(": {}", t)),
                                    kind: Some(InlayHintKind::TYPE),
                                    padding_left: Some(true),
                                    padding_right: Some(false),
                                    text_edits: None,
                                    tooltip: None,
                                    data: None,
                                });
                            }
                        }
                    }
                }
                Decl::Function(f) => {
                    if f.return_type.is_none() {
                        if let Some(rt) = fn_return_types.get(&f.name) {
                            let _params_end_col = f.span.start.column.saturating_sub(1)
                                + f.name.len()
                                + (if f.type_params.is_empty() { 1 } else { 0 });
                            hints.push(InlayHint {
                                position: Position {
                                    line: f.span.start.line.saturating_sub(1) as u32,
                                    character: f.span.start.column.saturating_sub(1) as u32 + f.name.len() as u32 + 3,
                                },
                                label: InlayHintLabel::String(format!(" -> {}", rt)),
                                kind: Some(InlayHintKind::TYPE),
                                padding_left: Some(true),
                                padding_right: Some(false),
                                text_edits: None,
                                tooltip: None,
                                data: None,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        let resp = Response::new_ok(req.id, serde_json::to_value(hints).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_code_lens(&mut self, req: Request) {
        let params: CodeLensParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = match self.sources.get(&uri) {
            Some(s) => s.clone(),
            None => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<CodeLens>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };

        let mut lexer = kyc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(_) => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<CodeLens>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };

        let mut lenses: Vec<CodeLens> = Vec::new();
        for decl in &program.declarations {
            if let Decl::Function(f) = decl {
                if f.is_test {
                    let range = span_to_range(&f.span);
                    lenses.push(CodeLens {
                        range,
                        command: Some(Command {
                            title: "▶ Run Test".to_string(),
                            command: "kl.runTest".to_string(),
                            arguments: Some(vec![
                                serde_json::json!(uri),
                                serde_json::json!(f.name),
                            ]),
                        }),
                        data: None,
                    });
                }
                if f.is_bench {
                    let range = span_to_range(&f.span);
                    lenses.push(CodeLens {
                        range,
                        command: Some(Command {
                            title: "▶ Run Bench".to_string(),
                            command: "kl.runBench".to_string(),
                            arguments: Some(vec![
                                serde_json::json!(uri),
                                serde_json::json!(f.name),
                            ]),
                        }),
                        data: None,
                    });
                }
            }
        }

        let resp = Response::new_ok(req.id, serde_json::to_value(lenses).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_completion(&mut self, req: Request) {
        let params: CompletionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position.position;

        // Check if this is a dot-triggered completion (user typed "expr.")
        let is_dot = self.is_after_dot(&source, pos.line as usize, pos.character as usize, &params);
        if is_dot {
            let expr = self.word_before_dot(&source, pos.line as usize, pos.character as usize);
            if !expr.is_empty() {
                if let Some(items) = self.dot_completions(&source, &expr, pos.line as usize, pos.character as usize) {
                    let result = CompletionResponse::Array(items);
                    let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
                    let _ = self.connection.sender.send(Message::Response(resp));
                    return;
                }
            }
        }

        let prefix = self.word_at(&source, pos.line as usize, pos.character as usize);
        let prefix_lower = prefix.to_lowercase();

        let mut items: Vec<CompletionItem> = Vec::new();

        // Built-in functions (28 total)
        let builtins: &[(&str, &str)] = &[
            ("print", "print(value) — Print to stdout"),
            ("println", "println(value) — Print with newline"),
            ("print_int", "print_int(n) — Print integer"),
            ("println_int", "println_int(n) — Print integer with newline"),
            ("print_err", "print_err(value) — Print to stderr"),
            ("str", "str(value) — Convert to string"),
            ("len", "len(value) — Get length"),
            ("int", "int(value) — Convert to integer"),
            ("float", "float(value) — Convert to float"),
            ("bool", "bool(value) — Convert to boolean"),
            ("input", "input() — Read line from stdin"),
            ("open", "open(path) i64 — Open file"),
            ("read_str", "read_str(fd) str — Read file content"),
            ("write_str", "write_str(fd, str) — Write to file"),
            ("close", "close(fd) — Close file"),
            ("sleep", "sleep(ms) — Sleep in milliseconds"),
            ("now", "now() i64 — Current timestamp"),
            ("contains", "contains(str, substr) bool"),
            ("to_upper", "to_upper(str) str"),
            ("to_lower", "to_lower(str) str"),
            ("trim", "trim(str) str"),
            ("replace", "replace(str, from, to) str"),
            ("substr", "substr(str, start, len) str"),
            ("char_at", "char_at(str, index) char"),
            ("ord", "ord(c) i32 — Char to ASCII code"),
            ("is_digit", "is_digit(c) bool"),
            ("is_alpha", "is_alpha(c) bool"),
            ("is_alnum", "is_alnum(c) bool"),
            ("is_whitespace", "is_whitespace(c) bool"),
            ("is_upper", "is_upper(c) bool"),
            ("is_lower", "is_lower(c) bool"),
            ("assert", "assert(condition)"),
            ("assert_eq", "assert_eq(a, b)"),
            ("assert_str", "assert_str(a, b)"),
            ("assert_ne", "assert_ne(a, b)"),
            ("range", "range(start, end) [i32]"),
            ("json_parse", "json_parse(str) i64"),
            ("json_stringify", "json_stringify(value) str"),
            ("list_push", "list_push(list, value)"),
            ("list_pop", "list_pop(list) i64"),
            ("list_len", "list_len(list) i32"),
            ("ceil", "ceil(f64) f64"),
            ("floor", "floor(f64) f64"),
            ("round", "round(f64) f64"),
        ];
        for (label, detail) in builtins {
            if prefix.is_empty() || label.starts_with(&prefix) || label.to_lowercase().starts_with(&prefix_lower) {
                items.push(CompletionItem {
                    label: label.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(detail.to_string()),
                    insert_text: Some(label.to_string()),
                    sort_text: Some(format!("1{}", label)),
                    ..Default::default()
                });
            }
        }

        // Project symbols — parse with import resolution to include package symbols
        if let Some((prog, analyzer, _)) = self.resolve_and_analyze(&uri, &source) {
            for decl in &prog.declarations {
                let (name, kind, detail): (String, CompletionItemKind, String) = match decl {
                    Decl::Function(f) => {
                        let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_))).collect();
                        let sig = format!("fn {}({})", f.name, params.join(", "));
                        (f.name.clone(), CompletionItemKind::FUNCTION, sig)
                    }
                    Decl::Variable(v) => (v.name.clone(), CompletionItemKind::VARIABLE, format!("var {}", v.name)),
                    Decl::Constant(c) => (c.name.clone(), CompletionItemKind::CONSTANT, c.name.clone()),
                    Decl::Class(c) => (c.name.clone(), CompletionItemKind::CLASS, format!("class {}", c.name)),
                    Decl::Struct(s) => (s.name.clone(), CompletionItemKind::STRUCT, format!("struct {}", s.name)),
                    Decl::Enum(e) => (e.name.clone(), CompletionItemKind::ENUM, format!("enum {}", e.name)),
                    Decl::Contract(c) => (c.name.clone(), CompletionItemKind::INTERFACE, format!("contract {}", c.name)),
                    Decl::TypeAlias(t) => (t.name.clone(), CompletionItemKind::TYPE_PARAMETER, format!("type {}", t.name)),
                    Decl::AbstractClass(c) => (c.name.clone(), CompletionItemKind::CLASS, format!("abstract class {}", c.name)),
                    _ => continue,
                };
                if prefix.is_empty() || name.starts_with(&prefix) || name.to_lowercase().starts_with(&prefix_lower) {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(kind),
                        detail: Some(detail),
                        insert_text: None,
                        sort_text: Some(format!("2{}", &name)),
                        ..Default::default()
                    });
                }
            }
            // Also add type info from the symbol table for richer completions
            let symbols = analyzer.type_checker.symbols();
            for name in symbols.all_top_level_names() {
                if (prefix.is_empty() || name.starts_with(&prefix) || name.to_lowercase().starts_with(&prefix_lower))
                    && !items.iter().any(|i| i.label == name)
                {
                    if let Some(sym) = symbols.lookup(&name) {
                        let detail = Self::format_symbol_hover(sym, &name);
                        items.push(CompletionItem {
                            label: name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(detail),
                            insert_text: None,
                            sort_text: Some(format!("2{}", &name)),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Keywords
        let keywords = [
            ("if", "if condition:"),
            ("elif", "elif condition:"),
            ("else", "else:"),
            ("while", "while condition:"),
            ("for", "for item in list:"),
            ("in", "in"),
            ("match", "match value:"),
            ("return", "return value"),
            ("pass", "pass — no-op placeholder"),
            ("fn", "fn name(params):"),
            ("final", "final class — lightweight class"),
            ("abstract", "abstract class/fn — abstract declaration"),
            ("class", "class Name :: Parent:"),
            ("enum", "enum Name:"),
            ("contract", "contract Name:"),
            ("impl", "Class :: Contract: — implement contract"),
            ("override", "override — override method from parent"),
            ("static", "static — static method"),
            ("import", "import module"),
            ("from", "from module import name"),
            ("as", "as alias | as Type — type cast"),
            ("type", "type Alias = Type — type alias"),
            ("is", "is — type test (value is Type)"),
            ("async", "async — async expression"),
            ("await", "await — await async task"),
            ("defer", "defer — defer function call"),
            ("guard", "guard condition else:"),
            ("loop", "loop: — infinite loop"),
            ("unsafe", "unsafe: — unsafe block"),
            ("break", "break — exit loop"),
            ("continue", "continue — next iteration"),
            ("true", "true — boolean literal"),
            ("false", "false — boolean literal"),
            ("None", "None — absent value"),
            ("ok", "ok(value) — success result"),
            ("error", "error(message) — error result"),
            ("this", "this — current instance"),
            ("super", "super — parent class"),
            (":=", "compile-time constant"),
            ("&T", "mutable type prefix"),
            ("^T", "move/ownership type prefix"),
        ];
        for (kw, detail) in &keywords {
            if prefix.is_empty() || kw.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: kw.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some(detail.to_string()),
                    insert_text: None,
                    sort_text: Some(format!("3{}", kw)),
                    ..Default::default()
                });
            }
        }

        // Suggest importable modules when typing "from " or "import "
        // and exported names after "from X import "
        let line_text: String = source.lines().nth(pos.line as usize).unwrap_or("").to_string();
        let trimmed = line_text.trim_start();
        if trimmed.starts_with("from ") || trimmed.starts_with("import ") {
            // Check if this is "from <module> import <prefix>" — suggest exported names
            if let Some(rest) = trimmed.strip_prefix("from ") {
                if let Some(import_pos) = rest.find(" import") {
                    let mod_path = rest[..import_pos].trim();
                    let after_import = rest[import_pos + 7..].trim_start();
                    let export_prefix = if after_import.is_empty() { "" } else { after_import };
                    if let Some(program) = self.resolve_module(mod_path) {
                        for decl in &program.declarations {
                            let (name, kind, detail) = match decl {
                                Decl::Class(c) => (c.name.clone(), CompletionItemKind::CLASS, "class"),
                                Decl::Struct(s) => (s.name.clone(), CompletionItemKind::STRUCT, "struct"),
                                Decl::Enum(e) => (e.name.clone(), CompletionItemKind::ENUM, "enum"),
                                Decl::Contract(c) => (c.name.clone(), CompletionItemKind::INTERFACE, "contract"),
                                Decl::Function(f) => (f.name.clone(), CompletionItemKind::FUNCTION, "fn"),
                                Decl::Variable(v) => (v.name.clone(), CompletionItemKind::VARIABLE, "var"),
                                Decl::Constant(c) => (c.name.clone(), CompletionItemKind::CONSTANT, "const"),
                                Decl::TypeAlias(t) => (t.name.clone(), CompletionItemKind::TYPE_PARAMETER, "type"),
                                _ => continue,
                            };
                            if name.starts_with('.') || name.starts_with('_') { continue; }
                            let match_prefix = if export_prefix.is_empty() { true }
                                else { name.starts_with(export_prefix) || name.to_lowercase().starts_with(&export_prefix.to_lowercase()) };
                            if match_prefix {
                                items.push(CompletionItem {
                                    label: name.clone(),
                                    kind: Some(kind),
                                    detail: Some(detail.to_string()),
                                    sort_text: Some(format!("2{}", &name)),
                                    ..Default::default()
                                });
                            }
                        }
                        // Skip module scanning if we're suggesting exports
                        // (don't add "from X import " AND module paths)
                    }
                }
            }
            // Only scan for modules if we're not in "from X import" context
            if !trimmed.contains(" import") {
                let search_dirs = self.collect_search_paths(&uri);
                let mut seen_mods = std::collections::HashSet::new();
                for dir in &search_dirs {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                                .unwrap_or_default();
                            if name.starts_with('.') || name.starts_with('_') { continue; }
                            if !seen_mods.insert(name.clone()) { continue; }
                            let is_pkg = path.is_dir() && path.join("lib.ky").exists();
                            if prefix.is_empty() || name.starts_with(&prefix) || name.to_lowercase().starts_with(&prefix_lower) {
                                items.push(CompletionItem {
                                    label: name.clone(),
                                    kind: Some(if is_pkg { CompletionItemKind::MODULE } else { CompletionItemKind::FILE }),
                                    detail: Some(if is_pkg { format!("package {}", name) } else { format!("module {}", name) }),
                                    insert_text: Some(name.clone()),
                                    sort_text: Some(format!("1{}", &name)),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }

        let result = CompletionResponse::Array(items);
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_hover(&mut self, req: Request) {
        let params: HoverParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position_params.position;

        let hover_text = self.find_hover_info(&source, pos.line as usize, pos.character as usize);

        // Compute word range for highlighting the hover target
        let word_range = self.word_range(&source, pos.line as usize, pos.character as usize);

        let result = Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_text)),
            range: word_range,
        };
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn find_hover_info(&self, source: &str, line: usize, col: usize) -> String {
        // We need a URI for import resolution — use a placeholder
        let uri = "file:///input.ky";

        // First try with import resolution
        if let Some((program, analyzer, _)) = self.resolve_and_analyze(uri, source) {
            return self.hover_from_analysis(&program, &analyzer, source, line, col);
        }

        // Fallback: parse + analyze without imports
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(_) => return "KL source file".to_string(),
        };
        let mut source_map = kyc_core::source_map::SourceMap::new();
        let _file_id = source_map.add("input.ky".to_string(), source.to_string());
        let mut analyzer = kyc_semantic::analyzer::SemanticAnalyzer::new()
            .with_source(source_map, "input.ky".to_string());
        analyzer.analyze(&program);
        self.hover_from_analysis(&program, &analyzer, source, line, col)
    }

    fn hover_from_analysis(&self, program: &Program, analyzer: &kyc_semantic::analyzer::SemanticAnalyzer, source: &str, line: usize, col: usize) -> String {
        let symbols = analyzer.type_checker.symbols();

        let word = self.word_at(source, line, col);
        if word.is_empty() {
            return "KL source file".to_string();
        }

        // 1. Check if it's a built-in function (prefer rich description over dummy symbol)
        if let Some(info) = Self::builtin_hover_info(&word) {
            return info;
        }

        // 2. Check local scope (variables within the enclosing function)
        let local_types = Self::build_local_scope(&program, line, col);
        if let Some(type_name) = local_types.get(&word) {
            return format!("`{}`: **{}**", word, type_name);
        }

        // 3. Check global symbol table
        if let Some(sym) = symbols.lookup(&word) {
            return Self::format_symbol_hover(sym, &word);
        }

        // 4. Check if it's a known type
        if symbols.lookup_type(&word).is_some() {
            return format!("`{}` — type", word);
        }

        format!("`{}` — KL identifier", word)
    }

    fn format_symbol_hover(sym: &Symbol, name: &str) -> String {
        match &sym.kind {
            SymKind::Function(f) => {
                let params: Vec<String> = f.params.iter()
                    .map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_)))
                    .collect();
                let mut info = format!("**fn {}({})", f.name, params.join(", "));
                if let Some(rt) = &f.return_type {
                    info.push_str(&format!(" {}", Self::fmt_ast_type(rt)));
                }
                info.push(')');
                info
            }
            SymKind::Variable { type_: Some(t), is_mutable, .. } => {
                let mut info = format!("`{}`: **{}**", name, Self::type_to_name(t));
                if *is_mutable {
                    info.push_str(" (mutable)");
                }
                info
            }
            SymKind::Variable { type_: None, is_mutable, .. } => {
                let mut info = format!("`{}`: inferred", name);
                if *is_mutable {
                    info.push_str(" (mutable)");
                }
                info
            }
            SymKind::Constant(t) => {
                format!("`{}`: **{}** (constant)", name, Self::type_to_name(t))
            }
            SymKind::Struct(s) => {
                let fields: Vec<String> = s.fields.iter()
                    .map(|f| format!("{}: {}", f.name, Self::fmt_ast_type(&f.type_)))
                    .collect();
                format!("**struct {}** — fields: {}", s.name, fields.join(", "))
            }
            SymKind::Class(c) => {
                let mut info = format!("**class {}**", c.name);
                if !c.members.is_empty() {
                    let members: Vec<String> = c.members.iter().map(|m| match m {
                        ClassMember::Field(f) => format!("{}: {}", f.name, Self::fmt_ast_type(&f.type_)),
                        ClassMember::Method(m) => m.name.clone(),
                        ClassMember::Constructor(_) => "constructor".to_string(),
                        ClassMember::Property(_) => "property".to_string(),
                    }).collect();
                    info.push_str(&format!(" — members: {}", members.join(", ")));
                }
                info
            }
            SymKind::Enum(e) => {
                let variants: Vec<String> = e.variants.iter().map(|v| v.name.clone()).collect();
                format!("**enum {}** — variants: {}", e.name, variants.join(", "))
            }
            SymKind::Contract(c) => {
                format!("**contract {}**", c.name)
            }
            SymKind::TypeAlias(t) => {
                format!("**type {}** = {}", t.name, Self::fmt_ast_type(&t.type_))
            }
            SymKind::Module(_) => {
                format!("**module {}**", name)
            }
            SymKind::TypeParam => {
                format!("`{}` — type parameter", name)
            }
        }
    }

    fn builtin_hover_info(word: &str) -> Option<String> {
        let info = match word {
            "print" => "`print(value)` — Print value to stdout",
            "println" => "`println(value)` — Print value with newline",
            "print_int" => "`print_int(n)` — Print integer to stdout",
            "println_int" => "`println_int(n)` — Print integer with newline",
            "print_err" => "`print_err(value)` — Print value to stderr",
            "str" => "`str(value)` — Convert to string",
            "len" => "`len(value)` — Get length of string, list, or dict",
            "int" => "`int(value)` — Convert to integer",
            "float" => "`float(value)` — Convert to float",
            "bool" => "`bool(value)` — Convert to boolean",
            "input" => "`input()` — Read a line from stdin",
            "open" => "`open(path) i64` — Open a file, returns file descriptor",
            "read_str" => "`read_str(fd) str` — Read file content as string",
            "write_str" => "`write_str(fd, str)` — Write string to file",
            "close" => "`close(fd)` — Close a file descriptor",
            "sleep" => "`sleep(ms)` — Sleep for given milliseconds",
            "now" => "`now() i64` — Current Unix timestamp in milliseconds",
            "contains" => "`contains(str, substr) bool` — Check if string contains substring",
            "to_upper" => "`to_upper(str) str` — Convert string to uppercase",
            "to_lower" => "`to_lower(str) str` — Convert string to lowercase",
            "trim" => "`trim(str) str` — Remove leading/trailing whitespace",
            "replace" => "`replace(str, from, to) str` — Replace all occurrences",
            "substr" => "`substr(str, start, len) str` — Extract substring",
            "char_at" => "`char_at(str, index) char` — Get character at index",
            "ord" => "`ord(c) i32` — Get ASCII code of character",
            "is_digit" => "`is_digit(c) bool` — Check if char is a digit",
            "is_alpha" => "`is_alpha(c) bool` — Check if char is alphabetic",
            "is_alnum" => "`is_alnum(c) bool` — Check if char is alphanumeric",
            "is_whitespace" => "`is_whitespace(c) bool` — Check if char is whitespace",
            "is_upper" => "`is_upper(c) bool` — Check if char is uppercase",
            "is_lower" => "`is_lower(c) bool` — Check if char is lowercase",
            "assert" => "`assert(condition)` — Assert condition is true",
            "assert_eq" => "`assert_eq(a, b)` — Assert values are equal",
            "assert_str" => "`assert_str(a, b)` — Assert strings are equal",
            "assert_ne" => "`assert_ne(a, b)` — Assert values are not equal",
            "range" => "`range(start, end) [i32]` — Create a range list",
            "json_parse" => "`json_parse(str)` — Parse JSON string",
            "json_stringify" => "`json_stringify(value) str` — Convert to JSON string",
            "list_push" => "`list_push(list, value)` — Push value to list (mutates)",
            "list_pop" => "`list_pop(list) i64` — Pop last value from list",
            "list_len" => "`list_len(list) i32` — Get list length",
            "ceil" => "`ceil(f64) f64` — Round up to nearest integer",
            "floor" => "`floor(f64) f64` — Round down to nearest integer",
            "round" => "`round(f64) f64` — Round to nearest integer",
            _ => return None,
        };
        Some(info.to_string())
    }

    /// Find the word range at a position (for hover highlighting).
    fn word_range(&self, source: &str, line: usize, col: usize) -> Option<Range> {
        for (i, l) in source.lines().enumerate() {
            if i == line {
                let chars: Vec<char> = l.chars().collect();
                if chars.is_empty() { return None; }
                let anchor = if col == 0 { 0 } else if col >= chars.len() { chars.len() - 1 } else { col - 1 };
                if !chars[anchor].is_alphanumeric() && chars[anchor] != '_' { return None; }
                let mut start = anchor;
                let mut end = anchor + 1;
                while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                    end += 1;
                }
                return Some(Range {
                    start: Position { line: line as u32, character: start as u32 },
                    end: Position { line: line as u32, character: end as u32 },
                });
            }
        }
        None
    }

    fn word_at(&self, source: &str, line: usize, col: usize) -> String {
        for (i, l) in source.lines().enumerate() {
            if i == line {
                let chars: Vec<char> = l.chars().collect();
                if chars.is_empty() {
                    return String::new();
                }
                // LSP `col` is the gap between characters; the character the
                // user just typed sits at chars[col-1].  Anchor on that.
                let anchor = if col == 0 { 0 } else { col - 1 };
                if !chars[anchor].is_alphanumeric() && chars[anchor] != '_' {
                    return String::new();
                }
                let mut start = anchor;
                let mut end = anchor + 1;
                while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                    end += 1;
                }
                return chars[start..end].iter().collect();
            }
        }
        String::new()
    }

    /// Check if the cursor is right after a `.` (either triggered by the
    /// character itself or invoked manually while positioned after one).
    fn is_after_dot(&self, source: &str, line: usize, col: usize, params: &CompletionParams) -> bool {
        if let Some(ctx) = &params.context {
            if ctx.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER
                && ctx.trigger_character.as_deref() == Some(".")
            {
                return true;
            }
        }
        // Manual invocation — check if char before cursor is '.'
        for (i, l) in source.lines().enumerate() {
            if i == line && col > 0 {
                let chars: Vec<char> = l.chars().collect();
                if col - 1 < chars.len() && chars[col - 1] == '.' {
                    return true;
                }
            }
        }
        false
    }

    /// Extract the identifier immediately before a `.` at the cursor.
    /// Cursor position (`col`) is *after* the dot, so we look at `col - 2 ..`.
    fn word_before_dot(&self, source: &str, line: usize, col: usize) -> String {
        for (i, l) in source.lines().enumerate() {
            if i == line {
                let chars: Vec<char> = l.chars().collect();
                if col == 0 {
                    return String::new();
                }
                let dot_idx = col.saturating_sub(1);
                if dot_idx >= chars.len() || chars[dot_idx] != '.' {
                    return String::new();
                }
                let mut start = dot_idx;
                while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                    start -= 1;
                }
                return chars[start..dot_idx].iter().collect();
            }
        }
        String::new()
    }

    /// Resolve dot completions for an expression by running the semantic
    /// pipeline and looking up the expression's type in the symbol table.
    /// `line`/`col` are the cursor position (after the dot) used to find the
    /// enclosing function for local variable resolution.
    fn dot_completions(&self, source: &str, expr: &str, line: usize, col: usize) -> Option<Vec<CompletionItem>> {
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = parser.parse().ok()?;

        let mut source_map = kyc_core::source_map::SourceMap::new();
        let _file_id = source_map.add("input.ky".to_string(), source.to_string());
        let mut analyzer = kyc_semantic::analyzer::SemanticAnalyzer::new()
            .with_source(source_map, "input.ky".to_string());
        analyzer.analyze(&program);

        let symbols = analyzer.type_checker.symbols();

        // Build local scope for variable-to-type mapping
        let local_types = Self::build_local_scope(&program, line, col);

        // Handle chained dot completions: "user.address" → resolve base "user" → get "address" type
        let parts: Vec<&str> = expr.split('.').collect();
        if parts.len() > 1 {
            let base = parts[0];
            // Walk the chain: resolve each part's type
            let mut current_type = Self::resolve_type_name(base, symbols, &local_types);
            for i in 1..parts.len() {
                let prop = parts[i];
                // Look up `prop` as a field of `current_type`
                if let Some(ref tn) = current_type {
                    if let Some(sym) = symbols.lookup(tn) {
                        let field_type = Self::field_type_for_sym(sym, prop);
                        if let Some(ft) = field_type {
                            current_type = Some(ft);
                        } else {
                            // If this is the last part, return completions for current type
                            if i == parts.len() - 1 {
                                return Self::completions_for_named_type(tn, symbols);
                            }
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            // Return completions for the resolved final type
            if let Some(tn) = current_type {
                return Self::completions_for_named_type(&tn, symbols);
            }
            return None;
        }

        // First, try global symbol table — but don't short-circuit on None
        // (variable may exist but have unresolved type)
        if let Some(sym) = symbols.lookup(expr) {
            if let Some(items) = Self::completions_for_sym(sym, symbols) {
                return Some(items);
            }
        }

        // Try local variables from the enclosing function (inferred types)
        if let Some(type_name) = local_types.get(expr) {
            return Self::completions_for_named_type(type_name, symbols);
        }

        // Try treating the expression as a type name directly
        if let Some(items) = Self::completions_for_named_type(expr, symbols) {
            return Some(items);
        }

        None
    }

    /// Resolve an expression name to its type name using symbol table and local scope.
    fn resolve_type_name<'a>(name: &str, symbols: &'a SymbolTable, local_types: &HashMap<String, String>) -> Option<String> {
        // Try global symbols first
        if let Some(sym) = symbols.lookup(name) {
            return match &sym.kind {
                SymKind::Variable { type_: Some(t), .. } => Some(Self::type_to_name(t)),
                SymKind::Constant(t) => Some(Self::type_to_name(t)),
                SymKind::Function(f) => f.return_type.as_ref().map(|t| Self::fmt_ast_type(t)),
                _ => None,
            };
        }
        // Try local scope
        if let Some(type_name) = local_types.get(name) {
            return Some(type_name.clone());
        }
        None
    }

    /// Extract a simple name from a Type for resolution purposes.
    fn type_to_name(t: &Type) -> String {
        match t {
            Type::Named(name) => name.clone(),
            Type::Generic(name, _) => name.clone(),
            Type::Str => "str".to_string(),
            Type::I32 | Type::I64 => "i32".to_string(),
            Type::Bool => "bool".to_string(),
            Type::F32 | Type::F64 => "f64".to_string(),
            Type::Char => "char".to_string(),
            Type::List(_) => "list".to_string(),
            Type::Dict(_, _) => "dict".to_string(),
            Type::Option(inner) => Self::type_to_name(inner),
            _ => String::new(),
        }
    }

    /// Return the field type name for a symbol at a given property, or None.
    fn field_type_for_sym(sym: &Symbol, property: &str) -> Option<String> {
        match &sym.kind {
            SymKind::Struct(s) => {
                s.fields.iter().find(|f| f.name == property)
                    .map(|f| Self::fmt_ast_type(&f.type_))
            }
            SymKind::Class(c) => {
                for member in &c.members {
                    if let ClassMember::Field(f) = member {
                        if f.name == property {
                            return Some(Self::fmt_ast_type(&f.type_));
                        }
                    }
                }
                None
            }
            SymKind::Enum(e) => {
                e.variants.iter().find(|v| v.name == property)
                    .map(|_| format!("{}", e.name)) // enum variant → returns the enum type
            }
            SymKind::Variable { type_: Some(t), .. } => Self::field_type_for_type(t, property),
            SymKind::Constant(t) => Self::field_type_for_type(t, property),
            _ => None,
        }
    }

    /// Return completions for a type name (resolves str, list, dict and user types).
    fn completions_for_named_type(type_name: &str, symbols: &SymbolTable) -> Option<Vec<CompletionItem>> {
        if let Some(sym) = symbols.lookup(type_name) {
            return Self::completions_for_sym(sym, symbols);
        }
        match type_name {
            "str" => Self::completions_for_type(&Type::Str, symbols),
            "list" => Self::completions_for_type(&Type::List(Box::new(Type::TypeVar(0))), symbols),
            "dict" => Self::completions_for_type(&Type::Dict(Box::new(Type::Str), Box::new(Type::TypeVar(0))), symbols),
            _ => None,
        }
    }

    /// Look up a field's type from a Type.
    fn field_type_for_type(t: &Type, _property: &str) -> Option<String> {
        match t {
            Type::Named(name) | Type::Generic(name, _) => {
                // We don't have symbol table access here, so return None
                Some(name.clone()) // best effort
            }
            _ => None,
        }
    }

    /// Walk the program AST to build a map of local variable names to their
    /// type names, scoped to the function containing `(line, col)`.
    fn build_local_scope(program: &Program, line: usize, _col: usize) -> HashMap<String, String> {
        let mut vars: HashMap<String, String> = HashMap::new();
        for decl in &program.declarations {
            if let Decl::Function(f) = decl {
                let l1 = f.span.start.line;
                let l2 = f.span.end.line;
                // KL spans are 1-indexed; cursor is 0-indexed
                if l1 <= line + 1 && line + 1 <= l2 {
                    // Register params
                    for param in &f.params {
                        if param.name == "this" { continue; }
                        let tn = Self::ast_type_name(&param.type_);
                        if !tn.is_empty() {
                            vars.insert(param.name.clone(), tn);
                        }
                    }
                    // Walk function body
                    if let Some(body) = &f.body {
                        Self::walk_block_for_vars(body, &mut vars);
                    }
                }
            }
        }
        vars
    }

    /// Walk a block to collect variable declarations with their types.
    fn walk_block_for_vars(block: &Block, vars: &mut HashMap<String, String>) {
        for stmt in &block.statements {
            match stmt {
                Stmt::Variable(vd) | Stmt::TypedVariable(vd) => {
                    if let Some(t) = &vd.type_ {
                        let tn = Self::ast_type_name(t);
                        if !tn.is_empty() {
                            vars.insert(vd.name.clone(), tn);
                        }
                    } else {
                        let val = &vd.value;
                        if let Some(tn) = Self::infer_type_from_expr(val) {
                            vars.insert(vd.name.clone(), tn);
                        }
                    }
                }
                // Auto-declared variables: `x = expr` (no mut/let)
                Stmt::Expression(Expr::Assignment { target, value, .. }) => {
                    if let Expr::Identifier { name, .. } = target.as_ref() {
                        if !vars.contains_key(name) {
                            if let Some(tn) = Self::infer_type_from_expr(value) {
                                vars.insert(name.clone(), tn);
                            }
                        }
                    }
                }
                Stmt::For(fs) => {
                    vars.insert(fs.variable.clone(), "i32".to_string());
                    if let Some(ref idx_var) = fs.index_variable {
                        vars.insert(idx_var.clone(), "i32".to_string());
                    }
                    Self::walk_block_for_vars(&fs.body, vars);
                    if let Some(eb) = &fs.else_branch {
                        Self::walk_block_for_vars(eb, vars);
                    }
                }
                Stmt::If(is) => {
                    Self::walk_block_for_vars(&is.body, vars);
                    for eb in &is.elif_branches {
                        Self::walk_block_for_vars(&eb.body, vars);
                    }
                    if let Some(eb) = &is.else_branch {
                        Self::walk_block_for_vars(eb, vars);
                    }
                }
                Stmt::While(ws) => {
                    Self::walk_block_for_vars(&ws.body, vars);
                }
                Stmt::Match(ms) => {
                    for arm in &ms.arms {
                        Self::walk_block_for_vars(&arm.body, vars);
                    }
                }
                Stmt::Guard(gs) => {
                    Self::walk_block_for_vars(&gs.body, vars);
                }
                Stmt::Unsafe(us) => {
                    Self::walk_block_for_vars(&us.body, vars);
                }
                _ => {}
            }
        }
    }

    /// Extract the type name string from an AstType.
    fn ast_type_name(t: &AstType) -> String {
        match t {
            AstType::Primitive { name, .. } => name.clone(),
            AstType::User { name, .. } => name.clone(),
            AstType::Generic { name, .. } => name.clone(),
            AstType::Optional { inner, .. } => Self::ast_type_name(inner),
            AstType::Error { inner, .. } => Self::ast_type_name(inner),
            AstType::Dict { .. } => "dict".to_string(),
            AstType::FnPtr { .. } => "fn_ptr".to_string(),
            AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => Self::ast_type_name(inner),
            AstType::Ptr { .. } => "ptr".to_string(),
            AstType::Set { inner, .. } => format!("set<{}>", Self::ast_type_name(inner)),
            AstType::Queue { inner, .. } => format!("queue<{}>", Self::ast_type_name(inner)),
            AstType::Stack { inner, .. } => format!("stack<{}>", Self::ast_type_name(inner)),
            AstType::Deque { inner, .. } => format!("deque<{}>", Self::ast_type_name(inner)),
            AstType::LinkedList { inner, .. } => format!("linked_list<{}>", Self::ast_type_name(inner)),
            AstType::List { inner, .. } => format!("[{}]", Self::ast_type_name(inner)),
            AstType::Array { inner, .. } => format!("[{}]", Self::ast_type_name(inner)),
            AstType::Slice { inner, .. } => format!("&[{}]", Self::ast_type_name(inner)),
            AstType::Chan { inner, .. } => format!("chan<{}>", Self::ast_type_name(inner)),
            AstType::Bytes { .. } => "bytes".to_string(),
        }
    }

    /// Rough type inference from an expression AST for building local scope.
    fn infer_type_from_expr(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Integer(_) => Some("i32".to_string()),
                Literal::Float(_) => Some("f64".to_string()),
                Literal::String(_) => Some("str".to_string()),
                Literal::Boolean(_) => Some("bool".to_string()),
                Literal::Char(_) => Some("char".to_string()),
                Literal::None => None,
                Literal::Null => Some("ptr".to_string()),
            },
            Expr::StructLiteral { struct_name, .. } => Some(struct_name.clone()),
            Expr::Identifier { name, .. } => {
                // Identifiers have unknown type without symbol table — return None
                // but special-case common patterns
                if name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()) {
                    Some("i32".to_string()) // UPPERCASE constants are usually i32
                } else {
                    None
                }
            }
            Expr::Unary { operand, .. } => Self::infer_type_from_expr(operand),
            Expr::Binary { left, operator, right, .. } => {
                use kyc_core::ast::BinaryOp;
                // String concatenation: if either side is str, result is str
                if matches!(operator, BinaryOp::Add) {
                    let lt = Self::infer_type_from_expr(left);
                    let rt = Self::infer_type_from_expr(right);
                    if lt.as_deref() == Some("str") || rt.as_deref() == Some("str") {
                        return Some("str".to_string());
                    }
                }
                // Comparisons always return bool
                if matches!(operator, BinaryOp::Eq | BinaryOp::Neq
                    | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge) {
                    return Some("bool".to_string());
                }
                // Arithmetic: use left operand type (both should match)
                Self::infer_type_from_expr(left)
            }
            Expr::PropertyAccess { object: _object, property, .. } => {
                // For chained completions like `obj.field`, infer from property name
                // This is a simplification — real type resolution needs symbol table
                if property == "len" { return Some("i32".to_string()); }
                if property == "contains" || property == "is_some" || property == "is_none" {
                    return Some("bool".to_string());
                }
                // For struct fields, we'd need the type — fallback to str
                Some("str".to_string())
            }
            Expr::FunctionCall { target, .. } => {
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    match name.as_str() {
                        "input" => Some("str".to_string()),
                        "len" => Some("i32".to_string()),
                        "str" => Some("str".to_string()),
                        "int" => Some("i32".to_string()),
                        "float" => Some("f64".to_string()),
                        "bool" => Some("bool".to_string()),
                        "open" => Some("i64".to_string()),
                        "now" => Some("i64".to_string()),
                        "abs" | "sqrt" | "pow" | "gcd" => Some("i32".to_string()),
                        "json_parse" => Some("dict".to_string()),
                        "json_stringify" => Some("str".to_string()),
                        _ => {
                            // Constructor call: Person(...) → type "Person"
                            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }
                    }
                } else if let Expr::PropertyAccess { object, property, .. } = target.as_ref() {
                    // Method call: obj.method() — infer from method name
                    if property == "to_upper" || property == "to_lower" || property == "trim" || property == "replace" || property == "substr" {
                        Some("str".to_string())
                    } else if property == "contains" || property == "is_some" || property == "is_none" {
                        Some("bool".to_string())
                    } else if property == "len" {
                        Some("i32".to_string())
                    } else if property == "pop" {
                        Some("i64".to_string())
                    } else {
                        Self::infer_type_from_expr(object)
                    }
                } else {
                    None
                }
            }
            Expr::Ternary { then_expr, .. } => Self::infer_type_from_expr(then_expr),
            Expr::List { .. } => Some("list".to_string()),
            Expr::Dictionary { .. } => Some("dict".to_string()),
            Expr::Index { target, .. } => {
                // For list[i], infer element type
                let target_type = Self::infer_type_from_expr(target);
                if target_type.as_deref() == Some("list") || target_type.as_deref() == Some("dict") {
                    Some("i64".to_string()) // default to i64 for list/dict elements
                } else {
                    Some("i64".to_string())
                }
            }
            Expr::OptionalChain { target, .. } => Self::infer_type_from_expr(target),
            Expr::StringInterp { .. } => Some("str".to_string()),
            Expr::Async { .. } | Expr::AsyncBlock { .. } => Some("i64".to_string()), // task handle
            Expr::Spread { expression, .. } => Self::infer_type_from_expr(expression),
            _ => None,
        }
    }

    /// Given a Symbol, return the appropriate dot completions (fields,
    /// methods, enum variants).
    fn completions_for_sym(sym: &Symbol, symbols: &SymbolTable) -> Option<Vec<CompletionItem>> {
        match &sym.kind {
            SymKind::Struct(s) => {
                let items: Vec<CompletionItem> = s.fields.iter().map(|f| {
                    CompletionItem {
                        label: f.name.clone(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(format!("{}: {}", f.name, Self::fmt_ast_type(&f.type_))),
                        sort_text: Some(format!("1{}", f.name)),
                        ..Default::default()
                    }
                }).collect();
                if items.is_empty() { None } else { Some(items) }
            }
            SymKind::Class(c) => {
                let mut items = Vec::new();
                for member in &c.members {
                    match member {
                        ClassMember::Field(f) => {
                            items.push(CompletionItem {
                                label: f.name.clone(),
                                kind: Some(CompletionItemKind::FIELD),
                                detail: Some(format!("{}: {}", f.name, Self::fmt_ast_type(&f.type_))),
                                sort_text: Some(format!("1{}", f.name)),
                                ..Default::default()
                            });
                        }
                        ClassMember::Method(m) => {
                            let params: Vec<String> = m.params.iter()
                                .filter(|p| p.name != "this")
                                .map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_)))
                                .collect();
                            let sig = format!("fn {}({})", m.name, params.join(", "));
                            items.push(CompletionItem {
                                label: m.name.clone(),
                                kind: Some(CompletionItemKind::METHOD),
                                detail: Some(sig),
                                sort_text: Some(format!("2{}", m.name)),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
                if items.is_empty() { None } else { Some(items) }
            }
            SymKind::Enum(e) => {
                let items: Vec<CompletionItem> = e.variants.iter().map(|v| {
                    CompletionItem {
                        label: v.name.clone(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: Some(format!("{}.{}", e.name, v.name)),
                        sort_text: Some(format!("1{}", v.name)),
                        ..Default::default()
                    }
                }).collect();
                if items.is_empty() { None } else { Some(items) }
            }
            SymKind::Variable { type_: Some(t), .. } => {
                Self::completions_for_type(t, symbols)
            }
            SymKind::Constant(t) => {
                Self::completions_for_type(t, symbols)
            }
            _ => None,
        }
    }

    /// Return dot completions for a given resolved Type.
    fn completions_for_type(ty: &Type, symbols: &SymbolTable) -> Option<Vec<CompletionItem>> {
        match ty {
            Type::Named(name) | Type::Generic(name, _) => {
                let sym = symbols.lookup(name)?;
                Self::completions_for_sym(sym, symbols)
            }
            Type::Str => {
                Some(vec![
                    CompletionItem { label: "contains".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn contains(substr: str) bool".into()), sort_text: Some("1contains".into()), ..Default::default() },
                    CompletionItem { label: "to_upper".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn to_upper() str".into()), sort_text: Some("2to_upper".into()), ..Default::default() },
                    CompletionItem { label: "to_lower".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn to_lower() str".into()), sort_text: Some("3to_lower".into()), ..Default::default() },
                    CompletionItem { label: "trim".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn trim() str".into()), sort_text: Some("4trim".into()), ..Default::default() },
                    CompletionItem { label: "replace".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn replace(from: str, to: str) str".into()), sort_text: Some("5replace".into()), ..Default::default() },
                    CompletionItem { label: "substr".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn substr(start: i64, len: i64) str".into()), sort_text: Some("6substr".into()), ..Default::default() },
                    CompletionItem { label: "char_at".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn char_at(index: i64) char".into()), sort_text: Some("7char_at".into()), ..Default::default() },
                    CompletionItem { label: "len".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn len() i32".into()), sort_text: Some("8len".into()), ..Default::default() },
                ])
            }
            Type::List(_) => {
                Some(vec![
                    CompletionItem { label: "push".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn push(value)".into()), sort_text: Some("1push".into()), ..Default::default() },
                    CompletionItem { label: "pop".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn pop() i64".into()), sort_text: Some("2pop".into()), ..Default::default() },
                    CompletionItem { label: "len".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn len() i32".into()), sort_text: Some("3len".into()), ..Default::default() },
                ])
            }
            Type::Dict(_, _) => {
                Some(vec![
                    CompletionItem { label: "len".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn len() i32".into()), sort_text: Some("1len".into()), ..Default::default() },
                    CompletionItem { label: "get".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn get(key)".into()), sort_text: Some("2get".into()), ..Default::default() },
                    CompletionItem { label: "set".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn set(key, value)".into()), sort_text: Some("3set".into()), ..Default::default() },
                    CompletionItem { label: "keys".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn keys() [str]".into()), sort_text: Some("4keys".into()), ..Default::default() },
                    CompletionItem { label: "values".into(), kind: Some(CompletionItemKind::METHOD), detail: Some("fn values() [T]".into()), sort_text: Some("5values".into()), ..Default::default() },
                ])
            }
            _ => None,
        }
    }

    fn fmt_ast_type(t: &AstType) -> String {
        match t {
            AstType::Primitive { name, .. } => name.clone(),
            AstType::User { name, .. } => name.clone(),
            AstType::Generic { name, args, .. } => {
                let args: Vec<String> = args.iter().map(Self::fmt_ast_type).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            AstType::Optional { inner, .. } => format!("{}?", Self::fmt_ast_type(inner)),
            AstType::Error { inner, .. } => format!("{}!", Self::fmt_ast_type(inner)),
            AstType::Dict { key, value, .. } => format!("dict<{}, {}>", Self::fmt_ast_type(key), Self::fmt_ast_type(value)),
            AstType::FnPtr { params, return_, .. } => {
                let args: Vec<String> = params.iter().map(Self::fmt_ast_type).collect();
                format!("fn({}) {}", args.join(", "), Self::fmt_ast_type(return_))
            }
            AstType::Mutable { inner, .. } => format!("^{}", Self::fmt_ast_type(inner)),
            AstType::Borrow { inner, .. } => format!("&{}", Self::fmt_ast_type(inner)),
            AstType::Ptr { .. } => "ptr".to_string(),
            AstType::Set { inner, .. } => format!("set<{}>", Self::fmt_ast_type(inner)),
            AstType::Queue { inner, .. } => format!("queue<{}>", Self::fmt_ast_type(inner)),
            AstType::Stack { inner, .. } => format!("stack<{}>", Self::fmt_ast_type(inner)),
            AstType::Deque { inner, .. } => format!("deque<{}>", Self::fmt_ast_type(inner)),
            AstType::LinkedList { inner, .. } => format!("linked_list<{}>", Self::fmt_ast_type(inner)),
            AstType::List { inner, .. } => format!("[{}]", Self::fmt_ast_type(inner)),
            AstType::Array { inner, size, .. } => format!("[{}; {}]", Self::fmt_ast_type(inner), size),
            AstType::Slice { inner, .. } => format!("&[{}]", Self::fmt_ast_type(inner)),
            AstType::Chan { inner, .. } => format!("chan<{}>", Self::fmt_ast_type(inner)),
            AstType::Bytes { .. } => "bytes".to_string(),
        }
    }

    fn handle_definition(&mut self, req: Request) {
        let params: GotoDefinitionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position_params.position;

        let word = self.word_at(&source, pos.line as usize, pos.character as usize);

        let result: Option<GotoDefinitionResponse> = if word.is_empty() {
            None
        } else {
            // First try current file
            let local_result = self.find_definition_in_source(&source, &word)
                .map(|(span, _)| {
                    let loc = Location {
                        uri: lsp_types::Url::parse(&uri).unwrap(),
                        range: span_to_range(&span),
                    };
                    GotoDefinitionResponse::Scalar(loc)
                });

            if local_result.is_some() {
                local_result
            } else {
                // Try dependency packages
                let file_path = uri.trim_start_matches("file://");
                let project_root = crate::package::find_project_root(
                    &std::path::Path::new(file_path)
                );
                if let Some(root) = project_root {
                    self.find_definition_in_dependencies(&root, &word)
                        .map(|(path, span)| {
                            let dep_uri = format!("file://{}", path.display());
                            let loc = Location {
                                uri: lsp_types::Url::parse(&dep_uri).unwrap(),
                                range: span_to_range(&span),
                            };
                            GotoDefinitionResponse::Scalar(loc)
                        })
                } else {
                    None
                }
            }
        };

        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn find_definition_in_source(&self, source: &str, name: &str) -> Option<(Span, String)> {
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        if let Ok(prog) = parser.parse() {
            for decl in &prog.declarations {
                let result = match decl {
                    Decl::Function(f) if f.name == name => Some((f.span, f.name.clone())),
                    Decl::Variable(v) if v.name == name => Some((v.span, v.name.clone())),
                    Decl::Constant(c) if c.name == name => Some((c.span, c.name.clone())),
                    Decl::Class(c) if c.name == name => Some((c.span, c.name.clone())),
                    Decl::Struct(s) if s.name == name => Some((s.span, s.name.clone())),
                    Decl::Enum(e) if e.name == name => Some((e.span, e.name.clone())),
                    Decl::Contract(c) if c.name == name => Some((c.span, c.name.clone())),
                    Decl::TypeAlias(t) if t.name == name => Some((t.span, t.name.clone())),
                    _ => None,
                };
                if result.is_some() {
                    return result;
                }
            }
        }
        None
    }

    /// Search for a symbol definition across all cached dependency packages.
    fn find_definition_in_dependencies(&self, project_root: &std::path::Path, name: &str) -> Option<(std::path::PathBuf, Span)> {
        use std::fs;

        let lock_path = project_root.join("ky.lock");
        if !lock_path.exists() {
            return None;
        }

        let lock_content = fs::read_to_string(&lock_path).ok()?;
        let lock: crate::package::LockFile = toml::from_str(&lock_content).ok()?;

        for entry in &lock.packages {
            let src_dir = crate::package::cache::package_src_dir(&entry.name, &entry.version);
            if !src_dir.exists() {
                continue;
            }

            let dep_files = match fs::read_dir(&src_dir) {
                Ok(entries) => entries.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |ext| ext == "ky")).collect::<Vec<_>>(),
                Err(_) => continue,
            };

            for dep_file in dep_files {
                let dep_source = match fs::read_to_string(&dep_file) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if let Some((span, _)) = self.find_definition_in_source(&dep_source, name) {
                    return Some((dep_file, span));
                }
            }
        }
        None
    }

    fn handle_document_symbol(&mut self, req: Request) {
        let params: DocumentSymbolParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let symbols = self.collect_symbols(&source, &uri);
        let result = DocumentSymbolResponse::Flat(symbols);
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_workspace_symbol(&mut self, req: Request) {
        let params: WorkspaceSymbolParams = serde_json::from_value(req.params).unwrap();
        let query = params.query.to_lowercase();
        let mut all_symbols: Vec<SymbolInformation> = Vec::new();
        for (uri, source) in &self.sources {
            for sym in self.collect_symbols(source, uri) {
                if query.is_empty() || sym.name.to_lowercase().contains(&query) {
                    all_symbols.push(sym);
                }
            }
        }
        let result = WorkspaceSymbolResponse::Flat(all_symbols);
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_signature_help(&mut self, req: Request) {
        let params: SignatureHelpParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position_params.position;

        let mut lexer = kyc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        if let Ok(prog) = parser.parse() {
            let word = self.word_at(&source, pos.line as usize, pos.character as usize);
            if !word.is_empty() {
                for decl in &prog.declarations {
                    if let Decl::Function(f) = decl {
                        if f.name == word {
                            let params_list: Vec<String> = f.params.iter()
                                .map(|p| format!("{}: {}", p.name, Self::fmt_ast_type(&p.type_)))
                                .collect();
                            let return_info = f.return_type.as_ref()
                                .map(|rt| format!(" -> {}", Self::fmt_ast_type(rt)))
                                .unwrap_or_default();
                            let label = format!("fn {}({}){}", f.name, params_list.join(", "), return_info);
                            let sig = SignatureInformation {
                                label,
                                parameters: Some(vec![]),
                                active_parameter: Some(0),
                                documentation: None,
                            };
                            let result = SignatureHelp {
                                signatures: vec![sig],
                                active_signature: Some(0),
                                active_parameter: Some(0),
                            };
                            let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
                            let _ = self.connection.sender.send(Message::Response(resp));
                            return;
                        }
                    }
                }
            }
        }
        let result = SignatureHelp {
            signatures: vec![],
            active_signature: Some(0),
            active_parameter: Some(0),
        };
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_references(&mut self, req: Request) {
        let params: ReferenceParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position.text_document.uri.to_string();
        let pos = params.text_document_position.position;
        let source = self.sources.get(&uri).cloned().unwrap_or_default();

        let word = self.word_at(&source, pos.line as usize, pos.character as usize);
        let mut locations = Vec::new();
        if !word.is_empty() {
            for (doc_uri, doc_source) in &self.sources {
                if let Ok(url) = lsp_types::Url::parse(doc_uri) {
                    locations.extend(self.find_references_in_source(doc_source, &word, url));
                }
            }
        }
        let resp = Response::new_ok(req.id, serde_json::to_value(locations).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn find_references_in_source(&self, source: &str, name: &str, uri: lsp_types::Url) -> Vec<Location> {
        let mut locations = Vec::new();
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        for tok in &tokens {
            if let TokenKind::Identifier(ref id) = tok.kind {
                if id == name {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: span_to_range(&tok.span),
                    });
                }
            }
        }
        locations
    }

    fn handle_code_action(&mut self, req: Request) {
        let params: CodeActionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        for diag in &params.context.diagnostics {
            if let Some(ref code) = diag.code {
                let code_str = match code {
                    NumberOrString::Number(n) => format!("E{:04}", n),
                    NumberOrString::String(s) => s.clone(),
                };
                if code_str == "E0009" {
                    // Undefined symbol — suggest creating the variable
                    let msg = diag.message.clone();
                    let name = self.extract_symbol_name(&msg);
                    if let Some(var_name) = name {
                        // Create variable code action
                        let edit = WorkspaceEdit {
                            changes: Some({
                                let mut map = HashMap::new();
                                map.insert(
                                    lsp_types::Url::parse(&uri).unwrap(),
                                    vec![TextEdit {
                                        range: Range {
                                            start: Position { line: 0, character: 0 },
                                            end: Position { line: 0, character: 0 },
                                        },
                                        new_text: format!("mut {} = None\n", var_name),
                                    }],
                                );
                                map
                            }),
                            ..Default::default()
                        };
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: format!("Create mutable variable '{}'", var_name),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diag.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));

                        // Import suggestion (if name looks like a module)
                        if var_name.chars().next().unwrap_or(' ').is_uppercase() {
                            let import_edit = WorkspaceEdit {
                                changes: Some({
                                    let mut map = HashMap::new();
                                    map.insert(
                                        lsp_types::Url::parse(&uri).unwrap(),
                                        vec![TextEdit {
                                            range: Range {
                                                start: Position { line: 0, character: 0 },
                                                end: Position { line: 0, character: 0 },
                                            },
                                            new_text: format!("import {}\n", var_name),
                                        }],
                                    );
                                    map
                                }),
                                ..Default::default()
                            };
                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!("Import module '{}'", var_name),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: Some(vec![diag.clone()]),
                                edit: Some(import_edit),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }
        }

        let resp = Response::new_ok(req.id, serde_json::to_value(actions).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_formatting(&mut self, req: Request) {
        let params: DocumentFormattingParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        // Skip formatting for ky.toml
        if uri.ends_with("ky.toml") {
            let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<TextEdit>>(vec![]).unwrap());
            let _ = self.connection.sender.send(Message::Response(resp));
            return;
        }
        let source = match self.sources.get(&uri) {
            Some(s) => s.clone(),
            None => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<TextEdit>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };

        let formatter = crate::formatter::Formatter::new();
        match formatter.format(&source) {
            Ok(formatted) => {
                let lines = source.lines().count();
                let last_line_len = source.lines().last().map(|l| l.len()).unwrap_or(0);
                let full_range = Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: (lines.max(1) as u32).saturating_sub(1), character: last_line_len.max(1) as u32 },
                };
                let edits = vec![TextEdit {
                    range: full_range,
                    new_text: formatted,
                }];
                let resp = Response::new_ok(req.id, serde_json::to_value(edits).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
            }
            Err(_) => {
                let resp = Response::new_ok(req.id, serde_json::to_value::<Vec<TextEdit>>(vec![]).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
            }
        }
    }

    /// Extract the symbol name from a diagnostic message like "'foo' is not defined"
    fn extract_symbol_name(&self, msg: &str) -> Option<String> {
        if let Some(start) = msg.find('\'') {
            let rest = &msg[start + 1..];
            if let Some(end) = rest.find('\'') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }

    fn handle_semantic_tokens_full(&mut self, req: Request) {
        let params: SemanticTokensParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();

        let mut lexer = kyc_frontend::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = match parser.parse() {
            Ok(p) => p,
            Err(_) => {
                let result = SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data: vec![] });
                let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
                let _ = self.connection.sender.send(Message::Response(resp));
                return;
            }
        };

        let mut source_map = kyc_core::source_map::SourceMap::new();
        let _file_id = source_map.add("input.ky".to_string(), source.to_string());
        let mut analyzer = kyc_semantic::analyzer::SemanticAnalyzer::new()
            .with_source(source_map, "input.ky".to_string());
        analyzer.analyze(&program);
        let symbols = analyzer.type_checker.symbols();

        // Token type indices matching the legend in server_capabilities
        const T_VARIABLE: u32 = 0;
        const T_TYPE: u32 = 1;
        const T_CLASS: u32 = 2;
        const T_STRUCT: u32 = 3;
        const T_ENUM: u32 = 4;
        const T_FUNCTION: u32 = 5;
        const T_METHOD: u32 = 6;
        const T_PARAMETER: u32 = 7;
        const T_PROPERTY: u32 = 8;

        // Modifier indices (bitmask values matching legend order)
        const M_MODIFICATION: u32 = 1;
        const M_DECLARATION: u32 = 2;
        const M_READONLY: u32 = 4;

        let mut semantic_tokens: Vec<(usize, usize, usize, u32, u32)> = Vec::new();

        // Walk all declarations
        for decl in &program.declarations {
            match decl {
                Decl::Function(f) => {
                    // Function name
                    semantic_tokens.push((f.span.start.line, f.span.start.column, f.name.len(), T_FUNCTION, M_DECLARATION));
                    // Parameters
                    for p in &f.params {
                        semantic_tokens.push((p.span.start.line, p.span.start.column, p.name.len(), T_PARAMETER, M_DECLARATION));
                    }
                    // Walk body
                    if let Some(body) = &f.body {
                        Self::walk_semantic_block(body, &mut semantic_tokens, symbols);
                    }
                }
                Decl::Variable(v) => {
                    let modifier = if v.is_mutable { M_MODIFICATION } else { M_READONLY };
                    semantic_tokens.push((v.span.start.line, v.span.start.column, v.name.len(), T_VARIABLE, M_DECLARATION | modifier));
                    if let Some(ref t) = v.type_ {
                        Self::walk_semantic_type(t, &mut semantic_tokens, symbols);
                    }
                    Self::walk_semantic_expr(&v.value, &mut semantic_tokens, symbols);
                }
                Decl::Constant(c) => {
                    semantic_tokens.push((c.span.start.line, c.span.start.column, c.name.len(), T_VARIABLE, M_DECLARATION | M_READONLY));
                    Self::walk_semantic_expr(&c.value, &mut semantic_tokens, symbols);
                }
                Decl::Class(c) => {
                    semantic_tokens.push((c.span.start.line, c.span.start.column, c.name.len(), T_CLASS, M_DECLARATION));
                    for member in &c.members {
                        match member {
                            ClassMember::Field(f) => {
                                semantic_tokens.push((f.span.start.line, f.span.start.column, f.name.len(), T_PROPERTY, M_DECLARATION));
                                Self::walk_semantic_type(&f.type_, &mut semantic_tokens, symbols);
                            }
                            ClassMember::Method(m) => {
                                semantic_tokens.push((m.span.start.line, m.span.start.column, m.name.len(), T_METHOD, M_DECLARATION));
                                for p in &m.params {
                                    semantic_tokens.push((p.span.start.line, p.span.start.column, p.name.len(), T_PARAMETER, M_DECLARATION));
                                }
                                if let Some(body) = &m.body {
                                    Self::walk_semantic_block(body, &mut semantic_tokens, symbols);
                                }
                            }
                            ClassMember::Constructor(ctor) => {
                                for p in &ctor.params {
                                    semantic_tokens.push((p.span.start.line, p.span.start.column, p.name.len(), T_PARAMETER, M_DECLARATION));
                                }
                                Self::walk_semantic_block(&ctor.body, &mut semantic_tokens, symbols);
                            }
                            ClassMember::Property(p) => {
                                semantic_tokens.push((p.span.start.line, p.span.start.column, p.name.len(), T_PROPERTY, M_DECLARATION));
                            }
                        }
                    }
                }
                Decl::Struct(s) => {
                    semantic_tokens.push((s.span.start.line, s.span.start.column, s.name.len(), T_STRUCT, M_DECLARATION));
                    for f in &s.fields {
                        semantic_tokens.push((f.span.start.line, f.span.start.column, f.name.len(), T_PROPERTY, M_DECLARATION));
                        Self::walk_semantic_type(&f.type_, &mut semantic_tokens, symbols);
                    }
                }
                Decl::Enum(e) => {
                    semantic_tokens.push((e.span.start.line, e.span.start.column, e.name.len(), T_ENUM, M_DECLARATION));
                }
                Decl::Contract(c) => {
                    semantic_tokens.push((c.span.start.line, c.span.start.column, c.name.len(), T_CLASS, M_DECLARATION));
                }
                Decl::FromImport(fi) => {
                    for name in &fi.imported_names {
                        // Heuristic: uppercase → type/class, lowercase → function/variable
                        let token_type = if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                            T_TYPE
                        } else {
                            T_FUNCTION
                        };
                        // Use rough position: span.end minus name length for each name
                        let col = if name.len() < fi.span.end.column as usize {
                            fi.span.end.column as usize - name.len()
                        } else { 0 };
                        semantic_tokens.push((fi.span.end.line, col, name.len(), token_type, 0));
                    }
                }
                _ => {}
            }
        }

        // Sort by (line, column) for encoding
        semantic_tokens.sort_by_key(|t| (t.0, t.1));

        // Encode as delta-encoded SemanticToken array
        let mut data: Vec<SemanticToken> = Vec::new();
        let mut prev_line: usize = 0;
        let mut prev_col: usize = 0;
        // Spans from the lexer are 1-indexed; LSP positions are 0-indexed.
        for (line, col, len, ty, mods) in &semantic_tokens {
            let line0 = line.saturating_sub(1);
            let col0 = col.saturating_sub(1);
            let d_line = if data.is_empty() { line0 as u32 } else { (line0 - prev_line) as u32 };
            let d_start = if data.is_empty() || line0 != prev_line { col0 as u32 } else { (col0 - prev_col) as u32 };
            data.push(SemanticToken {
                delta_line: d_line,
                delta_start: d_start,
                length: *len as u32,
                token_type: *ty,
                token_modifiers_bitset: *mods,
            });
            prev_line = line0;
            prev_col = col0;
        }

        let result = SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data });
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_prepare_rename(&mut self, req: Request) {
        let params: TextDocumentPositionParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.position;
        let word = self.word_at(&source, pos.line as usize, pos.character as usize);

        let result = if word.is_empty() {
            None
        } else {
            Some(PrepareRenameResponse::RangeWithPlaceholder {
                range: Range {
                    start: Position { line: pos.line, character: (pos.character as usize).saturating_sub(word.len()) as u32 },
                    end: pos,
                },
                placeholder: word,
            })
        };
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn handle_rename(&mut self, req: Request) {
        let params: RenameParams = serde_json::from_value(req.params).unwrap();
        let uri = params.text_document_position.text_document.uri.to_string();
        let source = self.sources.get(&uri).cloned().unwrap_or_default();
        let pos = params.text_document_position.position;
        let new_name = params.new_name.clone();

        let word = self.word_at(&source, pos.line as usize, pos.character as usize);
        let result: Option<WorkspaceEdit> = if word.is_empty() {
            None
        } else {
            let mut changes: HashMap<lsp_types::Url, Vec<TextEdit>> = HashMap::new();
            for (doc_uri, doc_source) in &self.sources {
                if let Ok(url) = lsp_types::Url::parse(doc_uri) {
                    let mut edits = Vec::new();
                    let mut lexer = kyc_frontend::lexer::Lexer::new(doc_source);
                    let tokens = lexer.tokenize();
                    for tok in &tokens {
                        if let TokenKind::Identifier(ref id) = tok.kind {
                            if id == &word {
                                edits.push(TextEdit {
                                    range: span_to_range(&tok.span),
                                    new_text: new_name.clone(),
                                });
                            }
                        }
                    }
                    if !edits.is_empty() {
                        changes.insert(url, edits);
                    }
                }
            }
            Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            })
        };
        let resp = Response::new_ok(req.id, serde_json::to_value(result).unwrap());
        let _ = self.connection.sender.send(Message::Response(resp));
    }

    fn collect_symbols(&self, source: &str, uri: &str) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let url = lsp_types::Url::parse(uri).unwrap_or_else(|_| lsp_types::Url::parse("file:///unknown.ky").unwrap());
        if let Ok(prog) = parser.parse() {
            for decl in &prog.declarations {
                let (name, kind, span) = match decl {
                    Decl::Function(f) => (f.name.clone(), SymbolKind::FUNCTION, f.span),
                    Decl::Variable(v) => (v.name.clone(), SymbolKind::VARIABLE, v.span),
                    Decl::Constant(c) => (c.name.clone(), SymbolKind::CONSTANT, c.span),
                    Decl::Class(c) => (c.name.clone(), SymbolKind::CLASS, c.span),
                    Decl::Struct(s) => (s.name.clone(), SymbolKind::STRUCT, s.span),
                    Decl::Enum(e) => (e.name.clone(), SymbolKind::ENUM, e.span),
                    Decl::Contract(c) => (c.name.clone(), SymbolKind::INTERFACE, c.span),
                    Decl::TypeAlias(t) => (t.name.clone(), SymbolKind::TYPE_PARAMETER, t.span),
                    _ => continue,
                };
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name,
                    kind,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: url.clone(),
                        range: span_to_range(&span),
                    },
                    container_name: None,
                });
            }
        }
        symbols
    }

    fn walk_semantic_block(block: &Block, tokens: &mut Vec<(usize, usize, usize, u32, u32)>, symbols: &SymbolTable) {
        const T_VARIABLE: u32 = 0;
        const M_DECLARATION: u32 = 2;
        const M_MODIFICATION: u32 = 1;
        const M_READONLY: u32 = 4;

        for stmt in &block.statements {
            match stmt {
                Stmt::Variable(vd) | Stmt::TypedVariable(vd) => {
                    let modifier = if vd.is_mutable { M_MODIFICATION } else { M_READONLY };
                    tokens.push((vd.span.start.line, vd.span.start.column, vd.name.len(), T_VARIABLE, M_DECLARATION | modifier));
                    if let Some(ref t) = vd.type_ {
                        Self::walk_semantic_type(t, tokens, symbols);
                    }
                    Self::walk_semantic_expr(&vd.value, tokens, symbols);
                }
                Stmt::Constant(c) => {
                    tokens.push((c.span.start.line, c.span.start.column, c.name.len(), T_VARIABLE, M_DECLARATION | M_READONLY));
                    Self::walk_semantic_expr(&c.value, tokens, symbols);
                }
                Stmt::Expression(expr) => {
                    Self::walk_semantic_expr(expr, tokens, symbols);
                }
                Stmt::Return(ret) => {
                    if let Some(e) = ret {
                        Self::walk_semantic_expr(e, tokens, symbols);
                    }
                }
                Stmt::Break(ret, _label) => {
                    if let Some(e) = ret {
                        Self::walk_semantic_expr(e, tokens, symbols);
                    }
                }
                Stmt::If(is) => {
                    Self::walk_semantic_expr(&is.condition, tokens, symbols);
                    Self::walk_semantic_block(&is.body, tokens, symbols);
                    for el in &is.elif_branches {
                        Self::walk_semantic_expr(&el.condition, tokens, symbols);
                        Self::walk_semantic_block(&el.body, tokens, symbols);
                    }
                    if let Some(ref eb) = is.else_branch {
                        Self::walk_semantic_block(eb, tokens, symbols);
                    }
                }
                Stmt::BindingIf(bi) => {
                    tokens.push((bi.span.start.line, bi.span.start.column, bi.name.len(), T_VARIABLE, M_DECLARATION));
                    Self::walk_semantic_expr(&bi.value, tokens, symbols);
                    Self::walk_semantic_block(&bi.body, tokens, symbols);
                    if let Some(ref eb) = bi.else_branch {
                        Self::walk_semantic_block(eb, tokens, symbols);
                    }
                }
                Stmt::While(ws) => {
                    Self::walk_semantic_expr(&ws.condition, tokens, symbols);
                    Self::walk_semantic_block(&ws.body, tokens, symbols);
                    if let Some(ref eb) = ws.else_branch {
                        Self::walk_semantic_block(eb, tokens, symbols);
                    }
                }
                Stmt::WhileBind(wb) => {
                    tokens.push((wb.span.start.line, wb.span.start.column, wb.name.len(), T_VARIABLE, M_DECLARATION));
                    Self::walk_semantic_expr(&wb.iterable, tokens, symbols);
                    Self::walk_semantic_block(&wb.body, tokens, symbols);
                }
                Stmt::For(fs) => {
                    let base_col = fs.span.start.column;
                    if let Some(ref idx_var) = fs.index_variable {
                        tokens.push((fs.span.start.line, base_col + 4, idx_var.len(), T_VARIABLE, M_DECLARATION));
                        tokens.push((fs.span.start.line, base_col + 5 + idx_var.len(), fs.variable.len(), T_VARIABLE, M_DECLARATION));
                    } else {
                        tokens.push((fs.span.start.line, fs.span.start.column, fs.variable.len(), T_VARIABLE, M_DECLARATION));
                    }
                    Self::walk_semantic_expr(&fs.iterable, tokens, symbols);
                    Self::walk_semantic_block(&fs.body, tokens, symbols);
                    if let Some(ref eb) = fs.else_branch {
                        Self::walk_semantic_block(eb, tokens, symbols);
                    }
                }
                Stmt::Match(ms) => {
                    Self::walk_semantic_expr(&ms.expression, tokens, symbols);
                    for arm in &ms.arms {
                        Self::walk_semantic_block(&arm.body, tokens, symbols);
                    }
                }
                Stmt::Guard(gs) => {
                    Self::walk_semantic_expr(&gs.condition, tokens, symbols);
                    Self::walk_semantic_block(&gs.body, tokens, symbols);
                }
                Stmt::Unsafe(us) => {
                    Self::walk_semantic_block(&us.body, tokens, symbols);
                }
                Stmt::Defer(ds) => {
                    Self::walk_semantic_expr(&ds.call, tokens, symbols);
                }
                Stmt::Continue(_) => {}
            }
        }
    }

    fn walk_semantic_expr(expr: &Expr, tokens: &mut Vec<(usize, usize, usize, u32, u32)>, symbols: &SymbolTable) {
        const T_VARIABLE: u32 = 0;
        const T_TYPE: u32 = 1;
        const T_FUNCTION: u32 = 5;
        const T_METHOD: u32 = 6;

        match expr {
            Expr::Identifier { name, span } => {
                if let Some(sym) = symbols.lookup(name) {
                    let (ty, mods) = match &sym.kind {
                        SymKind::Variable { is_mutable, .. } => {
                            (T_VARIABLE, if *is_mutable { 1 } else { 4 })
                        }
                        SymKind::Function(_) => (T_FUNCTION, 0),
                        SymKind::Class(_) => (T_TYPE, 0),
                        SymKind::Struct(_) => (T_TYPE, 0),
                        SymKind::Enum(_) => (T_TYPE, 0),
                        SymKind::Constant(_) => (T_VARIABLE, 4),
                        _ => return,
                    };
                    tokens.push((span.start.line, span.start.column, name.len(), ty, mods));
                } else {
                    match name.as_str() {
                        "i32" | "i64" | "i8" | "i16" | "u8" | "u16" | "u32" | "u64"
                        | "f32" | "f64" | "bool" | "str" | "char" | "void" | "any" => {
                            tokens.push((span.start.line, span.start.column, name.len(), T_TYPE, 0));
                        }
                        _ => {}
                    }
                }
            }
            Expr::FunctionCall { target, arguments, .. } => {
                Self::walk_semantic_expr(target, tokens, symbols);
                for arg in arguments {
                    Self::walk_semantic_expr(arg, tokens, symbols);
                }
            }
            Expr::PropertyAccess { object, property, span } => {
                Self::walk_semantic_expr(object, tokens, symbols);
                tokens.push((span.end.line, span.end.column.saturating_sub(property.len()), property.len(), T_METHOD, 0));
            }
            Expr::Binary { left, right, .. } => {
                Self::walk_semantic_expr(left, tokens, symbols);
                Self::walk_semantic_expr(right, tokens, symbols);
            }
            Expr::Unary { operand, .. } => {
                Self::walk_semantic_expr(operand, tokens, symbols);
            }
            Expr::Assignment { target, value, .. } => {
                Self::walk_semantic_expr(target, tokens, symbols);
                Self::walk_semantic_expr(value, tokens, symbols);
            }
            Expr::IfExpr { .. } => unreachable!("desugared in HIR"),
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                Self::walk_semantic_expr(cond, tokens, symbols);
                Self::walk_semantic_expr(then_expr, tokens, symbols);
                Self::walk_semantic_expr(else_expr, tokens, symbols);
            }
            Expr::Index { target, index, .. } => {
                Self::walk_semantic_expr(target, tokens, symbols);
                Self::walk_semantic_expr(index, tokens, symbols);
            }
            Expr::RangeSlice { target, start, end, .. } => {
                Self::walk_semantic_expr(target, tokens, symbols);
                if let Some(s) = start {
                    Self::walk_semantic_expr(s, tokens, symbols);
                }
                if let Some(e) = end {
                    Self::walk_semantic_expr(e, tokens, symbols);
                }
            }
            Expr::List { elements, .. } => {
                for e in elements {
                    Self::walk_semantic_expr(e, tokens, symbols);
                }
            }
            Expr::Tuple { elements, .. } => {
                for e in elements {
                    Self::walk_semantic_expr(e, tokens, symbols);
                }
            }
            Expr::Dictionary { entries, .. } => {
                for (_k, v) in entries {
                    Self::walk_semantic_expr(v, tokens, symbols);
                }
            }
            Expr::StructLiteral { fields, .. } => {
                for (_name, expr) in fields {
                    Self::walk_semantic_expr(expr, tokens, symbols);
                }
            }
            Expr::OptionalChain { target, .. } => {
                Self::walk_semantic_expr(target, tokens, symbols);
            }
            Expr::Spread { expression, .. } => {
                Self::walk_semantic_expr(expression, tokens, symbols);
            }
            Expr::Closure { body, .. } => {
                Self::walk_semantic_expr(body, tokens, symbols);
            }
            Expr::Await { expression, .. } => {
                Self::walk_semantic_expr(expression, tokens, symbols);
            }
            Expr::Async { expression, .. } => {
                Self::walk_semantic_expr(expression, tokens, symbols);
            }
            Expr::ErrorProp { expression, .. } => {
                Self::walk_semantic_expr(expression, tokens, symbols);
            }
            Expr::Loop { body, .. } => {
                Self::walk_semantic_block(body, tokens, symbols);
            }
            Expr::StringInterp { parts, .. } => {
                for p in parts {
                    Self::walk_semantic_expr(p, tokens, symbols);
                }
            }
            Expr::MatchExpr { expression, arms, .. } => {
                Self::walk_semantic_expr(expression, tokens, symbols);
                for arm in arms {
                    Self::walk_semantic_block(&arm.body, tokens, symbols);
                }
            }
            _ => {}
        }
    }

    fn walk_semantic_type(ast_type: &AstType, tokens: &mut Vec<(usize, usize, usize, u32, u32)>, symbols: &SymbolTable) {
        const T_TYPE: u32 = 1;
        match ast_type {
            AstType::Primitive { name, span } => {
                tokens.push((span.start.line, span.start.column, name.len(), T_TYPE, 0));
            }
            AstType::User { name, span } => {
                tokens.push((span.start.line, span.start.column, name.len(), T_TYPE, 0));
            }
            AstType::Generic { name, args, span } => {
                tokens.push((span.start.line, span.start.column, name.len(), T_TYPE, 0));
                for p in args {
                    Self::walk_semantic_type(p, tokens, symbols);
                }
            }
            AstType::Optional { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Error { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Dict { key, value, .. } => {
                Self::walk_semantic_type(key, tokens, symbols);
                Self::walk_semantic_type(value, tokens, symbols);
            }
            AstType::FnPtr { params, return_, .. } => {
                for p in params {
                    Self::walk_semantic_type(p, tokens, symbols);
                }
                Self::walk_semantic_type(return_, tokens, symbols);
            }
            AstType::Mutable { inner, .. } | AstType::Borrow { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Ptr { .. } => {}
            AstType::Set { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Queue { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Stack { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Deque { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::LinkedList { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::List { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Array { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Slice { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Chan { inner, .. } => {
                Self::walk_semantic_type(inner, tokens, symbols);
            }
            AstType::Bytes { .. } => {}
        }
    }
}

/// Convert KL Span (1-indexed) to LSP Range (0-indexed).
fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position {
            line: span.start.line.saturating_sub(1) as u32,
            character: span.start.column.saturating_sub(1) as u32,
        },
        end: Position {
            line: span.end.line.saturating_sub(1) as u32,
            character: span.end.column.saturating_sub(1) as u32,
        },
    }
}

/// Apply an LSP range-based text change to a source string.
/// Range is 0-indexed line/character.
fn apply_range_change(source: &str, range: &Range, new_text: &str) -> String {
    let start_offset = offset_from_position(source, range.start.line as usize, range.start.character as usize);
    let end_offset = offset_from_position(source, range.end.line as usize, range.end.character as usize);
    let mut result = String::with_capacity(source.len() + new_text.len());
    result.push_str(&source[..start_offset]);
    result.push_str(new_text);
    result.push_str(&source[end_offset..]);
    result
}

/// Convert a 0-indexed line/character position to a byte offset in the source.
fn offset_from_position(source: &str, line: usize, character: usize) -> usize {
    let mut current_line = 0usize;
    let mut offset = 0usize;
    let bytes = source.as_bytes();
    while current_line < line && offset < bytes.len() {
        if bytes[offset] == b'\n' {
            current_line += 1;
        }
        offset += 1;
    }
    // Now at start of target line; advance by `character` bytes
    let line_start = offset;
    let end = bytes.len().min(line_start + character);
    // Don't go past end of line
    let mut col_offset = line_start;
    while col_offset < end && col_offset < bytes.len() && bytes[col_offset] != b'\n' {
        col_offset += 1;
    }
    col_offset
}

/// Convert a byte span (`Range<usize>`) from TOML errors to an LSP range.
/// Returns `None` if the span is out of bounds.
fn byte_span_to_range(source: &str, span: &std::ops::Range<usize>) -> Option<Range> {
    let bytes = source.as_bytes();
    if span.start >= bytes.len() {
        return None;
    }
    let start_line = bytes[..span.start].iter().filter(|&&b| b == b'\n').count() as u32;
    let start_col = bytes[..span.start].iter().rev()
        .take_while(|&&b| b != b'\n')
        .count() as u32;

    let end = span.end.min(bytes.len());
    let end_line = bytes[..end].iter().filter(|&&b| b == b'\n').count() as u32;
    let end_col = bytes[..end].iter().rev()
        .take_while(|&&b| b != b'\n')
        .count() as u32;

    Some(Range {
        start: Position { line: start_line, character: start_col },
        end: Position { line: end_line, character: end_col },
    })
}
