use combine::{
    between, choice, many, many1, optional, token,
    parser::{
        char::{char, digit, letter, space, string},
        combinator::no_partial,
        function::parser,
        repeat::chainl1,
    },
    stream::Stream,
    Parser, ParseError, EasyParser,
};
use lox_ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Value};

pub fn parse_program(input: &str) -> Result<Program, String> {
    match program().easy_parse(input) {
        Ok((program, remaining)) => {
            if remaining.is_empty() {
                Ok(program)
            } else {
                Err(format!("Unexpected remaining input: {}", remaining))
            }
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

// Whitespace and comments
fn ws<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let whitespace = many::<Vec<_>, _, _>(choice((
        space().map(|_| ()),
        line_comment(),
    )))
    .map(|_| ());
    no_partial(whitespace)
}

fn line_comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        string("//"),
        many::<Vec<_>, _, _>(combine::satisfy(|c| c != '\n' && c != '\r')),
        optional(char('\n').or(char('\r'))),
    )
        .map(|_| ())
}

// Identifiers
fn identifier<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        letter().or(char('_')),
        many::<String, _, _>(letter().or(digit()).or(char('_'))),
    )
        .map(|(first, rest)| {
            let mut result = String::new();
            result.push(first);
            result.push_str(&rest);
            result
        })
}

// Keywords
fn keyword<Input>(word: &'static str) -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    string(word)
        .skip(combine::not_followed_by(
            letter().or(digit()).or(char('_'))
        ))
        .map(|_| ())
}

// Literals
fn number_literal<Input>() -> impl Parser<Input, Output = f64>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let integer = many1::<String, _, _>(digit());
    let decimal = char('.').with(many1::<String, _, _>(digit()));

    (integer, optional(decimal))
        .map(|(int_part, dec_part)| {
            let mut number_str = int_part;
            if let Some(dec) = dec_part {
                number_str.push('.');
                number_str.push_str(&dec);
            }
            number_str.parse().unwrap_or(0.0)
        })
}

fn string_literal<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    between(
        char('"'),
        char('"'),
        many::<String, _, _>(combine::satisfy(|c| c != '"')),
    )
}

fn boolean_literal<Input>() -> impl Parser<Input, Output = bool>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        keyword("true").map(|_| true),
        keyword("false").map(|_| false),
    ))
}

fn nil_literal<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    keyword("nil")
}

// Primary expressions
fn primary<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        number_literal().map(|n| Expr::Literal(Value::Number(n))),
        string_literal().map(|s| Expr::Literal(Value::String(s))),
        boolean_literal().map(|b| Expr::Literal(Value::Bool(b))),
        nil_literal().map(|_| Expr::Literal(Value::Nil)),
        identifier().map(Expr::Variable),
        between(char('(').skip(ws()), char(')'), expression()),
    ))
}

// Unary expressions
fn unary<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let unary_op = choice((
        char('!').map(|_| UnaryOp::Not),
        char('-').map(|_| UnaryOp::Minus),
    ));

    choice((
        (unary_op.skip(ws()), primary())
            .map(|(op, expr)| Expr::Unary {
                operator: op,
                operand: Box::new(expr),
            }),
        primary(),
    ))
}

// Binary expressions with precedence
fn factor<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let op = choice((
        char('*').skip(ws()).map(|_| BinaryOp::Multiply),
        char('/').skip(ws()).map(|_| BinaryOp::Divide),
    ));

    chainl1(unary().skip(ws()), op.map(|op| {
        move |left: Expr, right: Expr| Expr::Binary {
            left: Box::new(left),
            operator: op.clone(),
            right: Box::new(right),
        }
    }))
}

fn term<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let op = choice((
        char('+').skip(ws()).map(|_| BinaryOp::Add),
        char('-').skip(ws()).map(|_| BinaryOp::Subtract),
    ));

    chainl1(factor().skip(ws()), op.map(|op| {
        move |left: Expr, right: Expr| Expr::Binary {
            left: Box::new(left),
            operator: op.clone(),
            right: Box::new(right),
        }
    }))
}

fn comparison<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let op = choice((
        string(">=").skip(ws()).map(|_| BinaryOp::GreaterEqual),
        string("<=").skip(ws()).map(|_| BinaryOp::LessEqual),
        char('>').skip(ws()).map(|_| BinaryOp::Greater),
        char('<').skip(ws()).map(|_| BinaryOp::Less),
    ));

    chainl1(term().skip(ws()), op.map(|op| {
        move |left: Expr, right: Expr| Expr::Binary {
            left: Box::new(left),
            operator: op.clone(),
            right: Box::new(right),
        }
    }))
}

fn equality<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let op = choice((
        string("==").skip(ws()).map(|_| BinaryOp::Equal),
        string("!=").skip(ws()).map(|_| BinaryOp::NotEqual),
    ));

    chainl1(comparison().skip(ws()), op.map(|op| {
        move |left: Expr, right: Expr| Expr::Binary {
            left: Box::new(left),
            operator: op.clone(),
            right: Box::new(right),
        }
    }))
}

fn logical_and<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let op = keyword("and").skip(ws()).map(|_| BinaryOp::And);

    chainl1(equality().skip(ws()), op.map(|op| {
        move |left: Expr, right: Expr| Expr::Binary {
            left: Box::new(left),
            operator: op.clone(),
            right: Box::new(right),
        }
    }))
}

fn logical_or<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let op = keyword("or").skip(ws()).map(|_| BinaryOp::Or);

    chainl1(logical_and().skip(ws()), op.map(|op| {
        move |left: Expr, right: Expr| Expr::Binary {
            left: Box::new(left),
            operator: op.clone(),
            right: Box::new(right),
        }
    }))
}

fn assignment<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        (
            identifier().skip(ws()),
            char('=').skip(ws()),
            logical_or(),
        )
            .map(|(name, _, value)| Expr::Assignment {
                name,
                value: Box::new(value),
            }),
        logical_or(),
    ))
}

fn expression<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    assignment()
}

// Statements
fn print_statement<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("print").skip(ws()),
        expression().skip(ws()),
        char(';'),
    )
        .map(|(_, expr, _)| Stmt::Print(expr))
}

fn var_declaration<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        keyword("var").skip(ws()),
        identifier().skip(ws()),
        optional(char('=').skip(ws()).with(expression())).skip(ws()),
        char(';'),
    )
        .map(|(_, name, initializer, _)| Stmt::VarDeclaration { name, initializer })
}

fn expression_statement<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (expression().skip(ws()), char(';')).map(|(expr, _)| Stmt::Expression(expr))
}

fn statement<Input>() -> impl Parser<Input, Output = Stmt>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        print_statement(),
        var_declaration(),
        expression_statement(),
    ))
}

fn program<Input>() -> impl Parser<Input, Output = Program>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        ws(),
        many::<Vec<_>, _, _>(statement().skip(ws())),
    )
        .map(|(_, statements)| Program { statements })
}
