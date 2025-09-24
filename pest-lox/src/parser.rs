//! Pest-based parser for the Lox language

use pest::Parser;
use pest::iterators::Pair;
use lox_ast::{BinaryOp, Expr, Program, Stmt, UnaryOp, Value};

#[derive(pest_derive::Parser)]
#[grammar = "lox.pest"]
pub struct LoxParser;

pub fn parse_program(input: &str) -> Result<Program, Box<pest::error::Error<Rule>>> {
    let pairs = LoxParser::parse(Rule::program, input)?;
    let program_pair = pairs.into_iter().next().unwrap();

    let mut statements = Vec::new();
    for pair in program_pair.into_inner() {
        match pair.as_rule() {
            Rule::statement => {
                statements.push(parse_statement(pair)?);
            }
            Rule::EOI => break,
            _ => {}
        }
    }

    Ok(Program::new(statements))
}

fn parse_statement(pair: Pair<Rule>) -> Result<Stmt, Box<pest::error::Error<Rule>>> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::print_stmt => parse_print_statement(inner),
        Rule::var_decl => parse_var_declaration(inner),
        Rule::expression_stmt => parse_expression_statement(inner),
        _ => Err(Box::new(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Unknown statement type".to_string(),
            },
            pest::Position::from_start(""),
        ))),
    }
}

fn parse_print_statement(pair: Pair<Rule>) -> Result<Stmt, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();

    // The first (and should be only) inner pair is the expression
    let expr = parse_expression(inner.next().unwrap())?;
    Ok(Stmt::Print(expr))
}

fn parse_var_declaration(pair: Pair<Rule>) -> Result<Stmt, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();

    // First inner should be the identifier
    let name = inner.next().unwrap().as_str().to_string();

    // Check if there's an initializer (the "=" and expression are grouped together)
    let initializer = if let Some(expr_pair) = inner.next() {
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };

    Ok(Stmt::VarDeclaration { name, initializer })
}

fn parse_expression_statement(pair: Pair<Rule>) -> Result<Stmt, Box<pest::error::Error<Rule>>> {
    let expr = parse_expression(pair.into_inner().next().unwrap())?;
    Ok(Stmt::Expression(expr))
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    match pair.as_rule() {
        Rule::assignment => parse_assignment(pair),
        Rule::logical_or => parse_binary_expr(pair, BinaryOp::Or),
        Rule::logical_and => parse_binary_expr(pair, BinaryOp::And),
        Rule::equality => parse_equality(pair),
        Rule::comparison => parse_comparison(pair),
        Rule::term => parse_term(pair),
        Rule::factor => parse_factor(pair),
        Rule::unary => parse_unary(pair),
        Rule::primary => parse_primary(pair),
        Rule::expression => parse_expression(pair.into_inner().next().unwrap()),
        _ => Err(Box::new(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Unknown expression type".to_string(),
            },
            pest::Position::from_start(""),
        ))),
    }
}

fn parse_assignment(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    if let Some(second) = inner.next() {
        // This is an assignment
        let name = first.as_str().to_string();
        let value = parse_expression(second)?;
        Ok(Expr::Assignment {
            name,
            value: Box::new(value),
        })
    } else {
        // This is just a logical_or
        parse_expression(first)
    }
}

fn parse_binary_expr(pair: Pair<Rule>, default_op: BinaryOp) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();
    let mut expr = parse_expression(inner.next().unwrap())?;

    while let Some(next) = inner.next() {
        expr = Expr::Binary {
            left: Box::new(expr),
            operator: default_op.clone(),
            right: Box::new(parse_expression(next)?),
        };
    }

    Ok(expr)
}

fn parse_equality(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();
    let mut expr = parse_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let right_expr = inner.next().unwrap();
        let op = match op_pair.as_str() {
            "==" => BinaryOp::Equal,
            "!=" => BinaryOp::NotEqual,
            _ => return Err(Box::new(pest::error::Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: "Unknown equality operator".to_string(),
                },
                pest::Position::from_start(""),
            ))),
        };

        expr = Expr::Binary {
            left: Box::new(expr),
            operator: op,
            right: Box::new(parse_expression(right_expr)?),
        };
    }

    Ok(expr)
}

