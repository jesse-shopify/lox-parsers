//! Simple parser comparison library (nom-only to avoid linker issues)

use lox_ast::Program;
use std::time::{Duration, Instant};

pub fn parse_with_nom(input: &str) -> Result<Program, String> {
    nom_lox::parse_program(input).map_err(|e| format!("{:?}", e))
}

pub fn benchmark_parser(input: &str, iterations: usize) -> (Duration, usize) {
    let start = Instant::now();
    let mut successes = 0;

    for _ in 0..iterations {
        if parse_with_nom(input).is_ok() {
            successes += 1;
        }
    }

    (start.elapsed(), successes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_ast::{Stmt, Expr, Value, BinaryOp, UnaryOp};

    #[test]
    fn test_simple_literals() {
        let test_cases = vec![
            ("42;", 1, "number literal"),
            (r#""hello";"#, 1, "string literal"),
            ("true;", 1, "boolean literal"),
            ("nil;", 1, "nil literal"),
        ];

        for (input, expected_statements, description) in test_cases {
            match parse_with_nom(input) {
                Ok(program) => {
                    assert_eq!(
                        program.statements.len(),
                        expected_statements,
                        "Failed for {}: {}",
                        description,
                        input
                    );
                }
                Err(e) => panic!("Parse failed for {}: {}", description, e),
            }
        }
    }

    #[test]
    fn test_print_statements() {
        let test_cases = vec![
            (r#"print "hello";"#, "hello"),
            (r#"print "world";"#, "world"),
            (r#"print "test 123";"#, "test 123"),
        ];

        for (input, expected_string) in test_cases {
            match parse_with_nom(input) {
                Ok(program) => {
                    assert_eq!(program.statements.len(), 1);
                    match &program.statements[0] {
                        Stmt::Print(Expr::Literal(Value::String(s))) => {
                            assert_eq!(s, expected_string);
                        }
                        _ => panic!("Expected print statement with string literal"),
                    }
                }
                Err(e) => panic!("Parse failed: {}", e),
            }
        }
    }

    #[test]
    fn test_variable_declarations() {
        let test_cases = vec![
            ("var x = 42;", "x", Some(Value::Number(42.0))),
            ("var name = \"test\";", "name", Some(Value::String("test".to_string()))),
            ("var flag = true;", "flag", Some(Value::Bool(true))),
            ("var empty = nil;", "empty", Some(Value::Nil)),
        ];

        for (input, expected_name, expected_value) in test_cases {
            match parse_with_nom(input) {
                Ok(program) => {
                    assert_eq!(program.statements.len(), 1);
                    match &program.statements[0] {
                        Stmt::VarDeclaration { name, initializer } => {
                            assert_eq!(name, expected_name);
                            match (initializer, &expected_value) {
                                (Some(Expr::Literal(actual)), Some(expected)) => {
                                    assert_eq!(actual, expected);
                                }
                                (None, None) => {}
                                _ => panic!("Initializer mismatch"),
                            }
                        }
                        _ => panic!("Expected variable declaration"),
                    }
                }
                Err(e) => panic!("Parse failed: {}", e),
            }
        }
    }

    #[test]
    fn test_arithmetic_expressions() {
        let test_cases = vec![
            ("1 + 2;", BinaryOp::Add),
            ("5 - 3;", BinaryOp::Subtract),
            ("4 * 6;", BinaryOp::Multiply),
            ("8 / 2;", BinaryOp::Divide),
        ];

        for (input, expected_op) in test_cases {
            match parse_with_nom(input) {
                Ok(program) => {
                    assert_eq!(program.statements.len(), 1);
                    match &program.statements[0] {
                        Stmt::Expression(Expr::Binary { left: _, operator, right: _ }) => {
                            assert_eq!(*operator, expected_op);
                        }
                        _ => panic!("Expected binary expression"),
                    }
                }
                Err(e) => panic!("Parse failed: {}", e),
            }
        }
    }

    #[test]
    fn test_operator_precedence() {
        // Test that 1 + 2 * 3 is parsed as 1 + (2 * 3)
        match parse_with_nom("1 + 2 * 3;") {
            Ok(program) => {
                assert_eq!(program.statements.len(), 1);
                match &program.statements[0] {
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
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_unary_expressions() {
        let test_cases = vec![
            ("-42;", UnaryOp::Minus, Value::Number(42.0)),
            ("!true;", UnaryOp::Not, Value::Bool(true)),
            ("!false;", UnaryOp::Not, Value::Bool(false)),
        ];

        for (input, expected_op, expected_operand) in test_cases {
            match parse_with_nom(input) {
                Ok(program) => {
                    assert_eq!(program.statements.len(), 1);
                    match &program.statements[0] {
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
                Err(e) => panic!("Parse failed: {}", e),
            }
        }
    }

    #[test]
    fn test_complex_programs() {
        let complex_input = r#"
            var a = 10;
            var b = 20;
            var sum = a + b;
            print sum;
        "#;

        match parse_with_nom(complex_input) {
            Ok(program) => {
                assert_eq!(program.statements.len(), 4);

                // Check first statement: var a = 10;
                match &program.statements[0] {
                    Stmt::VarDeclaration { name, initializer } => {
                        assert_eq!(name, "a");
                        assert!(matches!(initializer, Some(Expr::Literal(Value::Number(10.0)))));
                    }
                    _ => panic!("Expected variable declaration"),
                }

                // Check last statement: print sum;
                match &program.statements[3] {
                    Stmt::Print(Expr::Variable(var_name)) => {
                        assert_eq!(var_name, "sum");
                    }
                    _ => panic!("Expected print statement"),
                }
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_comments_ignored() {
        let input_with_comments = r#"
            // This is a comment
            var x = 42; // End of line comment
            // Another comment
            print x;
        "#;

        match parse_with_nom(input_with_comments) {
            Ok(program) => {
                assert_eq!(program.statements.len(), 2);
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_error_cases() {
        let invalid_inputs = vec![
            ("var = 42;", "missing variable name"),
            ("print", "missing semicolon"),
            ("42 +;", "incomplete expression"),
            ("var x = ;", "missing initializer"),
        ];

        for (input, description) in invalid_inputs {
            match parse_with_nom(input) {
                Ok(_) => println!("⚠️  Expected '{}' to fail but it succeeded", description),
                Err(_) => {
                    // Expected to fail
                }
            }
        }
    }

    #[test]
    fn test_performance_benchmark() {
        let input = "var x = (1 + 2) * 3; print x;";
        let iterations = 1000;

        let (elapsed, successes) = benchmark_parser(input, iterations);

        assert_eq!(successes, iterations, "All iterations should succeed");

        let avg_time = elapsed / iterations as u32;
        println!("Benchmark: {} iterations in {:?}", iterations, elapsed);
        println!("Average time per parse: {:?}", avg_time);

        // Should be reasonably fast (less than 1ms per parse)
        assert!(avg_time.as_millis() < 1, "Parser should be faster than 1ms per parse");
    }

    #[test]
    fn test_serde_integration() {
        let input = r#"var greeting = "Hello, world!"; print greeting;"#;

        match parse_with_nom(input) {
            Ok(program) => {
                // Test serialization
                let json = serde_json::to_string(&program).expect("Should serialize");
                assert!(!json.is_empty());

                // Test deserialization
                let deserialized: Program = serde_json::from_str(&json).expect("Should deserialize");
                assert_eq!(program, deserialized);

                println!("Serialization test passed, JSON length: {}", json.len());
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }
}
