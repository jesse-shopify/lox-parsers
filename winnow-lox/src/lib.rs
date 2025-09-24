//! Winnow-based Lox parser library
//!
//! This library provides a parser for the Lox programming language using the winnow parser combinator library.
//!
//! # Example
//!
//! ```
//! use winnow_lox::parse_program;
//! use lox_ast::Program;
//!
//! let input = r#"print "Hello, world!";"#;
//! let result = parse_program(input);
//! // Note: currently has runtime issues with repeat assertions
//! ```

mod parser;

pub use parser::parse_program;
pub use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Parser information for identification
pub const PARSER_NAME: &str = "winnow";
pub const PARSER_VERSION: &str = "0.6";
pub const PARSER_DESCRIPTION: &str = "Modern successor to nom with better error handling";
