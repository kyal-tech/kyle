// kyc_core::source_map — File source storage and lookup
//
// The SourceMap stores all source files loaded by the compiler.
// It maps file IDs to file names and source text, enabling
// the diagnostic system to display source snippets.

use crate::span::Span;

/// A single source file loaded into the compiler.
#[derive(Clone, Debug)]
pub struct SourceFile {
    pub name: String,
    pub source: String,
}

impl SourceFile {
    pub fn new(name: String, source: String) -> Self {
        Self { name, source }
    }

    /// Returns the line at the given 1-indexed line number.
    /// Returns None for line 0 (dummy/unset spans).
    pub fn line(&self, line: usize) -> Option<&str> {
        if line == 0 { return None; }
        self.source.lines().nth(line - 1)
    }

    /// Returns the total number of lines.
    pub fn line_count(&self) -> usize {
        self.source.lines().count()
    }
}

/// Global source file registry.
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Register a new source file and return its file ID.
    pub fn add(&mut self, name: String, source: String) -> usize {
        let id = self.files.len();
        self.files.push(SourceFile::new(name, source));
        id
    }

    /// Look up a source file by its ID.
    pub fn get(&self, id: usize) -> Option<&SourceFile> {
        self.files.get(id)
    }

    /// Resolve a Span to a source snippet for error display.
    pub fn snippet(&self, span: Span) -> Option<String> {
        let file = self.get(span.file_id)?;
        let line = file.line(span.start.line)?;
        Some(line.to_string())
    }
}
