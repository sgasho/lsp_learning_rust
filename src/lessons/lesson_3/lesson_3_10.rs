// Lesson 3-10へようこそ！
// lesson_3_9で基本的な型システムができるようになりましたね。
// 今度は、型推論をより高度化します。

// あなたのタスク：
// 型注釈がない場合の型推論を強化してください。
// 例：let x = 5; のように型注釈がなくても型を推論する

use std::collections::HashMap;

// 型情報（lesson_3_9から拡張）
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
    // 新規追加：推論中の型
    Inferred(Box<Type>), // 推論された型を表す
}

impl Type {
    // 推論された型を実際の型に変換
    pub fn resolve(&self) -> &Type {
        match self {
            Type::Inferred(inner) => inner.resolve(),
            other => other,
        }
    }

    // 推論された型かどうか判定
    pub fn is_inferred(&self) -> bool {
        matches!(self, Type::Inferred(_))
    }
}

// AST構造（lesson_3_9と同じ）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
        type_annotation: Option<Type>,
    },
    Expression(Expr),
    Block {
        statements: Vec<Stmt>,
    },
    IfStatement {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    WhileStatement {
        condition: Expr,
        body: Box<Stmt>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Boolean(bool),
    String(String),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
    },
    // 新規追加：変数への代入式
    Assignment {
        name: String,
        value: Box<Expr>,
    },
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

// シンボル（lesson_3_9と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
    pub symbol_type: Type,
}

// スコープ（lesson_3_9と同じ）
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

