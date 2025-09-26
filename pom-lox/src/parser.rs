use pom::parser::{Parser, is_a, none_of, sym, seq, end};
use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

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

/// Parse expressions - add logical operators
fn expression<'a>() -> Parser<'a, u8, Expr> {
    logical_or()
}

/// Parse logical OR: logical_and ("or" logical_and)*
fn logical_or<'a>() -> Parser<'a, u8, Expr> {
    (logical_and() + (ws() * seq(b"or") + ws() * logical_and()).repeat(0..))
    .map(|(first, rest)| {
        rest.into_iter().fold(first, |left, (_, right)| {
            Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::Or,
                right: Box::new(right),
            }
        })
    })
}

/// Parse logical AND: equality ("and" equality)*
fn logical_and<'a>() -> Parser<'a, u8, Expr> {
    (equality() + (ws() * seq(b"and") + ws() * equality()).repeat(0..))
    .map(|(first, rest)| {
        rest.into_iter().fold(first, |left, (_, right)| {
            Expr::Binary {
                left: Box::new(left),
                operator: BinaryOp::And,
                right: Box::new(right),
            }
        })
    })
}

/// Parse equality: comparison ("==" comparison | "!=" comparison)*
fn equality<'a>() -> Parser<'a, u8, Expr> {
    (comparison() + (ws() * (seq(b"==") | seq(b"!=")) + ws() * comparison()).repeat(0..))
    .map(|(first, rest)| {
        rest.into_iter().fold(first, |left, (op, right)| {
            let binary_op = if op == b"==" { BinaryOp::Equal } else { BinaryOp::NotEqual };
            Expr::Binary {
                left: Box::new(left),
                operator: binary_op,
                right: Box::new(right),
            }
        })
    })
}

/// Parse comparison: term (">=" term | ">" term | "<=" term | "<" term)*
fn comparison<'a>() -> Parser<'a, u8, Expr> {
    (term() + (ws() * (seq(b">=") | seq(b">") | seq(b"<=") | seq(b"<")) + ws() * term()).repeat(0..))
    .map(|(first, rest)| {
        rest.into_iter().fold(first, |left, (op, right)| {
            let binary_op = match op {
                b">=" => BinaryOp::GreaterEqual,
                b">" => BinaryOp::Greater,
                b"<=" => BinaryOp::LessEqual,
                b"<" => BinaryOp::Less,
                _ => unreachable!(),
            };
            Expr::Binary {
                left: Box::new(left),
                operator: binary_op,
                right: Box::new(right),
            }
        })
    })
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

/// Parse factor: unary ("*" unary | "/" unary)*
fn factor<'a>() -> Parser<'a, u8, Expr> {
    (unary() + (ws() * (sym(b'*') | sym(b'/')) + ws() * unary()).repeat(0..))
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

/// Parse unary expressions: ("!" | "-")* primary
fn unary<'a>() -> Parser<'a, u8, Expr> {
    ((sym(b'!') | sym(b'-')).repeat(0..) + primary())
    .map(|(ops, expr)| {
        ops.into_iter().rev().fold(expr, |acc, op| {
            let unary_op = if op == b'!' {
                UnaryOp::Not
            } else {
                UnaryOp::Minus
            };
            Expr::Unary {
                operator: unary_op,
                operand: Box::new(acc),
            }
        })
    })
}

/// Parse primary expressions
fn primary<'a>() -> Parser<'a, u8, Expr> {
    literal() | variable() | simple_grouped()
}

/// Parse simple grouped expressions: allow arithmetic but avoid recursion
fn simple_grouped<'a>() -> Parser<'a, u8, Expr> {
    // Create a special parser for inside parentheses that doesn't call primary (to avoid recursion)
    let inside_parens = term_no_grouping();
    (sym(b'(') * ws() * inside_parens - ws() - sym(b')')).map(|expr| Expr::Grouping(Box::new(expr)))
}

/// Parse term without grouping to avoid recursion
fn term_no_grouping<'a>() -> Parser<'a, u8, Expr> {
    (factor_no_grouping() + (ws() * (sym(b'+') | sym(b'-')) + ws() * factor_no_grouping()).repeat(0..))
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

/// Parse factor without grouping to avoid recursion
fn factor_no_grouping<'a>() -> Parser<'a, u8, Expr> {
    (unary_no_grouping() + (ws() * (sym(b'*') | sym(b'/')) + ws() * unary_no_grouping()).repeat(0..))
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

/// Parse unary without grouping to avoid recursion
fn unary_no_grouping<'a>() -> Parser<'a, u8, Expr> {
    ((sym(b'!') | sym(b'-')).repeat(0..) + primary_no_grouping())
    .map(|(ops, expr)| {
        ops.into_iter().rev().fold(expr, |acc, op| {
            let unary_op = if op == b'!' {
                UnaryOp::Not
            } else {
                UnaryOp::Minus
            };
            Expr::Unary {
                operator: unary_op,
                operand: Box::new(acc),
            }
        })
    })
}

/// Parse primary without grouping to avoid recursion
fn primary_no_grouping<'a>() -> Parser<'a, u8, Expr> {
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