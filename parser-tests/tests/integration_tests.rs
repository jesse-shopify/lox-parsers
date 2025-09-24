//! Comprehensive integration tests for all Lox parsers
//!
//! This combines test cases from:
//! - parser-comparison/tests/integration_tests.rs
//! - parser-comparison-simple/tests/comprehensive_tests.rs
//! - simple-test functionality

use parser_tests::{
    get_working_parsers, get_all_parsers, run_parser_tests, compare_all_parsers,
    TestCase, TEST_CASES, LoxParser, NomParser, LalrpopParser, PomParser
};
use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

/// Test that all working parsers can handle basic cases
#[test]
fn test_working_parsers_basic_functionality() {
    let parsers = get_working_parsers();

    // Basic test cases that should work on all functional parsers
    let basic_cases = vec![
        ("Simple number", "42;", 1),
        ("Simple string", r#""hello";"#, 1),
        ("Print statement", r#"print "Hello, world!";"#, 1),
        ("Variable declaration", "var x = 42;", 1),
        ("Simple arithmetic", "1 + 2;", 1),
    ];

    for parser in parsers {
        println!("Testing {} parser...", parser.name());

        for (name, input, expected_statements) in &basic_cases {
            let result = parser.parse(input);
            assert!(result.success,
                "Parser {} failed on test '{}': {:?}",
                parser.name(), name, result.error
            );
            assert_eq!(result.statement_count, *expected_statements,
                "Parser {} returned wrong statement count for '{}': expected {}, got {}",
                parser.name(), name, expected_statements, result.statement_count
            );
        }
    }
}

/// Test operator precedence across working parsers
#[test]
fn test_operator_precedence() {
    let parsers = get_working_parsers();

    for parser in parsers {
        let result = parser.parse("1 + 2 * 3;");
        if result.success {
            if let Some(program) = result.program {
                if let Some(Stmt::Expression(Expr::Binary { left, operator, right })) = program.statements.first() {
                    // Should parse as 1 + (2 * 3), not (1 + 2) * 3
                    assert_eq!(*operator, BinaryOp::Add);
                    assert!(matches!(**left, Expr::Literal(Value::Number(1.0))));
                    // Right side should be a multiplication
                    if let Expr::Binary { operator: right_op, .. } = right.as_ref() {
                        assert_eq!(*right_op, BinaryOp::Multiply);
                    } else {
                        panic!("Expected multiplication on right side for parser {}", parser.name());
                    }
                }
            }
        }
    }
}

/// Test variable operations across working parsers
#[test]
fn test_variable_operations() {
    let parsers = get_working_parsers();

    let test_input = r#"
        var x = 10;
        var y = 20;
        var sum = x + y;
        print sum;
    "#;

    for parser in parsers {
        let result = parser.parse(test_input);
        if result.success {
            assert_eq!(result.statement_count, 4,
                "Parser {} should parse 4 statements, got {}",
                parser.name(), result.statement_count
            );
        }
    }
}

/// Test all standard test cases against working parsers
#[test]
fn test_standard_cases_working_parsers() {
    let parsers = get_working_parsers();

    for parser in parsers {
        let summary = run_parser_tests(parser.as_ref());

        // Working parsers should pass most or all tests
        assert!(summary.passed > 0,
            "Working parser {} should pass at least some tests",
            summary.parser_name
        );

        // Print results for visibility
        println!("Parser {}: {}/{} tests passed",
            summary.parser_name, summary.passed, summary.total
        );
    }
}

/// Test individual parser implementations
mod parser_specific_tests {
    use super::*;

    #[test]
    fn test_nom_parser() {
        let parser = NomParser;
        let summary = run_parser_tests(&parser);

        // nom-lox should be fully functional
        assert!(summary.passed > 0, "nom parser should pass some tests");
        println!("nom-lox: {}/{} tests passed", summary.passed, summary.total);
    }

    #[test]
    fn test_lalrpop_parser() {
        let parser = LalrpopParser;
        let summary = run_parser_tests(&parser);

        // lalrpop-lox should be fully functional
        assert!(summary.passed > 0, "lalrpop parser should pass some tests");
        println!("lalrpop-lox: {}/{} tests passed", summary.passed, summary.total);
    }

    #[test]
    fn test_pom_parser() {
        let parser = PomParser;
        let summary = run_parser_tests(&parser);

        // pom-lox should be fully functional
        assert!(summary.passed > 0, "pom parser should pass some tests");
        println!("pom-lox: {}/{} tests passed", summary.passed, summary.total);
    }
}

/// Test error handling and edge cases
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let parsers = get_working_parsers();

        for parser in parsers {
            let result = parser.parse("");
            // Empty input should either succeed with 0 statements or fail gracefully
            if result.success {
                assert_eq!(result.statement_count, 0);
            }
        }
    }

    #[test]
    fn test_invalid_syntax() {
        let parsers = get_working_parsers();
        let invalid_inputs = vec![
            "1 +", // Incomplete expression
            "var", // Incomplete variable declaration
            "print", // Incomplete print statement
            "1 2 3", // Invalid sequence
        ];

        for parser in parsers {
            for invalid_input in &invalid_inputs {
                let result = parser.parse(invalid_input);
                // Should fail gracefully with error message
                if !result.success {
                    assert!(result.error.is_some(),
                        "Parser {} should provide error message for invalid input: {}",
                        parser.name(), invalid_input
                    );
                }
            }
        }
    }
}

