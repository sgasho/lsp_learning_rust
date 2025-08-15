// Lesson 4-4: 到達不可能コード検出
// rust-analyzerの高度な診断機能

// あなたのタスク：
// 到達不可能なコードを検出する診断システムを実装してください。

use super::common::{
    diagnostic::{Diagnostic, DiagnosticCategory},
    span::Span,
};

// 制御フロー文を含む拡張された式
#[derive(Debug, Clone, PartialEq)]
pub enum FlowExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier(String, Span),
    Return {
        value: Option<Box<FlowExpr>>,
        span: Span,
    },
}

impl FlowExpr {
    pub fn span(&self) -> &Span {
        match self {
            FlowExpr::Number(_, span) => span,
            FlowExpr::Boolean(_, span) => span,
            FlowExpr::String(_, span) => span,
            FlowExpr::Identifier(_, span) => span,
            FlowExpr::Return { span, .. } => span,
        }
    }
}

// 制御フロー文を含む拡張された文
#[derive(Debug, Clone, PartialEq)]
pub enum FlowStmt {
    LetDeclaration {
        name: String,
        value: FlowExpr,
        span: Span,
    },
    Expression(FlowExpr),
    Block {
        statements: Vec<FlowStmt>,
        span: Span,
    },
    IfStatement {
        condition: FlowExpr,
        then_branch: Box<FlowStmt>,
        else_branch: Option<Box<FlowStmt>>,
        span: Span,
    },
    Return {
        value: Option<FlowExpr>,
        span: Span,
    },
}

impl FlowStmt {
    pub fn span(&self) -> &Span {
        match self {
            FlowStmt::LetDeclaration { span, .. } => span,
            FlowStmt::Expression(expr) => expr.span(),
            FlowStmt::Block { span, .. } => span,
            FlowStmt::IfStatement { span, .. } => span,
            FlowStmt::Return { span, .. } => span,
        }
    }
}

// 制御フローを含むプログラム
#[derive(Debug, Clone, PartialEq)]
pub struct FlowProgram {
    pub statements: Vec<FlowStmt>,
}

// コードの到達可能性情報
#[derive(Debug, Clone)]
pub struct ReachabilityInfo {
    pub statement: FlowStmt,
    pub is_reachable: bool,
    pub span: Span,
}

impl ReachabilityInfo {
    pub fn new(statement: FlowStmt, is_reachable: bool) -> Self {
        let span = statement.span().clone();
        ReachabilityInfo {
            statement,
            is_reachable,
            span,
        }
    }
}

// 到達不可能コード検出器
#[derive(Debug)]
pub struct UnreachableCodeChecker {
    reachable_statements: Vec<ReachabilityInfo>,
    diagnostics: Vec<Diagnostic>,
}

impl UnreachableCodeChecker {
    pub fn new() -> Self {
        UnreachableCodeChecker {
            reachable_statements: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &FlowProgram) -> Vec<Diagnostic> {
        self.reachable_statements.clear();
        self.diagnostics.clear();

        // Phase 1: 文の到達可能性を解析
        self.analyze_reachability(&program.statements, true);

        // Phase 2: 到達不可能コードの診断を生成
        self.generate_unreachable_diagnostics();

        self.diagnostics.clone()
    }

    // Phase 1: 到達可能性解析
    fn analyze_reachability(&mut self, statements: &[FlowStmt], mut is_reachable: bool) -> bool {
        let mut has_early_return = false;

        for stmt in statements {
            // 現在の文の到達可能性を記録
            let reachability = ReachabilityInfo::new(stmt.clone(), is_reachable);
            self.reachable_statements.push(reachability);

            // 文の種類に応じた到達可能性解析
            match stmt {
                FlowStmt::Return { .. } => {
                    // return文以降は到達不可能
                    has_early_return = true;
                    is_reachable = false;
                }
                FlowStmt::IfStatement {
                    condition: _null,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    // if文の分岐解析
                    if is_reachable {
                        let then_returns = self.analyze_stmt_reachability(then_branch, true);
                        let else_returns = if let Some(else_stmt) = else_branch {
                            self.analyze_stmt_reachability(else_stmt, true)
                        } else {
                            false
                        };

                        // 両方の分岐でreturnする場合、後続は到達不可能
                        if then_returns && else_returns {
                            is_reachable = false;
                            has_early_return = true;
                        }
                    }
                }
                FlowStmt::Block { statements, .. } => {
                    // ブロック内の解析
                    if is_reachable {
                        let block_returns = self.analyze_reachability(statements, true);
                        if block_returns {
                            is_reachable = false;
                            has_early_return = true;
                        }
                    }
                }
                FlowStmt::LetDeclaration { .. } | FlowStmt::Expression(_) => {
                    // 通常の文は到達可能性に影響しない
                }
            }
        }

        has_early_return
    }

    // 単一文の到達可能性解析
    fn analyze_stmt_reachability(&mut self, stmt: &FlowStmt, is_reachable: bool) -> bool {
        match stmt {
            FlowStmt::Return { .. } => {
                let reachability = ReachabilityInfo::new(stmt.clone(), is_reachable);
                self.reachable_statements.push(reachability);
                true
            }
            FlowStmt::Block { statements, .. } => {
                let reachability = ReachabilityInfo::new(stmt.clone(), is_reachable);
                self.reachable_statements.push(reachability);
                self.analyze_reachability(statements, is_reachable)
            }
            FlowStmt::IfStatement {
                then_branch,
                else_branch,
                ..
            } => {
                let reachability = ReachabilityInfo::new(stmt.clone(), is_reachable);
                self.reachable_statements.push(reachability);

                if is_reachable {
                    let then_returns = self.analyze_stmt_reachability(then_branch, true);
                    let else_returns = if let Some(else_stmt) = else_branch {
                        self.analyze_stmt_reachability(else_stmt, true)
                    } else {
                        false
                    };
                    then_returns && else_returns
                } else {
                    false
                }
            }
            _ => {
                let reachability = ReachabilityInfo::new(stmt.clone(), is_reachable);
                self.reachable_statements.push(reachability);
                false
            }
        }
    }

    // Phase 2: 到達不可能コードの診断生成
    fn generate_unreachable_diagnostics(&mut self) {
        // todo!("到達不可能コードの診断を実装してください")
        // ヒント：
        // 1. self.reachable_statements をイテレート
        // 2. is_reachable が false の文を見つける
        // 3. 到達不可能コードの警告を生成
        // 4. self.diagnostics に追加
        // 5. DiagnosticCategory::TypeError を使用（専用カテゴリがないため）
        // 6. メッセージは "unreachable code" を使用

        self.reachable_statements.iter().for_each(|info| {
            if !info.is_reachable {
                self.diagnostics.push(Diagnostic::warning(
                    DiagnosticCategory::TypeError,
                    "unreachable code".to_string(),
                    info.span.clone(),
                ))
            }
        })
    }
}

// 公開API
pub fn check_unreachable_code(program: &FlowProgram) -> Vec<Diagnostic> {
    let mut checker = UnreachableCodeChecker::new();
    checker.check(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lessons::lesson_4::common::span::Position;

    #[test]
    fn test_unreachable_after_return() {
        let program = FlowProgram {
            statements: vec![
                FlowStmt::Return {
                    value: Some(FlowExpr::Number(42, Span::single(Position::new(0, 7)))),
                    span: Span::new(Position::new(0, 0), Position::new(0, 9)),
                },
                FlowStmt::LetDeclaration {
                    name: "unreachable".to_string(),
                    value: FlowExpr::Number(10, Span::single(Position::new(1, 15))),
                    span: Span::new(Position::new(1, 0), Position::new(1, 17)),
                },
                FlowStmt::Expression(FlowExpr::Identifier(
                    "unreachable".to_string(),
                    Span::single(Position::new(2, 0)),
                )),
            ],
        };

        let diagnostics = check_unreachable_code(&program);

        // return後の2つの文が到達不可能として検出されるべき
        assert!(diagnostics.len() >= 2);
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("unreachable code")));
    }

