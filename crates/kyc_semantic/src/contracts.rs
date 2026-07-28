use std::collections::HashMap;
use kyc_core::ast::*;
use kyc_core::ast::AstType;
use kyc_core::diagnostic::{Diagnostic, ErrorCode, DiagnosticReporter};

/// Compare two AstType values ignoring span differences
fn types_match_ignore_span(a: &AstType, b: &AstType) -> bool {
    match (a, b) {
        (AstType::User { name: n1, .. }, AstType::User { name: n2, .. }) => n1 == n2,
        (AstType::Primitive { name: n1, .. }, AstType::Primitive { name: n2, .. }) => n1 == n2,
        (AstType::Mutable { inner: i1, .. }, AstType::Mutable { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Borrow { inner: i1, .. }, AstType::Borrow { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Generic { name: n1, args: a1, .. }, AstType::Generic { name: n2, args: a2, .. }) => {
            n1 == n2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| types_match_ignore_span(x, y))
        }
        (AstType::Optional { inner: i1, .. }, AstType::Optional { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Error { inner: i1, .. }, AstType::Error { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Dict { key: k1, value: v1, .. }, AstType::Dict { key: k2, value: v2, .. }) => {
            types_match_ignore_span(k1, k2) && types_match_ignore_span(v1, v2)
        }
        (AstType::FnPtr { params: p1, return_: r1, .. }, AstType::FnPtr { params: p2, return_: r2, .. }) => {
            p1.len() == p2.len() && p1.iter().zip(p2).all(|(x, y)| types_match_ignore_span(x, y))
            && types_match_ignore_span(r1, r2)
        }
        (AstType::Set { inner: i1, .. }, AstType::Set { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Queue { inner: i1, .. }, AstType::Queue { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Stack { inner: i1, .. }, AstType::Stack { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::List { inner: i1, .. }, AstType::List { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Array { inner: i1, size: s1, .. }, AstType::Array { inner: i2, size: s2, .. }) => {
            s1 == s2 && types_match_ignore_span(i1, i2)
        }
        (AstType::Slice { inner: i1, .. }, AstType::Slice { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Chan { inner: i1, .. }, AstType::Chan { inner: i2, .. }) => types_match_ignore_span(i1, i2),
        (AstType::Bytes { .. }, AstType::Bytes { .. }) => true,
        (AstType::Ptr { .. }, AstType::Ptr { .. }) => true,
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

pub struct ContractChecker {
    contracts: HashMap<String, ContractDecl>,
}

impl ContractChecker {
    pub fn new() -> Self {
        Self { contracts: HashMap::new() }
    }

    pub fn register_contract(&mut self, contract: &ContractDecl) {
        self.contracts.insert(contract.name.clone(), contract.clone());
    }

    pub fn check_all_classes(&self, classes: &[&ClassDecl], reporter: &mut DiagnosticReporter) {
        for class in classes {
            for contract_name in &class.contracts {
                if let Some(contract) = self.contracts.get(contract_name) {
                    self.validate_class_contract(class, contract, reporter);
                } else {
                    reporter.report(
                        Diagnostic::error(ErrorCode::E0009,
                            format!("contract '{}' not found", contract_name))
                            .with_span(class.span)
                    );
                }
            }
        }
    }

    fn validate_class_contract(&self, class: &ClassDecl, contract: &ContractDecl, reporter: &mut DiagnosticReporter) {
        for req_method in &contract.methods {
            let found = class.members.iter().any(|member| {
                if let ClassMember::Method(m) = member {
                    if m.name != req_method.name { return false; }
                    // Skip auto-injected `this`/`self` param for instance methods
                    let class_params = if !m.is_static && m.params.first().map_or(false, |p| p.name == "this" || p.name == "self") {
                        &m.params[1..]
                    } else {
                        &m.params[..]
                    };
                    if class_params.len() != req_method.params.len() { return false; }
                    class_params.iter().zip(&req_method.params).all(|(a, b)| {
                        types_match_ignore_span(&a.type_, &b.type_)
                    })
                } else {
                    false
                }
            });
            if !found {
                reporter.report(
                    Diagnostic::error(ErrorCode::E0001,
                        format!("class '{}' does not implement contract method '{}'",
                            class.name, req_method.name))
                        .with_span(class.span)
                );
            }
        }
    }
}