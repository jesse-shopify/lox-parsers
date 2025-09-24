//! LALRPOP-based Lox parser library
//!
//! This library provides a parser for the Lox programming language using the LALRPOP parser generator.
//! LALRPOP generates LR(1) parsers from grammar specifications.
//!
//! # Example
//!
//! ```
//! use lalrpop_lox::parse_program;
//! use lox_ast::Program;
//!
//! let input = r#"print "Hello, world!";"#;
//! let result = parse_program(input);
//! assert!(result.is_ok());
//! ```

mod parser;

pub use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Parser information for identification
pub const PARSER_NAME: &str = "lalrpop";
pub const PARSER_VERSION: &str = "0.20";
pub const PARSER_DESCRIPTION: &str = "LR(1) parser generator with excellent performance";

/// Parse a Lox program from input string
pub fn parse_program(input: &str) -> Result<Program, String> {
    parser::parse_program(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_ast::{Expr, Stmt, Value, BinaryOp, UnaryOp};

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

    #[test]
    fn test_variable_declaration_without_initializer() {
        let input = "var x;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::VarDeclaration { name, initializer } => {
                assert_eq!(name, "x");
                assert!(initializer.is_none());
            }
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_unary_expressions() {
        let test_cases = vec![
            ("-42;", UnaryOp::Minus, Value::Number(42.0)),
            ("!true;", UnaryOp::Not, Value::Bool(true)),
        ];

        for (input, expected_op, expected_operand) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result.statements.len(), 1);

            match &result.statements[0] {
                Stmt::Expression(Expr::Unary { operator, operand }) => {
                    assert_eq!(*operator, expected_op);
                    match &**operand {
                        Expr::Literal(value) => {
                            assert_eq!(*value, expected_operand);
                        }
                        _ => panic!("Expected literal operand"),
                    }
                }
                _ => panic!("Expected unary expression"),
            }
        }
    }

    #[test]
    fn test_operator_precedence() {
        // Test that 1 + 2 * 3 is parsed as 1 + (2 * 3)
        let input = "1 + 2 * 3;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Expression(Expr::Binary { left, operator, right }) => {
                assert!(matches!(**left, Expr::Literal(Value::Number(1.0))));
                assert_eq!(*operator, BinaryOp::Add);

                // Right side should be 2 * 3
                match &**right {
                    Expr::Binary { left: inner_left, operator: inner_op, right: inner_right } => {
                        assert!(matches!(**inner_left, Expr::Literal(Value::Number(2.0))));
                        assert_eq!(*inner_op, BinaryOp::Multiply);
                        assert!(matches!(**inner_right, Expr::Literal(Value::Number(3.0))));
                    }
                    _ => panic!("Expected multiplication on right side"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_grouping() {
        let input = "(1 + 2) * 3;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Expression(Expr::Binary { left, operator, right }) => {
                // Left side should be (1 + 2)
                match &**left {
                    Expr::Grouping(inner) => {
                        match &**inner {
                            Expr::Binary { left: inner_left, operator: inner_op, right: inner_right } => {
                                assert!(matches!(**inner_left, Expr::Literal(Value::Number(1.0))));
                                assert_eq!(*inner_op, BinaryOp::Add);
                                assert!(matches!(**inner_right, Expr::Literal(Value::Number(2.0))));
                            }
                            _ => panic!("Expected addition inside grouping"),
                        }
                    }
                    _ => panic!("Expected grouping on left side"),
                }

                assert_eq!(*operator, BinaryOp::Multiply);
                assert!(matches!(**right, Expr::Literal(Value::Number(3.0))));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_logical_operators() {
        let test_cases = vec![
            ("true and false;", BinaryOp::And),
            ("true or false;", BinaryOp::Or),
        ];

        for (input, expected_op) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result.statements.len(), 1);

            match &result.statements[0] {
                Stmt::Expression(Expr::Binary { left, operator, right }) => {
                    assert!(matches!(**left, Expr::Literal(Value::Bool(true))));
                    assert_eq!(*operator, expected_op);
                    assert!(matches!(**right, Expr::Literal(Value::Bool(false))));
                }
                _ => panic!("Expected binary expression"),
            }
        }
    }

    #[test]
    fn test_comparison_operators() {
        let test_cases = vec![
            ("5 > 3;", BinaryOp::Greater),
            ("5 >= 3;", BinaryOp::GreaterEqual),
            ("3 < 5;", BinaryOp::Less),
            ("3 <= 5;", BinaryOp::LessEqual),
            ("5 == 5;", BinaryOp::Equal),
            ("5 != 3;", BinaryOp::NotEqual),
        ];

        for (input, expected_op) in test_cases {
            let result = parse_program(input).unwrap();
            assert_eq!(result.statements.len(), 1);

            match &result.statements[0] {
                Stmt::Expression(Expr::Binary { operator, .. }) => {
                    assert_eq!(*operator, expected_op);
                }
                _ => panic!("Expected binary expression"),
            }
        }
    }

    #[test]
    fn test_assignment() {
        let input = "x = 42;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Expression(Expr::Assignment { name, value }) => {
                assert_eq!(name, "x");
                assert!(matches!(**value, Expr::Literal(Value::Number(42.0))));
            }
            _ => panic!("Expected assignment expression"),
        }
    }

    #[test]
    fn test_complex_program() {
        let input = r#"
            var a = 10;
            var b = 20;
            var sum = a + b;
            print sum;
        "#;

        let result = parse_program(input).unwrap();
        assert_eq!(result.statements.len(), 4);

        // Check first statement: var a = 10;
        match &result.statements[0] {
            Stmt::VarDeclaration { name, initializer } => {
                assert_eq!(name, "a");
                assert!(matches!(initializer, Some(Expr::Literal(Value::Number(10.0)))));
            }
            _ => panic!("Expected variable declaration"),
        }

        // Check last statement: print sum;
        match &result.statements[3] {
            Stmt::Print(Expr::Variable(var_name)) => {
                assert_eq!(var_name, "sum");
            }
            _ => panic!("Expected print statement"),
        }
    }
}
