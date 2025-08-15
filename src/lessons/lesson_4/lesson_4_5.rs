// Lesson 4-5: 変数リネーム機能
// rust-analyzerのリファクタリング支援機能の第1弾

// あなたのタスク：
// 変数を安全にリネームするシステムを実装してください。

use super::common::{
    diagnostic::{Diagnostic, DiagnosticCategory},
    span::{Position, Span},
};
use std::collections::HashMap;

// テキスト編集操作
#[derive(Debug, Clone, PartialEq)]
pub struct TextEdit {
    pub span: Span,
    pub new_text: String,
}

impl TextEdit {
    pub fn new(span: Span, new_text: String) -> Self {
        TextEdit { span, new_text }
    }
}

// リネーム結果
#[derive(Debug, Clone, PartialEq)]
pub struct RenameResult {
    pub edits: Vec<TextEdit>,
    pub diagnostics: Vec<Diagnostic>,
}

impl RenameResult {
    pub fn new() -> Self {
        RenameResult {
            edits: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_error(message: String, span: Span) -> Self {
        let mut result = Self::new();
        result.diagnostics.push(Diagnostic::warning(
            DiagnosticCategory::TypeError,
            message,
            span,
        ));
        result
    }

    pub fn add_edit(&mut self, edit: TextEdit) {
        self.edits.push(edit);
    }
}

// スコープ情報を含む拡張された式
#[derive(Debug, Clone, PartialEq)]
pub enum ScopedExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier {
        name: String,
        span: Span,
        scope_id: usize,
    },
}

impl ScopedExpr {
    pub fn span(&self) -> &Span {
        match self {
            ScopedExpr::Number(_, span) => span,
            ScopedExpr::Boolean(_, span) => span,
            ScopedExpr::String(_, span) => span,
            ScopedExpr::Identifier { span, .. } => span,
        }
    }
}

// スコープ情報を含む拡張された文
#[derive(Debug, Clone, PartialEq)]
pub enum ScopedStmt {
    LetDeclaration {
        name: String,
        value: ScopedExpr,
        span: Span,
        scope_id: usize,
    },
    Expression(ScopedExpr),
    Block {
        statements: Vec<ScopedStmt>,
        span: Span,
        scope_id: usize,
    },
}

impl ScopedStmt {
    pub fn span(&self) -> &Span {
        match self {
            ScopedStmt::LetDeclaration { span, .. } => span,
            ScopedStmt::Expression(expr) => expr.span(),
            ScopedStmt::Block { span, .. } => span,
        }
    }
}

// スコープ情報を含むプログラム
#[derive(Debug, Clone, PartialEq)]
pub struct ScopedProgram {
    pub statements: Vec<ScopedStmt>,
}

// 変数の定義情報
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub name: String,
    pub definition_span: Span,
    pub scope_id: usize,
    pub usages: Vec<Span>,
}

impl VariableDefinition {
    pub fn new(name: String, definition_span: Span, scope_id: usize) -> Self {
        VariableDefinition {
            name,
            definition_span,
            scope_id,
            usages: Vec::new(),
        }
    }

    pub fn add_usage(&mut self, span: Span) {
        self.usages.push(span);
    }
}

// 変数リネーム機能
#[derive(Debug)]
pub struct VariableRenamer {
    variables: HashMap<String, Vec<VariableDefinition>>,
    current_scope: usize,
}

impl VariableRenamer {
    pub fn new() -> Self {
        VariableRenamer {
            variables: HashMap::new(),
            current_scope: 0,
        }
    }

    pub fn rename_variable(
        &mut self,
        program: &ScopedProgram,
        target_position: Position,
        new_name: String,
    ) -> RenameResult {
        self.variables.clear();
        self.current_scope = 0;

        // Phase 1: 変数定義と使用箇所を収集
        self.collect_variables(program);

        // Phase 2: リネーム対象の変数を見つける
        if let Some(target_var) = self.find_target_variable(&target_position) {
            // Phase 3: リネームが安全かチェック
            if let Some(conflict) = self.check_rename_conflicts(&target_var, &new_name) {
                return RenameResult::with_error(conflict, target_var.definition_span.clone());
            }

            // Phase 4: リネーム操作を生成
            self.generate_rename_edits(&target_var, new_name)
        } else {
            RenameResult::with_error(
                "No variable found at the specified position".to_string(),
                Span::single(target_position),
            )
        }
    }

