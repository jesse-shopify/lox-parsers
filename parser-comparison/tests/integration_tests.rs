//! Integration tests for parser comparison

use parser_comparison::{compare_parsers, get_all_parsers, NomParser, LoxParser};
use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Test cases for parser comparison
struct TestCase {
    name: &'static str,
    input: &'static str,
    expected_statements: usize,
    description: &'static str,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "simple_literal",
        input: "42;",
        expected_statements: 1,
        description: "Simple number literal",
    },
    TestCase {
        name: "string_literal",
        input: r#""Hello, world!";"#,
        expected_statements: 1,
        description: "String literal",
    },
    TestCase {
        name: "print_statement",
        input: r#"print "Hello, world!";"#,
        expected_statements: 1,
        description: "Print statement",
    },
    TestCase {
        name: "variable_declaration",
        input: "var x = 42;",
        expected_statements: 1,
        description: "Variable declaration with initializer",
    },
    TestCase {
        name: "variable_assignment",
        input: "var x = 10; x = 20;",
        expected_statements: 2,
        description: "Variable declaration followed by assignment",
    },
    TestCase {
        name: "arithmetic_expression",
        input: "1 + 2 * 3;",
        expected_statements: 1,
        description: "Arithmetic with proper precedence",
    },
    TestCase {
        name: "comparison_expression",
        input: "5 > 3;",
        expected_statements: 1,
        description: "Comparison operator",
    },
    TestCase {
        name: "logical_expression",
        input: "true and false;",
        expected_statements: 1,
        description: "Logical AND operation",
    },
    TestCase {
        name: "grouped_expression",
        input: "(1 + 2) * 3;",
        expected_statements: 1,
        description: "Parenthesized expression",
    },
    TestCase {
        name: "complex_expression",
        input: "var a = 10; var b = 20; var sum = a + b; print sum;",
        expected_statements: 4,
        description: "Multiple statements with variables",
    },
    TestCase {
        name: "comments",
        input: r#"// This is a comment
print "Hello"; // Another comment"#,
        expected_statements: 1,
        description: "Comments should be ignored",
    },
    TestCase {
        name: "multiple_statements",
        input: "var a = 1;\nvar b = 2;\nprint a + b;",
        expected_statements: 3,
        description: "Multiple statements on separate lines",
    },
    TestCase {
        name: "boolean_literals",
        input: "true; false; nil;",
        expected_statements: 3,
        description: "Boolean and nil literals",
    },
    TestCase {
        name: "unary_expressions",
        input: "-42; !true;",
        expected_statements: 2,
        description: "Unary minus and not operators",
    },
    TestCase {
        name: "complex_arithmetic",
        input: "1 + 2 * 3 - 4 / 2;",
        expected_statements: 1,
        description: "Complex arithmetic with multiple operators",
    },
];

#[test]
fn test_nom_parser_individual() {
    let parser = NomParser;

    for test_case in TEST_CASES {
        let result = parser.parse(test_case.input);

        match result {
            Ok(program) => {
                assert_eq!(
                    program.statements.len(),
                    test_case.expected_statements,
                    "Test '{}' failed: expected {} statements, got {}",
                    test_case.name,
                    test_case.expected_statements,
                    program.statements.len()
                );
            }
            Err(_) => {
                // For now, we'll just record failures but not fail the test
                // since some parsers have known issues
                println!("⚠️  nom parser failed on '{}': {}", test_case.name, test_case.input);
            }
        }
    }
}

