// kyc_driver — Pipeline orchestration
//
// Orchestrates the full compilation pipeline:
//   Source → Lexer → Parser → AST → Semantic Analysis
//   → MIR → Optimization → LLVM Codegen → Linker
//
// Depends on: all other compiler crates

pub mod pipeline;
pub mod build;
pub mod config;
