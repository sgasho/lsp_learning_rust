// Lesson 3-9へようこそ！
// lesson_3_8で関数のスコープができるようになりましたね。
// 今度は、基本的な型システムを学びます。

// あなたのタスク：
// 基本的な型システムを実装してください。
// 例：変数に型情報を付与し、型の不一致を検出する

use std::collections::HashMap;

// 基本型の定義
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer, // i32
    Boolean, // bool
    String,  // String
    Function {
        // 関数型（パラメータ型 -> 戻り値型）
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    Unknown, // 型が不明
}

// AST構造（型注釈を追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
        type_annotation: Option<Type>, // 型注釈（let x: i32 = 5; の場合）
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
        return_type: Option<Type>, // 戻り値型注釈
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>, // パラメータ型注釈
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Boolean(bool),  // 新規追加
    String(String), // 新規追加
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,
    Equal,    // 新規追加：==
    NotEqual, // 新規追加：!=
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// シンボル（変数）の情報（型情報を追加）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
    pub symbol_type: Type, // 型情報を追加
}

// スコープ構造（lesson_3_8と同じ）
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

// シンボルテーブル（lesson_3_8と同じ）
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

    // 変数を定義（型情報付き）
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
}

// 型チェッカー
#[derive(Debug)]
pub struct TypeChecker {
    symbol_table: SymbolTable,
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    // プログラム全体を型チェック
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
        for statement in &program.statements {
            if let Err(e) = self.check_statement(statement) {
                self.errors.push(e);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    // 個別の文をチェック
    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::LetDeclaration {
                name,
                value,
                type_annotation,
            } => {
                // TODO
                let inferred_type = self.infer_expression_type(value)?;
                match type_annotation {
                    Some(type_annotation) => {
                        if inferred_type.ne(type_annotation) {
                            return Err("Unmatched type annotation".to_string());
                        }
                    }
                    None => {}
                }

                self.symbol_table.define(name.clone(), inferred_type)?;

                Ok(())
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
                if condition_type != Type::Boolean {
                    return Err(format!(
                        "If condition must be boolean, found {:?}",
                        condition_type
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
                if condition_type != Type::Boolean {
                    return Err(format!(
                        "While condition must be boolean, found {:?}",
                        condition_type
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
                // TODO
                // ヒント：
                // 1. パラメータ型をVec<Type>に変換: let param_types: Vec<Type> = parameters.iter().map(|p| p.param_type.clone().unwrap_or(Type::Unknown)).collect();
                // 2. 関数型を作成: Type::Function { parameters: param_types, return_type: Box::new(return_type.clone().unwrap_or(Type::Unknown)) }
                // 3. self.symbol_table.define(name.clone(), function_type)? で関数定義
                // 4. self.symbol_table.enter_scope() でスコープ開始
                // 5. パラメータを定義: for param in parameters { self.symbol_table.define(param.name.clone(), param.param_type.clone().unwrap_or(Type::Unknown))? }
                // 6. self.check_statement(body)? で本体チェック
                // 7. self.symbol_table.exit_scope() でスコープ終了

                let fun_type = Type::Function {
                    parameters: parameters
                        .iter()
                        .map(|param| param.param_type.clone().unwrap_or(Type::Unknown))
                        .collect(),
                    return_type: Box::new(return_type.clone().unwrap_or(Type::Unknown)),
                };

                self.symbol_table.define(name.clone(), fun_type)?;

                self.symbol_table.enter_scope();

                self.check_statement(body)?;

                self.symbol_table.exit_scope();

                Ok(())
            }
        }
    }

    // 式の型を推論
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
                // TODO
                // ヒント：
                // 1. let left_type = self.infer_expression_type(left)?;
                // 2. let right_type = self.infer_expression_type(right)?;
                // 3. オペレーター別に型チェック
                //    - Add/Sub/Mul/Div: Integer + Integer = Integer
                //    - GreaterThan/LessThan: Integer + Integer = Boolean
                //    - Equal/NotEqual: 同じ型同士 = Boolean

                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;

                match operator {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if left_type.ne(&right_type) {
                            return Err(format!(
                                "Type mismatch: left = {:?} but right = {:?}",
                                left_type, right_type
                            ));
                        }

                        if left_type.ne(&Type::Integer) {
                            return Err(format!(
                                "Arithmetic operations can only be applied to integers, got: {:?}",
                                left_type
                            ));
                        }

                        Ok(left_type)
                    }
                    BinaryOp::GreaterThan | BinaryOp::LessThan => {
                        if left_type.ne(&right_type) {
                            return Err(format!(
                                "Type mismatch: left = {:?} but right = {:?}",
                                left_type, right_type
                            ));
                        }

                        if left_type.ne(&Type::Integer) {
                            return Err(format!(
                                "Arithmetic operations can only be applied to integers, got: {:?}",
                                left_type
                            ));
                        }

                        Ok(Type::Boolean)
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if left_type.ne(&right_type) {
                            return Err(format!(
                                "Type mismatch: left = {:?} but right = {:?}",
                                left_type, right_type
                            ));
                        }

                        Ok(Type::Boolean)
                    }
                }
            }
            Expr::FunctionCall { name, arguments } => {
                // TODO
                // ヒント：
                // 1. let function_type = if let Some(symbol) = self.symbol_table.resolve(name) { symbol.symbol_type.clone() } else { return Err(...) }; で関数型取得
                // 2. if let Type::Function { parameters, return_type } = function_type で関数型確認
                // 3. 引数の型をチェック: arguments.iter().zip(parameters).try_for_each で引数数と型を確認
                // 4. Ok(*return_type) で戻り値型を返す

                match self.symbol_table.resolve(name) {
                    Some(fun_type) => {
                        if let Type::Function {
                            parameters,
                            return_type,
                        } = fun_type.symbol_type.clone()
                        {
                            arguments
                                .iter()
                                .zip(parameters)
                                .try_for_each(|(got, want)| {
                                    let inferred_arg = self.infer_expression_type(got)?;
                                    if inferred_arg.ne(&want) {
                                        return Err(format!("Unmatched argument {:?}", got));
                                    }
                                    Ok(())
                                })?;
                            Ok(*return_type.clone())
                        } else {
                            Err(format!("Function '{}' does not exist", name))
                        }
                    }
                    None => Err(format!("Function '{}' not defined", name)),
                }
            }
        }
    }

    // 型チェック結果を取得
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }
}

// 公開API
pub fn check_types(program: &Program) -> Result<SymbolTable, Vec<String>> {
    let mut checker = TypeChecker::new();
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
    fn test_basic_type_checking() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(42),
                    type_annotation: Some(Type::Integer),
                },
                Stmt::LetDeclaration {
                    name: "flag".to_string(),
                    value: Expr::Boolean(true),
                    type_annotation: Some(Type::Boolean),
                },
                Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Number(10)),
                }),
            ],
        };

        let result = check_types(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        let x_symbol = table.resolve("x").unwrap();
        assert_eq!(x_symbol.symbol_type, Type::Integer);

        let flag_symbol = table.resolve("flag").unwrap();
        assert_eq!(flag_symbol.symbol_type, Type::Boolean);
    }

    #[test]
    fn test_type_mismatch_error() {
        let program = Program {
            statements: vec![Stmt::LetDeclaration {
                name: "x".to_string(),
                value: Expr::Boolean(true),           // booleanを代入
                type_annotation: Some(Type::Integer), // integerと注釈
            }],
        };

        let result = check_types(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
    }

    #[test]
    fn test_if_condition_type_check() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(5),
                    type_annotation: None,
                },
                Stmt::IfStatement {
                    condition: Expr::Number(42), // 数値（非boolean）を条件に使用
                    then_branch: Box::new(Stmt::Expression(Expr::Identifier("x".to_string()))),
                    else_branch: None,
                },
            ],
        };

        let result = check_types(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("If condition must be boolean"));
    }
}
