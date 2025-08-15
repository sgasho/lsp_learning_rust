// Lesson 4-1: 未使用変数検出
// rust-analyzerで最も貢献しやすい診断機能の基本

// あなたのタスク：
// 未使用変数を検出する診断システムを実装してください。

use super::common::{
    ast::{Expr, Program, Stmt},
    diagnostic::{Diagnostic, DiagnosticCategory},
    span::Span,
};

// シンボル情報（使用状況追跡）
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub definition_span: Span,
    pub is_used: bool,
}

impl Symbol {
    pub fn new(name: String, definition_span: Span) -> Self {
        Symbol {
            name,
            definition_span,
            is_used: false,
        }
    }
}

// 未使用変数検出器
#[derive(Debug)]
pub struct UnusedVariableChecker {
    symbols: HashMap<String, Symbol>,
    diagnostics: Vec<Diagnostic>,
}

impl UnusedVariableChecker {
    pub fn new() -> Self {
        UnusedVariableChecker {
            symbols: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        self.symbols.clear();
        self.diagnostics.clear();

        // Phase 1: 変数定義を収集
        self.collect_definitions(program);

        /// comment
        /// rearea
        use std::collections::HashMap as Map;
        use std::collections::HashMap;

        // Phase 2: 変数使用を追跡
        self.track_usage(program);

        // Phase 3: 未使用変数の診断を生成
        self.generate_unused_diagnostics();

        self.diagnostics.clone()
    }

    // Phase 1: 変数定義の収集
    fn collect_definitions(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.collect_definitions_from_stmt(stmt);
        }
    }

    fn collect_definitions_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LetDeclaration { name, span, .. } => {
                let symbol = Symbol::new(name.clone(), span.clone());
                self.symbols.insert(name.clone(), symbol);
            }
            Stmt::Expression(_) => {
                // 式文では新しい変数定義は発生しない
            }
        }
    }

    // Phase 2: 変数使用の追跡
    fn track_usage(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.track_usage_in_stmt(stmt);
        }
    }

    fn track_usage_in_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LetDeclaration { value, .. } => {
                self.track_usage_in_expr(value);
            }
            Stmt::Expression(expr) => {
                self.track_usage_in_expr(expr);
            }
        }
    }

    fn track_usage_in_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(name, _) => {
                if let Some(symbol) = self.symbols.get_mut(name) {
                    symbol.is_used = true;
                }
            }
            Expr::Number(_, _) | Expr::Boolean(_, _) | Expr::String(_, _) => {
                // リテラルは変数使用ではない
            }
        }
    }

    // Phase 3: 未使用変数の診断生成
    fn generate_unused_diagnostics(&mut self) {
        // todo!("未使用変数の診断を実装してください")
        // ヒント：
        // 1. self.symbols をイテレート
        // 2. is_used が false のシンボルを見つける
        // 3. 未使用変数の警告を生成
        // 4. self.diagnostics に追加
        self.symbols.iter().for_each(|(name, symbol)| {
            if !symbol.is_used {
                self.diagnostics.push(Diagnostic::warning(
                    DiagnosticCategory::UnusedVariable,
                    format!("unused_var `{}`", name),
                    symbol.clone().definition_span,
                ))
            }
        });
    }
}

// 公開API
pub fn check_unused_variables(program: &Program) -> Vec<Diagnostic> {
    let mut checker = UnusedVariableChecker::new();
    checker.check(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lessons::lesson_4::common::span::Position;

    #[test]
    fn test_unused_variable_detection() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "unused_var".to_string(),
                    value: Expr::Number(42, Span::single(Position::new(0, 18))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 14)),
                },
                Stmt::LetDeclaration {
                    name: "used_var1".to_string(),
                    value: Expr::Number(10, Span::single(Position::new(1, 16))),
                    span: Span::new(Position::new(1, 4), Position::new(1, 12)),
                },
                Stmt::Expression(Expr::Identifier(
                    "used_var1".to_string(),
                    Span::single(Position::new(2, 0)),
                )),
            ],
        };

        let diagnostics = check_unused_variables(&program);

        // unused_var は警告されるべき
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.category, DiagnosticCategory::UnusedVariable)
                && d.message.contains("unused_var")));

        // used_var は警告されるべきではない
        assert!(!diagnostics
            .iter()
            .any(|d| matches!(d.category, DiagnosticCategory::UnusedVariable)
                && d.message.contains("used_var1")));
    }

    #[test]
    fn test_all_variables_used() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "var1".to_string(),
                    value: Expr::Number(1, Span::single(Position::new(0, 11))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 8)),
                },
                Stmt::LetDeclaration {
                    name: "var2".to_string(),
                    value: Expr::Identifier("var1".to_string(), Span::single(Position::new(1, 11))),
                    span: Span::new(Position::new(1, 4), Position::new(1, 8)),
                },
                Stmt::Expression(Expr::Identifier(
                    "var2".to_string(),
                    Span::single(Position::new(2, 0)),
                )),
            ],
        };

        let diagnostics = check_unused_variables(&program);

        // 全ての変数が使用されているので、診断は空であるべき
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_multiple_unused_variables() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "unused1".to_string(),
                    value: Expr::Number(1, Span::single(Position::new(0, 14))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 11)),
                },
                Stmt::LetDeclaration {
                    name: "unused2".to_string(),
                    value: Expr::Number(2, Span::single(Position::new(1, 14))),
                    span: Span::new(Position::new(1, 4), Position::new(1, 11)),
                },
            ],
        };

        let diagnostics = check_unused_variables(&program);

        // 2つの未使用変数が検出されるべき
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|d| d.message.contains("unused1")));
        assert!(diagnostics.iter().any(|d| d.message.contains("unused2")));
    }
}
