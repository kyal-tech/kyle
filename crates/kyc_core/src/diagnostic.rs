// kyc_core::diagnostic — Error and warning reporting system
//
// Reference: docs/14-error-catalog.md, docs/05-error-system.md

use std::fmt;
use crate::span::Span;
use crate::source_map::SourceMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Ice,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorCode {
    E0001, E0002, E0003, E0004, E0005, E0006, E0007, E0008, E0009,
    E0010, E0011, E0012, E0013, E0014, E0015, E0016, E0017, E0018,
    E0019, E0020,
    W0001, W0002, W0003,
    P0001,
}

impl ErrorCode {
    pub fn code_str(&self) -> &'static str {
        match self {
            ErrorCode::E0001 => "E0001",
            ErrorCode::E0002 => "E0002",
            ErrorCode::E0003 => "E0003",
            ErrorCode::E0004 => "E0004",
            ErrorCode::E0005 => "E0005",
            ErrorCode::E0006 => "E0006",
            ErrorCode::E0007 => "E0007",
            ErrorCode::E0008 => "E0008",
            ErrorCode::E0009 => "E0009",
            ErrorCode::E0010 => "E0010",
            ErrorCode::E0011 => "E0011",
            ErrorCode::E0012 => "E0012",
            ErrorCode::E0013 => "E0013",
            ErrorCode::E0014 => "E0014",
            ErrorCode::E0015 => "E0015",
            ErrorCode::E0016 => "E0016",
            ErrorCode::E0017 => "E0017",
            ErrorCode::E0018 => "E0018",
            ErrorCode::E0019 => "E0019",
            ErrorCode::E0020 => "E0020",
            ErrorCode::W0001 => "W0001",
            ErrorCode::W0002 => "W0002",
            ErrorCode::W0003 => "W0003",
            ErrorCode::P0001 => "P0001",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            ErrorCode::E0001 => "Type mismatch",
            ErrorCode::E0002 => "Unhandled error value",
            ErrorCode::E0003 => "Unsafe operation outside unsafe block",
            ErrorCode::E0004 => "Non-exhaustive match",
            ErrorCode::E0005 => "Unreachable code",
            ErrorCode::E0006 => "Circular dependency detected",
            ErrorCode::E0007 => "Cannot modify constant",
            ErrorCode::E0008 => "Optional value not checked",
            ErrorCode::E0009 => "Undefined symbol",
            ErrorCode::E0010 => "Potential data loss",
            ErrorCode::E0011 => "Integer overflow",
            ErrorCode::E0012 => "Division by zero",
            ErrorCode::E0013 => "Invalid UTF-8 sequence",
            ErrorCode::E0014 => "Cannot access private member",
            ErrorCode::E0015 => "Unused import",
            ErrorCode::E0016 => "Dead code",
            ErrorCode::E0017 => "Unknown generic type",
            ErrorCode::E0018 => "Fallible function not handled",
            ErrorCode::E0019 => "Cannot inherit from final class",
            ErrorCode::E0020 => "Invalid attribute argument",
            ErrorCode::W0001 => "Unused variable",
            ErrorCode::W0002 => "Shadowed variable",
            ErrorCode::W0003 => "Redundant cast",
            ErrorCode::P0001 => "Internal compiler error",
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            ErrorCode::E0001 | ErrorCode::E0002 | ErrorCode::E0003
            | ErrorCode::E0004 | ErrorCode::E0005 | ErrorCode::E0006
            | ErrorCode::E0007 | ErrorCode::E0008 | ErrorCode::E0009
            | ErrorCode::E0010 | ErrorCode::E0011 | ErrorCode::E0012
            | ErrorCode::E0013 | ErrorCode::E0014 | ErrorCode::E0015
            | ErrorCode::E0016 | ErrorCode::E0017 | ErrorCode::E0018
            | ErrorCode::E0019 | ErrorCode::E0020 => Severity::Error,
            ErrorCode::W0001 | ErrorCode::W0002 | ErrorCode::W0003 => Severity::Warning,
            ErrorCode::P0001 => Severity::Ice,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KL-{}", self.code_str())
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: ErrorCode,
    pub message: String,
    pub span: Option<Span>,
    pub suggestions: Vec<String>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            span: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code, message)
    }

    pub fn warning(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code, message)
    }

    pub fn ice(message: impl Into<String>) -> Self {
        Self::new(Severity::Ice, ErrorCode::P0001, message)
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

#[derive(Default)]
pub struct DiagnosticReporter {
    diagnostics: Vec<Diagnostic>,
    source_map: Option<SourceMap>,
    source_name: String,
}

impl DiagnosticReporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_source(mut self, source_map: SourceMap, source_name: String) -> Self {
        self.source_map = Some(source_map);
        self.source_name = source_name;
        self
    }

    pub fn report(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.severity, Severity::Error | Severity::Ice))
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| matches!(d.severity, Severity::Warning))
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn emit_all(&self) {
        for diag in &self.diagnostics {
            self.emit(diag);
        }
        let counts = self.summary();
        if !counts.is_empty() {
            eprintln!("{}", counts);
        }
    }

    pub fn summary(&self) -> String {
        let errors = self.diagnostics.iter().filter(|d| matches!(d.severity, Severity::Error)).count();
        let warnings = self.diagnostics.iter().filter(|d| matches!(d.severity, Severity::Warning)).count();
        let ices = self.diagnostics.iter().filter(|d| matches!(d.severity, Severity::Ice)).count();
        let mut parts = Vec::new();
        if errors > 0 { parts.push(format!("{} error(s)", errors)); }
        if warnings > 0 { parts.push(format!("{} warning(s)", warnings)); }
        if ices > 0 { parts.push(format!("{} internal error(s)", ices)); }
        parts.join(", ")
    }

    fn emit(&self, diag: &Diagnostic) {
        let _severity_str = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Ice => "internal error",
        };

        eprintln!("{}: {}", diag.code, diag.code.title());
        eprintln!();
        eprintln!("  {}", diag.message);

        if let Some(span) = &diag.span {
            if let Some(sm) = &self.source_map {
                let file = sm.get(span.file_id);
                let file_name = file.map(|f| f.name.as_str()).unwrap_or(&self.source_name);
                eprintln!();
                eprintln!("  --> {}:{}:{}", file_name, span.start.line, span.start.column);

                if let Some(file) = file {
                    if span.start.line > 0 {
                        if let Some(line) = file.line(span.start.line) {
                            eprintln!("   {} | {}", span.start.line, line);
                            let caret_col = span.start.column.saturating_sub(1);
                            let width = if span.end.line == span.start.line {
                                (span.end.column - span.start.column).max(1)
                            } else {
                                1
                            };
                            if caret_col <= line.len() {
                                eprintln!("     | {}{}", " ".repeat(caret_col), "^".repeat(width.min(line.len().saturating_sub(caret_col))));
                            }
                        }
                    }
                }
            } else {
                eprintln!("  --> {}:{}:{}", &self.source_name, span.start.line, span.start.column);
            }
        }

        for note in &diag.notes {
            eprintln!();
            eprintln!("  note: {}", note);
        }

        for suggestion in &diag.suggestions {
            eprintln!();
            eprintln!("  help: {}", suggestion);
        }

        eprintln!();
    }
}
