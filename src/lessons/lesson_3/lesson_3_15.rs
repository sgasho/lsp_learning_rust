// Lesson 3-15へようこそ！
// lesson_3_14で借用チェッカーの基本ができるようになりましたね。
// 今度は、ライフタイム推論システムを学びます。

// あなたのタスク：
// 関数のライフタイムパラメータを自動推論するシステムを実装してください。
// 例：fn get_first<'a>(x: &'a str, y: &str) -> &'a str のライフタイム推論

use std::collections::HashMap;

// 位置情報（lesson_3_14と同じ）
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

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    pub fn single(pos: Position) -> Self {
        Span {
            start: pos.clone(),
            end: Position::new(pos.line, pos.column + 1),
        }
    }
}

// 診断情報（lesson_3_14と同じ）
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub code: Option<String>,
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

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }
}

// 借用の種類（lesson_3_14と同じ）
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowKind {
    Immutable, // &T (不変借用)
    Mutable,   // &mut T (可変借用)
}

// ライフタイム（拡張）
#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub name: String,
    pub scope_level: usize,
    pub creation_span: Span,
    pub is_inferred: bool, // 新規追加：推論されたライフタイムかどうか
}

impl Lifetime {
    pub fn new(name: String, scope_level: usize, creation_span: Span) -> Self {
        Lifetime {
            name,
            scope_level,
            creation_span,
            is_inferred: false,
        }
    }

    pub fn inferred(name: String, scope_level: usize, creation_span: Span) -> Self {
        Lifetime {
            name,
            scope_level,
            creation_span,
            is_inferred: true,
        }
    }
}

// ライフタイム制約
#[derive(Debug, Clone, PartialEq)]
pub enum LifetimeConstraint {
    // 'a: 'b (ライフタイム'aは'b以上長生きしなければならない)
    Outlives {
        longer: String,
        shorter: String,
        span: Span,
    },
    // 'a == 'b (ライフタイム'aと'bは同じでなければならない)
    Equal {
        left: String,
        right: String,
        span: Span,
    },
}

// 借用情報（lesson_3_14と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Borrow {
    pub variable: String,    // 借用される変数名
    pub kind: BorrowKind,    // 借用の種類
    pub lifetime: Lifetime,  // 借用のライフタイム
    pub creation_span: Span, // 借用が作成された位置
}

impl Borrow {
    pub fn new(
        variable: String,
        kind: BorrowKind,
        lifetime: Lifetime,
        creation_span: Span,
    ) -> Self {
        Borrow {
            variable,
            kind,
            lifetime,
            creation_span,
        }
    }
}

// 型情報（拡張）
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    String,
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
        lifetime_params: Vec<Lifetime>, // 新規追加：ライフタイムパラメータ
    },
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    Reference {
        inner_type: Box<Type>,
        lifetime: Option<Lifetime>,
        mutability: BorrowKind,
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

    // 構造体のフィールドを検索
    pub fn get_field(&self, field_name: &str) -> Option<&Field> {
        match self.resolve() {
            Type::Struct { fields, .. } => fields.iter().find(|field| field.name == field_name),
            _ => None,
        }
    }

    // 参照型かどうかをチェック
    pub fn is_reference(&self) -> bool {
        matches!(self.resolve(), Type::Reference { .. })
    }

    // 参照の内部型を取得
    pub fn get_inner_type(&self) -> Option<&Type> {
        match self.resolve() {
            Type::Reference { inner_type, .. } => Some(inner_type),
            _ => None,
        }
    }

    // 参照のライフタイムを取得
    pub fn get_lifetime(&self) -> Option<&Lifetime> {
        match self.resolve() {
            Type::Reference { lifetime, .. } => lifetime.as_ref(),
            _ => None,
        }
    }

    // 参照の可変性を取得
    pub fn get_mutability(&self) -> Option<&BorrowKind> {
        match self.resolve() {
            Type::Reference { mutability, .. } => Some(mutability),
            _ => None,
        }
    }

    // 可変借用かどうかをチェック
    pub fn is_mutable_reference(&self) -> bool {
        matches!(
            self.resolve(),
            Type::Reference {
                mutability: BorrowKind::Mutable,
                ..
            }
        )
    }

    // 関数のライフタイムパラメータを取得
    pub fn get_lifetime_params(&self) -> Option<&Vec<Lifetime>> {
        match self.resolve() {
            Type::Function {
                lifetime_params, ..
            } => Some(lifetime_params),
            _ => None,
        }
    }
}