// シンボルテーブル（lesson_3_9と同じ）
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

    pub fn define(&mut self, name: String, symbol_type: Type) -> Result<(), String> {
        if self.current_scope.symbols.contains_key(&name) {
            return Err(format!("Variable '{}' already defined in this scope", name));
        }

        let symbol = Symbol {
            name: name.clone(),
            scope_level: self.scope_level,
            symbol_type,
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

    // 新規追加：変数の型を更新する（型推論で使用）
    pub fn update_type(&mut self, name: &str, new_type: Type) -> Result<(), String> {
        if let Some(symbol) = self.current_scope.symbols.get_mut(name) {
            symbol.symbol_type = new_type;
            return Ok(());
        }

        // 親スコープも探索
        if let Some(_parent) = &mut self.current_scope.parent {
            // Note: この実装は簡略化。実際にはより複雑
            return Err(format!("Cannot update type for variable '{}'", name));
        }

        Err(format!("Variable '{}' not found for type update", name))
    }
}

// 高度な型チェッカー
#[derive(Debug)]
pub struct AdvancedTypeChecker {
    symbol_table: SymbolTable,
    errors: Vec<String>,
    // 新規追加：型推論のためのコンテキスト
    inference_context: Vec<String>, // 推論が必要な変数のリスト
}

impl AdvancedTypeChecker {
    pub fn new() -> Self {
        AdvancedTypeChecker {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            inference_context: Vec::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
        // Phase 1: 基本的な型チェック
        for statement in &program.statements {
            if let Err(e) = self.check_statement(statement) {
                self.errors.push(e);
            }
        }

        // Phase 2: 型推論の解決
        if let Err(e) = self.resolve_type_inference() {
            self.errors.push(e);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::LetDeclaration {
                name,
                value,
                type_annotation,
            } => {
                // todo!("型注釈なしの変数定義を実装してください")
                // ヒント：
                // 1. 値の型を推論
                // 2. 型注釈がある場合は一致確認
                // 3. 型注釈がない場合は推論された型を使用
                // 4. 推論された型がUnknownの場合は後で解決するためマーク
                let inferred_value_type = self.infer_expression_type(value)?;

                if type_annotation
                    .clone()
                    .is_some_and(|annotation| annotation.ne(&inferred_value_type))
                {
                    return Err("Type mismatch".to_string());
                }

                self.symbol_table.define(name.clone(), inferred_value_type)
            }
            Stmt::Expression(expr) => {
                self.infer_expression_type(expr)?;
                Ok(())
            }
            Stmt::Block { statements } => {
                self.symbol_table.enter_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.symbol_table.exit_scope();
                Ok(())
            }
            Stmt::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.infer_expression_type(condition)?;
                if *condition_type.resolve() != Type::Boolean {
                    return Err(format!(
                        "If condition must be boolean, found {:?}",
                        condition_type.resolve()
                    ));
                }

                self.check_statement(then_branch)?;
                if let Some(else_stmt) = else_branch {
                    self.check_statement(else_stmt)?;
                }
                Ok(())
            }
            Stmt::WhileStatement { condition, body } => {
                let condition_type = self.infer_expression_type(condition)?;
                if *condition_type.resolve() != Type::Boolean {
                    return Err(format!(
                        "While condition must be boolean, found {:?}",
                        condition_type.resolve()
                    ));
                }

                self.check_statement(body)?;
                Ok(())
            }
            Stmt::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let param_types: Vec<Type> = parameters
                    .iter()
                    .map(|p| p.param_type.clone().unwrap_or(Type::Unknown))
                    .collect();

                let function_type = Type::Function {
                    parameters: param_types,
                    return_type: Box::new(return_type.clone().unwrap_or(Type::Unknown)),
                };

                self.symbol_table.define(name.clone(), function_type)?;
                self.symbol_table.enter_scope();

                for param in parameters {
                    let param_type = param.param_type.clone().unwrap_or(Type::Unknown);
                    self.symbol_table.define(param.name.clone(), param_type)?;
                }

                self.check_statement(body)?;
                self.symbol_table.exit_scope();
                Ok(())
            }
        }
    }

    fn infer_expression_type(&mut self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Number(_) => Ok(Type::Integer),
            Expr::Boolean(_) => Ok(Type::Boolean),
            Expr::String(_) => Ok(Type::String),
            Expr::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.resolve(name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(format!("Variable '{}' not defined", name))
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // todo!("高度な二項演算の型推論を実装してください")
                // ヒント：
                // 1. 左右の型を推論
                // 2. 推論された型を解決してから比較
                // 3. オペレーターに応じた型チェック
                let left_inferred = self.infer_expression_type(left)?;
                let right_inferred = self.infer_expression_type(right)?;

                let resolved_left = left_inferred.resolve();
                let resolved_right = right_inferred.resolve();

                match operator {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if *resolved_left == Type::Integer && *resolved_right == Type::Integer {
                            Ok(Type::Integer)
                        } else {
                            Err("Arithmetic requires integers".to_string())
                        }
                    }
                    BinaryOp::GreaterThan | BinaryOp::LessThan => {
                        if *resolved_left == Type::Integer && *resolved_right == Type::Integer {
                            Ok(Type::Boolean)
                        } else {
                            Err("Arithmetic requires integers".to_string())
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if resolved_left.ne(resolved_right) {
                            Err(format!(
                                "left type: {:?} is not equal to right type: {:?}",
                                resolved_left, resolved_right
                            ))
                        } else {
                            Ok(Type::Boolean)
                        }
                    }
                }
            }
            Expr::FunctionCall { name, arguments } => {
                let function_type = if let Some(symbol) = self.symbol_table.resolve(name) {
                    symbol.symbol_type.clone()
                } else {
                    return Err(format!("Function '{}' not defined", name));
                };

                if let Type::Function {
                    parameters,
                    return_type,
                } = function_type
                {
                    if arguments.len() != parameters.len() {
                        return Err(format!(
                            "Function '{}' expects {} arguments, found {}",
                            name,
                            parameters.len(),
                            arguments.len()
                        ));
                    }

                    for (arg, expected_type) in arguments.iter().zip(parameters.iter()) {
                        let arg_type = self.infer_expression_type(arg)?;
                        if *arg_type.resolve() != *expected_type.resolve()
                            && *expected_type.resolve() != Type::Unknown
                        {
                            return Err(format!(
                                "Argument type mismatch: expected {:?}, found {:?}",
                                expected_type.resolve(),
                                arg_type.resolve()
                            ));
                        }
                    }

                    Ok(*return_type)
                } else {
                    Err(format!("'{}' is not a function", name))
                }
            }
            Expr::Assignment { name, value } => {
                // todo!("代入式の型チェックを実装してください")
                // ヒント：
                // 1. 変数が定義されているかチェック
                // 2. 値の型を推論
                // 3. 既存の変数の型と一致するかチェック
                // 4. 代入される値の型を返す
                match self.symbol_table.resolve(name) {
                    Some(symbol) => {
                        let got_type = symbol.clone().symbol_type;
                        let inferred = self.infer_expression_type(value)?;
                        let resolved_type = inferred.resolve();
                        if got_type.ne(resolved_type) {
                            return Err(format!(
                                "Type mismatch: expected {:?}, found {:?}",
                                resolved_type, value
                            ));
                        }
                        Ok(got_type)
                    }
                    None => Err(format!("Variable '{}' not defined", name)),
                }
            }
        }
    }

    // 新規追加：型推論の解決
    fn resolve_type_inference(&mut self) -> Result<(), String> {
        // 簡単な型推論アルゴリズム
        // 実際のrust-analyzerはもっと複雑ですが、基本的な考え方を学習

        for var_name in &self.inference_context.clone() {
            // ヒント：
            // 1. 変数の使用箇所を探す
            // 2. 使用方法から型を推論
            // 3. 推論した型で変数の型を更新

            // 今回は簡単な実装として、Unknown型はIntegerとして推論
            if let Some(_symbol) = self.symbol_table.resolve(var_name) {
                // 実際の推論ロジックはここに実装
                // 今回は簡略化
            }
        }

        Ok(())
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }
}

