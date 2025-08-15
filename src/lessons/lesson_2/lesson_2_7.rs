// Lesson 2-7へようこそ！
// lesson_2_6で条件分岐（if-else）を学びましたね。
// 今度は、ループ文（while）の構文解析について学びます。

// あなたのタスク：
// while文の構文解析を実装してください。
// 例：while x > 0 { print(x); x = x - 1; }

// トークンを拡張（whileを追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,  // (
    RightParen, // )
    Comma,      // ,
    Let,        // let
    Assign,     // =
    Semicolon,  // ;
    If,         // if
    Else,       // else
    While,      // while
    LeftBrace,  // {
    RightBrace, // }
    Greater,    // >
    Less,       // <
    Equal,      // ==
    NotEqual,   // !=
    Eof,
}

// 抽象構文木（AST）のノード
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
    },
    Expression(Expr), // 式文
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
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
    Grouped(Box<Expr>), // 括弧でグループ化された式
    FunctionCall {
        // 関数呼び出し
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
    Greater,  // >
    Less,     // <
    Equal,    // ==
    NotEqual, // !=
}

// ブロック（文のリスト）
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

// プログラム全体を表現
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// パーサーの状態を管理する構造体
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // 現在のトークンを取得
    fn current_token(&self) -> &Token {
        if self.current >= self.tokens.len() {
            &Token::Eof
        } else {
            &self.tokens[self.current]
        }
    }

    // 次のトークンに進む
    fn advance(&mut self) {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
    }

    // 特定のトークンを期待して消費
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, found {:?}",
                expected,
                self.current_token()
            ))
        }
    }

    // ブロックを解析
    fn parse_block(&mut self) -> Result<Block, String> {
        // lesson_2_6と同じ実装
        self.expect(Token::LeftBrace)?;

        let mut statements = Vec::new();

        while !matches!(self.current_token(), Token::RightBrace) {
            if matches!(self.current_token(), Token::Eof) {
                return Err("Missing closing brace".to_string());
            }
            statements.push(self.parse_statement()?);
        }

        self.expect(Token::RightBrace)?;

        Ok(Block { statements })
    }

    // while文を解析
    fn parse_while_statement(&mut self) -> Result<Stmt, String> {
        // ヒント：
        // 1. expect(Token::While)でwhileキーワードを消費
        // 2. parse_expression()で条件式を解析
        // 3. parse_block()でボディブロックを解析
        // 4. While ASTノードを作成

        self.expect(Token::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::While { condition, body })
    }

    // if文を解析
    fn parse_if_statement(&mut self) -> Result<Stmt, String> {
        // lesson_2_6と同じ実装
        self.expect(Token::If)?;

        let condition = self.parse_comparison()?;

        let then_block = self.parse_block()?;

        let else_block = match self.current_token() {
            Token::Else => {
                self.advance();
                Some(self.parse_block()?)
            }
            _ => None,
        };

        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    // let文を解析
    fn parse_let_statement(&mut self) -> Result<Stmt, String> {
        // lesson_2_6と同じ実装
        self.expect(Token::Let)?;

        let ident = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            t => return Err(format!("Expected identifier, found {:?}", t)),
        };
        self.advance();

        self.expect(Token::Assign)?;

        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;

        Ok(Stmt::LetDeclaration {
            name: ident,
            value: expr,
        })
    }

    // 文を解析
    fn parse_statement(&mut self) -> Result<Stmt, String> {
        // ヒント：
        // 1. current_token()で現在のトークンを確認
        // 2. Token::Letの場合：parse_let_statement()を呼び出し
        // 3. Token::Ifの場合：parse_if_statement()を呼び出し
        // 4. Token::Whileの場合：parse_while_statement()を呼び出し
        // 5. その他の場合：式文として処理

        match self.current_token() {
            Token::If => {
                let stmt = self.parse_if_statement()?;
                Ok(stmt)
            }
            Token::Let => {
                let stmt = self.parse_let_statement()?;
                Ok(stmt)
            }
            Token::While => {
                let stmt = self.parse_while_statement()?;
                Ok(stmt)
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    // 関数の引数リストを解析
    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
        // lesson_2_6と同じ実装
        let mut arguments = Vec::new();

        if matches!(self.current_token(), Token::RightParen) {
            return Ok(arguments);
        }

        arguments.push(self.parse_expression()?);

        while matches!(self.current_token(), Token::Comma) {
            self.advance();

            if matches!(self.current_token(), Token::RightParen) {
                return Err("Trailing comma in function arguments".to_string());
            }

            arguments.push(self.parse_expression()?);
        }

        Ok(arguments)
    }

    // 数値、識別子、括弧式、または関数呼び出しを解析（最高優先度）
    fn parse_primary(&mut self) -> Result<Expr, String> {
        // lesson_2_6と同じ実装
        match self.current_token() {
            Token::Number(n) => {
                let result = Expr::Number(*n);
                self.advance();
                Ok(result)
            }
            Token::Identifier(name) => {
                let ident = name.clone();
                self.advance();

                match self.current_token() {
                    Token::LeftParen => {
                        self.advance();
                        let arguments = self.parse_arguments()?;
                        self.expect(Token::RightParen)?;
                        Ok(Expr::FunctionCall {
                            name: ident,
                            arguments,
                        })
                    }
                    _ => Ok(Expr::Identifier(ident)),
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::Grouped(Box::new(expr)))
            }
            token => Err(format!(
                "Expected number, identifier, or '(', found {:?}",
                token
            )),
        }
    }

    // 乗算・除算を解析（中優先度）
    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        // lesson_2_6と同じ実装
        let mut left = self.parse_primary()?;

        while matches!(self.current_token(), Token::Star | Token::Slash) {
            let op = match self.current_token() {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_primary()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 加算・減算を解析（中優先度）
    fn parse_additive(&mut self) -> Result<Expr, String> {
        // lesson_2_6と同じ実装
        let mut left = self.parse_multiplicative()?;

        while matches!(self.current_token(), Token::Plus | Token::Minus) {
            let op = match self.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_multiplicative()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 比較演算を解析（低優先度）
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        // lesson_2_6と同じ実装
        let mut left = self.parse_additive()?;

        while matches!(
            self.current_token(),
            Token::Greater | Token::Less | Token::Equal | Token::NotEqual
        ) {
            let op = match self.current_token() {
                Token::Greater => BinaryOp::Greater,
                Token::Less => BinaryOp::Less,
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };

            self.advance();

            let right = self.parse_additive()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    // 式全体を解析
    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_comparison()
    }

    // プログラム全体を解析
    fn parse_program(&mut self) -> Result<Program, String> {
        // lesson_2_6と同じ実装
        let mut statements = Vec::new();

        while !matches!(self.current_token(), Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        Ok(Program { statements })
    }
}

pub fn parse_program(tokens: Vec<Token>) -> Result<Program, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_while() {
        let tokens = vec![
            Token::While,
            Token::Identifier("x".to_string()),
            Token::Greater,
            Token::Number(0),
            Token::LeftBrace,
            Token::Identifier("print".to_string()),
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    operator: BinaryOp::Greater,
                    right: Box::new(Expr::Number(0)),
                },
                body: Block {
                    statements: vec![Stmt::Expression(Expr::FunctionCall {
                        name: "print".to_string(),
                        arguments: vec![Expr::Identifier("x".to_string())],
                    })],
                },
            }]
        );
    }

    #[test]
    fn test_parse_while_with_multiple_statements() {
        let tokens = vec![
            Token::While,
            Token::Identifier("i".to_string()),
            Token::Less,
            Token::Number(5),
            Token::LeftBrace,
            Token::Identifier("print".to_string()),
            Token::LeftParen,
            Token::Identifier("i".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Let,
            Token::Identifier("i".to_string()),
            Token::Assign,
            Token::Identifier("i".to_string()),
            Token::Plus,
            Token::Number(1),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier("i".to_string())),
                    operator: BinaryOp::Less,
                    right: Box::new(Expr::Number(5)),
                },
                body: Block {
                    statements: vec![
                        Stmt::Expression(Expr::FunctionCall {
                            name: "print".to_string(),
                            arguments: vec![Expr::Identifier("i".to_string())],
                        }),
                        Stmt::LetDeclaration {
                            name: "i".to_string(),
                            value: Expr::Binary {
                                left: Box::new(Expr::Identifier("i".to_string())),
                                operator: BinaryOp::Add,
                                right: Box::new(Expr::Number(1)),
                            },
                        },
                    ],
                },
            }]
        );
    }

    #[test]
    fn test_parse_while_with_complex_condition() {
        let tokens = vec![
            Token::While,
            Token::Identifier("x".to_string()),
            Token::Greater,
            Token::Number(0),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::LeftBrace,
            Token::Let,
            Token::Identifier("z".to_string()),
            Token::Assign,
            Token::Number(42),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    operator: BinaryOp::Greater,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Number(0)),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Identifier("y".to_string())),
                    }),
                },
                body: Block {
                    statements: vec![Stmt::LetDeclaration {
                        name: "z".to_string(),
                        value: Expr::Number(42),
                    }],
                },
            }]
        );
    }

    #[test]
    fn test_parse_empty_while_body() {
        let tokens = vec![
            Token::While,
            Token::Identifier("condition".to_string()),
            Token::Equal,
            Token::Number(1),
            Token::LeftBrace,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier("condition".to_string())),
                    operator: BinaryOp::Equal,
                    right: Box::new(Expr::Number(1)),
                },
                body: Block { statements: vec![] },
            }]
        );
    }

    #[test]
    fn test_parse_nested_while() {
        let tokens = vec![
            Token::While,
            Token::Identifier("x".to_string()),
            Token::Greater,
            Token::Number(0),
            Token::LeftBrace,
            Token::While,
            Token::Identifier("y".to_string()),
            Token::Less,
            Token::Number(10),
            Token::LeftBrace,
            Token::Let,
            Token::Identifier("z".to_string()),
            Token::Assign,
            Token::Number(5),
            Token::Semicolon,
            Token::RightBrace,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    operator: BinaryOp::Greater,
                    right: Box::new(Expr::Number(0)),
                },
                body: Block {
                    statements: vec![Stmt::While {
                        condition: Expr::Binary {
                            left: Box::new(Expr::Identifier("y".to_string())),
                            operator: BinaryOp::Less,
                            right: Box::new(Expr::Number(10)),
                        },
                        body: Block {
                            statements: vec![Stmt::LetDeclaration {
                                name: "z".to_string(),
                                value: Expr::Number(5),
                            }],
                        },
                    }],
                },
            }]
        );
    }

    #[test]
    fn test_parse_while_with_if_mixed() {
        let tokens = vec![
            Token::While,
            Token::Identifier("x".to_string()),
            Token::Greater,
            Token::Number(0),
            Token::LeftBrace,
            Token::If,
            Token::Identifier("x".to_string()),
            Token::Equal,
            Token::Number(5),
            Token::LeftBrace,
            Token::Identifier("break".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::Semicolon,
            Token::RightBrace,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::While {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    operator: BinaryOp::Greater,
                    right: Box::new(Expr::Number(0)),
                },
                body: Block {
                    statements: vec![Stmt::If {
                        condition: Expr::Binary {
                            left: Box::new(Expr::Identifier("x".to_string())),
                            operator: BinaryOp::Equal,
                            right: Box::new(Expr::Number(5)),
                        },
                        then_block: Block {
                            statements: vec![Stmt::Expression(Expr::FunctionCall {
                                name: "break".to_string(),
                                arguments: vec![],
                            })],
                        },
                        else_block: None,
                    }],
                },
            }]
        );
    }

    #[test]
    fn test_parse_program_with_multiple_statements() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(10),
            Token::Semicolon,
            Token::While,
            Token::Identifier("x".to_string()),
            Token::Greater,
            Token::Number(0),
            Token::LeftBrace,
            Token::Identifier("print".to_string()),
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Identifier("x".to_string()),
            Token::Minus,
            Token::Number(1),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(10),
                },
                Stmt::While {
                    condition: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::Greater,
                        right: Box::new(Expr::Number(0)),
                    },
                    body: Block {
                        statements: vec![
                            Stmt::Expression(Expr::FunctionCall {
                                name: "print".to_string(),
                                arguments: vec![Expr::Identifier("x".to_string())],
                            }),
                            Stmt::LetDeclaration {
                                name: "x".to_string(),
                                value: Expr::Binary {
                                    left: Box::new(Expr::Identifier("x".to_string())),
                                    operator: BinaryOp::Subtract,
                                    right: Box::new(Expr::Number(1)),
                                },
                            },
                        ],
                    },
                },
            ]
        );
    }

    #[test]
    fn test_parse_while_missing_condition() {
        let tokens = vec![
            Token::While,
            Token::LeftBrace,
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(1),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];
        let result = parse_program(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_while_missing_brace() {
        let tokens = vec![
            Token::While,
            Token::Identifier("x".to_string()),
            Token::Greater,
            Token::Number(0),
            Token::Let,
            Token::Identifier("y".to_string()),
            Token::Assign,
            Token::Number(1),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];
        let result = parse_program(tokens);
        assert!(result.is_err());
    }
}
