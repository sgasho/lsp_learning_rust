// Lesson 4-6: 関数抽出機能
// rust-analyzerの最重要リファクタリング機能

// あなたのタスク：
// 選択されたコードブロックを新しい関数として抽出するシステムを実装してください。

use super::common::{
    diagnostic::{Diagnostic, DiagnosticCategory},
    span::{Position, Span},
};
use std::collections::HashMap;

// テキスト編集操作（lesson_4_5から継承）
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

// 関数抽出結果
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractResult {
    pub edits: Vec<TextEdit>,
    pub diagnostics: Vec<Diagnostic>,
    pub extracted_function_name: Option<String>,
}

impl ExtractResult {
    pub fn new() -> Self {
        ExtractResult {
            edits: Vec::new(),
            diagnostics: Vec::new(),
            extracted_function_name: None,
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

    pub fn set_function_name(&mut self, name: String) {
        self.extracted_function_name = Some(name);
    }
}

// 変数の使用情報
#[derive(Debug, Clone)]
pub struct VariableUsage {
    pub name: String,
    pub is_read: bool,   // 変数が読み取られるか
    pub is_written: bool, // 変数が書き込まれるか
    pub first_use_span: Span,
}

impl VariableUsage {
    pub fn new(name: String, span: Span) -> Self {
        VariableUsage {
            name,
            is_read: false,
            is_written: false,
            first_use_span: span,
        }
    }

    pub fn mark_read(&mut self) {
        self.is_read = true;
    }

    pub fn mark_written(&mut self) {
        self.is_written = true;
    }
}

// 抽出可能なコード（スコープと変数使用を含む）
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractableExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier {
        name: String,
        span: Span,
    },
    Binary {
        left: Box<ExtractableExpr>,
        operator: BinaryOp,
        right: Box<ExtractableExpr>,
        span: Span,
    },
}

impl ExtractableExpr {
    pub fn span(&self) -> &Span {
        match self {
            ExtractableExpr::Number(_, span) => span,
            ExtractableExpr::Boolean(_, span) => span,
            ExtractableExpr::String(_, span) => span,
            ExtractableExpr::Identifier { span, .. } => span,
            ExtractableExpr::Binary { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtractableStmt {
    LetDeclaration {
        name: String,
        value: ExtractableExpr,
        span: Span,
    },
    Expression(ExtractableExpr),
    Assignment {
        name: String,
        value: ExtractableExpr,
        span: Span,
    },
}

impl ExtractableStmt {
    pub fn span(&self) -> &Span {
        match self {
            ExtractableStmt::LetDeclaration { span, .. } => span,
            ExtractableStmt::Expression(expr) => expr.span(),
            ExtractableStmt::Assignment { span, .. } => span,
        }
    }
}

// 抽出対象のコードブロック
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub statements: Vec<ExtractableStmt>,
    pub span: Span,
}

// 関数抽出機能
#[derive(Debug)]
pub struct FunctionExtractor {
    variable_usages: HashMap<String, VariableUsage>,
}

impl FunctionExtractor {
    pub fn new() -> Self {
        FunctionExtractor {
            variable_usages: HashMap::new(),
        }
    }

    pub fn extract_function(
        &mut self,
        code_block: &CodeBlock,
        function_name: String,
    ) -> ExtractResult {
        self.variable_usages.clear();

        // Phase 1: 変数使用パターンを分析
        self.analyze_variable_usage(code_block);

        // Phase 2: 抽出可能性をチェック
        if let Some(error) = self.check_extractability(code_block) {
            return ExtractResult::with_error(error, code_block.span.clone());
        }

        // Phase 3: 関数シグネチャを生成
        let signature = self.generate_function_signature(&function_name);

        // Phase 4: 関数本体を生成
        let body = self.generate_function_body(code_block);

        // Phase 5: リファクタリング操作を生成
        self.generate_extract_edits(code_block, signature, body, function_name)
    }

    // Phase 1: 変数使用パターンの分析
    fn analyze_variable_usage(&mut self, code_block: &CodeBlock) {
        for stmt in &code_block.statements {
            self.analyze_statement_usage(stmt);
        }
    }

    fn analyze_statement_usage(&mut self, stmt: &ExtractableStmt) {
        match stmt {
            ExtractableStmt::LetDeclaration { name, value, span } => {
                // 新しい変数定義
                let mut usage = VariableUsage::new(name.clone(), span.clone());
                usage.mark_written();
                self.variable_usages.insert(name.clone(), usage);

                // 初期化式の分析
                self.analyze_expression_usage(value);
            }
            ExtractableStmt::Expression(expr) => {
                self.analyze_expression_usage(expr);
            }
            ExtractableStmt::Assignment { name, value, span } => {
                // 既存変数への代入
                let usage = self.variable_usages
                    .entry(name.clone())
                    .or_insert_with(|| VariableUsage::new(name.clone(), span.clone()));
                usage.mark_written();

                self.analyze_expression_usage(value);
            }
        }
    }

    fn analyze_expression_usage(&mut self, expr: &ExtractableExpr) {
        match expr {
            ExtractableExpr::Identifier { name, span } => {
                let usage = self.variable_usages
                    .entry(name.clone())
                    .or_insert_with(|| VariableUsage::new(name.clone(), span.clone()));
                usage.mark_read();
            }
            ExtractableExpr::Binary { left, right, .. } => {
                self.analyze_expression_usage(left);
                self.analyze_expression_usage(right);
            }
            ExtractableExpr::Number(_, _)
            | ExtractableExpr::Boolean(_, _)
            | ExtractableExpr::String(_, _) => {
                // リテラルは変数使用ではない
            }
        }
    }

    // Phase 2: 抽出可能性のチェック
    fn check_extractability(&self, code_block: &CodeBlock) -> Option<String> {
        // 空のブロックは抽出できない
        if code_block.statements.is_empty() {
            return Some("Cannot extract empty code block".to_string());
        }

        // 複雑な制御フローがある場合は抽出を拒否
        // （この実装では基本的な文のみサポート）
        
        None // 基本的なケースではエラーなし
    }

    // Phase 3: 関数シグネチャの生成
    fn generate_function_signature(&self, function_name: &str) -> String {
        let mut params = Vec::new();
        let mut return_vars = Vec::new();

        for (var_name, usage) in &self.variable_usages {
            if usage.is_read && !usage.is_written {
                // 読み取り専用変数はパラメータ
                params.push(format!("{}: i32", var_name)); // 簡略化：全てi32とする
            } else if usage.is_written {
                // 書き込み変数は戻り値候補
                return_vars.push(var_name.clone());
            }
        }

        let param_str = params.join(", ");
        let return_str = if return_vars.is_empty() {
            String::new()
        } else if return_vars.len() == 1 {
            format!(" -> i32")
        } else {
            format!(" -> ({})", return_vars.iter().map(|_| "i32").collect::<Vec<_>>().join(", "))
        };

        format!("fn {}({}){}", function_name, param_str, return_str)
    }

    // Phase 4: 関数本体の生成
    fn generate_function_body(&self, code_block: &CodeBlock) -> String {
        let mut body_lines = Vec::new();

        for stmt in &code_block.statements {
            body_lines.push(self.statement_to_string(stmt));
        }

        // 戻り値の生成
        let return_vars: Vec<String> = self.variable_usages
            .iter()
            .filter(|(_, usage)| usage.is_written)
            .map(|(name, _)| name.clone())
            .collect();

        if !return_vars.is_empty() {
            if return_vars.len() == 1 {
                body_lines.push(return_vars[0].clone());
            } else {
                body_lines.push(format!("({})", return_vars.join(", ")));
            }
        }

        format!("    {}\n", body_lines.join("\n    "))
    }

    // Phase 5: リファクタリング操作の生成
    fn generate_extract_edits(
        &self,
        code_block: &CodeBlock,
        signature: String,
        body: String,
        function_name: String,
    ) -> ExtractResult {
        let mut result = ExtractResult::new();
        result.set_function_name(function_name.clone());

        // 1. 抽出された関数の定義を生成
        let function_definition = format!("{} {{\n{}}}\n\n", signature, body);

        // 関数定義を適切な位置（通常はファイルの最後）に挿入
        // 簡略化のため、元のコードブロックの前に挿入
        let function_insert_position = Span::new(
            Position::new(code_block.span.start.line, 0),
            Position::new(code_block.span.start.line, 0),
        );
        result.add_edit(TextEdit::new(function_insert_position, function_definition));

        // 2. 関数呼び出しの生成
        let mut call_args = Vec::new();
        let mut return_vars = Vec::new();

        // 引数と戻り値を収集
        for (var_name, usage) in &self.variable_usages {
            if usage.is_read && !usage.is_written {
                // 読み取り専用変数は引数
                call_args.push(var_name.clone());
            } else if usage.is_written {
                // 書き込み変数は戻り値
                return_vars.push(var_name.clone());
            }
        }

        // 3. 関数呼び出し文の生成
        let function_call = if return_vars.is_empty() {
            // 戻り値なし
            format!("{}({});", function_name, call_args.join(", "))
        } else if return_vars.len() == 1 {
            // 単一戻り値
            format!("let {} = {}({});", return_vars[0], function_name, call_args.join(", "))
        } else {
            // 複数戻り値
            format!(
                "let ({}) = {}({});",
                return_vars.join(", "),
                function_name,
                call_args.join(", ")
            )
        };

        // 4. 元のコードブロックを関数呼び出しに置換
        result.add_edit(TextEdit::new(code_block.span.clone(), function_call));

        result
    }

    // ヘルパーメソッド：文を文字列に変換
    fn statement_to_string(&self, stmt: &ExtractableStmt) -> String {
        match stmt {
            ExtractableStmt::LetDeclaration { name, value, .. } => {
                format!("let {} = {};", name, self.expression_to_string(value))
            }
            ExtractableStmt::Expression(expr) => {
                format!("{};", self.expression_to_string(expr))
            }
            ExtractableStmt::Assignment { name, value, .. } => {
                format!("{} = {};", name, self.expression_to_string(value))
            }
        }
    }

    fn expression_to_string(&self, expr: &ExtractableExpr) -> String {
        match expr {
            ExtractableExpr::Number(n, _) => n.to_string(),
            ExtractableExpr::Boolean(b, _) => b.to_string(),
            ExtractableExpr::String(s, _) => format!("\"{}\"", s),
            ExtractableExpr::Identifier { name, .. } => name.clone(),
            ExtractableExpr::Binary { left, operator, right, .. } => {
                format!(
                    "{} {} {}",
                    self.expression_to_string(left),
                    operator.as_str(),
                    self.expression_to_string(right)
                )
            }
        }
    }
}

// 公開API
pub fn extract_function(code_block: &CodeBlock, function_name: String) -> ExtractResult {
    let mut extractor = FunctionExtractor::new();
    extractor.extract_function(code_block, function_name)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression_extraction() {
        let code_block = CodeBlock {
            statements: vec![
                ExtractableStmt::LetDeclaration {
                    name: "result".to_string(),
                    value: ExtractableExpr::Binary {
                        left: Box::new(ExtractableExpr::Identifier {
                            name: "a".to_string(),
                            span: Span::single(Position::new(0, 12)),
                        }),
                        operator: BinaryOp::Add,
                        right: Box::new(ExtractableExpr::Identifier {
                            name: "b".to_string(),
                            span: Span::single(Position::new(0, 16)),
                        }),
                        span: Span::new(Position::new(0, 12), Position::new(0, 16)),
                    },
                    span: Span::new(Position::new(0, 4), Position::new(0, 17)),
                },
            ],
            span: Span::new(Position::new(0, 0), Position::new(0, 18)),
        };

        let result = extract_function(&code_block, "calculate".to_string());

        // エラーがないことを確認
        assert!(result.diagnostics.is_empty());

        // 関数名が設定されていることを確認
        assert_eq!(result.extracted_function_name, Some("calculate".to_string()));

        // 編集操作が生成されていることを確認（実装後）
        // assert!(!result.edits.is_empty());
    }

    #[test]
    fn test_multiple_statements_extraction() {
        let code_block = CodeBlock {
            statements: vec![
                ExtractableStmt::LetDeclaration {
                    name: "x".to_string(),
                    value: ExtractableExpr::Number(10, Span::single(Position::new(0, 8))),
                    span: Span::new(Position::new(0, 4), Position::new(0, 9)),
                },
                ExtractableStmt::LetDeclaration {
                    name: "y".to_string(),
                    value: ExtractableExpr::Binary {
                        left: Box::new(ExtractableExpr::Identifier {
                            name: "x".to_string(),
                            span: Span::single(Position::new(1, 8)),
                        }),
                        operator: BinaryOp::Multiply,
                        right: Box::new(ExtractableExpr::Number(2, Span::single(Position::new(1, 12)))),
                        span: Span::new(Position::new(1, 8), Position::new(1, 12)),
                    },
                    span: Span::new(Position::new(1, 4), Position::new(1, 13)),
                },
            ],
            span: Span::new(Position::new(0, 0), Position::new(1, 14)),
        };

        let result = extract_function(&code_block, "process".to_string());

        // エラーがないことを確認
        assert!(result.diagnostics.is_empty());

        // 関数名が設定されていることを確認
        assert_eq!(result.extracted_function_name, Some("process".to_string()));
    }

    #[test]
    fn test_empty_block_extraction() {
        let code_block = CodeBlock {
            statements: vec![],
            span: Span::new(Position::new(0, 0), Position::new(0, 1)),
        };

        let result = extract_function(&code_block, "empty".to_string());

        // エラーが発生することを確認
        assert!(!result.diagnostics.is_empty());
        assert!(result.diagnostics.iter().any(|d| d.message.contains("empty")));
    }

    #[test]
    fn test_variable_usage_analysis() {
        let code_block = CodeBlock {
            statements: vec![
                ExtractableStmt::Assignment {
                    name: "existing_var".to_string(),
                    value: ExtractableExpr::Binary {
                        left: Box::new(ExtractableExpr::Identifier {
                            name: "input".to_string(),
                            span: Span::single(Position::new(0, 15)),
                        }),
                        operator: BinaryOp::Add,
                        right: Box::new(ExtractableExpr::Number(5, Span::single(Position::new(0, 19)))),
                        span: Span::new(Position::new(0, 15), Position::new(0, 19)),
                    },
                    span: Span::new(Position::new(0, 0), Position::new(0, 20)),
                },
            ],
            span: Span::new(Position::new(0, 0), Position::new(0, 21)),
        };

        let result = extract_function(&code_block, "update".to_string());

        // エラーがないことを確認
        assert!(result.diagnostics.is_empty());

        // 関数名が設定されていることを確認
        assert_eq!(result.extracted_function_name, Some("update".to_string()));
    }
}