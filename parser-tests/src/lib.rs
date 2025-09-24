//! Comprehensive test suite for all Lox parser implementations
//!
//! This crate combines functionality from parser-comparison, parser-comparison-simple,
//! and simple-test into a unified testing framework for all Lox parsers.

use lox_ast::Program;
use colored::*;

/// Trait for unified parser testing
pub trait LoxParser: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parse(&self, input: &str) -> ParseResult;
}

/// Result of parsing with success/failure information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParseResult {
    pub success: bool,
    pub program: Option<Program>,
    pub error: Option<String>,
    pub statement_count: usize,
}

/// Parser implementation for nom-lox
pub struct NomParser;

impl LoxParser for NomParser {
    fn name(&self) -> &'static str { nom_lox::PARSER_NAME }
    fn version(&self) -> &'static str { nom_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { nom_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        match nom_lox::parse_program(input) {
            Ok(program) => ParseResult {
                success: true,
                statement_count: program.statements.len(),
                program: Some(program),
                error: None,
            },
            Err(e) => ParseResult {
                success: false,
                program: None,
                error: Some(format!("{:?}", e)),
                statement_count: 0,
            },
        }
    }
}

// ChumskyParser implementation commented out due to macOS linker issues
// The chumsky-lox parser works but causes linker assertion failures on macOS
// when included in the test framework

/// Parser implementation for pest-lox
pub struct PestParser;

impl LoxParser for PestParser {
    fn name(&self) -> &'static str { pest_lox::PARSER_NAME }
    fn version(&self) -> &'static str { pest_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { pest_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        match pest_lox::parse_program(input) {
            Ok(program) => ParseResult {
                success: true,
                statement_count: program.statements.len(),
                program: Some(program),
                error: None,
            },
            Err(e) => ParseResult {
                success: false,
                program: None,
                error: Some(format!("{:?}", e)),
                statement_count: 0,
            },
        }
    }
}

/// Parser implementation for lalrpop-lox
pub struct LalrpopParser;

impl LoxParser for LalrpopParser {
    fn name(&self) -> &'static str { lalrpop_lox::PARSER_NAME }
    fn version(&self) -> &'static str { lalrpop_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { lalrpop_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        match lalrpop_lox::parse_program(input) {
            Ok(program) => ParseResult {
                success: true,
                statement_count: program.statements.len(),
                program: Some(program),
                error: None,
            },
            Err(e) => ParseResult {
                success: false,
                program: None,
                error: Some(format!("{:?}", e)),
                statement_count: 0,
            },
        }
    }
}

/// Parser implementation for pom-lox
pub struct PomParser;

impl LoxParser for PomParser {
    fn name(&self) -> &'static str { pom_lox::PARSER_NAME }
    fn version(&self) -> &'static str { pom_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { pom_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        match pom_lox::parse_program(input) {
            Ok(program) => ParseResult {
                success: true,
                statement_count: program.statements.len(),
                program: Some(program),
                error: None,
            },
            Err(e) => ParseResult {
                success: false,
                program: None,
                error: Some(format!("{:?}", e)),
                statement_count: 0,
            },
        }
    }
}

/// Parser implementation for lelwel-lox
pub struct LelwelParser;

impl LoxParser for LelwelParser {
    fn name(&self) -> &'static str { lelwel_lox::PARSER_NAME }
    fn version(&self) -> &'static str { lelwel_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { lelwel_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        match lelwel_lox::parse_program(input) {
            Ok(program) => ParseResult {
                success: true,
                statement_count: program.statements.len(),
                program: Some(program),
                error: None,
            },
            Err(e) => ParseResult {
                success: false,
                program: None,
                error: Some(format!("{:?}", e)),
                statement_count: 0,
            },
        }
    }
}

/// Get all available parsers (only working ones to avoid linker issues)
pub fn get_all_parsers() -> Vec<Box<dyn LoxParser>> {
    vec![
        Box::new(NomParser),
        // Box::new(ChumskyParser),  // Disabled due to macOS linker issues
        Box::new(PestParser),
        Box::new(LalrpopParser),
        Box::new(PomParser),
        Box::new(LelwelParser),
    ]
}

/// Get only working parsers (those that don't have known issues)
pub fn get_working_parsers() -> Vec<Box<dyn LoxParser>> {
    vec![
        Box::new(NomParser),
        // Box::new(ChumskyParser),  // Disabled due to macOS linker issues
        Box::new(PestParser),
        Box::new(LalrpopParser),
        Box::new(PomParser),
        Box::new(LelwelParser),
    ]
}

