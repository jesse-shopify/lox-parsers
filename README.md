# Lox Parser Implementations

This workspace contains parser implementations for the [Lox programming language](https://craftinginterpreters.com/the-lox-language.html) as specified in Robert Nystrom's "Crafting Interpreters" book.

## Crates

### lox-ast ✅
A shared library crate containing the Abstract Syntax Tree definitions for the Lox programming language. This crate is used by all parser implementations to ensure consistency.

**Features**:
- Complete AST node definitions for all Lox language constructs
- Display implementations for pretty-printing
- Comprehensive test coverage
- Well-documented API

### nom-lox ✅
A complete parser implementation using the [nom](https://github.com/Geal/nom) parser combinator library. Nom is known for its zero-copy parsing approach and excellent performance.

**Status**: ✅ Complete and working
- Handles all basic Lox language constructs
- Uses shared `lox-ast` crate
- Passes all tests
- Includes REPL and file parsing modes
- Correct operator precedence and associativity

### pest-lox ⚠️
A parser implementation using the [pest](https://github.com/pest-parser/pest) PEG parser generator. Pest uses grammar files to generate parsers.

**Status**: ⚠️ Builds but has runtime issues
- Grammar file defined in `lox.pest`
- Uses shared `lox-ast` crate
- Builds successfully but crashes on parsing
- Needs debugging for proper operation

### chumsky-lox ❌
A parser implementation using the [chumsky](https://github.com/zesterer/chumsky) parser combinator library.

**Status**: ❌ Linker errors
- Hits macOS linker assertion failures
- Complex recursive type inference causes compiler issues
- Implementation complete but non-functional

### winnow-lox ❌
A parser implementation using the [winnow](https://github.com/winnow-rs/winnow) parser combinator library (successor to nom).

**Status**: ❌ API compatibility issues
- Uses deprecated winnow APIs
- Type annotation issues with new winnow version
- Needs updates for current winnow API

### lalrpop-lox 🚧
A parser implementation using the [LALRPOP](https://github.com/lalrpop/lalrpop) LR(1) parser generator.

**Status**: 🚧 Placeholder
- Directory structure created
- Implementation pending

### pom-lox 🚧
A parser implementation using the [pom](https://github.com/J-F-Liu/pom) parser combinator library.

**Status**: 🚧 Placeholder
- Directory structure created
- Implementation pending

## Language Features Supported

The nom-lox implementation currently supports:

- **Literals**: Numbers, strings, booleans, and nil
- **Expressions**: Arithmetic, comparison, equality, logical operations with correct precedence
- **Variables**: Declaration and assignment
- **Statements**: Expression statements and print statements
- **Comments**: Line comments starting with `//`
- **Grouping**: Parenthesized expressions
- **Operators**: `+`, `-`, `*`, `/`, `>`, `>=`, `<`, `<=`, `==`, `!=`, `and`, `or`, `!`, unary `-`

## Building and Running

### Build the parser
```bash
cargo build
```

### Run the nom-based parser
```bash
# Interactive REPL
cargo run --bin nom-lox

# Parse a file
cargo run --bin nom-lox examples/hello.lox
```

### Run the pest-based parser
```bash
# Interactive REPL
cargo run --bin pest-lox

# Parse a file
cargo run --bin pest-lox examples/hello.lox
```

### Run other parsers
```bash
# winnow-lox (currently not working - API issues)
cargo run --bin winnow-lox

# chumsky-lox (currently not working - linker errors)
cargo run --bin chumsky-lox

# lalrpop-lox and pom-lox (placeholders)
cargo run --bin lalrpop-lox
cargo run --bin pom-lox
```

### Run tests
```bash
cargo test
```

## Example Programs

The `examples/` directory contains sample Lox programs:

- `hello.lox` - Simple hello world program
- `arithmetic.lox` - Complex arithmetic expressions with variables
- `variables.lox` - Variable declarations, assignments, and different data types

### Example Usage

```bash
$ cargo run --bin nom-lox examples/arithmetic.lox
Successfully parsed file: examples/arithmetic.lox
Program {
    statements: [
        VarDeclaration { name: "a", initializer: Some(Literal(Number(10.0))) },
        VarDeclaration { name: "b", initializer: Some(Literal(Number(20.0))) },
        VarDeclaration { name: "sum", initializer: Some(Binary { ... }) },
        Print(Variable("sum")),
        ...
    ],
}
```

## Parser Details

### nom-lox Implementation
- **Performance**: Excellent (zero-copy parsing)
- **Error Messages**: Basic but functional
- **API Complexity**: Medium (typical for nom)
- **Memory Usage**: Low
- **Operator Precedence**: Correctly implemented following Lox specification
- **Features**: Complete basic Lox support with proper AST generation

## Architecture

The project is organized into multiple crates:

### lox-ast
- `lib.rs` - Complete AST definitions with Display implementations and utility methods
- Tests for all AST node types and operations

### nom-lox
- `parser.rs` - nom-based parser combinators for each language construct
- `main.rs` - CLI interface with REPL and file parsing modes
- Uses `lox-ast` crate for AST types

## Future Enhancements

The parser can be extended to support additional Lox language features:

- **Control Flow**: `if`/`else` statements, `while` and `for` loops
- **Functions**: Function declarations, calls, and closures
- **Classes**: Class declarations, methods, inheritance
- **Block Statements**: Scoped variable declarations
- **Better Error Recovery**: More descriptive error messages and recovery
- **Additional Operators**: Ternary operator, string concatenation

## Adding New Parser Implementations

To add a new parser implementation (e.g., using a different parsing library):

1. Create a new crate in the workspace: `new-parser-lox/`
2. Add it to the workspace members in `Cargo.toml`
3. Add `lox-ast = { workspace = true }` to the new crate's dependencies
4. Import the AST types: `use lox_ast::{Expr, Stmt, Program, Value, BinaryOp, UnaryOp};`
5. Implement the parser using your chosen library
6. Add tests and examples

The shared AST crate ensures all parser implementations use consistent data structures.

## Contributing

Feel free to:
- Extend the nom implementation with additional Lox language features
- Create new parser implementations using different libraries (pest, lalrpop, etc.)
- Improve error messages and recovery
- Add more comprehensive tests and examples

The foundation is solid and ready for extension!