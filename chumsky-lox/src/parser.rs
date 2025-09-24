//! Chumsky-based parser for the Lox language

use chumsky::prelude::*;
use lox_ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Value};

/// Parse whitespace and comments
fn whitespace() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    let comment = just("//")
        .then(take_until(just('\n')))
        .ignored();

    choice((
        one_of(" \t\r\n").ignored(),
        comment,
    ))
    .repeated()
    .ignored()
}

/// Parse an identifier
fn identifier() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    text::ident().padded_by(whitespace())
}

/// Parse a string literal
fn string_literal() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    just('"')
        .ignore_then(
            filter(|c| *c != '"')
                .repeated()
                .collect::<String>()
        )
        .then_ignore(just('"'))
        .padded_by(whitespace())
}

/// Parse a number literal
fn number_literal() -> impl Parser<char, f64, Error = Simple<char>> + Clone {
    text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(|s| s.parse().unwrap())
        .padded_by(whitespace())
}

/// Parse a boolean literal
fn boolean_literal() -> impl Parser<char, bool, Error = Simple<char>> + Clone {
    choice((
        just("true").to(true),
        just("false").to(false),
    ))
    .padded_by(whitespace())
}

/// Parse nil literal
fn nil_literal() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    just("nil").to(()).padded_by(whitespace())
}

/// Parse a literal value
fn literal() -> impl Parser<char, Value, Error = Simple<char>> + Clone {
    choice((
        nil_literal().to(Value::Nil),
        boolean_literal().map(Value::Bool),
        number_literal().map(Value::Number),
        string_literal().map(Value::String),
    ))
}

/// Parse expressions - simplified to avoid compiler issues
fn expression() -> impl Parser<char, Expr, Error = Simple<char>> + Clone {
    recursive(|expr| {
        let atom = choice((
            literal().map(Expr::Literal),
            identifier().map(Expr::Variable),
            expr.clone()
                .delimited_by(just('(').padded_by(whitespace()), just(')').padded_by(whitespace()))
                .map(|e| Expr::Grouping(Box::new(e))),
        ));

        // Unary expressions
        let unary = choice((
            just('!').to(UnaryOp::Not),
            just('-').to(UnaryOp::Minus),
        ))
        .padded_by(whitespace())
        .then(atom.clone())
        .map(|(op, operand)| Expr::Unary {
            operator: op,
            operand: Box::new(operand),
        })
        .or(atom);

        // Binary expressions - simplified precedence
        let product = unary
            .clone()
            .then(
                choice((
                    just('*').to(BinaryOp::Multiply),
                    just('/').to(BinaryOp::Divide),
                ))
                .padded_by(whitespace())
                .then(unary)
                .repeated()
            )
            .foldl(|left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            });

        let sum = product
            .clone()
            .then(
                choice((
                    just('+').to(BinaryOp::Add),
                    just('-').to(BinaryOp::Subtract),
                ))
                .padded_by(whitespace())
                .then(product)
                .repeated()
            )
            .foldl(|left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            });

        let comparison = sum
            .clone()
            .then(
                choice((
                    just(">=").to(BinaryOp::GreaterEqual),
                    just('>').to(BinaryOp::Greater),
                    just("<=").to(BinaryOp::LessEqual),
                    just('<').to(BinaryOp::Less),
                ))
                .padded_by(whitespace())
                .then(sum)
                .repeated()
            )
            .foldl(|left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            });

        let equality = comparison
            .clone()
            .then(
                choice((
                    just("!=").to(BinaryOp::NotEqual),
                    just("==").to(BinaryOp::Equal),
                ))
                .padded_by(whitespace())
                .then(comparison)
                .repeated()
            )
            .foldl(|left, (op, right)| Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            });

        let logical_and = equality
            .clone()
            .then(
                just("and")
                    .padded_by(whitespace())
                    .ignore_then(equality)
                    .repeated()
            )
            .foldl(|left, right| Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            });

        let logical_or = logical_and
            .clone()
            .then(
                just("or")
                    .padded_by(whitespace())
                    .ignore_then(logical_and)
                    .repeated()
            )
            .foldl(|left, right| Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            });

        // Assignment
        identifier()
            .then_ignore(just('=').padded_by(whitespace()))
            .then(logical_or.clone())
            .map(|(name, value)| Expr::Assignment {
                name,
                value: Box::new(value),
            })
            .or(logical_or)
    })
}

/// Parse a print statement
fn print_stmt() -> impl Parser<char, Stmt, Error = Simple<char>> + Clone {
    just("print")
        .padded_by(whitespace())
        .ignore_then(expression())
        .then_ignore(just(';').padded_by(whitespace()))
        .map(Stmt::Print)
}

/// Parse a variable declaration
fn var_declaration() -> impl Parser<char, Stmt, Error = Simple<char>> + Clone {
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
fn expr_stmt() -> impl Parser<char, Stmt, Error = Simple<char>> + Clone {
    expression()
        .then_ignore(just(';').padded_by(whitespace()))
        .map(Stmt::Expression)
}

/// Parse a statement
fn statement() -> impl Parser<char, Stmt, Error = Simple<char>> + Clone {
    choice((
        print_stmt(),
        var_declaration(),
        expr_stmt(),
    ))
}

/// Parse a program (list of statements)
fn program() -> impl Parser<char, Program, Error = Simple<char>> + Clone {
    whitespace()
        .ignore_then(statement().repeated())
        .then_ignore(end())
        .map(Program::new)
}

/// Parse a complete Lox program from a string
pub fn parse_program(input: &str) -> Result<Program, Vec<Simple<char>>> {
    program().parse(input)
}
