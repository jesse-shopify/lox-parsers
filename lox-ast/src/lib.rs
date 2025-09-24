//! Abstract Syntax Tree definitions for the Lox programming language
//!
//! This crate provides the core AST types used by various Lox parser implementations.
//! Based on the specification at https://craftinginterpreters.com/the-lox-language.html

use std::fmt;
use serde::{Deserialize, Serialize};

/// Represents a Lox value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Binary operators in Lox
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,

    // Comparison
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,

    // Logical
    And,
    Or,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        };
        write!(f, "{}", op_str)
    }
}

/// Unary operators in Lox
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Minus,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            UnaryOp::Minus => "-",
            UnaryOp::Not => "!",
        };
        write!(f, "{}", op_str)
    }
}

/// Lox expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal values
    Literal(Value),

    /// Variable reference
    Variable(String),

    /// Binary operations
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },

    /// Unary operations
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },

    /// Grouping (parentheses)
    Grouping(Box<Expr>),

    /// Assignment
    Assignment {
        name: String,
        value: Box<Expr>,
    },

    /// Function call
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },

    /// Property access
    Get {
        object: Box<Expr>,
        name: String,
    },

    /// Property assignment
    Set {
        object: Box<Expr>,
        name: String,
        value: Box<Expr>,
    },

    /// This expression
    This,

    /// Super expression
    Super {
        method: String,
    },
}

/// Lox statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Expression statement
    Expression(Expr),

    /// Print statement
    Print(Expr),

    /// Variable declaration
    VarDeclaration {
        name: String,
        initializer: Option<Expr>,
    },

    /// Block statement
    Block(Vec<Stmt>),

    /// If statement
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    /// While loop
    While {
        condition: Expr,
        body: Box<Stmt>,
    },

    /// For loop (desugared to while in some implementations)
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },

    /// Function declaration
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    /// Return statement
    Return {
        value: Option<Expr>,
    },

    /// Class declaration
    Class {
        name: String,
        superclass: Option<String>,
        methods: Vec<Stmt>, // Should be Function statements
    },
}

/// A complete Lox program
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    /// Create a new Lox program with the given statements
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements }
    }

    /// Get a reference to the statements in this program
    pub fn statements(&self) -> &[Stmt] {
        &self.statements
    }

    /// Get a mutable reference to the statements in this program
    pub fn statements_mut(&mut self) -> &mut Vec<Stmt> {
        &mut self.statements
    }

    /// Add a statement to this program
    pub fn add_statement(&mut self, stmt: Stmt) {
        self.statements.push(stmt);
    }

    /// Check if the program is empty
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Program with {} statements:", self.statements.len())?;
        for (i, stmt) in self.statements.iter().enumerate() {
            writeln!(f, "  {}: {:?}", i + 1, stmt)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Nil.to_string(), "nil");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Number(42.5).to_string(), "42.5");
        assert_eq!(Value::String("hello".to_string()).to_string(), "\"hello\"");
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(BinaryOp::Add.to_string(), "+");
        assert_eq!(BinaryOp::Equal.to_string(), "==");
        assert_eq!(BinaryOp::And.to_string(), "and");
    }

    #[test]
    fn test_unary_op_display() {
        assert_eq!(UnaryOp::Minus.to_string(), "-");
        assert_eq!(UnaryOp::Not.to_string(), "!");
    }

    #[test]
    fn test_program_creation() {
        let stmt = Stmt::Print(Expr::Literal(Value::String("test".to_string())));
        let program = Program::new(vec![stmt]);

        assert_eq!(program.statements().len(), 1);
        assert!(!program.is_empty());
    }

    #[test]
    fn test_program_mutation() {
        let mut program = Program::new(vec![]);
        assert!(program.is_empty());

        program.add_statement(Stmt::Print(Expr::Literal(Value::Nil)));
        assert_eq!(program.statements().len(), 1);
        assert!(!program.is_empty());
    }

    #[test]
    fn test_expr_equality() {
        let expr1 = Expr::Binary {
            left: Box::new(Expr::Literal(Value::Number(1.0))),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Literal(Value::Number(2.0))),
        };

        let expr2 = Expr::Binary {
            left: Box::new(Expr::Literal(Value::Number(1.0))),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Literal(Value::Number(2.0))),
        };

        assert_eq!(expr1, expr2);
    }
}