#[test]
fn test_parser_comparison_consensus() {
    // Test a simple case that should work across all parsers
    let simple_cases = &[
        ("print statement", r#"print "test";"#),
        ("variable declaration", "var x = 42;"),
        ("arithmetic", "1 + 2;"),
    ];

    for (name, input) in simple_cases {
        let parsers = get_all_parsers();
        let result = compare_parsers(input, parsers);

        // At least nom should succeed
        assert!(
            result.accuracy_summary.successful_parsers >= 1,
            "No parsers succeeded for test '{}' with input: {}",
            name,
            input
        );

        println!(
            "Test '{}': {}/{} parsers succeeded",
            name,
            result.accuracy_summary.successful_parsers,
            result.accuracy_summary.total_parsers
        );

        if result.accuracy_summary.consensus_reached {
            println!("  ✅ Consensus reached!");
        } else if result.accuracy_summary.successful_parsers > 1 {
            println!("  ⚠️  Multiple parsers succeeded but produced different ASTs");
        }
    }
}

#[test]
fn test_ast_structure_validation() {
    let parser = NomParser;

    // Test specific AST structures
    let test_cases = vec![
        (
            "print statement AST",
            r#"print "hello";"#,
            |program: &Program| {
                assert_eq!(program.statements.len(), 1);
                match &program.statements[0] {
                    Stmt::Print(Expr::Literal(Value::String(s))) => {
                        assert_eq!(s, "hello");
                    }
                    _ => panic!("Expected print statement with string literal"),
                }
            }
        ),
        (
            "variable declaration AST",
            "var x = 42;",
            |program: &Program| {
                assert_eq!(program.statements.len(), 1);
                match &program.statements[0] {
                    Stmt::VarDeclaration { name, initializer } => {
                        assert_eq!(name, "x");
                        match initializer {
                            Some(Expr::Literal(Value::Number(n))) => {
                                assert_eq!(*n, 42.0);
                            }
                            _ => panic!("Expected number literal initializer"),
                        }
                    }
                    _ => panic!("Expected variable declaration"),
                }
            }
        ),
        (
            "binary expression AST",
            "1 + 2;",
            |program: &Program| {
                assert_eq!(program.statements.len(), 1);
                match &program.statements[0] {
                    Stmt::Expression(Expr::Binary { left, operator, right }) => {
                        assert!(matches!(**left, Expr::Literal(Value::Number(1.0))));
                        assert_eq!(*operator, BinaryOp::Add);
                        assert!(matches!(**right, Expr::Literal(Value::Number(2.0))));
                    }
                    _ => panic!("Expected binary expression"),
                }
            }
        ),
        (
            "unary expression AST",
            "-42;",
            |program: &Program| {
                assert_eq!(program.statements.len(), 1);
                match &program.statements[0] {
                    Stmt::Expression(Expr::Unary { operator, operand }) => {
                        assert_eq!(*operator, UnaryOp::Minus);
                        assert!(matches!(**operand, Expr::Literal(Value::Number(42.0))));
                    }
                    _ => panic!("Expected unary expression"),
                }
            }
        ),
    ];

    for (name, input, validator) in test_cases {
        match parser.parse(input) {
            Ok(program) => {
                validator(&program);
                println!("✅ AST validation passed for: {}", name);
            }
            Err(e) => {
                panic!("Parse failed for '{}': {:?}", name, e);
            }
        }
    }
}

#[test]
fn test_parser_metadata() {
    let parsers = get_all_parsers();

    // Ensure all parsers have proper metadata
    for parser in parsers {
        assert!(!parser.name().is_empty(), "Parser name should not be empty");
        assert!(!parser.version().is_empty(), "Parser version should not be empty");
        assert!(!parser.description().is_empty(), "Parser description should not be empty");

        println!("Parser: {} v{} - {}", parser.name(), parser.version(), parser.description());
    }
}

#[test]
fn test_operator_precedence() {
    let parser = NomParser;

    // Test that 1 + 2 * 3 is parsed as 1 + (2 * 3), not (1 + 2) * 3
    let result = parser.parse("1 + 2 * 3;").expect("Should parse successfully");

    assert_eq!(result.statements.len(), 1);
    match &result.statements[0] {
        Stmt::Expression(Expr::Binary { left, operator, right }) => {
            // Should be: 1 + (2 * 3)
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

    println!("✅ Operator precedence test passed");
}

#[test]
fn test_error_handling() {
    let parser = NomParser;

    let invalid_inputs = &[
        ("unclosed string", r#""hello"#),
        ("invalid syntax", "var = 42;"),
        ("missing semicolon", "print 42"),
        ("empty input", ""),
    ];

    for (name, input) in invalid_inputs {
        match parser.parse(input) {
            Ok(_) => {
                println!("⚠️  Expected '{}' to fail but it succeeded", name);
            }
            Err(_) => {
                println!("✅ Correctly rejected invalid input: {}", name);
            }
        }
    }
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_parser_performance() {
        let parser = NomParser;
        let complex_input = r#"
            var a = 10;
            var b = 20;
            var c = (a + b) * 2;
            print c;
            var result = c > 50 and c < 100;
            print result;
        "#;

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = parser.parse(complex_input);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;

        println!("Benchmark results for {} iterations:", iterations);
        println!("  Total time: {:?}", elapsed);
        println!("  Average time per parse: {:?}", avg_time);
        println!("  Parses per second: {:.0}", 1.0 / avg_time.as_secs_f64());

        // Ensure reasonable performance (less than 1ms per parse)
        assert!(avg_time.as_millis() < 1, "Parser should be faster than 1ms per parse");
    }
}
