// Lesson 3-4へようこそ！
// lesson_3_3で単一レベルのブロックスコープができるようになりましたね。
// 今度は、複数レベルにネストしたスコープを扱います。

// あなたのタスク：
// ネストしたスコープとレベル管理を実装してください。
// 例：{ { { let x = 1; } } } のような深いネスト

use std::collections::HashMap;

// AST構造（lesson_3_3と同じ）
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

// シンボル（変数）の情報（レベル情報を追加）
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize, // どのレベルで定義されたか
}

// スコープ構造（レベル情報を追加）
#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize, // スコープの深さレベル
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

// シンボルテーブル（レベル管理を追加）
#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Scope,
    pub scope_level: usize, // 現在のレベル追跡
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            current_scope: Scope::new(),
            scope_level: 0,
        }
    }

    // 新しいスコープに入る（レベル管理付き）
    pub fn enter_scope(&mut self) {
        // ヒント：
        // 1. scope_levelを1増加
        // 2. Scope::with_parent()で新しいスコープを作成（現在のスコープを親に、新しいレベルで）
        // 3. current_scopeを新しいスコープに更新

        // TODO: ここを実装してください
        self.scope_level += 1;
        let new_parent = &self.current_scope;
        self.current_scope = Scope::with_parent(new_parent.clone(), self.scope_level);
    }

    // スコープから出る（レベル管理付き）
    pub fn exit_scope(&mut self) {
        // ヒント：
        // 1. current_scope.parentがある場合のみ処理
        // 2. scope_levelを1減少
        // 3. parentを取り出してcurrent_scopeに設定

        // TODO: ここを実装してください
        if self.scope_level == 0 {
            return;
        }

        self.scope_level -= 1;
        let parent = self.current_scope.parent.clone();

        if let Some(parent) = parent {
            self.current_scope = *parent;
        }
    }

    // 変数を定義（レベル情報付き）
    pub fn define(&mut self, name: String) -> Result<(), String> {
        // ヒント：
        // 1. 重複チェック（lesson_3_3と同じ）
        // 2. Symbolを作成時にscope_level: self.scope_levelを設定
        // 3. current_scope.symbolsに追加

        // TODO: ここを実装してください
        self.current_scope.symbols.insert(
            name.clone(),
            Symbol {
                name,
                scope_level: self.scope_level,
            },
        );
        Ok(())
    }

    // 変数を検索（lesson_3_3と同じ階層検索）
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

// スコープ解析を行う構造体（lesson_3_3と同じ）
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

    // プログラム全体を解析（lesson_3_3と同じ）
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

    // 個別の文を解析（lesson_3_3と同じ）
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

    // 式を解析（lesson_3_3と同じ）
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
    fn test_two_level_nesting() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "outer".to_string(),
                    value: Expr::Number(1),
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "middle".to_string(),
                            value: Expr::Number(2),
                        },
                        Stmt::Block {
                            statements: vec![Stmt::LetDeclaration {
                                name: "inner".to_string(),
                                value: Expr::Binary {
                                    left: Box::new(Expr::Identifier("outer".to_string())),
                                    operator: BinaryOp::Add,
                                    right: Box::new(Expr::Identifier("middle".to_string())),
                                },
                            }],
                        },
                    ],
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        // ネスト終了後、outerは見えるがmiddle、innerは見えない
        assert!(table.resolve("outer").is_some());
        assert!(table.resolve("middle").is_none());
        assert!(table.resolve("inner").is_none());

        // outerのスコープレベルは0
        assert_eq!(table.resolve("outer").unwrap().scope_level, 0);
    }

    #[test]
    fn test_three_level_nesting() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "level0".to_string(),
                    value: Expr::Number(0),
                },
                Stmt::Block {
                    statements: vec![
                        Stmt::LetDeclaration {
                            name: "level1".to_string(),
                            value: Expr::Number(1),
                        },
                        Stmt::Block {
                            statements: vec![
                                Stmt::LetDeclaration {
                                    name: "level2".to_string(),
                                    value: Expr::Number(2),
                                },
                                Stmt::Block {
                                    statements: vec![Stmt::Expression(Expr::Binary {
                                        left: Box::new(Expr::Binary {
                                            left: Box::new(Expr::Identifier("level0".to_string())),
                                            operator: BinaryOp::Add,
                                            right: Box::new(Expr::Identifier("level1".to_string())),
                                        }),
                                        operator: BinaryOp::Add,
                                        right: Box::new(Expr::Identifier("level2".to_string())),
                                    })],
                                },
                            ],
                        },
                    ],
                },
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());

        let table = result.unwrap();
        // 最外層のみ残る
        assert!(table.resolve("level0").is_some());
        assert!(table.resolve("level1").is_none());
        assert!(table.resolve("level2").is_none());
    }

    #[test]
    fn test_deep_nesting_variable_resolution() {
        let program = Program {
            statements: vec![
                Stmt::LetDeclaration {
                    name: "global".to_string(),
                    value: Expr::Number(42),
                },
                Stmt::Block {
                    statements: vec![Stmt::Block {
                        statements: vec![Stmt::Block {
                            statements: vec![
                                Stmt::Expression(Expr::Identifier("global".to_string())), // 3レベル上から参照
                            ],
                        }],
                    }],
                },
                Stmt::Expression(Expr::Identifier("global".to_string())), // まだ見える
            ],
        };

        let result = analyze_scope(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_scope_error() {
        let program = Program {
            statements: vec![Stmt::Block {
                statements: vec![
                    Stmt::Block {
                        statements: vec![Stmt::LetDeclaration {
                            name: "deep".to_string(),
                            value: Expr::Number(1),
                        }],
                    },
                    Stmt::Expression(Expr::Identifier("deep".to_string())), // deepは1レベル深い場所で定義
                ],
            }],
        };

        let result = analyze_scope(&program);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.len() > 0);
        assert!(errors[0].contains("not defined"));
    }
}