    #[test]
    fn test_reachable_code_only() {
        let program = FlowProgram {
            statements: vec![
                FlowStmt::LetDeclaration {
                    name: "x".to_string(),
                    value: FlowExpr::Number(42, Span::single(Position::new(0, 8))),
                    span: Span::new(Position::new(0, 0), Position::new(0, 10)),
                },
                FlowStmt::Expression(FlowExpr::Identifier(
                    "x".to_string(),
                    Span::single(Position::new(1, 0)),
                )),
                FlowStmt::Return {
                    value: Some(FlowExpr::Identifier(
                        "x".to_string(),
                        Span::single(Position::new(2, 7)),
                    )),
                    span: Span::new(Position::new(2, 0), Position::new(2, 8)),
                },
            ],
        };

        let diagnostics = check_unreachable_code(&program);

        // 全てのコードが到達可能なので、診断は空であるべき
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_unreachable_after_if_both_return() {
        let program = FlowProgram {
            statements: vec![
                FlowStmt::IfStatement {
                    condition: FlowExpr::Boolean(true, Span::single(Position::new(0, 3))),
                    then_branch: Box::new(FlowStmt::Return {
                        value: Some(FlowExpr::Number(1, Span::single(Position::new(1, 11)))),
                        span: Span::new(Position::new(1, 4), Position::new(1, 13)),
                    }),
                    else_branch: Some(Box::new(FlowStmt::Return {
                        value: Some(FlowExpr::Number(2, Span::single(Position::new(3, 11)))),
                        span: Span::new(Position::new(3, 4), Position::new(3, 13)),
                    })),
                    span: Span::new(Position::new(0, 0), Position::new(4, 1)),
                },
                FlowStmt::LetDeclaration {
                    name: "unreachable".to_string(),
                    value: FlowExpr::Number(42, Span::single(Position::new(5, 15))),
                    span: Span::new(Position::new(5, 0), Position::new(5, 17)),
                },
            ],
        };

        let diagnostics = check_unreachable_code(&program);

        // if文で両方の分岐がreturnするため、後続のlet文が到達不可能
        assert!(diagnostics.len() >= 1);
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("unreachable code")));
    }

    #[test]
    fn test_reachable_after_if_partial_return() {
        let program = FlowProgram {
            statements: vec![
                FlowStmt::IfStatement {
                    condition: FlowExpr::Boolean(true, Span::single(Position::new(0, 3))),
                    then_branch: Box::new(FlowStmt::Return {
                        value: Some(FlowExpr::Number(1, Span::single(Position::new(1, 11)))),
                        span: Span::new(Position::new(1, 4), Position::new(1, 13)),
                    }),
                    else_branch: Some(Box::new(FlowStmt::Expression(FlowExpr::Number(
                        2,
                        Span::single(Position::new(3, 4)),
                    )))),
                    span: Span::new(Position::new(0, 0), Position::new(4, 1)),
                },
                FlowStmt::LetDeclaration {
                    name: "reachable".to_string(),
                    value: FlowExpr::Number(42, Span::single(Position::new(5, 15))),
                    span: Span::new(Position::new(5, 0), Position::new(5, 17)),
                },
            ],
        };

        let diagnostics = check_unreachable_code(&program);

        // else分岐がreturnしないため、後続のコードは到達可能
        assert!(diagnostics.is_empty());
    }
}
