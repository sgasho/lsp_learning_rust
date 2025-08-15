// Lesson 4-3: 未使用関数検出
// rust-analyzerの実用診断機能の第3弾

// あなたのタスク：
// 未使用関数を検出する診断システムを実装してください。

use super::common::{
    diagnostic::{Diagnostic, DiagnosticCategory},
    span::Span,
};
use std::collections::HashMap;

// 拡張された式（関数呼び出しを含む）
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier(String, Span),
    FunctionCall {
        name: String,
        arguments: Vec<ExtendedExpr>,
        span: Span,
    },
}

impl ExtendedExpr {
    pub fn span(&self) -> &Span {
        match self {
            ExtendedExpr::Number(_, span) => span,
            ExtendedExpr::Boolean(_, span) => span,
            ExtendedExpr::String(_, span) => span,
            ExtendedExpr::Identifier(_, span) => span,
            ExtendedExpr::FunctionCall { span, .. } => span,
        }
    }
}

// 拡張された文（関数定義を含む）
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedStmt {
    LetDeclaration {
        name: String,
        value: ExtendedExpr,
        span: Span,
    },
    Expression(ExtendedExpr),
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<ExtendedStmt>,
        span: Span,
    },
}

// 拡張されたプログラム（関数定義を含む）
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedProgram {
    pub statements: Vec<ExtendedStmt>,
}

// 関数情報（使用状況追跡）
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub definition_span: Span,
    pub is_used: bool,
    pub is_main: bool, // main関数は常に使用済みとみなす
}

impl FunctionInfo {
    pub fn new(name: String, definition_span: Span) -> Self {
        let is_main = name == "main";
        FunctionInfo {
            name,
            definition_span,
            is_used: is_main, // main関数は最初から使用済み
            is_main,
        }
    }
}

// 未使用関数検出器
#[derive(Debug)]
pub struct UnusedFunctionChecker {
    functions: HashMap<String, FunctionInfo>,
    diagnostics: Vec<Diagnostic>,
}

