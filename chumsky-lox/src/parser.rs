//! Chumsky-based parser for the Lox language (1.0.0-alpha.8)

use chumsky::prelude::*;
use lox_ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Value};

/// Parse whitespace and comments
fn whitespace<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .ignored();

    choice((
        one_of(" \t\r\n").ignored(),
        comment,
    ))
    .repeated()
    .ignored()
}

/// Parse an identifier
fn identifier<'src>() -> impl Parser<'src, &'src str, String> + Clone {
    text::ident()
        .map(|s: &str| s.to_string())
        .padded_by(whitespace())
}

/// Parse a string literal
fn string_literal<'src>() -> impl Parser<'src, &'src str, String> + Clone {
    just('"')
        .ignore_then(
            any().and_is(just('"').not())
                .repeated()
                .to_slice()
                .map(|s: &str| s.to_string())
        )
        .then_ignore(just('"'))
        .padded_by(whitespace())
}

/// Parse a number literal
fn number_literal<'src>() -> impl Parser<'src, &'src str, f64> + Clone {
    text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| s.parse().unwrap())
        .padded_by(whitespace())
}

/// Parse a boolean literal
fn boolean_literal<'src>() -> impl Parser<'src, &'src str, bool> + Clone {
    choice((
        just("true").to(true),
        just("false").to(false),
    ))
    .padded_by(whitespace())
}

/// Parse nil literal
fn nil_literal<'src>() -> impl Parser<'src, &'src str, ()> + Clone {
    just("nil").ignored().padded_by(whitespace())
}

/// Parse a literal value
fn literal<'src>() -> impl Parser<'src, &'src str, Value> + Clone {
    choice((
        nil_literal().to(Value::Nil),
        boolean_literal().map(Value::Bool),
        number_literal().map(Value::Number),
        string_literal().map(Value::String),
    ))
}

/// Forward declaration for recursive parsing
fn expression<'src>() -> impl Parser<'src, &'src str, Expr> + Clone {
    recursive(|expr| {
        let atom = choice((
            literal().map(Expr::Literal),
            identifier().map(Expr::Variable),
            expr.clone()
                .delimited_by(just('(').padded_by(whitespace()), just(')').padded_by(whitespace()))
                .map(|e| Expr::Grouping(Box::new(e))),
        ));

        let unary = choice((
            just('!')
                .padded_by(whitespace())
                .ignore_then(expr.clone())
                .map(|operand| Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                }),
            just('-')
                .padded_by(whitespace())
                .ignore_then(expr.clone())
                .map(|operand| Expr::Unary {
                    operator: UnaryOp::Minus,
                    operand: Box::new(operand),
                }),
            atom,
        ));

        let factor = unary.clone().foldl(
            choice((
                just('*').padded_by(whitespace()).to(BinaryOp::Multiply),
                just('/').padded_by(whitespace()).to(BinaryOp::Divide),
            ))
            .then(unary)
            .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        );

        let term = factor.clone().foldl(
            choice((
                just('+').padded_by(whitespace()).to(BinaryOp::Add),
                just('-').padded_by(whitespace()).to(BinaryOp::Subtract),
            ))
            .then(factor)
            .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        );

        let comparison = term.clone().foldl(
            choice((
                just(">=").padded_by(whitespace()).to(BinaryOp::GreaterEqual),
                just('>').padded_by(whitespace()).to(BinaryOp::Greater),
                just("<=").padded_by(whitespace()).to(BinaryOp::LessEqual),
                just('<').padded_by(whitespace()).to(BinaryOp::Less),
            ))
            .then(term)
            .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        );

        let equality = comparison.clone().foldl(
            choice((
                just("!=").padded_by(whitespace()).to(BinaryOp::NotEqual),
                just("==").padded_by(whitespace()).to(BinaryOp::Equal),
            ))
            .then(comparison)
            .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        );

        let logical_and = equality.clone().foldl(
            just("and")
                .padded_by(whitespace())
                .to(BinaryOp::And)
                .then(equality)
                .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        );

        let logical_or = logical_and.clone().foldl(
            just("or")
                .padded_by(whitespace())
                .to(BinaryOp::Or)
                .then(logical_and)
                .repeated(),
            |left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        );

        let assignment = choice((
            identifier()
                .then_ignore(just('=').padded_by(whitespace()))
                .then(expr.clone())
                .map(|(name, value)| Expr::Assignment {
                    name,
                    value: Box::new(value),
                }),
            logical_or,
        ));

        assignment
    })
}

/// Parse a print statement
fn print_stmt<'src>() -> impl Parser<'src, &'src str, Stmt> + Clone {
    just("print")
        .padded_by(whitespace())
        .ignore_then(expression())
        .then_ignore(just(';').padded_by(whitespace()))
        .map(Stmt::Print)
}

/// Parse a variable declaration
fn var_declaration<'src>() -> impl Parser<'src, &'src str, Stmt> + Clone {
    just("var")
        .padded_by(whitespace())
        .ignore_then(identifier())
        .then(
            just('=')
                .padded_by(whitespace())
                .ignore_then(expression())
                .or_not()
        )
        .then_ignore(just(';').padded_by(whitespace()))
        .map(|(name, initializer)| Stmt::VarDeclaration { name, initializer })
}

/// Parse an expression statement
fn expr_stmt<'src>() -> impl Parser<'src, &'src str, Stmt> + Clone {
    expression()
        .then_ignore(just(';').padded_by(whitespace()))
        .map(Stmt::Expression)
}

/// Parse a statement
fn statement<'src>() -> impl Parser<'src, &'src str, Stmt> + Clone {
    choice((
        print_stmt(),
        var_declaration(),
        expr_stmt(),
    ))
}

/// Parse a program (list of statements)
fn program<'src>() -> impl Parser<'src, &'src str, Program> + Clone {
    whitespace()
        .ignore_then(statement().repeated().collect::<Vec<_>>())
        .then_ignore(end())
        .map(Program::new)
}

/// Parse a complete Lox program from a string
pub fn parse_program(input: &str) -> Result<Program, String> {
    match program().parse(input).into_result() {
        Ok(program) => Ok(program),
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .into_iter()
                .map(|e| format!("{:?}", e))
                .collect();
            Err(error_messages.join("; "))
        }
    }
}