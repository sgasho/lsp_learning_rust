// Lesson 2-5へようこそ！
// lesson_2_4で関数呼び出しを学びましたね。
// 今度は、変数宣言とlet文の構文解析について学びます。

// あなたのタスク：
// let文の構文解析を実装してください。
// 例：let x = 42; let y = x + 1; let z = func(x, y);

use crate::lessons::lesson_2::lesson_2_5::Stmt::LetDeclaration;

// トークンを拡張（let, =, ; を追加）
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
    Eof,
}

// 抽象構文木（AST）のノード
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // 文（Statement）
    LetDeclaration { name: String, value: Expr },
    Expression(Expr), // 式文
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

    // let文を解析
    fn parse_let_statement(&mut self) -> Result<Stmt, String> {
        // ヒント：
        // 1. expect(Token::Let)でletキーワードを消費
        // 2. current_token()で変数名（Identifier）を取得
        // 3. advance()で変数名を消費
        // 4. expect(Token::Assign)で=を確認
        // 5. parse_expression()で値の式を解析
        // 6. expect(Token::Semicolon)で;を確認
        // 7. LetDeclaration ASTノードを作成
        self.expect(Token::Let)?;

        let ident = match self.current_token() {
            Token::Identifier(s) => s.clone(),
            t => return Err(format!("Expected identifier, found {:?}", t)),
        };
        self.advance();

        self.expect(Token::Assign)?;

        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;

        Ok(LetDeclaration {
            name: ident,
            value: expr,
        })
    }

    // 文を解析
    fn parse_statement(&mut self) -> Result<Stmt, String> {
        // ヒント：
        // 1. current_token()で現在のトークンを確認
        // 2. Token::Letの場合：parse_let_statement()を呼び出し
        // 3. その他の場合：式文として処理
        //    - parse_expression()で式を解析
        //    - expect(Token::Semicolon)で;を確認
        //    - Expression ASTノードを作成

        match self.current_token() {
            Token::Let => self.parse_let_statement(),
            _ => {
                let expr = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    // 関数の引数リストを解析
    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
        // lesson_2_4と同じ実装
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
        // lesson_2_4と同じ実装
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
        // lesson_2_4と同じ実装
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

    // 加算・減算を解析（低優先度）
    fn parse_additive(&mut self) -> Result<Expr, String> {
        // lesson_2_4と同じ実装
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

    // 式全体を解析
    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_additive()
    }

    // プログラム全体を解析
    fn parse_program(&mut self) -> Result<Program, String> {
        // ヒント：
        // 1. 空の文リストを作成
        // 2. Eofでない限りループ
        // 3. parse_statement()で文を解析
        // 4. 文リストに追加
        // 5. Program構造体を作成して返す
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
    fn test_parse_simple_let() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(42),
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::LetDeclaration {
                name: "x".to_string(),
                value: Expr::Number(42),
            }]
        );
    }

    #[test]
    fn test_parse_let_with_expression() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("result".to_string()),
            Token::Assign,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Star,
            Token::Number(3),
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::LetDeclaration {
                name: "result".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Number(1)),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Number(2)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expr::Number(3)),
                    }),
                },
            }]
        );
    }

    #[test]
    fn test_parse_let_with_function_call() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("value".to_string()),
            Token::Assign,
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(1),
            Token::Comma,
            Token::Number(2),
            Token::RightParen,
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::LetDeclaration {
                name: "value".to_string(),
                value: Expr::FunctionCall {
                    name: "func".to_string(),
                    arguments: vec![Expr::Number(1), Expr::Number(2)],
                },
            }]
        );
    }

    #[test]
    fn test_parse_let_with_identifier() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("y".to_string()),
            Token::Assign,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Number(1),
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::LetDeclaration {
                name: "y".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".to_string())),
                    operator: BinaryOp::Add,
                    right: Box::new(Expr::Number(1)),
                },
            }]
        );
    }

    #[test]
    fn test_parse_multiple_let_statements() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(10),
            Token::Semicolon,
            Token::Let,
            Token::Identifier("y".to_string()),
            Token::Assign,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Number(5),
            Token::Semicolon,
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
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Number(5)),
                    },
                }
            ]
        );
    }

    #[test]
    fn test_parse_expression_statement() {
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(42),
            Token::RightParen,
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::Expression(Expr::FunctionCall {
                name: "func".to_string(),
                arguments: vec![Expr::Number(42)],
            })]
        );
    }

    #[test]
    fn test_parse_mixed_statements() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(5),
            Token::Semicolon,
            Token::Identifier("print".to_string()),
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Let,
            Token::Identifier("y".to_string()),
            Token::Assign,
            Token::Identifier("x".to_string()),
            Token::Star,
            Token::Number(2),
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![
                Stmt::LetDeclaration {
                    name: "x".to_string(),
                    value: Expr::Number(5),
                },
                Stmt::Expression(Expr::FunctionCall {
                    name: "print".to_string(),
                    arguments: vec![Expr::Identifier("x".to_string())],
                }),
                Stmt::LetDeclaration {
                    name: "y".to_string(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Identifier("x".to_string())),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expr::Number(2)),
                    },
                }
            ]
        );
    }

    #[test]
    fn test_parse_missing_semicolon() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Number(42),
            Token::Eof,
        ];
        let result = parse_program(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_assignment() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Number(42),
            Token::Semicolon,
            Token::Eof,
        ];
        let result = parse_program(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_variable_name() {
        let tokens = vec![
            Token::Let,
            Token::Assign,
            Token::Number(42),
            Token::Semicolon,
            Token::Eof,
        ];
        let result = parse_program(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_program() {
        let tokens = vec![Token::Eof];
        let program = parse_program(tokens).unwrap();
        assert_eq!(program.statements, vec![]);
    }

    #[test]
    fn test_parse_let_with_parentheses() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("result".to_string()),
            Token::Assign,
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::RightParen,
            Token::Star,
            Token::Number(3),
            Token::Semicolon,
            Token::Eof,
        ];
        let program = parse_program(tokens).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::LetDeclaration {
                name: "result".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                        left: Box::new(Expr::Number(1)),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Number(2)),
                    }))),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expr::Number(3)),
                },
            }]
        );
    }
}
