// Simplified AST for lesson_4 diagnostic system

use super::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    String,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
        span: Span,
    },
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier(String, Span),
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Number(_, span) => span,
            Expr::Boolean(_, span) => span,
            Expr::String(_, span) => span,
            Expr::Identifier(_, span) => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
