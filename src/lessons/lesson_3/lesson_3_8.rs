// Lesson 3-8へようこそ！
// lesson_3_7でwhile文のスコープができるようになりましたね。
// 今度は、関数でのスコープ管理を学びます。

// あなたのタスク：
// 関数でのスコープ管理を実装してください。
// 例：関数内で定義された変数やパラメータは外で見えない

use std::collections::HashMap;

// AST構造（関数定義を追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
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
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// シンボル（変数）の情報（lesson_3_7と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
}

// スコープ構造（lesson_3_7と同じ）
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

// シンボルテーブル（lesson_3_7と同じ）
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

    // 新しいスコープに入る（lesson_3_7と同じ）
    pub fn enter_scope(&mut self) {
        self.scope_level += 1;
        self.current_scope = Scope::with_parent(self.current_scope.clone(), self.scope_level);
    }

    // スコープから出る（lesson_3_7と同じ）
    pub fn exit_scope(&mut self) {
        if self.scope_level == 0 {
            return;
        }

        self.scope_level -= 1;
        if let Some(parent) = self.current_scope.parent.take() {
            self.current_scope = *parent;
        }
    }

    // 変数を定義（lesson_3_7と同じ）
    pub fn define(&mut self, name: String) -> Result<(), String> {
        if self.current_scope.symbols.contains_key(&name) {
            return Err(format!("Variable '{}' already defined in this scope", name));
        }

        let symbol = Symbol {
            name: name.clone(),
            scope_level: self.scope_level,
        };

        self.current_scope.symbols.insert(name, symbol);
        Ok(())
    }

    // 変数を検索（lesson_3_7と同じ）
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

// スコープ解析を行う構造体（関数解析を追加）
#[derive(Debug)]
pub struct ScopeAnalyzer {
    symbol_table: SymbolTable,
    errors: Vec<String>,
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        ScopeAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    // プログラム全体を解析（lesson_3_7と同じ）
    pub fn analyze_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
        for statement in &program.statements {
            if let Err(e) = self.analyze_statement(statement) {
                self.errors.push(e);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    // 個別の文を解析（関数解析を追加）
    fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::LetDeclaration { name, value } => {
                self.analyze_expression(value)?;
                self.symbol_table.define(name.clone())
            }
            Stmt::Expression(expr) => self.analyze_expression(expr),
            Stmt::Block { statements } => {
                self.symbol_table.enter_scope();

                for stmt in statements {
                    self.analyze_statement(stmt)?;
                }

                self.symbol_table.exit_scope();
                Ok(())
            }
            Stmt::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(then_branch)?;

                if let Some(else_stmt) = else_branch {
                    self.analyze_statement(else_stmt)?;
                }

                Ok(())
            }
            Stmt::WhileStatement { condition, body } => {
                self.analyze_expression(condition)?;
                self.analyze_statement(body)?;
                Ok(())
            }
            Stmt::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                // ヒント：
                // 1. self.symbol_table.define(name.clone())? で関数名をグローバルに定義
                // 2. self.symbol_table.enter_scope() で関数スコープ開始
                // 3. パラメータをループで定義: for param in parameters { self.symbol_table.define(param.name.clone())?; }
                // 4. self.analyze_statement(body)? で関数本体解析
                // 5. self.symbol_table.exit_scope() で関数スコープ終了
                self.symbol_table.define(name.clone())?;

                for parameter in parameters {
                    self.symbol_table.define(parameter.name.clone())?;
                }

                self.analyze_statement(body)?;

                self.symbol_table.exit_scope();

                Ok(())
            }
        }
    }

    // 式を解析（関数呼び出し確認を修正）
    fn analyze_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Identifier(name) => match self.symbol_table.resolve(name) {
                Some(_) => Ok(()),
                None => Err(format!("Variable '{}' not defined", name)),
            },
            Expr::Binary { left, right, .. } => {
                self.analyze_expression(left)?;
                self.analyze_expression(right)
            }
            Expr::FunctionCall { name, arguments } => {
                // ヒント：
                // 1. self.symbol_table.resolve(name).ok_or_else(|| format!("Function '{}' not defined", name))?; で関数存在確認
                // 2. arguments.iter().try_for_each(|arg| self.analyze_expression(arg)) で引数解析
                self.symbol_table
                    .resolve(name)
                    .ok_or_else(|| format!("Function '{}' not defined", name))?;
                arguments
                    .iter()
                    .try_for_each(|arg| self.analyze_expression(arg))
            }
            Expr::Number(_) => Ok(()),
        }
    }

    // 解析結果を取得
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }
}

// 公開API
pub fn analyze_scope(program: &Program) -> Result<SymbolTable, Vec<String>> {
    let mut analyzer = ScopeAnalyzer::new();
    match analyzer.analyze_program(program) {
        Ok(()) => Ok(analyzer.symbol_table),
        Err(errors) => Err(errors),
    }
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_function_declaration() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "global".to_string(),
                    value: Expr::Number(42),
                },
                Stmt::FunctionDeclaration {
                    name: "add".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "a".to_string(),
                        },
                        Parameter {
                            name: "b".to_string(),
                        },
                    ],
                    body: Box::new(Stmt::Block {
                        statements: vec![
                            Stmt::LetDeclaration {
                                name: "result".to_string(),
                                value: Expr::Binary {
                                    left: Box::new(Expr::Identifier("a".to_string())),
                                    operator: BinaryOp::Add,
                                    right: Box::new(Expr::Identifier("b".to_string())),
                                },
                            },
                            Stmt::Expression(Expr::Identifier("global".to_string())), // OK: グローバル変数
                            Stmt::Expression(Expr::Identifier("result".to_string())), // OK: ローカル変数
                        ],
                    }),
                },
                Stmt::Expression(Expr::Identifier("global".to_string())), // OK
                Stmt::Expression(Expr::Identifier("result".to_string())), // Error: resultは見えない
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("Variable 'result' not defined"));
    }

    #[test]
    fn test_function_call() {
        let program = Program {
            statements: vec![
                Stmt::FunctionDeclaration {
                    name: "multiply".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "x".to_string(),
                        },
                        Parameter {
                            name: "y".to_string(),
                        },
                    ],
                    body: Box::new(Stmt::Block {
                        statements: vec![Stmt::Expression(Expr::Binary {
                            left: Box::new(Expr::Identifier("x".to_string())),
                            operator: BinaryOp::Multiply,
                            right: Box::new(Expr::Identifier("y".to_string())),
                        })],
                    }),
                },
                Stmt::LetDeclaration {
                    name: "a".to_string(),
                    value: Expr::Number(5),
                },
                Stmt::LetDeclaration {
                    name: "b".to_string(),
                    value: Expr::Number(6),
                },
                Stmt::Expression(Expr::FunctionCall {
                    name: "multiply".to_string(),
                    arguments: vec![
                        Expr::Identifier("a".to_string()),
                        Expr::Identifier("b".to_string()),
                    ],
                }), // OK: 関数とパラメータが定義済み
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok()); // エラーなし

        let table = result.unwrap();
        assert!(table.resolve("multiply").is_some());
        assert!(table.resolve("a").is_some());
        assert!(table.resolve("b").is_some());
    }

    #[test]
    fn test_undefined_function_call() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(10),
                },
                Stmt::Expression(Expr::FunctionCall {
                    name: "unknown_function".to_string(), // 未定義関数
                    arguments: vec![Expr::Identifier("x".to_string())],
                }),
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("Function 'unknown_function' not defined"));
    }
}
