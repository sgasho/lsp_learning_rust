// Lesson 3-11へようこそ！
// lesson_3_10で型推論の高度化ができるようになりましたね。
// 今度は、エラー回復とエラー報告を学びます。

// あなたのタスク：
// 複数のエラーを収集し、位置情報付きで報告するシステムを実装してください。
// 例：エラーが1つ見つかっても解析を続行し、すべてのエラーを収集する

use std::collections::HashMap;

// 位置情報（ソースコード内の位置）
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Position { line, column }
    }
}

// 位置範囲（開始位置から終了位置まで）
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    // 1つの位置からスパンを作成（1文字分）
    pub fn single(pos: Position) -> Self {
        Span {
            start: pos.clone(),
            end: Position::new(pos.line, pos.column + 1),
        }
    }
}

// エラーの重要度
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,   // エラー（赤色、コンパイル不可）
    Warning, // 警告（黄色、コンパイル可能）
    Info,    // 情報（青色、ヒント）
}

// 診断情報（エラー・警告・情報）
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub code: Option<String>, // エラーコード（例：E0001）
}

impl Diagnostic {
    pub fn error(message: String, span: Span) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message,
            span,
            code: None,
        }
    }

    pub fn warning(message: String, span: Span) -> Self {
        Diagnostic {
            severity: Severity::Warning,
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

// 型情報（lesson_3_10と同じ）
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    String,
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    Unknown,
    Inferred(Box<Type>),
}

impl Type {
    pub fn resolve(&self) -> &Type {
        match self {
            Type::Inferred(inner) => inner.resolve(),
            other => other,
        }
    }
}

// AST構造（位置情報を追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
        type_annotation: Option<Type>,
        span: Span, // 位置情報を追加
    },
    Expression(Expr),
    Block {
        statements: Vec<Stmt>,
        span: Span,
    },
    IfStatement {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        span: Span,
    },
    WhileStatement {
        condition: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Box<Stmt>,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> &Span {
        match self {
            Stmt::LetDeclaration { span, .. } => span,
            Stmt::Expression(expr) => expr.span(),
            Stmt::Block { span, .. } => span,
            Stmt::IfStatement { span, .. } => span,
            Stmt::WhileStatement { span, .. } => span,
            Stmt::FunctionDeclaration { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier(String, Span),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    Assignment {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Number(_, span) => span,
            Expr::Boolean(_, span) => span,
            Expr::String(_, span) => span,
            Expr::Identifier(_, span) => span,
            Expr::Binary { span, .. } => span,
            Expr::FunctionCall { span, .. } => span,
            Expr::Assignment { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// シンボル（lesson_3_10と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
    pub symbol_type: Type,
    pub definition_span: Span, // 定義位置を追加
}

// スコープ（lesson_3_10と同じ）
#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            level: 0,
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope, level: usize) -> Self {
        Scope {
            level,
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
}

// シンボルテーブル（定義位置を追加）
#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Scope,
    pub scope_level: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            current_scope: Scope::new(),
            scope_level: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_level += 1;
        self.current_scope = Scope::with_parent(self.current_scope.clone(), self.scope_level);
    }

    pub fn exit_scope(&mut self) {
        if self.scope_level == 0 {
            return;
        }

        self.scope_level -= 1;
        if let Some(parent) = self.current_scope.parent.take() {
            self.current_scope = *parent;
        }
    }

    pub fn define(
        &mut self,
        name: String,
        symbol_type: Type,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(_existing) = self.current_scope.symbols.get(&name) {
            return Err(Diagnostic::error(
                format!("Variable '{}' already defined in this scope", name),
                span.clone(),
            )
            .with_code("E0001".to_string()));
        }

        let symbol = Symbol {
            name: name.clone(),
            scope_level: self.scope_level,
            symbol_type,
            definition_span: span,
        };

        self.current_scope.symbols.insert(name, symbol);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        let mut current = &self.current_scope;

        loop {
            if let Some(symbol) = current.symbols.get(name) {
                return Some(symbol);
            }

            if let Some(parent) = &current.parent {
                current = parent;
            } else {
                break;
            }
        }

        None
    }
}

// エラー回復型チェッカー
#[derive(Debug)]
pub struct DiagnosticTypeChecker {
    symbol_table: SymbolTable,
    diagnostics: Vec<Diagnostic>, // エラー・警告・情報を収集
}

impl DiagnosticTypeChecker {
    pub fn new() -> Self {
        DiagnosticTypeChecker {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Vec<Diagnostic> {
        // エラーが発生しても解析を続行
        for statement in &program.statements {
            self.check_statement(statement);
        }

        self.diagnostics.clone()
    }

    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LetDeclaration {
                name,
                value,
                type_annotation,
                span,
            } => {
                // todo!("エラー回復型の変数定義チェックを実装してください")
                // ヒント：
                // 1. 値の型を推論（エラーでも続行）
                // 2. 型注釈との一致チェック（エラーでも続行）
                // 3. 変数定義（重複チェック）
                // 4. すべてのエラーをdiagnosticsに追加
                if let Some(inferred_value_type) = self.infer_expression_type(value) {
                    if type_annotation
                        .clone()
                        .is_some_and(|annotation| annotation.ne(&inferred_value_type))
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Type mismatch: expected {:?}, found {:?}",
                                    type_annotation, inferred_value_type,
                                ),
                                span.clone(),
                            )
                            .with_code("E0002".to_string()),
                        );
                        if let Err(err) = self.symbol_table.define(
                            name.clone(),
                            inferred_value_type.clone(),
                            span.clone(),
                        ) {
                            self.diagnostics.push(
                                Diagnostic::error(format!("{:?}", err), span.clone())
                                    .with_code("E0003".to_string()),
                            )
                        }
                    }
                }

                if let Some(_) = self.symbol_table.resolve(name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!("Variable '{}' already defined in this scope", name),
                            span.clone(),
                        )
                        .with_code("E0003".to_string()),
                    );
                }
            }
            Stmt::Expression(expr) => {
                self.infer_expression_type(expr);
            }
            Stmt::Block { statements, .. } => {
                self.symbol_table.enter_scope();
                for stmt in statements {
                    self.check_statement(stmt);
                }
                self.symbol_table.exit_scope();
            }
            Stmt::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // todo!("エラー回復型のif文チェックを実装してください")
                // ヒント：
                // 1. 条件の型チェック（エラーでも続行）
                // 2. then_branchのチェック
                // 3. else_branchのチェック（もしあれば）

                if let Some(condition_type) = self.infer_expression_type(condition) {
                    if condition_type.ne(&Type::Boolean) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "If condition must be boolean, found {:?}",
                                    condition_type.resolve()
                                ),
                                condition.span().clone(),
                            )
                            .with_code("E0003".to_string()),
                        );
                    }
                }

                self.check_statement(then_branch);
                if let Some(else_block) = else_branch {
                    self.check_statement(else_block);
                }
            }
            Stmt::WhileStatement {
                condition, body, ..
            } => {
                if let Some(condition_type) = self.infer_expression_type(condition) {
                    if *condition_type.resolve() != Type::Boolean {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "While condition must be boolean, found {:?}",
                                    condition_type.resolve()
                                ),
                                condition.span().clone(),
                            )
                            .with_code("E0003".to_string()),
                        );
                    }
                }

                self.check_statement(body);
            }
            Stmt::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
                span,
            } => {
                let param_types: Vec<Type> = parameters
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Unknown))
                    .collect();

                let function_type = Type::Function {
                    parameters: param_types,
                    return_type: Box::new(return_type.clone().unwrap_or(Type::Unknown)),
                };

                if let Err(diagnostic) =
                    self.symbol_table
                        .define(name.clone(), function_type, span.clone())
                {
                    self.diagnostics.push(diagnostic);
                }

                self.symbol_table.enter_scope();

                for param in parameters {
                    let param_type = param.param_type.clone().unwrap_or(Type::Unknown);
                    if let Err(diagnostic) =
                        self.symbol_table
                            .define(param.name.clone(), param_type, param.span.clone())
                    {
                        self.diagnostics.push(diagnostic);
                    }
                }

                self.check_statement(body);
                self.symbol_table.exit_scope();
            }
        }
    }

    fn infer_expression_type(&mut self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Number(_, _) => Some(Type::Integer),
            Expr::Boolean(_, _) => Some(Type::Boolean),
            Expr::String(_, _) => Some(Type::String),
            Expr::Identifier(name, span) => {
                if let Some(symbol) = self.symbol_table.resolve(name) {
                    Some(symbol.symbol_type.clone())
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("Variable '{}' not defined", name), span.clone())
                            .with_code("E0004".to_string()),
                    );
                    None
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
                span,
            } => {
                // todo!("エラー回復型の二項演算チェックを実装してください")
                // ヒント：
                // 1. 左右の型を推論（どちらがエラーでも続行）
                // 2. 両方の型が取得できた場合のみ演算子チェック
                // 3. エラーはdiagnosticsに追加
                let inferred_left = self.infer_expression_type(left);
                let inferred_right = self.infer_expression_type(right);

                if let (Some(left_type), Some(right_type)) = (inferred_left, inferred_right) {
                    return match operator {
                        BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Divide => {
                            if left_type.resolve().ne(&Type::Integer)
                                || right_type.resolve().ne(&Type::Integer)
                            {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "Arithmetic must be integers".to_string(),
                                        span.clone(),
                                    )
                                    .with_code("E0005".to_string()),
                                );
                                return None;
                            }
                            Some(Type::Integer)
                        }
                        BinaryOp::GreaterThan | BinaryOp::LessThan => {
                            if left_type.resolve().ne(&Type::Integer)
                                || right_type.resolve().ne(&Type::Integer)
                            {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "Arithmetic must be integers".to_string(),
                                        span.clone(),
                                    )
                                    .with_code("E0005".to_string()),
                                );
                                return None;
                            }
                            Some(Type::Boolean)
                        }
                        BinaryOp::Equal | BinaryOp::NotEqual => {
                            if left_type.resolve().ne(&right_type.resolve()) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "Type mismatch: expected {:?}, found {:?}",
                                            left_type, right_type
                                        ),
                                        span.clone(),
                                    )
                                    .with_code("E0005".to_string()),
                                );
                                return None;
                            }
                            Some(Type::Boolean)
                        }
                    };
                }
                None
            }
            Expr::FunctionCall {
                name,
                arguments,
                span,
            } => {
                // 借用問題を回避するため、関数型を先にクローン
                let function_type = if let Some(symbol) = self.symbol_table.resolve(name) {
                    Some(symbol.symbol_type.clone())
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("Function '{}' not defined", name), span.clone())
                            .with_code("E0004".to_string()),
                    );
                    return None;
                };

                if let Some(Type::Function {
                    parameters,
                    return_type,
                }) = function_type
                {
                    if arguments.len() != parameters.len() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Function '{}' expects {} arguments, found {}",
                                    name,
                                    parameters.len(),
                                    arguments.len()
                                ),
                                span.clone(),
                            )
                            .with_code("E0005".to_string()),
                        );
                    }

                    // 引数の型チェック（エラーでも続行）
                    for (arg, expected_type) in arguments.iter().zip(parameters.iter()) {
                        if let Some(arg_type) = self.infer_expression_type(arg) {
                            if *arg_type.resolve() != *expected_type.resolve()
                                && *expected_type.resolve() != Type::Unknown
                            {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "Argument type mismatch: expected {:?}, found {:?}",
                                            expected_type.resolve(),
                                            arg_type.resolve()
                                        ),
                                        arg.span().clone(),
                                    )
                                    .with_code("E0006".to_string()),
                                );
                            }
                        }
                    }

                    Some(*return_type)
                } else if function_type.is_some() {
                    self.diagnostics.push(
                        Diagnostic::error(format!("'{}' is not a function", name), span.clone())
                            .with_code("E0007".to_string()),
                    );
                    None
                } else {
                    None
                }
            }
            Expr::Assignment { name, value, span } => {
                // todo!("エラー回復型の代入チェックを実装してください")
                // ヒント：
                // 1. 変数の存在チェック
                // 2. 値の型推論
                // 3. 型の一致チェック
                // 4. エラーはdiagnosticsに追加、戻り値はOption<Type>
                if self.symbol_table.resolve(name).is_none() {
                    self.diagnostics.push(
                        Diagnostic::error(format!("Variable '{}' not defined", name), span.clone())
                            .with_code("E0008".to_string()),
                    );
                    return None;
                }

                self.infer_expression_type(value)
            }
        }
    }

    pub fn get_diagnostics(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    // 診断結果をフォーマットして表示
    pub fn format_diagnostics(&self, source: &str) -> String {
        let mut result = String::new();
        let lines: Vec<&str> = source.lines().collect();

        for diagnostic in &self.diagnostics {
            result.push_str(&format!(
                "{}:{}: ",
                diagnostic.span.start.line + 1,
                diagnostic.span.start.column + 1
            ));

            match diagnostic.severity {
                Severity::Error => result.push_str("error: "),
                Severity::Warning => result.push_str("warning: "),
                Severity::Info => result.push_str("info: "),
            }

            result.push_str(&diagnostic.message);

            if let Some(code) = &diagnostic.code {
                result.push_str(&format!(" [{}]", code));
            }

            result.push('\n');

            // ソースコードの該当行を表示
            if diagnostic.span.start.line < lines.len() {
                result.push_str(&format!(" --> {}\n", lines[diagnostic.span.start.line]));

                // エラー位置を指す矢印
                let mut pointer = String::new();
                for _ in 0..diagnostic.span.start.column {
                    pointer.push(' ');
                }
                for _ in diagnostic.span.start.column..diagnostic.span.end.column {
                    pointer.push('^');
                }
                result.push_str(&format!("     {}\n", pointer));
            }

            result.push('\n');
        }

        result
    }
}