fn parse_comparison(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();
    let mut expr = parse_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let right_expr = inner.next().unwrap();
        let op = match op_pair.as_str() {
            ">" => BinaryOp::Greater,
            ">=" => BinaryOp::GreaterEqual,
            "<" => BinaryOp::Less,
            "<=" => BinaryOp::LessEqual,
            _ => return Err(Box::new(pest::error::Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: "Unknown comparison operator".to_string(),
                },
                pest::Position::from_start(""),
            ))),
        };

        expr = Expr::Binary {
            left: Box::new(expr),
            operator: op,
            right: Box::new(parse_expression(right_expr)?),
        };
    }

    Ok(expr)
}

fn parse_term(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();
    let mut expr = parse_factor(inner.next().unwrap())?;

    // Now we expect alternating term_op and factor pairs
    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::term_op {
            let op_str = op_pair.as_str();
            let right_pair = inner.next().unwrap(); // Should be a factor

            let op = match op_str {
                "+" => BinaryOp::Add,
                "-" => BinaryOp::Subtract,
                _ => return Err(Box::new(pest::error::Error::new_from_pos(
                    pest::error::ErrorVariant::CustomError {
                        message: "Unknown term operator".to_string(),
                    },
                    pest::Position::from_start(""),
                ))),
            };

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(parse_factor(right_pair)?),
            };
        }
    }

    Ok(expr)
}

fn parse_factor(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner();
    let mut expr = parse_unary(inner.next().unwrap())?;

    // Now we expect alternating factor_op and unary pairs
    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::factor_op {
            let op_str = op_pair.as_str();
            let right_pair = inner.next().unwrap(); // Should be a unary

            let op = match op_str {
                "*" => BinaryOp::Multiply,
                "/" => BinaryOp::Divide,
                _ => return Err(Box::new(pest::error::Error::new_from_pos(
                    pest::error::ErrorVariant::CustomError {
                        message: "Unknown factor operator".to_string(),
                    },
                    pest::Position::from_start(""),
                ))),
            };

            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(parse_unary(right_pair)?),
            };
        }
    }

    Ok(expr)
}

fn parse_unary(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let mut inner = pair.into_inner().collect::<Vec<_>>();
    let primary = inner.pop().unwrap();

    let mut expr = parse_primary(primary)?;

    // Apply unary operators in reverse order
    for op_pair in inner.into_iter().rev() {
        let op = match op_pair.as_str() {
            "!" => UnaryOp::Not,
            "-" => UnaryOp::Minus,
            _ => return Err(Box::new(pest::error::Error::new_from_pos(
                pest::error::ErrorVariant::CustomError {
                    message: "Unknown unary operator".to_string(),
                },
                pest::Position::from_start(""),
            ))),
        };

        expr = Expr::Unary {
            operator: op,
            operand: Box::new(expr),
        };
    }

    Ok(expr)
}

fn parse_primary(pair: Pair<Rule>) -> Result<Expr, Box<pest::error::Error<Rule>>> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::nil => Ok(Expr::Literal(Value::Nil)),
        Rule::boolean => {
            let value = inner.as_str() == "true";
            Ok(Expr::Literal(Value::Bool(value)))
        }
        Rule::number => {
            let value = inner.as_str().parse::<f64>().unwrap();
            Ok(Expr::Literal(Value::Number(value)))
        }
        Rule::string => {
            let s = inner.as_str();
            let value = s[1..s.len()-1].to_string(); // Remove quotes
            Ok(Expr::Literal(Value::String(value)))
        }
        Rule::identifier => {
            let name = inner.as_str().to_string();
            Ok(Expr::Variable(name))
        }
        Rule::expression => parse_expression(inner),
        _ => Err(Box::new(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Unknown primary expression".to_string(),
            },
            pest::Position::from_start(""),
        ))),
    }
}