impl UnusedFunctionChecker {
    pub fn new() -> Self {
        UnusedFunctionChecker {
            functions: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &ExtendedProgram) -> Vec<Diagnostic> {
        self.functions.clear();
        self.diagnostics.clear();

        // Phase 1: 関数定義を収集
        self.collect_functions(program);

        // Phase 2: 関数使用を追跡
        self.track_function_usage(program);

        // Phase 3: 未使用関数の診断を生成
        self.generate_unused_function_diagnostics();

        self.diagnostics.clone()
    }

    // Phase 1: 関数定義の収集
    fn collect_functions(&mut self, program: &ExtendedProgram) {
        for stmt in &program.statements {
            self.collect_functions_from_stmt(stmt);
        }
    }

    fn collect_functions_from_stmt(&mut self, stmt: &ExtendedStmt) {
        match stmt {
            ExtendedStmt::FunctionDeclaration { name, span, .. } => {
                let function_info = FunctionInfo::new(name.clone(), span.clone());
                self.functions.insert(name.clone(), function_info);
            }
            ExtendedStmt::LetDeclaration { .. } | ExtendedStmt::Expression(_) => {
                // 関数定義ではない
            }
        }
    }

    // Phase 2: 関数使用の追跡
    fn track_function_usage(&mut self, program: &ExtendedProgram) {
        for stmt in &program.statements {
            self.track_usage_in_stmt(stmt);
        }
    }

    fn track_usage_in_stmt(&mut self, stmt: &ExtendedStmt) {
        match stmt {
            ExtendedStmt::LetDeclaration { value, .. } => {
                self.track_usage_in_expr(value);
            }
            ExtendedStmt::Expression(expr) => {
                self.track_usage_in_expr(expr);
            }
            ExtendedStmt::FunctionDeclaration { body, .. } => {
                // 関数本体内での関数使用も追跡
                for body_stmt in body {
                    self.track_usage_in_stmt(body_stmt);
                }
            }
        }
    }

    fn track_usage_in_expr(&mut self, expr: &ExtendedExpr) {
        match expr {
            ExtendedExpr::FunctionCall {
                name, arguments, ..
            } => {
                // 関数呼び出しを検出
                if let Some(function_info) = self.functions.get_mut(name) {
                    function_info.is_used = true;
                }
                // 引数内の関数呼び出しも追跡
                for arg in arguments {
                    self.track_usage_in_expr(arg);
                }
            }
            ExtendedExpr::Identifier(_, _)
            | ExtendedExpr::Number(_, _)
            | ExtendedExpr::Boolean(_, _)
            | ExtendedExpr::String(_, _) => {
                // これらは関数使用ではない
            }
        }
    }

    // Phase 3: 未使用関数の診断生成
    fn generate_unused_function_diagnostics(&mut self) {
        // todo!("未使用関数の診断を実装してください")
        // ヒント：
        // 1. self.functions をイテレート
        // 2. is_used が false の関数を見つける
        // 3. main関数は除外する（is_main フラグをチェック）
        // 4. 未使用関数の警告を生成
        // 5. self.diagnostics に追加
        // 6. DiagnosticCategory::UnusedVariable を使用（関数専用カテゴリがないため）

        for (function_name, function_info) in &self.functions {
            if !function_info.is_used && !function_info.is_main {
                self.diagnostics.push(Diagnostic::warning(
                    DiagnosticCategory::UnusedVariable,
                    function_name.clone(),
                    function_info.definition_span.clone(),
                ))
            }
        }
    }
}

// 公開API
pub fn check_unused_functions(program: &ExtendedProgram) -> Vec<Diagnostic> {
    let mut checker = UnusedFunctionChecker::new();
    checker.check(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lessons::lesson_4::common::span::Position;

    #[test]
    fn test_unused_function_detection() {
        let program = ExtendedProgram {
            statements: vec![
                ExtendedStmt::FunctionDeclaration {
                    name: "unused_function".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::Number(
                        42,
                        Span::single(Position::new(1, 4)),
                    ))],
                    span: Span::new(Position::new(0, 0), Position::new(2, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "used_function1".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::Number(
                        10,
                        Span::single(Position::new(4, 4)),
                    ))],
                    span: Span::new(Position::new(3, 0), Position::new(5, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "main".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::FunctionCall {
                        name: "used_function1".to_string(),
                        arguments: vec![],
                        span: Span::single(Position::new(7, 4)),
                    })],
                    span: Span::new(Position::new(6, 0), Position::new(8, 1)),
                },
            ],
        };

        let diagnostics = check_unused_functions(&program);

        // unused_function は警告されるべき
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("unused_function")));

        // used_function は警告されるべきではない（mainから呼ばれている）
        assert!(!diagnostics
            .iter()
            .any(|d| d.message.contains("used_function1")));

        // main は警告されるべきではない（常に使用済み扱い）
        assert!(!diagnostics.iter().any(|d| d.message.contains("main")));
    }

    #[test]
    fn test_all_functions_used() {
        let program = ExtendedProgram {
            statements: vec![
                ExtendedStmt::FunctionDeclaration {
                    name: "helper".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::Number(
                        42,
                        Span::single(Position::new(1, 4)),
                    ))],
                    span: Span::new(Position::new(0, 0), Position::new(2, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "main".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::FunctionCall {
                        name: "helper".to_string(),
                        arguments: vec![],
                        span: Span::single(Position::new(4, 4)),
                    })],
                    span: Span::new(Position::new(3, 0), Position::new(5, 1)),
                },
            ],
        };

        let diagnostics = check_unused_functions(&program);

        // 全ての関数が使用されているので、診断は空であるべき
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_multiple_unused_functions() {
        let program = ExtendedProgram {
            statements: vec![
                ExtendedStmt::FunctionDeclaration {
                    name: "unused1".to_string(),
                    parameters: vec![],
                    body: vec![],
                    span: Span::new(Position::new(0, 0), Position::new(1, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "unused2".to_string(),
                    parameters: vec![],
                    body: vec![],
                    span: Span::new(Position::new(2, 0), Position::new(3, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "main".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::Number(
                        42,
                        Span::single(Position::new(5, 4)),
                    ))],
                    span: Span::new(Position::new(4, 0), Position::new(6, 1)),
                },
            ],
        };

        let diagnostics = check_unused_functions(&program);

        // 2つの未使用関数が検出されるべき（mainは除く）
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|d| d.message.contains("unused1")));
        assert!(diagnostics.iter().any(|d| d.message.contains("unused2")));
        assert!(!diagnostics.iter().any(|d| d.message.contains("main")));
    }

    #[test]
    fn test_nested_function_calls() {
        let program = ExtendedProgram {
            statements: vec![
                ExtendedStmt::FunctionDeclaration {
                    name: "helper1".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::Number(
                        1,
                        Span::single(Position::new(1, 4)),
                    ))],
                    span: Span::new(Position::new(0, 0), Position::new(2, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "helper2".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::FunctionCall {
                        name: "helper1".to_string(),
                        arguments: vec![],
                        span: Span::single(Position::new(4, 4)),
                    })],
                    span: Span::new(Position::new(3, 0), Position::new(5, 1)),
                },
                ExtendedStmt::FunctionDeclaration {
                    name: "main".to_string(),
                    parameters: vec![],
                    body: vec![ExtendedStmt::Expression(ExtendedExpr::FunctionCall {
                        name: "helper2".to_string(),
                        arguments: vec![],
                        span: Span::single(Position::new(7, 4)),
                    })],
                    span: Span::new(Position::new(6, 0), Position::new(8, 1)),
                },
            ],
        };

        let diagnostics = check_unused_functions(&program);

        // 全ての関数がネストした呼び出しで使用されているので、診断は空であるべき
        assert!(diagnostics.is_empty());
    }
}