/// Test case structure
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: &'static str,
    pub input: &'static str,
    pub expected_statements: usize,
    pub description: &'static str,
}

/// Standard test cases covering all Lox language features
pub const TEST_CASES: &[TestCase] = &[
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
        description: "Print statement with string",
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
        description: "Variable declaration and assignment",
    },
    TestCase {
        name: "arithmetic_expression",
        input: "1 + 2 * 3;",
        expected_statements: 1,
        description: "Arithmetic with operator precedence",
    },
    TestCase {
        name: "comparison_expression",
        input: "5 > 3;",
        expected_statements: 1,
        description: "Comparison operation",
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
        description: "Grouped expression with parentheses",
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
        description: "Unary minus and logical not",
    },
    TestCase {
        name: "complex_arithmetic",
        input: "1 + 2 * 3 - 4 / 2;",
        expected_statements: 1,
        description: "Complex arithmetic with multiple operators",
    },
    TestCase {
        name: "multiple_statements",
        input: "var a = 10; var b = 20; var sum = a + b; print sum;",
        expected_statements: 4,
        description: "Multiple statements with variables and operations",
    },
];

/// Run a single test case against a parser
pub fn run_test_case(parser: &dyn LoxParser, test_case: &TestCase) -> bool {
    let result = parser.parse(test_case.input);
    result.success && result.statement_count == test_case.expected_statements
}

/// Run all test cases against a parser and return summary
pub fn run_parser_tests(parser: &dyn LoxParser) -> TestSummary {
    let mut passed = 0;
    let mut failed = 0;
    let mut results = Vec::new();

    for test_case in TEST_CASES {
        let success = run_test_case(parser, test_case);
        if success {
            passed += 1;
        } else {
            failed += 1;
        }
        results.push(TestResult {
            test_name: test_case.name.to_string(),
            success,
            description: test_case.description.to_string(),
        });
    }

    TestSummary {
        parser_name: parser.name().to_string(),
        passed,
        failed,
        total: TEST_CASES.len(),
        results,
    }
}

/// Summary of test results for a parser
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestSummary {
    pub parser_name: String,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub results: Vec<TestResult>,
}

/// Individual test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub description: String,
}

impl TestSummary {
    /// Print a colored summary of the test results
    pub fn print_summary(&self) {
        let success_rate = (self.passed as f64 / self.total as f64) * 100.0;

        println!("\n=== {} Test Results ===", self.parser_name.bold());

        if self.passed == self.total {
            println!("✅ {} All tests passed! ({}/{})",
                "SUCCESS:".green().bold(),
                self.passed.to_string().green().bold(),
                self.total
            );
        } else if self.passed > 0 {
            println!("⚠️  {} {}/{} tests passed ({:.1}%)",
                "PARTIAL:".yellow().bold(),
                self.passed.to_string().yellow().bold(),
                self.total,
                success_rate
            );
        } else {
            println!("❌ {} All tests failed ({}/{})",
                "FAILED:".red().bold(),
                self.failed.to_string().red().bold(),
                self.total
            );
        }

        // Show failed tests
        if self.failed > 0 {
            println!("\n{}", "Failed tests:".red().bold());
            for result in &self.results {
                if !result.success {
                    println!("  ❌ {}: {}", result.test_name.red(), result.description.dimmed());
                }
            }
        }
    }
}

/// Compare all parsers and return comprehensive results
pub fn compare_all_parsers() -> Vec<TestSummary> {
    let parsers = get_all_parsers();
    let mut summaries = Vec::new();

    println!("{}", "=== Lox Parser Comparison Test Suite ===".bold().cyan());
    println!("Testing {} parsers with {} test cases\n", parsers.len(), TEST_CASES.len());

    for parser in parsers {
        let summary = run_parser_tests(parser.as_ref());
        summary.print_summary();
        summaries.push(summary);
    }

    // Print overall comparison
    println!("\n{}", "=== Overall Comparison ===".bold().cyan());
    for summary in &summaries {
        let status = if summary.passed == summary.total {
            "✅ WORKING".green().bold()
        } else if summary.passed > 0 {
            "⚠️  PARTIAL".yellow().bold()
        } else {
            "❌ BROKEN".red().bold()
        };

        println!("{:12} {}/{:2} tests passed - {}",
            status,
            summary.passed,
            summary.total,
            summary.parser_name.bold()
        );
    }

    summaries
}
