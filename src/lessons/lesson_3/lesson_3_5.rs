// Lesson 3-5へようこそ！
// lesson_3_4でネストしたスコープができるようになりましたね。
// 今度は、シャドウイング（変数の隠蔽）の詳細な動作を学びます。

// あなたのタスク：
// シャドウイングの動作を理解し、テストで確認してください。
// 例：let x = 1; { let x = 2; } でのxの隠蔽と復活

use std::collections::HashMap;

// AST構造（lesson_3_4と同じ）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
    Block { statements: Vec<Stmt> },
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// シンボル（変数）の情報（lesson_3_4と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
}

// スコープ構造（lesson_3_4と同じ）
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

// シンボルテーブル（lesson_3_4と同じ）
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

    // 新しいスコープに入る（lesson_3_4と同じ）
    pub fn enter_scope(&mut self) {
        self.scope_level += 1;
        self.current_scope = Scope::with_parent(self.current_scope.clone(), self.scope_level);
    }

    // スコープから出る（lesson_3_4と同じ）
    pub fn exit_scope(&mut self) {
        if self.scope_level == 0 {
            return;
        }

        self.scope_level -= 1;
        if let Some(parent) = self.current_scope.parent.take() {
            self.current_scope = *parent;
        }
    }

    // 変数を定義（lesson_3_4と同じ - シャドウイングは既に正しく動作）
    pub fn define(&mut self, name: String) -> Result<(), String> {
        // 同一スコープでの重複のみチェック（シャドウイングは許可）
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

    // 変数を検索（lesson_3_4と同じ - シャドウイングを正しく処理）
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        let mut current = &self.current_scope;
        
        loop {
            // 内側のスコープから検索（シャドウイングの核心）
            if let Some(symbol) = current.symbols.get(name) {
                return Some(symbol);  // 見つかった瞬間に返す
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

// スコープ解析を行う構造体（lesson_3_4と同じ）
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

    // プログラム全体を解析（lesson_3_4と同じ）
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

    // 個別の文を解析（lesson_3_4と同じ）
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
        }
    }

    // 式を解析（lesson_3_4と同じ）
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
    fn test_basic_shadowing() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "x".to_string(),  // xをシャドウ
                            value: Expr::Number(2),
                        },
                        Stmt::Expression(Expr::Identifier("x".to_string())), // 内側のx（2）を参照
                    ],
                },
                Stmt::Expression(Expr::Identifier("x".to_string())), // 外側のx（1）が復活
            ],
        };
        
        let result = analyze_scope(&program);
        assert!(result.is_ok());
        
        let table = result.unwrap();
        // ブロック終了後、外側のxが見える
        assert!(table.resolve("x").is_some());
        assert_eq!(table.resolve("x").unwrap().scope_level, 0); // レベル0のx
    }

    #[test]
    fn test_multi_level_shadowing() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),  // レベル0のx
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "x".to_string(),
                            value: Expr::Number(2),  // レベル1のx
                        },
                        Stmt::Block {
                            statements: vec![
                                Stmt::LetDeclaration {
                                    name: "x".to_string(),
                                    value: Expr::Number(3),  // レベル2のx
                                },
                                Stmt::Expression(Expr::Identifier("x".to_string())), // レベル2のxを参照
                            ],
                        },
                        Stmt::Expression(Expr::Identifier("x".to_string())), // レベル1のxを参照
                    ],
                },
                Stmt::Expression(Expr::Identifier("x".to_string())), // レベル0のxを参照
            ],
        };
        
        let result = analyze_scope(&program);
        assert!(result.is_ok());
        
        let table = result.unwrap();
        // 最終的にレベル0のxが見える
        assert!(table.resolve("x").is_some());
        assert_eq!(table.resolve("x").unwrap().scope_level, 0);
    }

    #[test]
    fn test_partial_shadowing() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Number(100),
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "x".to_string(),  // xだけシャドウ（yはシャドウしない）
                            value: Expr::Number(10),
                        },
                        Stmt::Expression(Expr::Binary {
                            left: Box::new(Expr::Identifier("x".to_string())),   // シャドウされたx（10）
                            operator: BinaryOp::Add,
                            right: Box::new(Expr::Identifier("y".to_string())),  // 外側のy（100）
                        }),
                    ],
                },
                Stmt::Expression(Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),  // 元のx（1）が復活
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Identifier("y".to_string())), // y（100）
                }),
            ],
        };
        
        let result = analyze_scope(&program);
        assert!(result.is_ok());
        
        let table = result.unwrap();
        // ブロック終了後、両方ともレベル0の変数
        assert!(table.resolve("x").is_some());
        assert!(table.resolve("y").is_some());
        assert_eq!(table.resolve("x").unwrap().scope_level, 0);
        assert_eq!(table.resolve("y").unwrap().scope_level, 0);
    }
}