// フィールド定義（lesson_3_14と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub span: Span,
}

// パラメータ（拡張）
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
    pub span: Span,
    pub has_lifetime: bool, // 新規追加：ライフタイムを持つかどうか
}

// AST構造（lesson_3_14と同じ）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
        type_annotation: Option<Type>,
        span: Span,
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
    StructDeclaration {
        name: String,
        fields: Vec<Field>,
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
            Stmt::StructDeclaration { span, .. } => span,
        }
    }
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
    FieldAccess {
        object: Box<Expr>,
        field_name: String,
        span: Span,
    },
    StructConstructor {
        struct_name: String,
        field_values: Vec<(String, Expr)>,
        span: Span,
    },
    Reference {
        inner: Box<Expr>,
        span: Span,
    },
    MutableReference {
        inner: Box<Expr>,
        span: Span,
    },
    Dereference {
        inner: Box<Expr>,
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
            Expr::FieldAccess { span, .. } => span,
            Expr::StructConstructor { span, .. } => span,
            Expr::Reference { span, .. } => span,
            Expr::MutableReference { span, .. } => span,
            Expr::Dereference { span, .. } => span,
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

// シンボル（lesson_3_14と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
    pub symbol_type: Type,
    pub definition_span: Span,
}

// スコープ（lesson_3_14と同じ）
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

// シンボルテーブル（lesson_3_14と同じ）
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

// ライフタイム推論エンジン
#[derive(Debug)]
pub struct LifetimeInferenceEngine {
    next_lifetime_id: usize,
    constraints: Vec<LifetimeConstraint>,
    inferred_lifetimes: HashMap<String, Lifetime>,
}

impl LifetimeInferenceEngine {
    pub fn new() -> Self {
        LifetimeInferenceEngine {
            next_lifetime_id: 0,
            constraints: Vec::new(),
            inferred_lifetimes: HashMap::new(),
        }
    }

    // 新しいライフタイム変数を生成
    pub fn fresh_lifetime(&mut self, span: Span) -> Lifetime {
        let name = format!("'_{}", self.next_lifetime_id);
        self.next_lifetime_id += 1;
        Lifetime::inferred(name, 0, span)
    }

    // ライフタイム制約を追加
    pub fn add_constraint(&mut self, constraint: LifetimeConstraint) {
        self.constraints.push(constraint);
    }

