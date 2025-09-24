//! Nom-based parser for the Lox language

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1, take_until},
    character::complete::{
        alpha1, alphanumeric1, char, multispace0, multispace1,
    },
    combinator::{map, opt, recognize, value},
    multi::many0,
    number::complete::double,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult
};

use lox_ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Value};


/// Parse a line comment
fn line_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("//"), take_until("\n")))(input)
}

/// Parse whitespace including comments
fn whitespace(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(alt((value((), multispace1), line_comment))),
    )(input)
}

/// Parse an identifier
fn identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(input)
}

/// Parse a string literal
fn string_literal(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        map(
            take_while1(|c| c != '"'),
            |s: &str| s.to_string(),
        ),
        char('"'),
    )(input)
}

/// Parse a number literal
fn number_literal(input: &str) -> IResult<&str, f64> {
    double(input)
}

/// Parse a boolean literal
fn boolean_literal(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("true")),
        value(false, tag("false")),
    ))(input)
}

/// Parse nil literal
fn nil_literal(input: &str) -> IResult<&str, ()> {
    value((), tag("nil"))(input)
}

/// Parse a literal value
fn literal(input: &str) -> IResult<&str, Value> {
    alt((
        map(nil_literal, |_| Value::Nil),
        map(boolean_literal, Value::Bool),
        map(number_literal, Value::Number),
        map(string_literal, Value::String),
    ))(input)
}

/// Parse a primary expression (literals, identifiers, groupings)
fn primary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(literal, Expr::Literal),
        map(identifier, Expr::Variable),
        delimited(
            delimited(multispace0, char('('), multispace0),
            map(expression, |e| Expr::Grouping(Box::new(e))),
            delimited(multispace0, char(')'), multispace0),
        ),
    ))(input)
}

/// Parse unary expressions
fn unary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            pair(
                delimited(
                    multispace0,
                    alt((
                        value(UnaryOp::Not, char('!')),
                        value(UnaryOp::Minus, char('-')),
                    )),
                    multispace0,
                ),
                unary,
            ),
            |(op, expr)| Expr::Unary {
                operator: op,
                operand: Box::new(expr),
            },
        ),
        primary,
    ))(input)
}

/// Parse multiplication and division
fn factor(input: &str) -> IResult<&str, Expr> {
    let (input, init) = unary(input)?;

    let (input, ops) = many0(pair(
        delimited(
            multispace0,
            alt((
                value(BinaryOp::Multiply, char('*')),
                value(BinaryOp::Divide, char('/')),
            )),
            multispace0,
        ),
        unary,
    ))(input)?;

    Ok((input, ops.into_iter().fold(init, |acc, (op, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    })))
}

/// Parse addition and subtraction
fn term(input: &str) -> IResult<&str, Expr> {
    let (input, init) = factor(input)?;

    let (input, ops) = many0(pair(
        delimited(
            multispace0,
            alt((
                value(BinaryOp::Add, char('+')),
                value(BinaryOp::Subtract, char('-')),
            )),
            multispace0,
        ),
        factor,
    ))(input)?;

    Ok((input, ops.into_iter().fold(init, |acc, (op, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    })))
}

/// Parse comparison operators
fn comparison(input: &str) -> IResult<&str, Expr> {
    let (input, init) = term(input)?;

    let (input, ops) = many0(pair(
        delimited(
            multispace0,
            alt((
                value(BinaryOp::GreaterEqual, tag(">=")),
                value(BinaryOp::Greater, char('>')),
                value(BinaryOp::LessEqual, tag("<=")),
                value(BinaryOp::Less, char('<')),
            )),
            multispace0,
        ),
        term,
    ))(input)?;

    Ok((input, ops.into_iter().fold(init, |acc, (op, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    })))
}

/// Parse equality operators
fn equality(input: &str) -> IResult<&str, Expr> {
    let (input, init) = comparison(input)?;

    let (input, ops) = many0(pair(
        delimited(
            multispace0,
            alt((
                value(BinaryOp::NotEqual, tag("!=")),
                value(BinaryOp::Equal, tag("==")),
            )),
            multispace0,
        ),
        comparison,
    ))(input)?;

    Ok((input, ops.into_iter().fold(init, |acc, (op, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: op,
            right: Box::new(expr),
        }
    })))
}

/// Parse logical AND
fn logical_and(input: &str) -> IResult<&str, Expr> {
    let (input, init) = equality(input)?;

    let (input, ops) = many0(pair(
        delimited(multispace0, tag("and"), multispace0),
        equality,
    ))(input)?;

    Ok((input, ops.into_iter().fold(init, |acc, (_, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: BinaryOp::And,
            right: Box::new(expr),
        }
    })))
}

/// Parse logical OR
fn logical_or(input: &str) -> IResult<&str, Expr> {
    let (input, init) = logical_and(input)?;

    let (input, ops) = many0(pair(
        delimited(multispace0, tag("or"), multispace0),
        logical_and,
    ))(input)?;

    Ok((input, ops.into_iter().fold(init, |acc, (_, expr)| {
        Expr::Binary {
            left: Box::new(acc),
            operator: BinaryOp::Or,
            right: Box::new(expr),
        }
    })))
}

/// Parse assignment
fn assignment(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            tuple((
                identifier,
                delimited(multispace0, char('='), multispace0),
                assignment,
            )),
            |(name, _, value)| Expr::Assignment {
                name,
                value: Box::new(value),
            },
        ),
        logical_or,
    ))(input)
}

/// Parse a full expression
fn expression(input: &str) -> IResult<&str, Expr> {
    assignment(input)
}

/// Parse a print statement
fn print_stmt(input: &str) -> IResult<&str, Stmt> {
    map(
        tuple((
            delimited(multispace0, tag("print"), multispace0),
            expression,
            delimited(multispace0, char(';'), multispace0),
        )),
        |(_, expr, _)| Stmt::Print(expr),
    )(input)
}

/// Parse a variable declaration
fn var_declaration(input: &str) -> IResult<&str, Stmt> {
    map(
        tuple((
            delimited(multispace0, tag("var"), multispace0),
            delimited(multispace0, identifier, multispace0),
            opt(preceded(delimited(multispace0, char('='), multispace0), expression)),
            delimited(multispace0, char(';'), multispace0),
        )),
        |(_, name, initializer, _)| Stmt::VarDeclaration { name, initializer },
    )(input)
}

/// Parse an expression statement
fn expr_stmt(input: &str) -> IResult<&str, Stmt> {
    map(
        terminated(expression, delimited(multispace0, char(';'), multispace0)),
        Stmt::Expression,
    )(input)
}

/// Parse a statement
fn statement(input: &str) -> IResult<&str, Stmt> {
    preceded(
        whitespace,
        alt((
            print_stmt,
            var_declaration,
            expr_stmt,
        )),
    )(input)
}

/// Parse a program (list of statements)
pub fn program(input: &str) -> IResult<&str, Program> {
    map(
        terminated(many0(statement), whitespace),
        Program::new,
    )(input)
}

/// Parse a complete Lox program from a string
pub fn parse_program(input: &str) -> Result<Program, String> {
    match program(input) {
        Ok(("", program)) => Ok(program),
        Ok((remaining, _)) => Err(format!("Unexpected input: {}", remaining)),
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}
