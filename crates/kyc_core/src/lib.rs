// kyc_core — Foundation types used by all compiler crates
//
// This crate contains no dependencies on other KL crates.
// It defines:
//   - AST node definitions
//   - Span (source location)
//   - SourceMap
//   - Symbol IDs
//   - Diagnostic types
//   - Type representations

#![allow(dead_code)]

pub mod ast;
pub mod span;
pub mod source_map;
pub mod symbol;
pub mod diagnostic;
pub mod types;
pub mod semver;
pub mod resolver;
