//! Chumsky-based Lox parser library
//!
//! This library provides a parser for the Lox programming language using the chumsky parser combinator library.
//!
//! # Example
//!
//! ```
//! use chumsky_lox::parse_program;
//! use lox_ast::Program;
//!
//! let input = r#"print "Hello, world!";"#;
//! let result = parse_program(input);
//! // Note: currently has linker issues on macOS
//! ```

mod parser;

pub use parser::parse_program;
pub use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Parser information for identification
pub const PARSER_NAME: &str = "chumsky";
pub const PARSER_VERSION: &str = "0.9";
pub const PARSER_DESCRIPTION: &str = "Parser combinator focused on excellent error messages";