/// Test comprehensive language features
mod language_feature_tests {
    use super::*;

    #[test]
    fn test_all_literal_types() {
        let parsers = get_working_parsers();

        let literals = vec![
            ("Number", "42;"),
            ("String", r#""hello";"#),
            ("Boolean true", "true;"),
            ("Boolean false", "false;"),
            ("Nil", "nil;"),
        ];

        for parser in parsers {
            for (name, input) in &literals {
                let result = parser.parse(input);
                if result.success {
                    assert_eq!(result.statement_count, 1,
                        "Parser {} failed on literal test '{}': expected 1 statement, got {}",
                        parser.name(), name, result.statement_count
                    );
                }
            }
        }
    }

    #[test]
    fn test_all_binary_operators() {
        let parsers = get_working_parsers();

        let operators = vec![
            ("Addition", "1 + 2;"),
            ("Subtraction", "5 - 3;"),
            ("Multiplication", "2 * 4;"),
            ("Division", "8 / 2;"),
            ("Greater", "5 > 3;"),
            ("Less", "3 < 5;"),
            ("Greater equal", "5 >= 5;"),
            ("Less equal", "3 <= 5;"),
            ("Equal", "5 == 5;"),
            ("Not equal", "5 != 3;"),
        ];

        for parser in parsers {
            for (name, input) in &operators {
                let result = parser.parse(input);
                if result.success {
                    assert_eq!(result.statement_count, 1,
                        "Parser {} failed on operator test '{}': expected 1 statement, got {}",
                        parser.name(), name, result.statement_count
                    );
                }
            }
        }
    }

    #[test]
    fn test_unary_operators() {
        let parsers = get_working_parsers();

        let unary_ops = vec![
            ("Unary minus", "-42;"),
            ("Logical not", "!true;"),
        ];

        for parser in parsers {
            for (name, input) in &unary_ops {
                let result = parser.parse(input);
                if result.success {
                    assert_eq!(result.statement_count, 1,
                        "Parser {} failed on unary test '{}': expected 1 statement, got {}",
                        parser.name(), name, result.statement_count
                    );
                }
            }
        }
    }
}

/// Integration test that runs the full comparison suite
#[test]
fn test_full_parser_comparison() {
    println!("\n=== Running Full Parser Comparison ===");
    let summaries = compare_all_parsers();

    // Should test all parsers
    assert_eq!(summaries.len(), 3, "Should test all 3 working parsers");

    // At least some parsers should work
    let working_count = summaries.iter().filter(|s| s.passed > 0).count();
    assert!(working_count >= 3, "At least 3 parsers should pass some tests");

    // Print summary
    println!("\n=== Test Summary ===");
    for summary in &summaries {
        println!("{}: {}/{} tests passed",
            summary.parser_name, summary.passed, summary.total
        );
    }
}