// 公開API
pub fn check_advanced_types(program: &Program) -> Result<SymbolTable, Vec<String>> {
    let mut checker = AdvancedTypeChecker::new();
    match checker.check_program(program) {
        Ok(()) => Ok(checker.symbol_table),
        Err(errors) => Err(errors),
    }
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference_without_annotation() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(42),
                    type_annotation: None, // 型注釈なし
                },
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Number(10)),
                    },
                    type_annotation: None, // 型注釈なし
                },
            ],
        };

        let result = check_advanced_types(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        let x_symbol = table.resolve("x").unwrap();
        assert_eq!(*x_symbol.symbol_type.resolve(), Type::Integer);

        let y_symbol = table.resolve("y").unwrap();
        assert_eq!(*y_symbol.symbol_type.resolve(), Type::Integer);
    }

    #[test]
    fn test_boolean_inference() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "flag".to_string(),
                    value: Expr::Boolean(true),
                    type_annotation: None,
                },
                Stmt::LetDeclaration {
                    name: "result".to_string(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Number(5)),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expr::Number(3)),
                    },
                    type_annotation: None,
                },
            ],
        };

        let result = check_advanced_types(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        let flag_symbol = table.resolve("flag").unwrap();
        assert_eq!(*flag_symbol.symbol_type.resolve(), Type::Boolean);

        let result_symbol = table.resolve("result").unwrap();
        assert_eq!(*result_symbol.symbol_type.resolve(), Type::Boolean);
    }

    #[test]
    fn test_assignment_type_check() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(42),
                    type_annotation: None,
                },
                Stmt::Expression(Expr::Assignment {
                    name: "x".to_string(),
                    value: Box::new(Expr::Number(100)), // OK: Integer代入
                }),
                Stmt::Expression(Expr::Assignment {
                    name: "x".to_string(),
                    value: Box::new(Expr::Boolean(true)), // Error: Boolean代入
                }),
            ],
        };

        let result = check_advanced_types(&program);
        assert!(result.is_err()); // 型不一致エラーが発生
    }
}
