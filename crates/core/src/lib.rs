//! # SIGMOS Core
//!
//! Core parsing, grammar, and AST definitions for the SIGMOS DSL.
//!
//! This crate provides the foundational components for parsing SIGMOS specifications:
//! - PEG grammar definition using pest
//! - Abstract Syntax Tree (AST) types
//! - Parser implementation with error handling
//! - Type system definitions
//!
//! # Examples
//!
//! ```rust
//! use sigmos_core::{Parser, SigmosRule};
//! use pest::Parser as PestParser;
//!
//! let input = r#"
//! spec "Agent" v1.0 {
//!   inputs:
//!     name: string
//! }
//! "#;
//!
//! let parsed = Parser::parse(SigmosRule::sigmos_file, input).unwrap();
//! ```

use pest_derive::Parser;
use thiserror::Error;

pub mod ast;
pub mod parser;
pub mod types;

/// SIGMOS parser using pest grammar
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SigmosParser;

// Re-export the Rule enum from pest_derive
pub use SigmosParser as Parser;

/// Parse errors for SIGMOS specifications
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Grammar parsing failed: {0}")]
    Grammar(String),
    #[error("Semantic validation failed: {0}")]
    Semantic(String),
    #[error("Type error: {0}")]
    Type(String),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;