    // Phase 1: 変数定義と使用箇所の収集
    fn collect_variables(&mut self, program: &ScopedProgram) {
        for stmt in &program.statements {
            self.collect_variables_from_stmt(stmt);
        }
    }

    fn collect_variables_from_stmt(&mut self, stmt: &ScopedStmt) {
        match stmt {
            ScopedStmt::LetDeclaration {
                name,
                value,
                span,
                scope_id,
            } => {
                // 変数定義を記録
                let var_def = VariableDefinition::new(name.clone(), span.clone(), *scope_id);
                self.variables
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(var_def);

                // 初期化式内の変数使用を収集
                self.collect_variables_from_expr(value);
            }
            ScopedStmt::Expression(expr) => {
                self.collect_variables_from_expr(expr);
            }
            ScopedStmt::Block {
                statements,
                scope_id,
                ..
            } => {
                let old_scope = self.current_scope;
                self.current_scope = *scope_id;

                for stmt in statements {
                    self.collect_variables_from_stmt(stmt);
                }

                self.current_scope = old_scope;
            }
        }
    }

    fn collect_variables_from_expr(&mut self, expr: &ScopedExpr) {
        match expr {
            ScopedExpr::Identifier {
                name,
                span,
                scope_id,
            } => {
                // 変数使用を記録
                if let Some(var_defs) = self.variables.get_mut(name) {
                    // 同じスコープまたは外側のスコープの変数定義を探す
                    if let Some(var_def) = var_defs.iter_mut().find(|def| def.scope_id <= *scope_id)
                    {
                        var_def.add_usage(span.clone());
                    }
                }
            }
            ScopedExpr::Number(_, _) | ScopedExpr::Boolean(_, _) | ScopedExpr::String(_, _) => {
                // リテラルは変数使用ではない
            }
        }
    }

    // Phase 2: リネーム対象の変数を見つける
    fn find_target_variable(&self, target_position: &Position) -> Option<&VariableDefinition> {
        for var_defs in self.variables.values() {
            for var_def in var_defs {
                // 定義箇所がターゲット位置にあるかチェック
                if self.position_in_span(target_position, &var_def.definition_span) {
                    return Some(var_def);
                }
                // 使用箇所がターゲット位置にあるかチェック
                for usage_span in &var_def.usages {
                    if self.position_in_span(target_position, usage_span) {
                        return Some(var_def);
                    }
                }
            }
        }
        None
    }

    fn position_in_span(&self, position: &Position, span: &Span) -> bool {
        position.line >= span.start.line
            && position.line <= span.end.line
            && position.column >= span.start.column
            && position.column <= span.end.column
    }

    // Phase 3: リネームの衝突チェック
    fn check_rename_conflicts(
        &self,
        _target_var: &VariableDefinition,
        new_name: &str,
    ) -> Option<String> {
        // todo!("リネームの衝突チェックを実装してください")
        // ヒント：
        // 1. 同じスコープに同じ名前の変数がないかチェック
        // 2. self.variables から new_name の変数定義を探す
        // 3. target_var.scope_id と同じスコープの定義があれば衝突
        // 4. 衝突がある場合は適切なエラーメッセージを返す
        // 5. 衝突がない場合は None を返す
        for (var_name, var_defs) in &self.variables {
            if new_name == var_name {
                if let Some(_conflict_def) = var_defs
                    .iter()
                    .find(|def| def.scope_id == self.current_scope)
                {
                    return Some(
                        "can not rename to existing variable name in the same scope".to_string(),
                    );
                }
            }
        }
        None
    }