// 公開API
pub fn check_with_diagnostics(program: &Program) -> Vec<Diagnostic> {
    let mut checker = DiagnosticTypeChecker::new();
    checker.check_program(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_errors_collection() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Identifier(
                        "undefined".to_string(),
                        Span::single(Position::new(0, 8)),
                    ),
                    type_annotation: None,
                    span: Span::new(Position::new(0, 0), Position::new(0, 20)),
                },
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Boolean(true, Span::single(Position::new(1, 8))),
                    type_annotation: Some(Type::Integer),
                    span: Span::new(Position::new(1, 0), Position::new(1, 20)),
                },
                Stmt::LetDeclaration {
                    name: "x".to_string(), // 重複定義
                    value: Expr::Number(42, Span::single(Position::new(2, 8))),
                    type_annotation: None,
                    span: Span::new(Position::new(2, 0), Position::new(2, 15)),
                },
            ],
        };

        let diagnostics = check_with_diagnostics(&program);

        // 3つのエラーが収集されることを確認
        assert_eq!(diagnostics.len(), 3);

        // 各エラーの内容を確認
        assert!(diagnostics[0].message.contains("not defined"));
        assert!(diagnostics[1].message.contains("Type mismatch"));
        assert!(diagnostics[2].message.contains("already defined"));
    }

    #[test]
    fn test_error_recovery_in_expressions() {
        let program = Program {
            statements: vec![Stmt::Expression(Expr::Binary {
                left: Box::new(Expr::Identifier(
                    "undefined1".to_string(),
                    Span::single(Position::new(0, 0)),
                )),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Identifier(
                    "undefined2".to_string(),
                    Span::single(Position::new(0, 15)),
                )),
                span: Span::new(Position::new(0, 0), Position::new(0, 25)),
            })],
        };

        let diagnostics = check_with_diagnostics(&program);

        // 2つの未定義変数エラーが収集されることを確認
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].message.contains("undefined1"));
        assert!(diagnostics[1].message.contains("undefined2"));
    }

    #[test]
    fn test_if_condition_error_recovery() {
        let program = Program {
            statements: vec![Stmt::IfStatement {
                condition: Expr::Number(42, Span::single(Position::new(0, 3))), // 数値を条件に使用
                then_branch: Box::new(Stmt::Expression(Expr::Identifier(
                    "undefined".to_string(),
                    Span::single(Position::new(1, 4)),
                ))),
                else_branch: None,
                span: Span::new(Position::new(0, 0), Position::new(2, 1)),
            }],
        };

        let diagnostics = check_with_diagnostics(&program);

        // if条件のエラーと未定義変数のエラーが両方収集されることを確認
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].message.contains("boolean"));
        assert!(diagnostics[1].message.contains("not defined"));
    }
}
