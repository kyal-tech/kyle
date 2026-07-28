// kyc_semantic::analyzer — Top-level semantic analysis orchestrator
//
// Coordinates the full semantic analysis pipeline:
//   1. Scope resolution & symbol table construction
//   2. Type checking & inference
//   3. Contract validation
//   4. Diagnostics reporting

use kyc_core::ast::{Program, Decl};
use kyc_core::source_map::SourceMap;
use kyc_core::diagnostic::DiagnosticReporter;

use crate::type_checker::TypeChecker;
use crate::contracts::ContractChecker;

pub struct SemanticAnalyzer {
    pub type_checker: TypeChecker,
    pub contract_checker: ContractChecker,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            type_checker: TypeChecker::new(),
            contract_checker: ContractChecker::new(),
        }
    }

    pub fn with_source(mut self, source_map: SourceMap, name: String) -> Self {
        self.type_checker = std::mem::take(&mut self.type_checker)
            .with_source(source_map, name);
        self
    }

    pub fn analyze(&mut self, program: &Program) {
        // Phase 1: Register contracts
        for decl in &program.declarations {
            if let Decl::Contract(c) = decl {
                self.contract_checker.register_contract(c);
            }
        }

        // Phase 2: Type checking
        self.type_checker.check_program(program);

        // Phase 3: Contract validation
        let classes: Vec<&kyc_core::ast::ClassDecl> = program.declarations.iter()
            .filter_map(|d| {
                if let Decl::Class(c) = d { Some(c) } else { None }
            })
            .collect();
        if !classes.is_empty() {
            self.contract_checker.check_all_classes(&classes, &mut self.type_checker.reporter);
        }
    }

    pub fn has_errors(&self) -> bool {
        self.type_checker.has_errors()
    }

    pub fn reporter(&self) -> &DiagnosticReporter {
        &self.type_checker.reporter
    }

    pub fn emit_diagnostics(&self) {
        self.type_checker.emit_diagnostics();
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
