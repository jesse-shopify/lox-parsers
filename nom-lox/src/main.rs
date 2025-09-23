//! Nom-based Lox parser implementation
//!
//! This crate implements a parser for the Lox programming language using the nom parser combinator library.
//! Lox specification: https://craftinginterpreters.com/the-lox-language.html

mod parser;

use std::env;
use std::fs;
use std::io::{self, Write};

use parser::parse_program;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_repl(),
        2 => run_file(&args[1]),
        _ => {
            eprintln!("Usage: nom-lox [script]");
            std::process::exit(64);
        }
    }
}

fn run_repl() {
    println!("Nom-based Lox Parser REPL");
    println!("Type 'exit' to quit.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();
                if input == "exit" {
                    break;
                }

                if input.is_empty() {
                    continue;
                }

                match parse_program(input) {
                    Ok(program) => {
                        println!("Parsed successfully:");
                        println!("{:#?}", program);
                    }
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(content) => {
            match parse_program(&content) {
                Ok(program) => {
                    println!("Successfully parsed file: {}", path);
                    println!("{:#?}", program);
                }
                Err(e) => {
                    eprintln!("Parse error in {}: {}", path, e);
                    std::process::exit(65);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", path, e);
            std::process::exit(66);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lox_ast::{Expr, Stmt, Value, BinaryOp};

    #[test]
    fn test_simple_expression() {
        let input = "1 + 2;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Expression(Expr::Binary { left, operator, right }) => {
                assert!(matches!(**left, Expr::Literal(Value::Number(1.0))));
                assert_eq!(*operator, BinaryOp::Add);
                assert!(matches!(**right, Expr::Literal(Value::Number(2.0))));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_print_statement() {
        let input = r#"print "Hello, world!";"#;
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::Print(Expr::Literal(Value::String(s))) => {
                assert_eq!(s, "Hello, world!");
            }
            _ => panic!("Expected print statement with string literal"),
        }
    }

    #[test]
    fn test_variable_declaration() {
        let input = "var x = 42;";
        let result = parse_program(input).unwrap();

        assert_eq!(result.statements.len(), 1);
        match &result.statements[0] {
            Stmt::VarDeclaration { name, initializer } => {
                assert_eq!(name, "x");
                assert!(matches!(
                    initializer,
                    Some(Expr::Literal(Value::Number(42.0)))
                ));
            }
            _ => panic!("Expected variable declaration"),
        }
    }
}
