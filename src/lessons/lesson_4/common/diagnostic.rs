// Diagnostic system for lesson_4

use super::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticCategory {
    UnusedVariable,
    UnusedImport,
    TypeError,
    NamingConvention,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    pub message: String,
    pub span: Span,
    pub code: Option<String>,
}

impl Diagnostic {
    pub fn warning(category: DiagnosticCategory, message: String, span: Span) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::Warning,
            category,
            message,
            span,
            code: None,
        }
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }
}