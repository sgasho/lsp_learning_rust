// Lesson 3-1へようこそ！
// lesson_2シリーズで構文解析エンジンが完成しましたね。
// 今度は、セマンティック解析の第一歩として、基本的な変数管理について学びます。

// あなたのタスク：
// 基本的な変数定義と参照を実装してください。
// 例：let x = 5; let y = x; のような変数管理

use std::collections::HashMap;

// lesson_2で作成したAST構造を再利用（簡略版）
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

// シンボル（変数）の情報（シンプル版）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
}

// シンボルテーブル（グローバルスコープのみ）
#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>, // グローバル変数のみ
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    // 変数を定義
    pub fn define(&mut self, name: String) -> Result<(), String> {
        // ヒント：
        // 1. 新しいSymbolを作成（name: nameで）
        // 2. self.symbolsに追加
        // 今回は重複チェックなし（lesson_3_2で追加予定）
        self.symbols.insert(name.clone(), Symbol { name });
        Ok(())
    }

    // 変数を検索
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        // ヒント：
        // 1. self.symbols.get(name)でHashMapから検索
        // 2. 結果をそのまま返す
        self.symbols.get(name)
    }
}

// スコープ解析を行う構造体（シンプル版）
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

    // プログラム全体を解析
    pub fn analyze_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
        // ヒント：
        // 1. program.statementsをforループで処理
        // 2. 各statementをanalyze_statement()で解析
        // 3. エラーが発生した場合はself.errorsに追加（継続処理）
        // 4. 最後にself.errorsが空ならOk(())、そうでなければErr(errors)
        
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

    // 個別の文を解析
    fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        // ヒント：
        // 1. Stmt::LetDeclarationの場合:
        //    - まず右辺（value）をanalyze_expression()で解析
        //    - 次に変数名（name）をdefine()で定義
        // 2. Stmt::Expressionの場合:
        //    - 式をanalyze_expression()で解析

        match stmt {
            Stmt::LetDeclaration { name, value } => {
                self.analyze_expression(value)?;
                self.symbol_table.define(name.clone())
            }
            Stmt::Expression(expr) => self.analyze_expression(expr),
        }
    }

    // 式を解析
    fn analyze_expression(&mut self, expr: &Expr) -> Result<(), String> {
        // ヒント：
        // 1. Expr::Identifierの場合:
        //    - resolve()で変数が定義されているかチェック
        //    - 定義されていない場合はエラー
        // 2. Expr::Binaryの場合:
        //    - left、rightの両方をanalyze_expression()で解析
        // 3. Expr::FunctionCallの場合:
        //    - argumentsをループで処理し、それぞれをanalyze_expression()で解析
        // 4. Expr::Numberの場合:
        //    - 何もしない（リテラルなのでエラーなし）

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
            Expr::Number(_) => {
                // 数値リテラルはエラーなし
                Ok(())
            }
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

// 公開API：プログラムのスコープ解析を実行
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
    fn test_basic_variable_definition_and_reference() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(5),
                },
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Identifier("x".to_string()),
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert!(table.resolve("x").is_some());
        assert!(table.resolve("y").is_some());
        assert!(table.resolve("z").is_none());
    }

    #[test]
    fn test_undefined_variable_error() {
        let program = Program {
            statements: vec![Stmt::LetDeclaration {
                name: "y".to_string(),
                value: Expr::Identifier("x".to_string()), // xは未定義
            }],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("not defined"));
    }

    #[test]
    fn test_function_call_with_variables() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(5),
                },
                Stmt::Expression(Expr::FunctionCall {
                    name: "print".to_string(),
                    arguments: vec![Expr::Identifier("x".to_string()), Expr::Number(10)],
                }),
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_expression_with_variables() {
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
                    name: "c".to_string(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Identifier("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Identifier("b".to_string())),
                    },
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert!(table.resolve("a").is_some());
        assert!(table.resolve("b").is_some());
        assert!(table.resolve("c").is_some());
    }
}
