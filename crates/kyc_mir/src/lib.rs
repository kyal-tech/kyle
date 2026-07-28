// kyc_mir — Mid-level IR and optimization
//
// Depends on: kyc_core
//
// Responsibilities:
//   - MIR definition
//   - AST to MIR lowering
//   - Optimization passes
//   - Control flow analysis

pub mod mir;
pub mod lower;
pub mod optimize;
pub mod borrow_analysis;
pub mod ssa;
