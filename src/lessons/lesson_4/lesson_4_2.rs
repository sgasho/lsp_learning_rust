// Lesson 4-2: 未使用インポート検出
// rust-analyzerで2番目によく貢献される診断機能

// あなたのタスク：
// 未使用インポートを検出する診断システムを実装してください。

use super::common::{
    ast::{Expr, Stmt},
    diagnostic::{Diagnostic, DiagnosticCategory},
    span::Span,
};
use std::collections::HashMap;

// インポート文（ASTに追加）
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module_name: String,
    pub imported_name: String,
    pub span: Span,
}

// 拡張されたプログラム（インポートを含む）
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramWithImports {
    pub imports: Vec<Import>,
    pub statements: Vec<Stmt>,
}

// インポート情報（使用状況追跡）
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub import: Import,
    pub is_used: bool,
}

impl ImportInfo {
    pub fn new(import: Import) -> Self {
        ImportInfo {
            import,
            is_used: false,
        }
    }
}

// 未使用インポート検出器
#[derive(Debug)]
pub struct UnusedImportChecker {
    imports: HashMap<String, ImportInfo>,
    diagnostics: Vec<Diagnostic>,
}

impl UnusedImportChecker {
    pub fn new() -> Self {
        UnusedImportChecker {
            imports: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &ProgramWithImports) -> Vec<Diagnostic> {
        self.imports.clear();
        self.diagnostics.clear();

        // Phase 1: インポートを収集
        self.collect_imports(program);

        // Phase 2: インポートの使用を追跡
        self.track_import_usage(program);

        // Phase 3: 未使用インポートの診断を生成
        self.generate_unused_import_diagnostics();

        self.diagnostics.clone()
    }

    // Phase 1: インポートの収集
    fn collect_imports(&mut self, program: &ProgramWithImports) {
        for import in &program.imports {
            let import_info = ImportInfo::new(import.clone());
            self.imports
                .insert(import.imported_name.clone(), import_info);
        }
    }

    // Phase 2: インポートの使用追跡
    fn track_import_usage(&mut self, program: &ProgramWithImports) {
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
                if let Some(import_info) = self.imports.get_mut(name) {
                    import_info.is_used = true;
                }
            }
            Expr::Number(_, _) | Expr::Boolean(_, _) | Expr::String(_, _) => {
                // リテラルはインポート使用ではない
            }
        }
    }

    // Phase 3: 未使用インポートの診断生成
    fn generate_unused_import_diagnostics(&mut self) {
        // todo!("未使用インポートの診断を実装してください")
        // ヒント：
        // 1. self.imports をイテレート
        // 2. is_used が false のインポートを見つける
        // 3. 未使用インポートの警告を生成
        // 4. self.diagnostics に追加
        // 5. DiagnosticCategory::UnusedImport を使用
        self.imports.iter().for_each(|(name, import_info)| {
            if !import_info.is_used {
                self.diagnostics.push(
                    Diagnostic::warning(
                        DiagnosticCategory::UnusedImport,
                        format!("unused import `{}`", name),
                        import_info.import.span.clone(),
                    )
                    .with_code("unused_import".to_string()),
                )
            }
        })
    }
}

// 公開API
pub fn check_unused_imports(program: &ProgramWithImports) -> Vec<Diagnostic> {
    let mut checker = UnusedImportChecker::new();
    checker.check(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lessons::lesson_4::common::span::Position;

    #[test]
    fn test_unused_import_detection() {
        let program = ProgramWithImports {
            imports: vec![
                Import {
                    module_name: "std::collections".to_string(),
                    imported_name: "HashMap".to_string(),
                    span: Span::new(Position::new(0, 0), Position::new(0, 32)),
                },
                Import {
                    module_name: "std::vec".to_string(),
                    imported_name: "Vec".to_string(),
                    span: Span::new(Position::new(1, 0), Position::new(1, 18)),
                },
            ],
            statements: vec![Stmt::LetDeclaration {
                name: "data".to_string(),
                value: Expr::Identifier("Vec".to_string(), Span::single(Position::new(2, 12))),
                span: Span::new(Position::new(2, 4), Position::new(2, 16)),
            }],
        };

        let diagnostics = check_unused_imports(&program);

        // HashMap は警告されるべき（未使用）
        assert!(diagnostics
            .iter()
            .any(|d| matches!(d.category, DiagnosticCategory::UnusedImport)
                && d.message.contains("HashMap")));

        // Vec は警告されるべきではない（使用済み）
        assert!(!diagnostics
            .iter()
            .any(|d| matches!(d.category, DiagnosticCategory::UnusedImport)
                && d.message.contains("Vec")));
    }

    #[test]
    fn test_all_imports_used() {
        let program = ProgramWithImports {
            imports: vec![
                Import {
                    module_name: "std::collections".to_string(),
                    imported_name: "HashMap".to_string(),
                    span: Span::new(Position::new(0, 0), Position::new(0, 32)),
                },
                Import {
                    module_name: "std::vec".to_string(),
                    imported_name: "Vec".to_string(),
                    span: Span::new(Position::new(1, 0), Position::new(1, 18)),
                },
            ],
            statements: vec![
                Stmt::LetDeclaration {
                    name: "map".to_string(),
                    value: Expr::Identifier(
                        "HashMap".to_string(),
                        Span::single(Position::new(2, 10)),
                    ),
                    span: Span::new(Position::new(2, 4), Position::new(2, 17)),
                },
                Stmt::LetDeclaration {
                    name: "list".to_string(),
                    value: Expr::Identifier("Vec".to_string(), Span::single(Position::new(3, 11))),
                    span: Span::new(Position::new(3, 4), Position::new(3, 14)),
                },
            ],
        };

        let diagnostics = check_unused_imports(&program);

        // 全てのインポートが使用されているので、診断は空であるべき
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_multiple_unused_imports() {
        let program = ProgramWithImports {
            imports: vec![
                Import {
                    module_name: "std::collections".to_string(),
                    imported_name: "HashMap".to_string(),
                    span: Span::new(Position::new(0, 0), Position::new(0, 32)),
                },
                Import {
                    module_name: "std::collections".to_string(),
                    imported_name: "HashSet".to_string(),
                    span: Span::new(Position::new(1, 0), Position::new(1, 32)),
                },
                Import {
                    module_name: "std::vec".to_string(),
                    imported_name: "Vec".to_string(),
                    span: Span::new(Position::new(2, 0), Position::new(2, 18)),
                },
            ],
            statements: vec![Stmt::LetDeclaration {
                name: "data".to_string(),
                value: Expr::Number(42, Span::single(Position::new(3, 11))),
                span: Span::new(Position::new(3, 4), Position::new(3, 13)),
            }],
        };

        let diagnostics = check_unused_imports(&program);

        // 3つの未使用インポートが検出されるべき
        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().any(|d| d.message.contains("HashMap")));
        assert!(diagnostics.iter().any(|d| d.message.contains("HashSet")));
        assert!(diagnostics.iter().any(|d| d.message.contains("Vec")));
    }
}
