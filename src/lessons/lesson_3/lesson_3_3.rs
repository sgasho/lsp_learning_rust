// Lesson 3-3へようこそ！
// lesson_3_2で重複定義のチェックができるようになりましたね。
// 今度は、ブロックスコープ（{}）の概念を導入します。

// あなたのタスク：
// 単一レベルのブロックスコープを実装してください。
// 例：{ let x = 5; } の外側でxは見えない

use std::collections::HashMap;

// AST構造を拡張（Block文を追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
    Block { statements: Vec<Stmt> }, // 新規追加
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

// シンボル（変数）の情報（lesson_3_2と同じ）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
}

// スコープ構造（階層的）
#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>, // 親スコープへの参照
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
}

// シンボルテーブル（階層的スコープ管理）
#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Scope,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            current_scope: Scope::new(),
        }
    }

    // 新しいスコープに入る
    pub fn enter_scope(&mut self) {
        // ヒント：
        // 1. 現在のスコープをclone()して保存
        // 2. Scope::with_parent()で新しいスコープを作成
        // 3. current_scopeを新しいスコープに更新
        self.current_scope = Scope::with_parent(self.current_scope.clone());
    }

    // スコープから出る
    pub fn exit_scope(&mut self) {
        // ヒント：
        // 1. current_scope.parentがある場合のみ処理
        // 2. parentを取り出してcurrent_scopeに設定
        // 3. 元のスコープは自動的に削除される
        match &self.current_scope.parent {
            Some(parent) => {
                self.current_scope = *parent.clone();
            }
            None => {}
        }
    }

    // 変数を定義（lesson_3_2と同じ）
    pub fn define(&mut self, name: String) -> Result<(), String> {
        if self.current_scope.symbols.contains_key(&name) {
            return Err(format!("Variable '{}' already defined in this scope", name));
        }

        self.current_scope
            .symbols
            .insert(name.clone(), Symbol { name });
        Ok(())
    }

    // 変数を検索（階層検索）
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        // ヒント：
        // 1. current_scopeから開始
        // 2. 現在のスコープでsymbols.get(name)をチェック
        // 3. 見つからなければparentスコープをチェック
        // 4. parentがNoneになるまで繰り返し
        // 5. 最終的に見つからなければNone
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

// スコープ解析を行う構造体
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

    // プログラム全体を解析（lesson_3_2と同じ）
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

    // 個別の文を解析（Block文の処理を追加）
    fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::LetDeclaration { name, value } => {
                self.analyze_expression(value)?;
                self.symbol_table.define(name.clone())
            }
            Stmt::Expression(expr) => self.analyze_expression(expr),
            Stmt::Block { statements } => {
                // ヒント：
                // 1. enter_scope()で新しいスコープに入る
                // 2. 各statementをanalyze_statement()で解析
                // 3. exit_scope()でスコープから出る
                // 4. エラーが発生してもexit_scope()は必ず実行
                self.symbol_table.enter_scope();

                for stmt in statements {
                    self.analyze_statement(stmt)?;
                }

                self.symbol_table.exit_scope();
                Ok(())
            }
        }
    }

    // 式を解析（lesson_3_2と同じ）
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
    fn test_basic_block_scope() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "y".to_string(),
                            value: Expr::Number(2),
                        },
                        Stmt::Expression(Expr::Identifier("x".to_string())), // xは外側で定義
                        Stmt::Expression(Expr::Identifier("y".to_string())), // yはブロック内で定義
                    ],
                },
                Stmt::Expression(Expr::Identifier("x".to_string())), // xはまだ見える
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        // ブロック終了後、xは見えるがyは見えない
        assert!(table.resolve("x").is_some());
        assert!(table.resolve("y").is_none());
    }

    #[test]
    fn test_block_scope_error() {
        let program = Program {
            statements: vec![
                Stmt::Block {
                    statements: vec![Stmt::LetDeclaration {
                        name: "temp".to_string(),
                        value: Expr::Number(42),
                    }],
                },
                Stmt::Expression(Expr::Identifier("temp".to_string())), // tempはスコープ外
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("not defined"));
    }

    #[test]
    fn test_outer_variable_access() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "outer".to_string(),
                    value: Expr::Number(10),
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "inner".to_string(),
                            value: Expr::Identifier("outer".to_string()), // 外側の変数を参照
                        },
                        Stmt::Expression(Expr::Binary {
                            left: Box::new(Expr::Identifier("outer".to_string())),
                            operator: BinaryOp::Add,
                            right: Box::new(Expr::Identifier("inner".to_string())),
                        }),
                    ],
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_block() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::Block {
                    statements: vec![], // 空のブロック
                },
                Stmt::Expression(Expr::Identifier("x".to_string())), // xはまだ見える
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert!(table.resolve("x").is_some());
    }
}
