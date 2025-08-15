// Lesson 3-6へようこそ！
// lesson_3_5でシャドウイングができるようになりましたね。
// 今度は、if文でのスコープ管理を学びます。

// あなたのタスク：
// if文でのスコープ管理を実装してください。
// 例：if文内で定義された変数は外で見えない

use std::collections::HashMap;

// AST構造（if文を追加）
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
        // 新規追加
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
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
    GreaterThan, // 比較演算子を追加
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// シンボル（変数）の情報（lesson_3_5と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
}

// スコープ構造（lesson_3_5と同じ）
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

// シンボルテーブル（lesson_3_5と同じ）
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

    // 新しいスコープに入る（lesson_3_5と同じ）
    pub fn enter_scope(&mut self) {
        self.scope_level += 1;
        self.current_scope = Scope::with_parent(self.current_scope.clone(), self.scope_level);
    }

    // スコープから出る（lesson_3_5と同じ）
    pub fn exit_scope(&mut self) {
        if self.scope_level == 0 {
            return;
        }

        self.scope_level -= 1;
        if let Some(parent) = self.current_scope.parent.take() {
            self.current_scope = *parent;
        }
    }

    // 変数を定義（lesson_3_5と同じ）
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

    // 変数を検索（lesson_3_5と同じ）
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

// スコープ解析を行う構造体（if文解析を追加）
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

    // プログラム全体を解析（lesson_3_5と同じ）
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

    // 個別の文を解析（if文解析を追加）
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
                // ヒント：
                // 1. 条件式を現在のスコープで解析
                // 2. then_branchを解析（Blockの場合は自動でスコープ管理）
                // 3. else_branchがある場合、解析（Blockの場合は自動でスコープ管理）

                self.analyze_expression(condition)?;
                self.analyze_statement(then_branch)?;

                if let Some(else_stmt) = else_branch {
                    self.analyze_statement(else_stmt)?;
                }

                Ok(())
            }
        }
    }

    // 式を解析（比較演算子を追加）
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
            Expr::FunctionCall { arguments, .. } => arguments
                .iter()
                .try_for_each(|expr| self.analyze_expression(expr)),
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
    fn test_basic_if_statement() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::IfStatement {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expr::Number(0)),
                    },
                    then_branch: Box::new(Stmt::Block {
                        statements: vec![
                            Stmt::LetDeclaration {
                                name: "y".to_string(),
                                value: Expr::Number(2),
                            },
                            Stmt::Expression(Expr::Binary {
                                left: Box::new(Expr::Identifier("x".to_string())),
                                operator: BinaryOp::Add,
                                right: Box::new(Expr::Identifier("y".to_string())),
                            }),
                        ],
                    }),
                    else_branch: None,
                },
                Stmt::Expression(Expr::Identifier("x".to_string())), // OK: 外側のx
                Stmt::Expression(Expr::Identifier("y".to_string())), // Error: yは見えない
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("Variable 'y' not defined"));
    }

    #[test]
    fn test_if_else_statement() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::IfStatement {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expr::Number(0)),
                    },
                    then_branch: Box::new(Stmt::Block {
                        statements: vec![Stmt::LetDeclaration {
                            name: "msg".to_string(),
                            value: Expr::Number(1), // "positive"のつもり
                        }],
                    }),
                    else_branch: Some(Box::new(Stmt::Block {
                        statements: vec![Stmt::LetDeclaration {
                            name: "msg".to_string(), // 異なるスコープなので同名でもOK
                            value: Expr::Number(0),  // "not positive"のつもり
                        }],
                    })),
                },
                Stmt::Expression(Expr::Identifier("x".to_string())), // OK
                Stmt::Expression(Expr::Identifier("msg".to_string())), // Error: どちらのmsgも見えない
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("Variable 'msg' not defined"));
    }

    #[test]
    fn test_if_statement_access_outer_variable() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "outer".to_string(),
                    value: Expr::Number(42),
                },
                Stmt::IfStatement {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Identifier("outer".to_string())),
                        operator: BinaryOp::GreaterThan,
                        right: Box::new(Expr::Number(0)),
                    },
                    then_branch: Box::new(Stmt::Block {
                        statements: vec![
                            Stmt::Expression(Expr::Identifier("outer".to_string())), // OK: 外側の変数にアクセス
                        ],
                    }),
                    else_branch: None,
                },
                Stmt::Expression(Expr::Identifier("outer".to_string())), // OK
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok()); // エラーなし

        let table = result.unwrap();
        assert!(table.resolve("outer").is_some());
        assert_eq!(table.resolve("outer").unwrap().scope_level, 0);
    }
}
