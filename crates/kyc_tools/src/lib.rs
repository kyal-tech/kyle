// kyc_tools — Developer tooling
//
// Depends on: kyc_core, kyc_frontend
//
// Responsibilities:
//   - LSP (Language Server Protocol) implementation
//   - Code formatter (kl fmt)
//   - Code completion
//   - Diagnostics
//   - Refactoring tools

pub mod lsp;
pub mod formatter;
pub mod completion;
pub mod package;
