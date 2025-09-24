//! Nom-based Lox parser library
//!
//! This library provides a parser for the Lox programming language using the nom parser combinator library.
//!
//! # Example
//!
//! ```
//! use nom_lox::parse_program;
//! use lox_ast::Program;
//!
//! let input = r#"print "Hello, world!";"#;
//! let result = parse_program(input);
//! assert!(result.is_ok());
//! ```

mod parser;

pub use parser::parse_program;
pub use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Parser information for identification
pub const PARSER_NAME: &str = "nom";
pub const PARSER_VERSION: &str = "7.1";
pub const PARSER_DESCRIPTION: &str = "Zero-copy parser combinator library with excellent performance";
