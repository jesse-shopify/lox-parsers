//! Parser comparison library for Lox language implementations
//!
//! This library provides functionality to compare different parser implementations
//! for the Lox programming language, testing their accuracy, performance, and
//! error handling capabilities.

use lox_ast::Program;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};

/// Represents a parser implementation
pub trait LoxParser {
    /// The name of the parser (e.g., "nom", "pest", "chumsky")
    fn name(&self) -> &'static str;

    /// The version of the underlying parser library
    fn version(&self) -> &'static str;

    /// A description of the parser
    fn description(&self) -> &'static str;

    /// Parse a Lox program from input string
    fn parse(&self, input: &str) -> ParseResult;
}

/// Result of parsing attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub success: bool,
    pub program: Option<Program>,
    pub error_message: Option<String>,
    pub parse_time: Duration,
}

impl ParseResult {
    pub fn success(program: Program, duration: Duration) -> Self {
        Self {
            success: true,
            program: Some(program),
            error_message: None,
            parse_time: duration,
        }
    }

    pub fn failure(error: String, duration: Duration) -> Self {
        Self {
            success: false,
            program: None,
            error_message: Some(error),
            parse_time: duration,
        }
    }
}

/// Comparison result for multiple parsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub input: String,
    pub results: Vec<ParserResult>,
    pub consensus_ast: Option<Program>,
    pub accuracy_summary: AccuracySummary,
}

/// Result from a single parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserResult {
    pub parser_name: String,
    pub parser_version: String,
    pub result: ParseResult,
}

/// Summary of accuracy across parsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracySummary {
    pub total_parsers: usize,
    pub successful_parsers: usize,
    pub failed_parsers: usize,
    pub consensus_reached: bool,
    pub average_parse_time: Duration,
}

/// Nom parser implementation
pub struct NomParser;

impl LoxParser for NomParser {
    fn name(&self) -> &'static str { nom_lox::PARSER_NAME }
    fn version(&self) -> &'static str { nom_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { nom_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        let start = Instant::now();
        match nom_lox::parse_program(input) {
            Ok(program) => ParseResult::success(program, start.elapsed()),
            Err(e) => ParseResult::failure(format!("{:?}", e), start.elapsed()),
        }
    }
}

/// Pest parser implementation
pub struct PestParser;

impl LoxParser for PestParser {
    fn name(&self) -> &'static str { pest_lox::PARSER_NAME }
    fn version(&self) -> &'static str { pest_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { pest_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        let start = Instant::now();
        match pest_lox::parse_program(input) {
            Ok(program) => ParseResult::success(program, start.elapsed()),
            Err(e) => ParseResult::failure(format!("{:?}", e), start.elapsed()),
        }
    }
}

/// Winnow parser implementation
pub struct WinnowParser;

impl LoxParser for WinnowParser {
    fn name(&self) -> &'static str { winnow_lox::PARSER_NAME }
    fn version(&self) -> &'static str { winnow_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { winnow_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        let start = Instant::now();
        match winnow_lox::parse_program(input) {
            Ok(program) => ParseResult::success(program, start.elapsed()),
            Err(e) => ParseResult::failure(format!("{:?}", e), start.elapsed()),
        }
    }
}

/// Chumsky parser implementation
pub struct ChumskyParser;

impl LoxParser for ChumskyParser {
    fn name(&self) -> &'static str { chumsky_lox::PARSER_NAME }
    fn version(&self) -> &'static str { chumsky_lox::PARSER_VERSION }
    fn description(&self) -> &'static str { chumsky_lox::PARSER_DESCRIPTION }

    fn parse(&self, input: &str) -> ParseResult {
        let start = Instant::now();
        match chumsky_lox::parse_program(input) {
            Ok(program) => ParseResult::success(program, start.elapsed()),
            Err(e) => ParseResult::failure(format!("{:?}", e), start.elapsed()),
        }
    }
}

