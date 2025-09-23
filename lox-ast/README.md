# lox-ast

Abstract Syntax Tree definitions for the Lox programming language.

This crate provides the core AST types used by various Lox parser implementations, ensuring consistency across different parsing approaches.

## Features

- **Complete AST Coverage**: Supports all Lox language constructs including expressions, statements, and declarations
- **Display Implementations**: Pretty-printing support for all AST nodes
- **Utility Methods**: Convenient methods for working with programs and AST nodes
- **Well-Tested**: Comprehensive test coverage for all functionality
- **Zero Dependencies**: Pure Rust implementation with no external dependencies

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
lox-ast = { path = "../lox-ast" }
```

## Example

```rust
use lox_ast::{Program, Stmt, Expr, Value, BinaryOp};

// Create a simple expression: 1 + 2
let expr = Expr::Binary {
    left: Box::new(Expr::Literal(Value::Number(1.0))),
    operator: BinaryOp::Add,
    right: Box::new(Expr::Literal(Value::Number(2.0))),
};

// Create a print statement
let stmt = Stmt::Print(expr);

// Create a program
let program = Program::new(vec![stmt]);

println!("{}", program);
```

## AST Node Types

### Values
- `Value::Nil` - The nil value
- `Value::Bool(bool)` - Boolean values
- `Value::Number(f64)` - Numeric values
- `Value::String(String)` - String values

### Expressions
- `Expr::Literal(Value)` - Literal values
- `Expr::Variable(String)` - Variable references
- `Expr::Binary { left, operator, right }` - Binary operations
- `Expr::Unary { operator, operand }` - Unary operations
- `Expr::Grouping(Box<Expr>)` - Parenthesized expressions
- `Expr::Assignment { name, value }` - Variable assignments
- And more for function calls, property access, etc.

### Statements
- `Stmt::Expression(Expr)` - Expression statements
- `Stmt::Print(Expr)` - Print statements
- `Stmt::VarDeclaration { name, initializer }` - Variable declarations
- `Stmt::Block(Vec<Stmt>)` - Block statements
- `Stmt::If { condition, then_branch, else_branch }` - Conditional statements
- And more for loops, functions, classes, etc.

### Operators
- `BinaryOp`: Arithmetic (`+`, `-`, `*`, `/`), comparison (`>`, `>=`, `<`, `<=`, `==`, `!=`), logical (`and`, `or`)
- `UnaryOp`: Negation (`-`), logical not (`!`)

## Design Philosophy

This crate follows these principles:

1. **Completeness**: Support for all Lox language constructs, including future extensions
2. **Consistency**: All parser implementations use the same AST representation
3. **Usability**: Clean API with helpful utility methods and Display implementations
4. **Performance**: Efficient data structures with minimal overhead
5. **Maintainability**: Well-documented and thoroughly tested code

## License

This crate is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.