    // ライフタイム推論を実行
    pub fn infer_lifetimes(&mut self) -> Result<(), Vec<Diagnostic>> {
        // todo!("ライフタイム推論を実装してください")
        // ヒント：
        // 1. 制約を解決する
        // 2. 循環参照をチェック
        // 3. 推論結果を記録
        // 4. エラーがあれば診断情報を返す

        // 簡単な実装：制約をチェックするだけ
        let mut diagnostics = Vec::new();

        for constraint in &self.constraints {
            match constraint {
                LifetimeConstraint::Outlives {
                    longer,
                    shorter,
                    span,
                } => {
                    // 実際の実装では、ライフタイムの関係をチェックする
                    if longer == shorter {
                        diagnostics.push(Diagnostic::error(
                            format!("Lifetime '{}' cannot outlive itself", longer),
                            span.clone(),
                        ));
                    }
                }
                LifetimeConstraint::Equal { left, right, .. } => {
                    // 等価制約の処理
                    if left != right {
                        // 実際の実装では、統合を行う
                    }
                }
            }
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    // 関数の型からライフタイムパラメータを推論
    pub fn infer_function_lifetimes(
        &mut self,
        parameters: &[Parameter],
        return_type: &Option<Type>,
        span: &Span,
    ) -> Vec<Lifetime> {
        // todo!("関数のライフタイム推論を実装してください")
        // ヒント：
        // 1. パラメータ内の参照型を収集
        // 2. 戻り値型の参照を収集
        // 3. ライフタイム省略ルールを適用
        // 4. 必要なライフタイムパラメータを生成

        let mut lifetime_params = Vec::new();

        // パラメータ内の参照型をチェック
        for param in parameters {
            if param.has_lifetime {
                let lifetime = self.fresh_lifetime(span.clone());
                lifetime_params.push(lifetime);
            }
        }

        // 戻り値型の参照をチェック
        if let Some(return_type) = return_type {
            if return_type.is_reference() {
                if lifetime_params.is_empty() {
                    // 新しいライフタイムが必要
                    let lifetime = self.fresh_lifetime(span.clone());
                    lifetime_params.push(lifetime);
                }
            }
        }

        lifetime_params
    }

    pub fn get_constraints(&self) -> &Vec<LifetimeConstraint> {
        &self.constraints
    }

    pub fn get_inferred_lifetimes(&self) -> &HashMap<String, Lifetime> {
        &self.inferred_lifetimes
    }
}

// ライフタイム推論付き借用チェッカー
#[derive(Debug)]
pub struct LifetimeAwareBorrowChecker {
    symbol_table: SymbolTable,
    diagnostics: Vec<Diagnostic>,
    active_borrows: Vec<Borrow>,
    lifetime_inference: LifetimeInferenceEngine, // 新規追加
}

impl LifetimeAwareBorrowChecker {
    pub fn new() -> Self {
        LifetimeAwareBorrowChecker {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
            active_borrows: Vec::new(),
            lifetime_inference: LifetimeInferenceEngine::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Vec<Diagnostic> {
        for statement in &program.statements {
            self.check_statement(statement);
        }

        // ライフタイム推論を実行
        if let Err(inference_diagnostics) = self.lifetime_inference.infer_lifetimes() {
            self.diagnostics.extend(inference_diagnostics);
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
                let inferred_type = self.infer_expression_type(value).unwrap_or(Type::Unknown);

                if let Some(annotation) = type_annotation {
                    if *inferred_type.resolve() != *annotation.resolve() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Type mismatch: expected {:?}, found {:?}",
                                    annotation, inferred_type
                                ),
                                span.clone(),
                            )
                            .with_code("E0002".to_string()),
                        );
                    }
                }

                if let Err(diagnostic) =
                    self.symbol_table
                        .define(name.clone(), inferred_type, span.clone())
                {
                    self.diagnostics.push(diagnostic);
                }
            }
            Stmt::Expression(expr) => {
                self.infer_expression_type(expr);
            }
            Stmt::Block { statements, .. } => {
                self.symbol_table.enter_scope();

                // スコープ開始時のアクティブ借用数を記録
                let _borrows_before = self.active_borrows.len();

                for stmt in statements {
                    self.check_statement(stmt);
                }

                // スコープ終了時に該当スコープの借用を削除
                self.active_borrows
                    .retain(|borrow| borrow.lifetime.scope_level <= self.symbol_table.scope_level);

                self.symbol_table.exit_scope();
            }
            Stmt::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                if let Some(condition_type) = self.infer_expression_type(condition) {
                    if *condition_type.resolve() != Type::Boolean {
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
                if let Some(else_stmt) = else_branch {
                    self.check_statement(else_stmt);
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
                // ライフタイム推論を実行
                let lifetime_params =
                    self.lifetime_inference
                        .infer_function_lifetimes(parameters, return_type, span);

                let param_types: Vec<Type> = parameters
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Unknown))
                    .collect();

                let function_type = Type::Function {
                    parameters: param_types,
                    return_type: Box::new(return_type.clone().unwrap_or(Type::Unknown)),
                    lifetime_params, // 推論されたライフタイムパラメータを追加
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
            Stmt::StructDeclaration { name, fields, span } => {
                let struct_type = Type::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                };

                let mut field_names = std::collections::HashSet::new();
                for field in fields {
                    if !field_names.insert(&field.name) {
                        self.diagnostics.push(Diagnostic::error(
                            format!("Field '{}' is defined multiple times", field.name),
                            field.span.clone(),
                        ));
                    }
                }

                if let Err(diagnostic) =
                    self.symbol_table
                        .define(name.clone(), struct_type, span.clone())
                {
                    self.diagnostics.push(diagnostic);
                }
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
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);

                match (left_type, right_type) {
                    (Some(l), Some(r)) => match operator {
                        BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Divide => {
                            if *l.resolve() == Type::Integer && *r.resolve() == Type::Integer {
                                Some(Type::Integer)
                            } else {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "Arithmetic operation requires integers".to_string(),
                                        span.clone(),
                                    )
                                    .with_code("E0005".to_string()),
                                );
                                None
                            }
                        }
                        BinaryOp::GreaterThan | BinaryOp::LessThan => {
                            if *l.resolve() == Type::Integer && *r.resolve() == Type::Integer {
                                Some(Type::Boolean)
                            } else {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "Comparison requires integers".to_string(),
                                        span.clone(),
                                    )
                                    .with_code("E0005".to_string()),
                                );
                                None
                            }
                        }
                        BinaryOp::Equal | BinaryOp::NotEqual => {
                            if l.resolve() == r.resolve() {
                                Some(Type::Boolean)
                            } else {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "Equality comparison requires same types".to_string(),
                                        span.clone(),
                                    )
                                    .with_code("E0005".to_string()),
                                );
                                None
                            }
                        }
                    },
                    _ => None,
                }
            }
            Expr::FunctionCall {
                name,
                arguments,
                span,
            } => {
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
                    lifetime_params,
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
                            .with_code("E0006".to_string()),
                        );
                    }

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
                                    .with_code("E0007".to_string()),
                                );
                            }
                        }
                    }

                    // ライフタイム制約を追加
                    self.add_function_call_constraints(&lifetime_params, arguments);

                    Some(*return_type)
                } else {
                    None
                }
            }
            Expr::Assignment { name, value, span } => {
                if self.symbol_table.resolve(name).is_none() {
                    self.diagnostics.push(
                        Diagnostic::error(format!("Variable '{}' not defined", name), span.clone())
                            .with_code("E0004".to_string()),
                    );
                    return None;
                }

                self.infer_expression_type(value)
            }
            Expr::FieldAccess {
                object,
                field_name,
                span,
            } => {
                if let Some(object_type) = self.infer_expression_type(object) {
                    if let Some(field) = object_type.get_field(field_name) {
                        Some(field.field_type.clone())
                    } else {
                        match object_type.resolve() {
                            Type::Struct { name, .. } => {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("Struct '{}' has no field '{}'", name, field_name),
                                    span.clone(),
                                ));
                            }
                            _ => {
                                self.diagnostics.push(Diagnostic::error(
                                    format!(
                                        "Cannot access field '{}' on non-struct type {:?}",
                                        field_name,
                                        object_type.resolve()
                                    ),
                                    span.clone(),
                                ));
                            }
                        }
                        None
                    }
                } else {
                    None
                }
            }
            Expr::StructConstructor {
                struct_name,
                field_values,
                span,
            } => {
                let struct_symbol = if let Some(symbol) = self.symbol_table.resolve(struct_name) {
                    symbol.clone()
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!("Struct '{}' not defined", struct_name),
                            span.clone(),
                        )
                        .with_code("E0004".to_string()),
                    );
                    return None;
                };

                if let Type::Struct { fields, .. } = &struct_symbol.symbol_type {
                    for (field_name, _value) in field_values {
                        if !fields.iter().any(|f| f.name == *field_name) {
                            self.diagnostics.push(Diagnostic::error(
                                format!("Struct '{}' has no field '{}'", struct_name, field_name),
                                span.clone(),
                            ));
                        }
                    }

                    for (field_name, value) in field_values {
                        if let Some(field) = fields.iter().find(|f| f.name == *field_name) {
                            if let Some(value_type) = self.infer_expression_type(value) {
                                if *value_type.resolve() != *field.field_type.resolve() {
                                    self.diagnostics.push(Diagnostic::error(
                                        format!("Type mismatch for field '{}': expected {:?}, found {:?}",
                                            field_name, field.field_type, value_type),
                                        value.span().clone()
                                    ));
                                }
                            }
                        }
                    }

                    Some(struct_symbol.symbol_type.clone())
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        format!("'{}' is not a struct", struct_name),
                        span.clone(),
                    ));
                    None
                }
            }
            Expr::Reference { inner, .. } => {
                let inner_type = self.infer_expression_type(inner)?;

                match inner.as_ref() {
                    Expr::Identifier(var_name, span) => {
                        let borrow =
                            self.create_borrow(var_name.clone(), BorrowKind::Immutable, span)?;

                        // 競合チェックを先に実行（自分自身を除外）
                        self.check_borrow_conflicts(&borrow);
                        self.active_borrows.push(borrow.clone());

                        Some(Type::Reference {
                            inner_type: Box::new(inner_type),
                            lifetime: Some(borrow.lifetime),
                            mutability: BorrowKind::Immutable,
                        })
                    }
                    _ => Some(Type::Reference {
                        inner_type: Box::new(inner_type),
                        lifetime: None,
                        mutability: BorrowKind::Immutable,
                    }),
                }
            }
            Expr::MutableReference { inner, .. } => {
                let inner_type = self.infer_expression_type(inner)?;

                match inner.as_ref() {
                    Expr::Identifier(var_name, span) => {
                        let borrow =
                            self.create_borrow(var_name.clone(), BorrowKind::Mutable, span)?;

                        // 競合チェックを先に実行（自分自身を除外）
                        self.check_borrow_conflicts(&borrow);
                        self.active_borrows.push(borrow.clone());

                        Some(Type::Reference {
                            inner_type: Box::new(inner_type),
                            lifetime: Some(borrow.lifetime),
                            mutability: BorrowKind::Mutable,
                        })
                    }
                    _ => Some(Type::Reference {
                        inner_type: Box::new(inner_type),
                        lifetime: None,
                        mutability: BorrowKind::Mutable,
                    }),
                }
            }
            Expr::Dereference { inner, span } => {
                if let Some(inner_type) = self.infer_expression_type(inner) {
                    match inner_type.get_inner_type() {
                        Some(deref_type) => Some(deref_type.clone()),
                        _ => {
                            self.diagnostics.push(Diagnostic::error(
                                format!(
                                    "Cannot dereference non-reference type {:?}",
                                    inner_type.resolve()
                                ),
                                span.clone(),
                            ));
                            None
                        }
                    }
                } else {
                    None
                }
            }
        }
    }

    // 借用チェック関連のヘルパーメソッド（lesson_3_14と同じ）
    fn check_borrow_conflicts(&mut self, new_borrow: &Borrow) {
        self.active_borrows
            .iter()
            .filter(|borrow| borrow.variable == new_borrow.variable)
            .for_each(|existing_borrow| {
                match (existing_borrow.kind.clone(), new_borrow.kind.clone()) {
                    (BorrowKind::Mutable, _) | (_, BorrowKind::Mutable) => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                "Cannot borrow `{}` as {:?} because it is already borrowed as {:?}",
                                new_borrow.variable,
                                new_borrow.kind,
                                existing_borrow.kind
                            ),
                                new_borrow.creation_span.clone(),
                            )
                            .with_code("E0502".to_string()),
                        );
                    }
                    // Rule 2: 複数の不変借用は OK
                    (BorrowKind::Immutable, BorrowKind::Immutable) => {
                        // 競合なし
                    }
                }
            })
    }

    fn create_borrow(&mut self, variable: String, kind: BorrowKind, span: &Span) -> Option<Borrow> {
        // ライフタイムを推論エンジンから生成
        let lifetime = self.create_lifetime_for_variable(&variable, span)?;

        Some(Borrow::new(variable.clone(), kind, lifetime, span.clone()))
    }

    fn create_lifetime_for_variable(&mut self, var_name: &str, span: &Span) -> Option<Lifetime> {
        if let Some(_symbol) = self.symbol_table.resolve(var_name) {
            // ライフタイム推論エンジンを使用して新しいライフタイムを生成
            Some(self.lifetime_inference.fresh_lifetime(span.clone()))
        } else {
            None
        }
    }

    // 関数呼び出しのライフタイム制約を追加
    fn add_function_call_constraints(&mut self, lifetime_params: &[Lifetime], arguments: &[Expr]) {
        // todo!("関数呼び出しのライフタイム制約を実装してください")
        // ヒント：
        // 1. 引数の参照型からライフタイムを抽出
        // 2. ライフタイムパラメータとの関係を制約として追加
        // 3. 戻り値のライフタイム制約を追加

        for (i, arg) in arguments.iter().enumerate() {
            if let Some(lifetime_param) = lifetime_params.get(i) {
                // 引数が参照型の場合、ライフタイム制約を追加
                if let Some(arg_type) = self.infer_expression_type(arg) {
                    if let Some(arg_lifetime) = arg_type.get_lifetime() {
                        // 引数のライフタイムがパラメータのライフタイム以上長生きする制約
                        self.lifetime_inference
                            .add_constraint(LifetimeConstraint::Outlives {
                                longer: arg_lifetime.name.clone(),
                                shorter: lifetime_param.name.clone(),
                                span: arg.span().clone(),
                            });
                    }
                }
            }
        }
    }

    pub fn get_diagnostics(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_active_borrows(&self) -> &Vec<Borrow> {
        &self.active_borrows
    }

    pub fn get_lifetime_inference(&self) -> &LifetimeInferenceEngine {
        &self.lifetime_inference
    }
}