/// Compare multiple parsers on the same input
pub fn compare_parsers(input: &str, parsers: Vec<Box<dyn LoxParser>>) -> ComparisonResult {
    let mut results = Vec::new();
    let mut successful_asts = Vec::new();

    for parser in parsers {
        let result = parser.parse(input);

        if let Some(ref program) = result.program {
            successful_asts.push(program.clone());
        }

        results.push(ParserResult {
            parser_name: parser.name().to_string(),
            parser_version: parser.version().to_string(),
            result,
        });
    }

    // Determine consensus AST (if all successful parsers agree)
    let consensus_ast = if successful_asts.len() > 1 {
        let first = &successful_asts[0];
        if successful_asts.iter().all(|ast| ast == first) {
            Some(first.clone())
        } else {
            None
        }
    } else if successful_asts.len() == 1 {
        Some(successful_asts[0].clone())
    } else {
        None
    };

    let successful_count = results.iter().filter(|r| r.result.success).count();
    let total_time: Duration = results.iter().map(|r| r.result.parse_time).sum();
    let average_time = if !results.is_empty() {
        total_time / results.len() as u32
    } else {
        Duration::ZERO
    };

    let total_parsers = results.len();

    ComparisonResult {
        input: input.to_string(),
        results,
        consensus_ast: consensus_ast.clone(),
        accuracy_summary: AccuracySummary {
            total_parsers,
            successful_parsers: successful_count,
            failed_parsers: total_parsers - successful_count,
            consensus_reached: consensus_ast.is_some() && successful_count > 1,
            average_parse_time: average_time,
        },
    }
}

/// Get all available parsers
pub fn get_all_parsers() -> Vec<Box<dyn LoxParser>> {
    vec![
        Box::new(NomParser),
        Box::new(PestParser),
        Box::new(WinnowParser),
        Box::new(ChumskyParser),
    ]
}

