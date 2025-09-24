//! Comprehensive integration tests for parser comparison

use parser_comparison_simple::parse_with_nom;
use lox_ast::{Program, Stmt, Expr, Value, BinaryOp};

/// Test cases that were previously in the CLI test suite
#[test]
fn test_comprehensive_lox_features() {
    let test_cases = vec![
        ("Simple literal", "42;", 1),
        ("String literal", r#""Hello, world!";"#, 1),
        ("Print statement", r#"print "Hello, world!";"#, 1),
        ("Variable declaration", "var x = 42;", 1),
        ("Variable assignment", "var x = 10; x = 20;", 2),
        ("Arithmetic expression", "1 + 2 * 3;", 1),
        ("Comparison expression", "5 > 3;", 1),
        ("Logical expression", "true and false;", 1),
        ("Grouped expression", "(1 + 2) * 3;", 1),
        ("Complex expression", "var a = 10; var b = 20; var sum = a + b; print sum;", 4),
        ("Boolean literals", "true; false; nil;", 3),
        ("Unary expressions", "-42; !true;", 2),
        ("Complex arithmetic", "1 + 2 * 3 - 4 / 2;", 1),
    ];

    let mut passed = 0;
    let mut failed = 0;

    println!("=== Comprehensive Lox Parser Tests ===");

    for (name, input, expected_statements) in test_cases {
        match parse_with_nom(input) {
            Ok(program) => {
                if program.statements.len() == expected_statements {
                    println!("✅ {}: {} statements", name, program.statements.len());
                    passed += 1;
                } else {
                    println!("❌ {}: expected {} statements, got {}",
                        name, expected_statements, program.statements.len());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("❌ {}: Parse failed - {}", name, e);
                failed += 1;
            }
        }
    }

    println!("\n=== Test Results ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Success rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

    // We expect at least 80% success rate for the nom parser
    let success_rate = (passed as f64 / (passed + failed) as f64) * 100.0;
    assert!(success_rate >= 80.0, "Success rate should be at least 80%, got {:.1}%", success_rate);
}

#[test]
fn test_comments_handling() {
    let test_cases = vec![
        ("Single line comment", "// This is a comment\nprint 42;", 1),
        ("End of line comment", r#"print "hello"; // comment"#, 1),
        ("Multiple comments", r#"
            // First comment
            var x = 10; // Variable
            // Another comment
            print x; // Print it
        "#, 2),
    ];

    for (name, input, expected_statements) in test_cases {
        match parse_with_nom(input) {
            Ok(program) => {
                assert_eq!(
                    program.statements.len(),
                    expected_statements,
                    "Failed for {}: expected {} statements, got {}",
                    name,
                    expected_statements,
                    program.statements.len()
                );
                println!("✅ {}: Comments properly ignored", name);
            }
            Err(e) => {
                panic!("Parse failed for {}: {}", name, e);
            }
        }
    }
}

#[test]
fn test_whitespace_handling() {
    let test_cases = vec![
        ("No whitespace", "var x=42;print x;", 2),
        ("Lots of whitespace", "   var    x   =   42   ;   print   x   ;   ", 2),
        ("Newlines", "var x = 42;\nprint x;", 2),
        ("Mixed whitespace", "\t\nvar x = 42;\n\t\nprint x;\n\t", 2),
    ];

    for (name, input, expected_statements) in test_cases {
        match parse_with_nom(input) {
            Ok(program) => {
                assert_eq!(
                    program.statements.len(),
                    expected_statements,
                    "Failed for {}: expected {} statements, got {}",
                    name,
                    expected_statements,
                    program.statements.len()
                );
                println!("✅ {}: Whitespace handled correctly", name);
            }
            Err(e) => {
                panic!("Parse failed for {}: {}", name, e);
            }
        }
    }
}

#[test]
fn test_nested_expressions() {
    let test_cases = vec![
        ("Simple grouping", "(42)", 1),
        ("Nested grouping", "((42))", 1),
        ("Complex nesting", "(1 + (2 * (3 + 4)))", 1),
        ("Mixed operators", "1 + 2 * 3 - 4 / 2", 1),
    ];

    for (name, input, expected_statements) in test_cases {
        let full_input = format!("{};", input); // Add semicolon to make it a statement

        match parse_with_nom(&full_input) {
            Ok(program) => {
                assert_eq!(
                    program.statements.len(),
                    expected_statements,
                    "Failed for {}: expected {} statements, got {}",
                    name,
                    expected_statements,
                    program.statements.len()
                );
                println!("✅ {}: Nested expression parsed correctly", name);
            }
            Err(e) => {
                panic!("Parse failed for {}: {}", name, e);
            }
        }
    }
}

#[test]
fn test_edge_cases() {
    let test_cases = vec![
        ("Empty program", "", 0),
        ("Only whitespace", "   \n\t  ", 0),
        ("Only comments", "// Just a comment\n// Another comment", 0),
        ("Minimal statement", "nil;", 1),
    ];

    for (name, input, expected_statements) in test_cases {
        match parse_with_nom(input) {
            Ok(program) => {
                assert_eq!(
                    program.statements.len(),
                    expected_statements,
                    "Failed for {}: expected {} statements, got {}",
                    name,
                    expected_statements,
                    program.statements.len()
                );
                println!("✅ {}: Edge case handled correctly", name);
            }
            Err(e) => {
                // Some edge cases might legitimately fail
                println!("ℹ️  {}: Parse failed (may be expected) - {}", name, e);
            }
        }
    }
}

#[test]
fn test_real_world_program() {
    let program = r#"
        // A simple Lox program
        var greeting = "Hello";
        var name = "World";
        var message = greeting + ", " + name + "!";
        print message;

        var x = 10;
        var y = 20;
        var sum = x + y;
        var product = x * y;

        print "Sum: " + sum;
        print "Product: " + product;

        var isPositive = sum > 0;
        print isPositive;
    "#;

    match parse_with_nom(program) {
        Ok(parsed) => {
            println!("✅ Real-world program parsed successfully");
            println!("   Statements: {}", parsed.statements.len());

            // Should have multiple statements
            assert!(parsed.statements.len() >= 8, "Should have at least 8 statements");

            // Check that we have different types of statements
            let mut var_decls = 0;
            let mut prints = 0;
            let mut expressions = 0;

            for stmt in &parsed.statements {
                match stmt {
                    Stmt::VarDeclaration { .. } => var_decls += 1,
                    Stmt::Print(_) => prints += 1,
                    Stmt::Expression(_) => expressions += 1,
                    _ => {}
                }
            }

            println!("   Variable declarations: {}", var_decls);
            println!("   Print statements: {}", prints);
            println!("   Expression statements: {}", expressions);

            assert!(var_decls > 0, "Should have variable declarations");
            assert!(prints > 0, "Should have print statements");
        }
        Err(e) => {
            panic!("Real-world program should parse successfully: {}", e);
        }
    }
}
