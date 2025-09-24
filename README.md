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

**Status**: ✅ Complete and working (library-only)
- Handles all basic Lox language constructs
- Uses shared `lox-ast` crate
- All tests passing (3 test cases)
- Library-only implementation for use as dependency
- Correct operator precedence and associativity

### pest-lox ⚠️
A parser implementation using the [pest](https://github.com/pest-parser/pest) PEG parser generator. Pest uses grammar files to generate parsers.

**Status**: ⚠️ Library-only with runtime issues
- Grammar file defined in `lox.pest`
- Uses shared `lox-ast` crate
- Compiles successfully with warnings
- Runtime crashes during parsing (Option::unwrap() on None)
- All 3 tests fail due to parser implementation bugs

### chumsky-lox ⚠️
A parser implementation using the [chumsky](https://github.com/zesterer/chumsky) parser combinator library.

**Status**: ⚠️ Library-only (linker issues)
- Library compiles successfully
- Hits macOS linker assertion failures during test execution
- Implementation complete but tests non-functional due to linker bugs
- Available as library dependency for other projects

### winnow-lox ❌
A parser implementation using the [winnow](https://github.com/winnow-rs/winnow) parser combinator library (successor to nom).

**Status**: ❌ Library-only with runtime issues
- Uses winnow 0.6 API
- Library compiles successfully with warnings
- All 3 tests fail due to `repeat` parser assertion failures
- Runtime panics with "repeat parsers must always consume" error
- Needs significant debugging for winnow API compatibility

### lalrpop-lox ✅
A parser implementation using the [LALRPOP](https://github.com/lalrpop/lalrpop) LR(1) parser generator.

**Status**: ✅ Complete and working
- Grammar-based parser specification in `lox.lalrpop`
- Generated LR(1) parser with excellent performance
- Uses shared `lox-ast` crate
- All tests passing (11 test cases)
- Comprehensive error handling with detailed messages

### pom-lox ✅
A parser implementation using the [pom](https://github.com/J-F-Liu/pom) parser combinator library.

**Status**: ✅ Complete and working (library-only)
- Handles basic Lox language constructs
- Uses shared `lox-ast` crate
- All 5 tests passing
- Library-only implementation for use as dependency
- Supports arithmetic expressions, variables, print statements
- Clean and simple pom combinator implementation

## Language Features Supported

The nom-lox (library), lalrpop-lox (library), and pom-lox (library) implementations currently support:

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

# nom-lox is now library-only (no binary)
# Use as a dependency in other projects

# pest-lox is now library-only (no binary)
# Use as a dependency in other projects
# Note: currently has runtime issues

### Run other parsers
```bash
# winnow-lox is now library-only (no binary)
# Use as a dependency in other projects
# Note: currently has runtime issues with repeat parsers

# chumsky-lox (library-only, no binary due to linker issues)

# pom-lox (placeholder)
cargo run --bin pom-lox

# lalrpop-lox (library-only, no binary - use as dependency)
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


# Notes

- pom - 140 (u8, concise but syntax is cryptic due to operators)
- chumsky - 250 (clunky/verbose return types, return impl)
- winnow - 320
- nom - 340
- pest - 340
- combine - 370 (return impl)
- lalrpop - (grammar file)