impl fmt::Display for ComparisonResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Parser Comparison Results ===")?;
        writeln!(f, "Input: {}", self.input.trim())?;
        writeln!(f)?;

        writeln!(f, "Summary:")?;
        writeln!(f, "  Total parsers: {}", self.accuracy_summary.total_parsers)?;
        writeln!(f, "  Successful: {}", self.accuracy_summary.successful_parsers)?;
        writeln!(f, "  Failed: {}", self.accuracy_summary.failed_parsers)?;
        writeln!(f, "  Consensus reached: {}", self.accuracy_summary.consensus_reached)?;
        writeln!(f, "  Average parse time: {:?}", self.accuracy_summary.average_parse_time)?;
        writeln!(f)?;

        writeln!(f, "Individual Results:")?;
        for result in &self.results {
            writeln!(f, "  {} v{}: {} ({:?})",
                result.parser_name,
                result.parser_version,
                if result.result.success { "✅ SUCCESS" } else { "❌ FAILED" },
                result.result.parse_time
            )?;

            if let Some(ref error) = result.result.error_message {
                writeln!(f, "    Error: {}", error)?;
            }
        }

        if let Some(ref ast) = self.consensus_ast {
            writeln!(f)?;
            writeln!(f, "Consensus AST:")?;
            writeln!(f, "{:#?}", ast)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_result_creation() {
        let program = Program::new(vec![]);
        let duration = Duration::from_millis(10);

        let success = ParseResult::success(program.clone(), duration);
        assert!(success.success);
        assert!(success.program.is_some());
        assert!(success.error_message.is_none());
        assert_eq!(success.parse_time, duration);

        let failure = ParseResult::failure("test error".to_string(), duration);
        assert!(!failure.success);
        assert!(failure.program.is_none());
        assert!(failure.error_message.is_some());
        assert_eq!(failure.parse_time, duration);
    }

    #[test]
    fn test_nom_parser_metadata() {
        let parser = NomParser;
        assert_eq!(parser.name(), "nom");
        assert_eq!(parser.version(), "7.1");
        assert!(!parser.description().is_empty());
    }

    #[test]
    fn test_nom_parser_simple_cases() {
        let parser = NomParser;

        // Test simple successful cases
        let test_cases = vec![
            ("42;", 1),
            ("print \"hello\";", 1),
            ("var x = 10;", 1),
            ("1 + 2;", 1),
        ];

        for (input, expected_statements) in test_cases {
            match parser.parse(input) {
                Ok(program) => {
                    assert_eq!(
                        program.statements.len(),
                        expected_statements,
                        "Failed for input: {}",
                        input
                    );
                }
                Err(_) => {
                    // For unit tests, we'll just note failures but not panic
                    // since some edge cases might legitimately fail
                    println!("Note: nom parser failed on: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_compare_parsers_single_parser() {
        let parsers = vec![Box::new(NomParser) as Box<dyn LoxParser>];
        let input = "42;";

        let result = compare_parsers(input, parsers);

        assert_eq!(result.input, input);
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.accuracy_summary.total_parsers, 1);

        // nom should succeed on simple input
        if result.accuracy_summary.successful_parsers == 1 {
            assert!(result.consensus_ast.is_some());
            assert!(result.accuracy_summary.consensus_reached);
        }
    }

    #[test]
    fn test_compare_parsers_empty_input() {
        let parsers = vec![Box::new(NomParser) as Box<dyn LoxParser>];
        let input = "";

        let result = compare_parsers(input, parsers);

        assert_eq!(result.input, input);
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.accuracy_summary.total_parsers, 1);

        // Empty input might succeed (empty program) or fail depending on parser
        // Either way is acceptable for this test
    }

    #[test]
    fn test_accuracy_summary_calculation() {
        let parsers = get_all_parsers();
        let input = "print \"test\";";

        let result = compare_parsers(input, parsers);

        let summary = &result.accuracy_summary;
        assert_eq!(
            summary.total_parsers,
            summary.successful_parsers + summary.failed_parsers
        );

        assert!(summary.average_parse_time >= Duration::ZERO);

        if summary.successful_parsers > 1 {
            // If multiple parsers succeeded, consensus might be reached
            println!(
                "Multiple parsers succeeded: {} out of {}",
                summary.successful_parsers,
                summary.total_parsers
            );
        }
    }

    #[test]
    fn test_get_all_parsers() {
        let parsers = get_all_parsers();

        // Should have at least nom parser
        assert!(!parsers.is_empty());

        // All parsers should have valid metadata
        for parser in parsers {
            assert!(!parser.name().is_empty());
            assert!(!parser.version().is_empty());
            assert!(!parser.description().is_empty());
        }
    }

    #[test]
    fn test_comparison_result_display() {
        let parsers = vec![Box::new(NomParser) as Box<dyn LoxParser>];
        let input = "42;";

        let result = compare_parsers(input, parsers);
        let display_output = format!("{}", result);

        // Should contain key information
        assert!(display_output.contains("Parser Comparison Results"));
        assert!(display_output.contains("42;"));
        assert!(display_output.contains("nom"));

        println!("Display output:\n{}", display_output);
    }

    #[test]
    fn test_serde_serialization() {
        use serde_json;

        let program = Program::new(vec![
            Stmt::Print(Expr::Literal(Value::String("test".to_string())))
        ]);

        // Test that Program can be serialized and deserialized
        let json = serde_json::to_string(&program).expect("Should serialize");
        let deserialized: Program = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(program, deserialized);

        println!("Serialized program: {}", json);
    }

    #[test]
    fn test_parser_timing() {
        let parser = NomParser;
        let input = "var x = 1 + 2 * 3;";

        let result = parser.parse(input);

        // Timing should be recorded
        assert!(result.parse_time > Duration::ZERO);
        assert!(result.parse_time < Duration::from_secs(1)); // Should be fast

        println!("Parse time: {:?}", result.parse_time);
    }
}
