// Lesson 3-12へようこそ！
// lesson_3_11でエラー回復システムができるようになりましたね。
// 今度は、構造体とフィールドアクセスを学びます。

// あなたのタスク：
// 構造体の定義とフィールドアクセスを実装してください。
// 例：struct Person { name: String } と person.name のような操作

use std::collections::HashMap;

// 位置情報（lesson_3_11と同じ）
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

// 診断情報（lesson_3_11と同じ）
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

// フィールド定義
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub span: Span,
}

// 型情報（構造体型を追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    String,
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    // 新規追加：構造体型
    Struct {
        name: String,
        fields: Vec<Field>,
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
}

// AST構造（構造体とフィールドアクセスを追加）
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
    // 新規追加：構造体定義
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
    // 新規追加：フィールドアクセス
    FieldAccess {
        object: Box<Expr>,
        field_name: String,
        span: Span,
    },
    // 新規追加：構造体コンストラクタ
    StructConstructor {
        struct_name: String,
        field_values: Vec<(String, Expr)>, // (フィールド名, 値)
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

// シンボル（構造体情報を追加）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
    pub symbol_type: Type,
    pub definition_span: Span,
}

// スコープ（lesson_3_11と同じ）
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

// シンボルテーブル（lesson_3_11と同じ）
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

// 構造体対応型チェッカー
#[derive(Debug)]
pub struct StructTypeChecker {
    symbol_table: SymbolTable,
    diagnostics: Vec<Diagnostic>,
}

