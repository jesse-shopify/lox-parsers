use super::parser::{Diagnostic, Span};
use codespan_reporting::diagnostic::Label;
use logos::Logos;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LexerError {
    #[default]
    Invalid,
    // TODO: add more errors if required
}

impl LexerError {
    pub fn into_diagnostic(self, span: Span) -> Diagnostic {
        match self {
            Self::Invalid => Diagnostic::error()
                .with_message("invalid token")
                .with_label(Label::primary((), span)),
        }
    }
}

// TODO: implement lexer
#[allow(clippy::upper_case_acronyms)]
#[derive(Logos, Debug, PartialEq, Copy, Clone)]
#[logos(error = LexerError)]
pub enum Token {
    EOF,
    #[token("true")]
    TRUE,
    #[token("false")]
    FALSE,
    #[token("nil")]
    NIL,
    #[token("print")]
    PRINT,
    #[token("var")]
    VAR,
    #[token("and")]
    AND,
    #[token("or")]
    OR,
    #[token("+")]
    PLUS,
    #[token("-")]
    MINUS,
    #[token("*")]
    STAR,
    #[token("/")]
    SLASH,
    #[token("!")]
    BANG,
    #[token("!=")]
    BANG_EQUAL,
    #[token("=")]
    EQUAL,
    #[token("==")]
    EQUAL_EQUAL,
    #[token(">")]
    GREATER,
    #[token(">=")]
    GREATER_EQUAL,
    #[token("<")]
    LESS,
    #[token("<=")]
    LESS_EQUAL,
    #[token("(")]
    LEFT_PAREN,
    #[token(")")]
    RIGHT_PAREN,
    #[token(";")]
    SEMICOLON,
    NUMBER,
    STRING,
    IDENTIFIER,
    Error,
}

// TODO: extend tokenization (e.g. check for mismatched parentheses)
pub fn tokenize(
    source: &str,
    diags: &mut Vec<Diagnostic>,
) -> (Vec<Token>, Vec<Span>) {
    let lexer = Token::lexer(source);
    let mut tokens = vec![];
    let mut spans = vec![];

    for (token, span) in lexer.spanned() {
        match token {
            Ok(token) => {
                tokens.push(token);
            }
            Err(err) => {
                diags.push(err.into_diagnostic(span.clone()));
                tokens.push(Token::Error);
            }
        }
        spans.push(span);
    }
    (tokens, spans)
}