    // Phase 4: リネーム操作の生成
    fn generate_rename_edits(
        &self,
        target_var: &VariableDefinition,
        new_name: String,
    ) -> RenameResult {
        let mut result = RenameResult::new();

        // 定義箇所をリネーム
        result.add_edit(TextEdit::new(
            target_var.definition_span.clone(),
            new_name.clone(),
        ));

        // 使用箇所をリネーム
        for usage_span in &target_var.usages {
            result.add_edit(TextEdit::new(usage_span.clone(), new_name.clone()));
        }

        result
    }
}

// 公開API
pub fn rename_variable(
    program: &ScopedProgram,
    target_position: Position,
    new_name: String,
) -> RenameResult {
    let mut renamer = VariableRenamer::new();
    renamer.rename_variable(program, target_position, new_name)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_variable_rename() {
        let program = ScopedProgram {
            statements: vec![
                ScopedStmt::LetDeclaration {
                    name: "old_name".to_string(),
                    value: ScopedExpr::Number(42, Span::single(Position::new(0, 14))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 12)),
                    scope_id: 0,
                },
                ScopedStmt::Expression(ScopedExpr::Identifier {
                    name: "old_name".to_string(),
                    span: Span::single(Position::new(1, 0)),
                    scope_id: 0,
                }),
            ],
        };

        let result = rename_variable(&program, Position::new(0, 8), "new_name".to_string());

        // エラーがないことを確認
        assert!(result.diagnostics.is_empty());

        // 2つの編集（定義と使用）があることを確認
        assert_eq!(result.edits.len(), 2);

        // 定義箇所の編集を確認
        assert!(result
            .edits
            .iter()
            .any(|edit| edit.span.start.line == 0 && edit.new_text == "new_name"));

        // 使用箇所の編集を確認
        assert!(result
            .edits
            .iter()
            .any(|edit| edit.span.start.line == 1 && edit.new_text == "new_name"));
    }

    #[test]
    fn test_rename_conflict_detection() {
        let program = ScopedProgram {
            statements: vec![
                ScopedStmt::LetDeclaration {
                    name: "existing".to_string(),
                    value: ScopedExpr::Number(1, Span::single(Position::new(0, 14))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 12)),
                    scope_id: 0,
                },
                ScopedStmt::LetDeclaration {
                    name: "target".to_string(),
                    value: ScopedExpr::Number(2, Span::single(Position::new(1, 12))),
                    span: Span::new(Position::new(1, 4), Position::new(1, 10)),
                    scope_id: 0,
                },
            ],
        };

        let result = rename_variable(&program, Position::new(1, 7), "existing".to_string());

        // 衝突エラーが発生することを確認
        assert!(!result.diagnostics.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("existing")));
    }

    #[test]
    fn test_multiple_usages_rename() {
        let program = ScopedProgram {
            statements: vec![
                ScopedStmt::LetDeclaration {
                    name: "var".to_string(),
                    value: ScopedExpr::Number(42, Span::single(Position::new(0, 10))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 7)),
                    scope_id: 0,
                },
                ScopedStmt::Expression(ScopedExpr::Identifier {
                    name: "var".to_string(),
                    span: Span::single(Position::new(1, 0)),
                    scope_id: 0,
                }),
                ScopedStmt::Expression(ScopedExpr::Identifier {
                    name: "var".to_string(),
                    span: Span::single(Position::new(2, 0)),
                    scope_id: 0,
                }),
            ],
        };

        let result = rename_variable(&program, Position::new(0, 5), "renamed".to_string());

        // エラーがないことを確認
        assert!(result.diagnostics.is_empty());

        // 3つの編集（定義と2つの使用）があることを確認
        assert_eq!(result.edits.len(), 3);

        // 全ての編集が正しい名前になっていることを確認
        assert!(result.edits.iter().all(|edit| edit.new_text == "renamed"));
    }

    #[test]
    fn test_variable_not_found() {
        let program = ScopedProgram {
            statements: vec![ScopedStmt::Expression(ScopedExpr::Number(
                42,
                Span::single(Position::new(0, 0)),
            ))],
        };

        let result = rename_variable(&program, Position::new(0, 0), "new_name".to_string());

        // 変数が見つからないエラーが発生することを確認
        assert!(!result.diagnostics.is_empty());
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("No variable found")));
    }
}
