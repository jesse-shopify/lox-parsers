use lox_ast::{Program, Stmt, Expr, Value, BinaryOp, UnaryOp};

// Define the Token enum that Lelwel expects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    // Literals and keywords
    TRUE, FALSE, NIL, PRINT, VAR, AND, OR,
    // Operators
    PLUS, MINUS, STAR, SLASH, BANG, BANG_EQUAL, EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL, LESS, LESS_EQUAL,
    // Delimiters
    LEFT_PAREN, RIGHT_PAREN, SEMICOLON,
    // Complex tokens
    NUMBER, STRING, IDENTIFIER,
    // Special
    Error, EOF,
}

pub type Diagnostic = String;
// Span is defined in the generated code

#[derive(Default)]
pub struct Context<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

// Simple lexer
fn lex(source: &str, _diags: &mut Vec<Diagnostic>) -> (Vec<Token>, Vec<Span>) {
    let mut tokens = Vec::new();
    let mut spans = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => continue,
            '(' => { tokens.push(Token::LEFT_PAREN); spans.push(start..start+1); }
            ')' => { tokens.push(Token::RIGHT_PAREN); spans.push(start..start+1); }
            ';' => { tokens.push(Token::SEMICOLON); spans.push(start..start+1); }
            '+' => { tokens.push(Token::PLUS); spans.push(start..start+1); }
            '-' => { tokens.push(Token::MINUS); spans.push(start..start+1); }
            '*' => { tokens.push(Token::STAR); spans.push(start..start+1); }
            '/' => {
                if chars.peek() == Some(&(start + 1, '/')) {
                    // Skip comment
                    while let Some((_, ch)) = chars.next() {
                        if ch == '\n' { break; }
                    }
                    continue;
                } else {
                    tokens.push(Token::SLASH); spans.push(start..start+1);
                }
            }
            '!' => {
                if chars.peek() == Some(&(start + 1, '=')) {
                    chars.next();
                    tokens.push(Token::BANG_EQUAL); spans.push(start..start+2);
                } else {
                    tokens.push(Token::BANG); spans.push(start..start+1);
                }
            }
            '=' => {
                if chars.peek() == Some(&(start + 1, '=')) {
                    chars.next();
                    tokens.push(Token::EQUAL_EQUAL); spans.push(start..start+2);
                } else {
                    tokens.push(Token::EQUAL); spans.push(start..start+1);
                }
            }
            '>' => {
                if chars.peek() == Some(&(start + 1, '=')) {
                    chars.next();
                    tokens.push(Token::GREATER_EQUAL); spans.push(start..start+2);
                } else {
                    tokens.push(Token::GREATER); spans.push(start..start+1);
                }
            }
            '<' => {
                if chars.peek() == Some(&(start + 1, '=')) {
                    chars.next();
                    tokens.push(Token::LESS_EQUAL); spans.push(start..start+2);
                } else {
                    tokens.push(Token::LESS); spans.push(start..start+1);
                }
            }
            '"' => {
                let mut end = start + 1;
                while let Some((pos, ch)) = chars.next() {
                    end = pos + ch.len_utf8();
                    if ch == '"' { break; }
                }
                tokens.push(Token::STRING); spans.push(start..end);
            }
            c if c.is_ascii_digit() => {
                let mut end = start + c.len_utf8();
                while let Some(&(pos, ch)) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        end = pos + ch.len_utf8();
                        chars.next();
                    } else { break; }
                }
                tokens.push(Token::NUMBER); spans.push(start..end);
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut end = start + c.len_utf8();
                while let Some(&(pos, ch)) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        end = pos + ch.len_utf8();
                        chars.next();
                    } else { break; }
                }
                let text = &source[start..end];
                let token = match text {
                    "true" => Token::TRUE, "false" => Token::FALSE, "nil" => Token::NIL,
                    "print" => Token::PRINT, "var" => Token::VAR, "and" => Token::AND, "or" => Token::OR,
                    _ => Token::IDENTIFIER,
                };
                tokens.push(token); spans.push(start..end);
            }
            _ => { tokens.push(Token::Error); spans.push(start..start+ch.len_utf8()); }
        }
    }

    tokens.push(Token::EOF);
    spans.push(source.len()..source.len());
    (tokens, spans)
}

// Include generated parser
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

// Implement required trait
impl<'a> ParserCallbacks for Parser<'a> {
    fn create_tokens(source: &str, diags: &mut Vec<Diagnostic>) -> (Vec<Token>, Vec<Span>) {
        lex(source, diags)
    }
    fn create_diagnostic(&self, _span: Span, message: String) -> Diagnostic { message }
}

// Simple AST conversion - focusing on basic cases first
fn convert_cst(cst: &Cst, node: NodeRef, source: &str) -> Result<Program, String> {
    let mut statements = Vec::new();

    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Statement) {
            if let Ok(stmt) = convert_statement(cst, child, source) {
                statements.push(stmt);
            }
        }
    }

    Ok(Program::new(statements))
}

fn convert_statement(cst: &Cst, node: NodeRef, source: &str) -> Result<Stmt, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::PrintStmt) {
            for grandchild in cst.children(child) {
                if cst.match_rule(grandchild, Rule::Expression) {
                    let expr = convert_expression(cst, grandchild, source)?;
                    return Ok(Stmt::Print(expr));
                }
            }
        } else if cst.match_rule(child, Rule::VarDeclaration) {
            return convert_var_declaration(cst, child, source);
        } else if cst.match_rule(child, Rule::ExpressionStmt) {
            for grandchild in cst.children(child) {
                if cst.match_rule(grandchild, Rule::Expression) {
                    let expr = convert_expression(cst, grandchild, source)?;
                    return Ok(Stmt::Expression(expr));
                }
            }
        }
    }
    Err("Unknown statement".to_string())
}

