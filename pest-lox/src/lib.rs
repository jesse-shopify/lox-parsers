//! Pest-based Lox parser library
//!
//! This library provides a parser for the Lox programming language using the pest PEG parser generator.
//!
//! # Example
//!
//! ```
//! use pest_lox::parse_program;
//! use lox_ast::Program;
//!
//! let input = r#"print "Hello, world!";"#;
//! let result = parse_program(input);
//! // Note: currently has runtime issues
//! ```

mod parser;

pub use parser::parse_program;
pub use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Parser information for identification
pub const PARSER_NAME: &str = "pest";
pub const PARSER_VERSION: &str = "2.7";
pub const PARSER_DESCRIPTION: &str = "PEG parser generator with grammar-based approach";
