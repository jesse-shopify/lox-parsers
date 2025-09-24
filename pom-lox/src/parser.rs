use pom::parser::{Parser, is_a, none_of, sym, seq, end};
use lox_ast::{Program, Stmt, Expr, Value, BinaryOp};

pub fn parse_program(input: &str) -> Result<Program, String> {
    match program().parse(input.as_bytes()) {
        Ok(program) => Ok(program),
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

/// Parse a complete program
fn program<'a>() -> Parser<'a, u8, Program> {
    (ws() * statements() - ws() - end()).map(|statements| Program { statements })
}

/// Parse a list of statements
fn statements<'a>() -> Parser<'a, u8, Vec<Stmt>> {
    statement().repeat(0..)
}

/// Parse a single statement
fn statement<'a>() -> Parser<'a, u8, Stmt> {
    ws() * (var_declaration() | assignment_statement() | expression_statement() | print_statement()) - ws()
}

/// Parse assignment statement: IDENTIFIER = EXPRESSION;
fn assignment_statement<'a>() -> Parser<'a, u8, Stmt> {
    (identifier() - ws() - sym(b'=') - ws() + expression() - ws() - sym(b';'))
    .map(|(name, value)| Stmt::Expression(Expr::Assignment {
        name,
        value: Box::new(value),
    }))
}

/// Parse variable declaration: var IDENTIFIER = EXPRESSION;
fn var_declaration<'a>() -> Parser<'a, u8, Stmt> {
    (seq(b"var") * ws() * identifier() + (ws() * sym(b'=') * ws() * expression()).opt() - ws() - sym(b';'))
    .map(|(name, initializer)| Stmt::VarDeclaration { name, initializer })
}

/// Parse print statement: print EXPRESSION;
fn print_statement<'a>() -> Parser<'a, u8, Stmt> {
    (seq(b"print") * ws() * expression() - ws() - sym(b';')).map(Stmt::Print)
}

/// Parse expression statement: EXPRESSION;
fn expression_statement<'a>() -> Parser<'a, u8, Stmt> {
    (expression() - ws() - sym(b';')).map(Stmt::Expression)
}

/// Parse expressions with basic arithmetic
fn expression<'a>() -> Parser<'a, u8, Expr> {
    term()
}

/// Parse term: factor ("+" factor | "-" factor)*
fn term<'a>() -> Parser<'a, u8, Expr> {
    (factor() + (ws() * (sym(b'+') | sym(b'-')) + ws() * factor()).repeat(0..))
    .map(|(first, rest)| {
        rest.into_iter().fold(first, |left, (op, right)| {
            let binary_op = if op == b'+' { BinaryOp::Add } else { BinaryOp::Subtract };
            Expr::Binary {
                left: Box::new(left),
                operator: binary_op,
                right: Box::new(right),
            }
        })
    })
}

/// Parse factor: primary ("*" primary | "/" primary)*
fn factor<'a>() -> Parser<'a, u8, Expr> {
    (primary() + (ws() * (sym(b'*') | sym(b'/')) + ws() * primary()).repeat(0..))
    .map(|(first, rest)| {
        rest.into_iter().fold(first, |left, (op, right)| {
            let binary_op = if op == b'*' { BinaryOp::Multiply } else { BinaryOp::Divide };
            Expr::Binary {
                left: Box::new(left),
                operator: binary_op,
                right: Box::new(right),
            }
        })
    })
}

/// Parse primary expressions
fn primary<'a>() -> Parser<'a, u8, Expr> {
    literal() | variable()
}

/// Parse variable reference
fn variable<'a>() -> Parser<'a, u8, Expr> {
    identifier().map(Expr::Variable)
}

/// Parse literals (simplified)
fn literal<'a>() -> Parser<'a, u8, Expr> {
    (seq(b"true").map(|_| Expr::Literal(Value::Bool(true)))) |
    (seq(b"false").map(|_| Expr::Literal(Value::Bool(false)))) |
    (seq(b"nil").map(|_| Expr::Literal(Value::Nil))) |
    (number().map(|n| Expr::Literal(Value::Number(n)))) |
    (string().map(|s| Expr::Literal(Value::String(s))))
}

/// Parse number literals (simplified - just integers)
fn number<'a>() -> Parser<'a, u8, f64> {
    is_a(|c: u8| (b'0'..=b'9').contains(&c)).repeat(1..)
    .map(|digits| {
        let number_str = String::from_utf8(digits).unwrap_or_default();
        number_str.parse::<f64>().unwrap_or(0.0)
    })
}

/// Parse string literals
fn string<'a>() -> Parser<'a, u8, String> {
    (sym(b'"') * none_of(b"\"").repeat(0..) - sym(b'"'))
    .map(|chars| String::from_utf8(chars).unwrap_or_default())
}

/// Parse identifiers
fn identifier<'a>() -> Parser<'a, u8, String> {
    (is_a(|c: u8| (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_') +
     is_a(|c: u8| (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'_').repeat(0..))
    .map(|(first_char, rest_chars)| {
        let mut name = String::new();
        name.push(first_char as char);
        for &c in &rest_chars {
            name.push(c as char);
        }
        name
    })
}

/// Parse whitespace
fn ws<'a>() -> Parser<'a, u8, ()> {
    is_a(|c: u8| c == b' ' || c == b'\t' || c == b'\r' || c == b'\n').repeat(0..).map(|_| ())
}