fn convert_var_declaration(cst: &Cst, node: NodeRef, source: &str) -> Result<Stmt, String> {
    let mut name = None;
    let mut initializer = None;

    for child in cst.children(node) {
        if let Some((text, _)) = cst.match_token(child, Token::IDENTIFIER) {
            name = Some(text.to_string());
        } else if cst.match_rule(child, Rule::Expression) {
            initializer = Some(convert_expression(cst, child, source)?);
        }
    }

    Ok(Stmt::VarDeclaration {
        name: name.ok_or("Missing variable name")?,
        initializer
    })
}

fn convert_expression(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Assignment) {
            return convert_assignment(cst, child, source);
        }
    }
    Err("No expression found".to_string())
}

fn convert_assignment(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::LogicalOr) {
            return convert_logical_or(cst, child, source);
        }
    }
    Err("Invalid assignment".to_string())
}

fn convert_logical_or(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::LogicalAnd) {
            return convert_logical_and(cst, child, source);
        }
    }
    Err("Invalid logical or".to_string())
}

fn convert_logical_and(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Equality) {
            return convert_equality(cst, child, source);
        }
    }
    Err("Invalid logical and".to_string())
}

fn convert_equality(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Comparison) {
            return convert_comparison(cst, child, source);
        }
    }
    Err("Invalid equality".to_string())
}

fn convert_comparison(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Term) {
            return convert_term(cst, child, source);
        }
    }
    Err("Invalid comparison".to_string())
}

fn convert_term(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    let children: Vec<_> = cst.children(node).collect();

    // Check for binary operation: factor [+ term]
    if children.len() == 3 {
        let left = convert_factor(cst, children[0], source)?;
        let right = convert_term(cst, children[2], source)?;
        let op = if cst.match_token(children[1], Token::PLUS).is_some() {
            BinaryOp::Add
        } else {
            BinaryOp::Subtract
        };
        return Ok(Expr::Binary { left: Box::new(left), operator: op, right: Box::new(right) });
    }

    // Single factor
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Factor) {
            return convert_factor(cst, child, source);
        }
    }
    Err("Invalid term".to_string())
}

fn convert_factor(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    let children: Vec<_> = cst.children(node).collect();

    // Check for binary operation: unary [* factor]
    if children.len() == 3 {
        let left = convert_unary(cst, children[0], source)?;
        let right = convert_factor(cst, children[2], source)?;
        let op = if cst.match_token(children[1], Token::STAR).is_some() {
            BinaryOp::Multiply
        } else {
            BinaryOp::Divide
        };
        return Ok(Expr::Binary { left: Box::new(left), operator: op, right: Box::new(right) });
    }

    // Single unary
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Unary) {
            return convert_unary(cst, child, source);
        }
    }
    Err("Invalid factor".to_string())
}

fn convert_unary(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    let children: Vec<_> = cst.children(node).collect();

    // Check for unary operation: (! | -) unary
    if children.len() == 2 {
        let op = if cst.match_token(children[0], Token::BANG).is_some() {
            UnaryOp::Not
        } else {
            UnaryOp::Minus
        };
        let operand = convert_unary(cst, children[1], source)?;
        return Ok(Expr::Unary { operator: op, operand: Box::new(operand) });
    }

    // Single primary
    for child in cst.children(node) {
        if cst.match_rule(child, Rule::Primary) {
            return convert_primary(cst, child, source);
        }
    }
    Err("Invalid unary".to_string())
}

fn convert_primary(cst: &Cst, node: NodeRef, source: &str) -> Result<Expr, String> {
    for child in cst.children(node) {
        if cst.match_token(child, Token::TRUE).is_some() {
            return Ok(Expr::Literal(Value::Bool(true)));
        } else if cst.match_token(child, Token::FALSE).is_some() {
            return Ok(Expr::Literal(Value::Bool(false)));
        } else if cst.match_token(child, Token::NIL).is_some() {
            return Ok(Expr::Literal(Value::Nil));
        } else if let Some((text, _)) = cst.match_token(child, Token::NUMBER) {
            let num: f64 = text.parse().map_err(|_| "Invalid number")?;
            return Ok(Expr::Literal(Value::Number(num)));
        } else if let Some((text, _)) = cst.match_token(child, Token::STRING) {
            let content = &text[1..text.len()-1];
            return Ok(Expr::Literal(Value::String(content.to_string())));
        } else if let Some((text, _)) = cst.match_token(child, Token::IDENTIFIER) {
            return Ok(Expr::Variable(text.to_string()));
        } else if cst.match_rule(child, Rule::Expression) {
            return convert_expression(cst, child, source);
        }
    }
    Err("Invalid primary".to_string())
}

/// Parse a complete Lox program from a string
pub fn parse_program(input: &str) -> Result<Program, String> {
    let mut diags = Vec::new();
    let cst = Parser::parse(input, &mut diags);

    if !diags.is_empty() {
        return Err(format!("Parse errors: {:?}", diags));
    }

    convert_cst(&cst, NodeRef(0), input)
}