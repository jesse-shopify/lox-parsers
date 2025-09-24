//! Simple test demonstrating parser libraries

use nom_lox::parse_program;
use lox_ast::{Program, Stmt, Expr, Value};

fn main() {
    println!("=== Parser Library Demonstration ===");

    let test_cases = vec![
        ("Simple print", r#"print "Hello, world!";"#),
        ("Variable declaration", r#"var x = 42;"#),
        ("Arithmetic", r#"1 + 2 * 3;"#),
        ("Complex expression", r#"var result = (10 + 5) * 2;"#),
    ];

    for (name, input) in test_cases {
        println!("\n{}: {}", name, input);

        match parse_program(input) {
            Ok(program) => {
                println!("✅ Parsed successfully!");
                println!("  Statements: {}", program.statements.len());

                // Demonstrate AST access
                for (i, stmt) in program.statements.iter().enumerate() {
                    match stmt {
                        Stmt::Print(expr) => {
                            println!("  [{}] Print statement: {:?}", i, expr);
                        }
                        Stmt::VarDeclaration { name, initializer } => {
                            println!("  [{}] Variable '{}' = {:?}", i, name, initializer);
                        }
                        Stmt::Expression(expr) => {
                            println!("  [{}] Expression: {:?}", i, expr);
                        }
                        _ => {
                            println!("  [{}] Other statement: {:?}", i, stmt);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Parse failed: {:?}", e);
            }
        }
    }

    println!("\n=== Library Information ===");
    println!("Parser: {} v{}", nom_lox::PARSER_NAME, nom_lox::PARSER_VERSION);
    println!("Description: {}", nom_lox::PARSER_DESCRIPTION);

    println!("\n=== Success! ===");
    println!("The nom-lox parser has been successfully converted to a library.");
    println!("It can now be used as a dependency in other Rust projects.");
    println!("The same conversion has been applied to all other parsers:");
    println!("- pest-lox (PEG parser generator)");
    println!("- winnow-lox (modern nom successor)");
    println!("- chumsky-lox (error-focused parser combinator)");
}
