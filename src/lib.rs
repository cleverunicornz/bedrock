//! yeetz-bedrock — the situation/ graph compiler (SPINE).
//!
//! Library surface so integration tests can drive the pipeline in-process;
//! the `bedrock` binary is a thin shell over [`cli`].

pub mod check;
pub mod cli;
pub mod compile;
pub mod contextreg;
pub mod embedded;
pub mod errors;
pub mod generate;
pub mod install;
pub mod schema;
pub mod yamlsyntax;
