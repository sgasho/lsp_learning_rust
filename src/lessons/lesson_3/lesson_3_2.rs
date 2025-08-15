// Lesson 3-2へようこそ！
// lesson_3_1で基本的な変数管理ができるようになりましたね。
// 今度は、同一スコープでの重複定義を検出する機能を追加します。

// あなたのタスク：
// 重複定義のチェック機能を実装してください。
// 例：let x = 5; let x = 10; → エラー

use std::collections::HashMap;

// lesson_3_1と同じAST構造
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
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

// シンボル（変数）の情報（lesson_3_1と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
}

// シンボルテーブル（lesson_3_1と同じ基本構造）
#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    // 変数を定義（重複チェック付き）
    pub fn define(&mut self, name: String) -> Result<(), String> {
        // ヒント：
        // 1. self.symbols.contains_key(&name)で既に存在するかチェック
        // 2. 存在する場合は"Variable '{}' already defined in this scope"エラー
        // 3. 存在しない場合のみSymbolを作成してinsert()

        if self.symbols.contains_key(&name) {
            return Err(format!("Variable '{}' already defined in this scope", name));
        }

        self.symbols.insert(name.clone(), Symbol { name });
        Ok(())
    }

    // 変数を検索（lesson_3_1と同じ）
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

// スコープ解析を行う構造体（lesson_3_1と同じ）
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

    // プログラム全体を解析（lesson_3_1と同じ）
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

    // 個別の文を解析（lesson_3_1と同じ）
    fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::LetDeclaration { name, value } => {
                self.analyze_expression(value)?;
                self.symbol_table.define(name.clone())
            }
            Stmt::Expression(expr) => self.analyze_expression(expr),
        }
    }

    // 式を解析（lesson_3_1と同じ）
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

    // 解析結果を取得（lesson_3_1と同じ）
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }
}

// 公開API（lesson_3_1と同じ）
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
    fn test_duplicate_variable_definition() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(5),
                },
                Stmt::LetDeclaration {
                    name: "x".to_string(), // 重複定義
                    value: Expr::Number(10),
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("already defined"));
    }

    #[test]
    fn test_different_variable_names() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(5),
                },
                Stmt::LetDeclaration {
                    name: "y".to_string(), // 異なる名前なのでOK
                    value: Expr::Number(10),
                },
                Stmt::LetDeclaration {
                    name: "z".to_string(), // 異なる名前なのでOK
                    value: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Identifier("y".to_string())),
                    },
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert!(table.resolve("x").is_some());
        assert!(table.resolve("y").is_some());
        assert!(table.resolve("z").is_some());
    }

    #[test]
    fn test_multiple_duplicate_definitions() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "a".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::LetDeclaration {
                    name: "b".to_string(),
                    value: Expr::Number(2),
                },
                Stmt::LetDeclaration {
                    name: "a".to_string(), // aの重複定義
                    value: Expr::Number(3),
                },
                Stmt::LetDeclaration {
                    name: "b".to_string(), // bの重複定義
                    value: Expr::Number(4),
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2); // aとbの2つのエラー
        assert!(errors
            .iter()
            .any(|e| e.contains("'a'") && e.contains("already defined")));
        assert!(errors
            .iter()
            .any(|e| e.contains("'b'") && e.contains("already defined")));
    }
}
