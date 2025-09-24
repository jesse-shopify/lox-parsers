//! Winnow-based parser for the Lox language

use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, delimited, opt, preceded, repeat, terminated},
    token::{take_while},
    ModalResult, Parser, stream::AsChar,
};
use lox_ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Value};

/// Parse whitespace and comments
fn ws(input: &mut &str) -> ModalResult<()> {
    repeat::<_, _, (), _, _>(0.., alt((
        multispace0.void(),
        ("//", take_while(0.., |c| c != '\n')).void(),
    )))
    .void()
    .parse_next(input)
}

/// Parse an identifier
fn identifier(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| c.is_alphanum() || c == '_')
        .verify(|s: &str| s.chars().next().unwrap().is_alpha() || s.starts_with('_'))
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

/// Parse a string literal
fn string_literal(input: &mut &str) -> ModalResult<String> {
    delimited(
        '"',
        take_while(0.., |c| c != '"').map(|s: &str| s.to_string()),
        '"',
    )
    .parse_next(input)
}

/// Parse a number literal
fn number_literal(input: &mut &str) -> ModalResult<f64> {
    (digit1, opt(('.', digit1)))
        .take()
        .try_map(|s: &str| s.parse::<f64>())
        .parse_next(input)
}

/// Parse a boolean literal
fn boolean_literal(input: &mut &str) -> ModalResult<bool> {
    alt(("true".value(true), "false".value(false))).parse_next(input)
}

/// Parse nil literal
fn nil_literal(input: &mut &str) -> ModalResult<()> {
    "nil".value(()).parse_next(input)
}

/// Parse a literal value
fn literal(input: &mut &str) -> ModalResult<Value> {
    alt((
        nil_literal.map(|_| Value::Nil),
        boolean_literal.map(Value::Bool),
        number_literal.map(Value::Number),
        string_literal.map(Value::String),
    ))
    .parse_next(input)
}

/// Parse a primary expression
fn primary(input: &mut &str) -> ModalResult<Expr> {
    alt((
        literal.map(Expr::Literal),
        identifier.map(Expr::Variable),
        delimited(
            (ws, '(', ws),
            expression,
            (ws, ')', ws),
        )
        .map(|e| Expr::Grouping(Box::new(e))),
    ))
    .parse_next(input)
}

/// Parse unary expressions
fn unary(input: &mut &str) -> ModalResult<Expr> {
    alt((
        (
            alt((
                '!'.value(UnaryOp::Not),
                '-'.value(UnaryOp::Minus),
            )),
            ws,
            unary,
        )
            .map(|(op, _, expr)| Expr::Unary {
                operator: op,
                operand: Box::new(expr),
            }),
        primary,
    ))
    .parse_next(input)
}

/// Parse multiplication and division
fn factor(input: &mut &str) -> ModalResult<Expr> {
    let (init, ops): (Expr, Vec<((), BinaryOp, (), Expr)>) = (
        unary,
        repeat(
            0..,
            (
                ws,
                alt((
                    '*'.value(BinaryOp::Multiply),
                    '/'.value(BinaryOp::Divide),
                )),
                ws,
                unary,
            ),
        ),
    )
        .parse_next(input)?;

    Ok(ops.into_iter().fold(init, |acc, (_, op, _, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    }))
}

/// Parse addition and subtraction
fn term(input: &mut &str) -> ModalResult<Expr> {
    let (init, ops): (Expr, Vec<((), BinaryOp, (), Expr)>) = (
        factor,
        repeat(
            0..,
            (
                ws,
                alt((
                    '+'.value(BinaryOp::Add),
                    '-'.value(BinaryOp::Subtract),
                )),
                ws,
                factor,
            ),
        ),
    )
        .parse_next(input)?;

    Ok(ops.into_iter().fold(init, |acc, (_, op, _, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    }))
}