// 公開API
pub fn check_with_lifetime_inference(program: &Program) -> Vec<Diagnostic> {
    let mut checker = LifetimeAwareBorrowChecker::new();
    checker.check_program(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lifetime_inference() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(42, Span::single(Position::new(0, 8))),
                    type_annotation: None,
                    span: Span::new(Position::new(0, 0), Position::new(0, 14)),
                },
                Stmt::LetDeclaration {
                    name: "r".to_string(),
                    value: Expr::Reference {
                        inner: Box::new(Expr::Identifier(
                            "x".to_string(),
                            Span::single(Position::new(1, 9)),
                        )),
                        span: Span::single(Position::new(1, 8)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(1, 0), Position::new(1, 10)),
                },
            ],
        };

        let diagnostics = check_with_lifetime_inference(&program);
        assert!(diagnostics.is_empty()); // エラーなし
    }

    #[test]
    fn test_function_lifetime_inference() {
        let program = Program {
            statements: vec![Stmt::FunctionDeclaration {
                name: "get_first".to_string(),
                parameters: vec![
                    Parameter {
                        name: "x".to_string(),
                        param_type: Some(Type::Reference {
                            inner_type: Box::new(Type::String),
                            lifetime: None,
                            mutability: BorrowKind::Immutable,
                        }),
                        span: Span::single(Position::new(0, 14)),
                        has_lifetime: true, // 参照型なのでライフタイムあり
                    },
                    Parameter {
                        name: "y".to_string(),
                        param_type: Some(Type::Reference {
                            inner_type: Box::new(Type::String),
                            lifetime: None,
                            mutability: BorrowKind::Immutable,
                        }),
                        span: Span::single(Position::new(0, 25)),
                        has_lifetime: true, // 参照型なのでライフタイムあり
                    },
                ],
                return_type: Some(Type::Reference {
                    inner_type: Box::new(Type::String),
                    lifetime: None,
                    mutability: BorrowKind::Immutable,
                }),
                body: Box::new(Stmt::Expression(Expr::Identifier(
                    "x".to_string(),
                    Span::single(Position::new(0, 50)),
                ))),
                span: Span::new(Position::new(0, 0), Position::new(0, 52)),
            }],
        };

        let diagnostics = check_with_lifetime_inference(&program);
        assert!(diagnostics.is_empty()); // ライフタイム推論が成功
    }

    #[test]
    fn test_lifetime_constraint_generation() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(42, Span::single(Position::new(0, 8))),
                    type_annotation: None,
                    span: Span::new(Position::new(0, 0), Position::new(0, 14)),
                },
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Number(10, Span::single(Position::new(1, 8))),
                    type_annotation: None,
                    span: Span::new(Position::new(1, 0), Position::new(1, 14)),
                },
                Stmt::LetDeclaration {
                    name: "r1".to_string(),
                    value: Expr::Reference {
                        inner: Box::new(Expr::Identifier(
                            "x".to_string(),
                            Span::single(Position::new(2, 10)),
                        )),
                        span: Span::single(Position::new(2, 9)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(2, 0), Position::new(2, 11)),
                },
                Stmt::LetDeclaration {
                    name: "r2".to_string(),
                    value: Expr::Reference {
                        inner: Box::new(Expr::Identifier(
                            "y".to_string(),
                            Span::single(Position::new(3, 10)),
                        )),
                        span: Span::single(Position::new(3, 9)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(3, 0), Position::new(3, 11)),
                },
            ],
        };

        let mut checker = LifetimeAwareBorrowChecker::new();
        let diagnostics = checker.check_program(&program);

        assert!(diagnostics.is_empty()); // エラーなし

        // ライフタイム推論エンジンが制約を生成していることを確認
        let inference_engine = checker.get_lifetime_inference();
        assert!(!inference_engine.get_inferred_lifetimes().is_empty());
    }

    #[test]
    fn test_multiple_reference_types() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "data".to_string(),
                    value: Expr::String("hello".to_string(), Span::single(Position::new(0, 10))),
                    type_annotation: None,
                    span: Span::new(Position::new(0, 0), Position::new(0, 17)),
                },
                Stmt::LetDeclaration {
                    name: "ref1".to_string(),
                    value: Expr::Reference {
                        inner: Box::new(Expr::Identifier(
                            "data".to_string(),
                            Span::single(Position::new(1, 11)),
                        )),
                        span: Span::single(Position::new(1, 10)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(1, 0), Position::new(1, 15)),
                },
                Stmt::LetDeclaration {
                    name: "ref2".to_string(),
                    value: Expr::Reference {
                        inner: Box::new(Expr::Identifier(
                            "data".to_string(),
                            Span::single(Position::new(2, 11)),
                        )),
                        span: Span::single(Position::new(2, 10)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(2, 0), Position::new(2, 15)),
                },
            ],
        };

        let diagnostics = check_with_lifetime_inference(&program);
        assert!(diagnostics.is_empty()); // 複数の不変借用は OK
    }
}
