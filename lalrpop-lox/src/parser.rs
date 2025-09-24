//! Parser module for LALRPOP-generated Lox parser

use lalrpop_util::{lalrpop_mod, ParseError, lexer::Token};
use lox_ast::Program;

// Include the generated parser
lalrpop_mod!(pub lox);

/// Parse a Lox program from input string
pub fn parse_program(input: &str) -> Result<Program, String> {
    let parser = lox::ProgramParser::new();

    match parser.parse(input) {
        Ok(program) => Ok(program),
        Err(e) => Err(format_parse_error(e, input)),
    }
}

/// Format LALRPOP parse errors into user-friendly messages
fn format_parse_error(error: ParseError<usize, Token<'_>, &str>, input: &str) -> String {
    match error {
        ParseError::InvalidToken { location } => {
            format!("Invalid token at position {}", location)
        }
        ParseError::UnrecognizedEof { location, expected } => {
            format!("Unexpected end of input at position {}. Expected one of: {}",
                location, expected.join(", "))
        }
        ParseError::UnrecognizedToken { token: (start, _tok, end), expected } => {
            let token_text = &input[start..end];
            format!("Unexpected token '{}' at position {}. Expected one of: {}",
                token_text, start, expected.join(", "))
        }
        ParseError::ExtraToken { token: (start, _tok, end) } => {
            let token_text = &input[start..end];
            format!("Extra token '{}' at position {}", token_text, start)
        }
        ParseError::User { error } => {
            format!("Parse error: {}", error)
        }
    }
}