/// Parse comparison operators
fn comparison(input: &mut &str) -> ModalResult<Expr> {
    let (init, ops): (Expr, Vec<((), BinaryOp, (), Expr)>) = (
        term,
        repeat(
            0..,
            (
                ws,
                alt((
                    ">=".value(BinaryOp::GreaterEqual),
                    '>'.value(BinaryOp::Greater),
                    "<=".value(BinaryOp::LessEqual),
                    '<'.value(BinaryOp::Less),
                )),
                ws,
                term,
            ),
        ),
    )
        .parse_next(input)?;

    Ok(ops.into_iter().fold(init, |acc, (_, op, _, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    }))
}

/// Parse equality operators
fn equality(input: &mut &str) -> ModalResult<Expr> {
    let (init, ops): (Expr, Vec<((), BinaryOp, (), Expr)>) = (
        comparison,
        repeat(
            0..,
            (
                ws,
                alt((
                    "!=".value(BinaryOp::NotEqual),
                    "==".value(BinaryOp::Equal),
                )),
                ws,
                comparison,
            ),
        ),
    )
        .parse_next(input)?;

    Ok(ops.into_iter().fold(init, |acc, (_, op, _, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    }))
}

/// Parse logical AND
fn logical_and(input: &mut &str) -> ModalResult<Expr> {
    let (init, ops): (Expr, Vec<((), &str, (), Expr)>) = (
        equality,
        repeat(0.., (ws, "and", ws, equality)),
    )
        .parse_next(input)?;

    Ok(ops.into_iter().fold(init, |acc, (_, _, _, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: BinaryOp::And,
            right: Box::new(expr),
        }
    }))
}

/// Parse logical OR
fn logical_or(input: &mut &str) -> ModalResult<Expr> {
    let (init, ops): (Expr, Vec<((), &str, (), Expr)>) = (
        logical_and,
        repeat(0.., (ws, "or", ws, logical_and)),
    )
        .parse_next(input)?;

    Ok(ops.into_iter().fold(init, |acc, (_, _, _, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: BinaryOp::Or,
            right: Box::new(expr),
        }
    }))
}

/// Parse assignment
fn assignment(input: &mut &str) -> ModalResult<Expr> {
    alt((
        (identifier, ws, '=', ws, assignment).map(|(name, _, _, _, value)| {
            Expr::Assignment {
                name,
                value: Box::new(value),
            }
        }),
        logical_or,
    ))
    .parse_next(input)
}

/// Parse a full expression
fn expression(input: &mut &str) -> ModalResult<Expr> {
    assignment.parse_next(input)
}

/// Parse a print statement
fn print_stmt(input: &mut &str) -> ModalResult<Stmt> {
    ("print", ws, expression, ws, ';')
        .map(|(_, _, expr, _, _)| Stmt::Print(expr))
        .parse_next(input)
}

/// Parse a variable declaration
fn var_declaration(input: &mut &str) -> ModalResult<Stmt> {
    (
        "var",
        ws,
        identifier,
        opt((ws, '=', ws, expression)),
        ws,
        ';',
    )
        .map(|(_, _, name, initializer, _, _)| Stmt::VarDeclaration {
            name,
            initializer: initializer.map(|(_, _, _, expr)| expr),
        })
        .parse_next(input)
}

/// Parse an expression statement
fn expr_stmt(input: &mut &str) -> ModalResult<Stmt> {
    terminated(expression, (ws, ';'))
        .map(Stmt::Expression)
        .parse_next(input)
}

/// Parse a statement
fn statement(input: &mut &str) -> ModalResult<Stmt> {
    preceded(
        ws,
        alt((print_stmt, var_declaration, expr_stmt)),
    )
    .parse_next(input)
}

/// Parse a program (list of statements)
fn program(input: &mut &str) -> ModalResult<Program> {
    terminated(repeat(0.., statement), ws)
        .map(Program::new)
        .parse_next(input)
}

/// Parse a complete Lox program from a string
pub fn parse_program(input: &str) -> Result<Program, winnow::error::ParseError<&str, winnow::error::ContextError>> {
    program.parse(input)
}
