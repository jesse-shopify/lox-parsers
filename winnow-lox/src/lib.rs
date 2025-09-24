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

#[cfg(test)]
mod tests {
    use super::*;
    use lox_ast::{Expr, Stmt, Value, BinaryOp};

    #[test]
    fn test_simple_expression() {
        let input = "1 + 2;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Expression(Expr::Binary { left, operator, right }) => {
                assert!(matches!(**left, Expr::Literal(Value::Number(1.0))));
                assert_eq!(*operator, BinaryOp::Add);
                assert!(matches!(**right, Expr::Literal(Value::Number(2.0))));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_print_statement() {
        let input = r#"print "Hello, world!";"#;
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Print(Expr::Literal(Value::String(s))) => {
                assert_eq!(s, "Hello, world!");
            }
            _ => panic!("Expected print statement with string literal"),
        }
    }

    #[test]
    fn test_variable_declaration() {
        let input = "var x = 42;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::VarDeclaration { name, initializer } => {
                assert_eq!(name, "x");
                assert!(matches!(
                    initializer,
                    Some(Expr::Literal(Value::Number(42.0)))
                ));
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}
