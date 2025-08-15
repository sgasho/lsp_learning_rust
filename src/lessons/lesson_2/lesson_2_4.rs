// Lesson 2-4へようこそ！
// lesson_2_3で括弧の処理を学びましたね。
// 今度は、関数呼び出しの構文解析について学びます。

// あなたのタスク：
// 関数呼び出しの構文解析を実装してください。
// 例：func(), add(1, 2), nested(func(x), y + z)

// トークンを拡張（カンマを追加）
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
    Eof,
}

// 抽象構文木（AST）のノード
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

    // 関数の引数リストを解析
    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
        // ヒント：
        // 1. 空の引数リストを作成
        // 2. ')'をチェック（引数なしの場合）
        // 3. 最初の引数をparse_expression()で解析
        // 4. ','が続く限りループ：
        //    - expect(Comma)でカンマを消費
        //    - parse_expression()で次の引数を解析
        //    - 引数リストに追加
        // 5. 引数リストを返す
        let mut arguments: Vec<Expr> = Vec::new();

        if matches!(self.current_token(), Token::RightParen) {
            return Ok(arguments);
        }

        arguments.push(self.parse_expression()?);

        while matches!(self.current_token(), Token::Comma) {
            self.advance();

            // 末尾カンマをチェック: func(1, 2,)
            if matches!(self.current_token(), Token::RightParen) {
                return Err("Trailing comma in function arguments".to_string());
            }

            arguments.push(self.parse_expression()?);
        }

        Ok(arguments)
    }

    // 数値、識別子、括弧式、または関数呼び出しを解析（最高優先度）
    fn parse_primary(&mut self) -> Result<Expr, String> {
        // ヒント：
        // 1. current_token()で現在のトークンを確認
        // 2. Number/LeftParenの場合は従来通り
        // 3. Identifierの場合は：
        //    - 識別子の名前を保存
        //    - advance()で識別子を消費
        //    - next_token()で次のトークンをチェック
        //    - '('の場合：関数呼び出し
        //      - advance()で'('を消費
        //      - parse_arguments()で引数を解析
        //      - expect(RightParen)で')'を確認
        //      - FunctionCall ASTノードを作成
        //    - '('以外の場合：通常の識別子
        //      - Identifier ASTノードを作成
        // 4. その他のトークンの場合はエラーを返す
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
                        let expr = self.parse_arguments()?;
                        self.expect(Token::RightParen)?;
                        Ok(Expr::FunctionCall {
                            name: ident,
                            arguments: expr,
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
        // ヒント：lesson_2_3と同じ実装でOK
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
            }
        }

        Ok(left)
    }

    // 加算・減算を解析（低優先度）
    fn parse_additive(&mut self) -> Result<Expr, String> {
        // ヒント：lesson_2_3と同じ実装でOK
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
            }
        }

        Ok(left)
    }

    // 式全体を解析
    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_additive()
    }
}

pub fn parse_expression(tokens: Vec<Token>) -> Result<Expr, String> {
    let mut parser = Parser::new(tokens);
    let result = parser.parse_expression()?;

    // 全てのトークンが消費されたかチェック
    match parser.current_token() {
        Token::Eof => Ok(result),
        token => Err(format!("Unexpected token after expression: {:?}", token)),
    }
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_call_no_args() {
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "func".to_string(),
                arguments: vec![],
            }
        );
    }

    #[test]
    fn test_parse_function_call_one_arg() {
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(42),
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "func".to_string(),
                arguments: vec![Expr::Number(42)],
            }
        );
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let tokens = vec![
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::Number(1),
            Token::Comma,
            Token::Number(2),
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "add".to_string(),
                arguments: vec![Expr::Number(1), Expr::Number(2)],
            }
        );
    }

    #[test]
    fn test_parse_function_call_expression_args() {
        // func(1 + 2, 3 * 4)
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Comma,
            Token::Number(3),
            Token::Star,
            Token::Number(4),
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "func".to_string(),
                arguments: vec![
                    Expr::Binary {
                        left: Box::new(Expr::Number(1)),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Number(2)),
                    },
                    Expr::Binary {
                        left: Box::new(Expr::Number(3)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expr::Number(4)),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_parse_nested_function_calls() {
        // outer(inner(42))
        let tokens = vec![
            Token::Identifier("outer".to_string()),
            Token::LeftParen,
            Token::Identifier("inner".to_string()),
            Token::LeftParen,
            Token::Number(42),
            Token::RightParen,
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "outer".to_string(),
                arguments: vec![Expr::FunctionCall {
                    name: "inner".to_string(),
                    arguments: vec![Expr::Number(42)],
                },],
            }
        );
    }

    #[test]
    fn test_parse_function_call_in_expression() {
        // 1 + func(2) * 3
        let tokens = vec![
            Token::Number(1),
            Token::Plus,
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(2),
            Token::RightParen,
            Token::Star,
            Token::Number(3),
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Number(1)),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::FunctionCall {
                        name: "func".to_string(),
                        arguments: vec![Expr::Number(2)],
                    }),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expr::Number(3)),
                }),
            }
        );
    }

    #[test]
    fn test_parse_identifier_vs_function_call() {
        // x + y (identifiers, not function calls)
        let tokens = vec![
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Identifier("y".to_string())),
            }
        );
    }

    #[test]
    fn test_parse_function_call_with_identifier_arg() {
        // func(x)
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "func".to_string(),
                arguments: vec![Expr::Identifier("x".to_string())],
            }
        );
    }

    #[test]
    fn test_parse_complex_function_call() {
        // func(a + b, inner(c), d * e + f)
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::Comma,
            Token::Identifier("inner".to_string()),
            Token::LeftParen,
            Token::Identifier("c".to_string()),
            Token::RightParen,
            Token::Comma,
            Token::Identifier("d".to_string()),
            Token::Star,
            Token::Identifier("e".to_string()),
            Token::Plus,
            Token::Identifier("f".to_string()),
            Token::RightParen,
            Token::Eof,
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(
            ast,
            Expr::FunctionCall {
                name: "func".to_string(),
                arguments: vec![
                    Expr::Binary {
                        left: Box::new(Expr::Identifier("a".to_string())),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Identifier("b".to_string())),
                    },
                    Expr::FunctionCall {
                        name: "inner".to_string(),
                        arguments: vec![Expr::Identifier("c".to_string())],
                    },
                    Expr::Binary {
                        left: Box::new(Expr::Binary {
                            left: Box::new(Expr::Identifier("d".to_string())),
                            operator: BinaryOp::Multiply,
                            right: Box::new(Expr::Identifier("e".to_string())),
                        }),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Identifier("f".to_string())),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_parse_missing_closing_paren() {
        // func(42
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(42),
            Token::Eof,
        ];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_comma() {
        // func(1 2)
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(1),
            Token::Number(2),
            Token::RightParen,
            Token::Eof,
        ];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_trailing_comma() {
        // func(1, 2,)
        let tokens = vec![
            Token::Identifier("func".to_string()),
            Token::LeftParen,
            Token::Number(1),
            Token::Comma,
            Token::Number(2),
            Token::Comma,
            Token::RightParen,
            Token::Eof,
        ];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }
}