impl StructTypeChecker {
    pub fn new() -> Self {
        StructTypeChecker {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Vec<Diagnostic> {
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
            Stmt::StructDeclaration { name, fields, span } => {
                // TODO
                // ヒント：
                // 1. 構造体型を作成
                // 2. フィールドの重複チェック
                // 3. 構造体をシンボルテーブルに定義
                let s = Type::Struct {
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

                if let Err(diagnostic) = self.symbol_table.define(name.clone(), s, span.clone()) {
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
                // TODO
                // ヒント：
                // 1. オブジェクトの型を推論
                // 2. 構造体型かチェック
                // 3. フィールドが存在するかチェック
                // 4. フィールドの型を返す
                match self.infer_expression_type(object) {
                    Some(Type::Struct { name: _, fields }) => {
                        match fields.iter().find(|f| f.name == *field_name) {
                            Some(field) => Some(field.field_type.clone()),
                            None => {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("no field: {}", field_name),
                                    span.clone(),
                                ));
                                None
                            }
                        }
                    }
                    Some(other) => {
                        self.diagnostics.push(Diagnostic::error(
                            format!("{:?} is not a struct", other),
                            span.clone(),
                        ));
                        None
                    }
                    None => None,
                }
            }
            Expr::StructConstructor {
                struct_name,
                field_values,
                span,
            } => {
                // todo!("構造体コンストラクタの処理を実装してください")
                // ヒント：
                // 1. 構造体型が定義されているかチェック
                // 2. 提供されたフィールドが全て存在するかチェック
                // 3. 各フィールドの値の型をチェック
                // 4. 構造体型を返す

                // Step 1: 構造体型が定義されているかチェック
                let struct_symbol = self.symbol_table.resolve(struct_name)?.clone();

                if let Type::Struct { fields, .. } = &struct_symbol.symbol_type {
                    // Step 2: 提供されたフィールドが全て存在するかチェック
                    for (field_name, _value) in field_values {
                        if !fields.iter().any(|f| f.name == *field_name) {
                            self.diagnostics.push(Diagnostic::error(
                                format!("Struct '{}' has no field '{}'", struct_name, field_name),
                                span.clone(),
                            ));
                        }
                    }

                    // Step 3: 各フィールドの値の型をチェック
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

                    // Step 4: 構造体型を返す
                    Some(struct_symbol.symbol_type.clone())
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        format!("'{}' is not a struct", struct_name),
                        span.clone(),
                    ));
                    None
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
}

// 公開API
pub fn check_with_structs(program: &Program) -> Vec<Diagnostic> {
    let mut checker = StructTypeChecker::new();
    checker.check_program(program)
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_struct_definition() {
        let program = Program {
            statements: vec![
                Stmt::StructDeclaration {
                    name: "Person".to_string(),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            field_type: Type::String,
                            span: Span::single(Position::new(1, 4)),
                        },
                        Field {
                            name: "age".to_string(),
                            field_type: Type::Integer,
                            span: Span::single(Position::new(2, 4)),
                        },
                    ],
                    span: Span::new(Position::new(0, 0), Position::new(3, 1)),
                },
                Stmt::LetDeclaration {
                    name: "person".to_string(),
                    value: Expr::StructConstructor {
                        struct_name: "Person".to_string(),
                        field_values: vec![
                            (
                                "name".to_string(),
                                Expr::String(
                                    "Alice".to_string(),
                                    Span::single(Position::new(4, 12)),
                                ),
                            ),
                            (
                                "age".to_string(),
                                Expr::Number(25, Span::single(Position::new(4, 26))),
                            ),
                        ],
                        span: Span::single(Position::new(4, 8)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(4, 0), Position::new(4, 30)),
                },
            ],
        };

        let diagnostics = check_with_structs(&program);
        assert!(diagnostics.is_empty()); // エラーなし
    }

    #[test]
    fn test_field_access() {
        let program = Program {
            statements: vec![
                Stmt::StructDeclaration {
                    name: "Point".to_string(),
                    fields: vec![
                        Field {
                            name: "x".to_string(),
                            field_type: Type::Integer,
                            span: Span::single(Position::new(1, 4)),
                        },
                        Field {
                            name: "y".to_string(),
                            field_type: Type::Integer,
                            span: Span::single(Position::new(2, 4)),
                        },
                    ],
                    span: Span::new(Position::new(0, 0), Position::new(3, 1)),
                },
                Stmt::LetDeclaration {
                    name: "point".to_string(),
                    value: Expr::StructConstructor {
                        struct_name: "Point".to_string(),
                        field_values: vec![
                            (
                                "x".to_string(),
                                Expr::Number(10, Span::single(Position::new(4, 14))),
                            ),
                            (
                                "y".to_string(),
                                Expr::Number(20, Span::single(Position::new(4, 21))),
                            ),
                        ],
                        span: Span::single(Position::new(4, 8)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(4, 0), Position::new(4, 25)),
                },
                Stmt::Expression(Expr::FieldAccess {
                    object: Box::new(Expr::Identifier(
                        "point".to_string(),
                        Span::single(Position::new(5, 0)),
                    )),
                    field_name: "x".to_string(),
                    span: Span::new(Position::new(5, 0), Position::new(5, 7)),
                }),
            ],
        };

        let diagnostics = check_with_structs(&program);
        assert!(diagnostics.is_empty()); // エラーなし
    }

    #[test]
    fn test_field_access_errors() {
        let program = Program {
            statements: vec![
                Stmt::StructDeclaration {
                    name: "Person".to_string(),
                    fields: vec![Field {
                        name: "name".to_string(),
                        field_type: Type::String,
                        span: Span::single(Position::new(1, 4)),
                    }],
                    span: Span::new(Position::new(0, 0), Position::new(2, 1)),
                },
                Stmt::Expression(Expr::FieldAccess {
                    object: Box::new(Expr::Number(42, Span::single(Position::new(3, 0)))), // 構造体ではない
                    field_name: "name".to_string(),
                    span: Span::new(Position::new(3, 0), Position::new(3, 7)),
                }),
                Stmt::LetDeclaration {
                    name: "person".to_string(),
                    value: Expr::StructConstructor {
                        struct_name: "Person".to_string(),
                        field_values: vec![(
                            "name".to_string(),
                            Expr::String("Bob".to_string(), Span::single(Position::new(4, 12))),
                        )],
                        span: Span::single(Position::new(4, 8)),
                    },
                    type_annotation: None,
                    span: Span::new(Position::new(4, 0), Position::new(4, 20)),
                },
                Stmt::Expression(Expr::FieldAccess {
                    object: Box::new(Expr::Identifier(
                        "person".to_string(),
                        Span::single(Position::new(5, 0)),
                    )),
                    field_name: "age".to_string(), // 存在しないフィールド
                    span: Span::new(Position::new(5, 0), Position::new(5, 10)),
                }),
            ],
        };

        let diagnostics = check_with_structs(&program);
        assert_eq!(diagnostics.len(), 2);

        // 1つ目のエラー：数値に対するフィールドアクセス
        assert!(diagnostics[0].message.contains("not a struct"));

        // 2つ目のエラー：存在しないフィールドへのアクセス
        assert!(diagnostics[1].message.contains("no field"));
    }
}
