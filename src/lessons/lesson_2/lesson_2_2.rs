// Lesson 2-2へようこそ！
// lesson_2_1で字句解析を学びましたね。
// 今度は、rust-analyzerの次のステップ：構文解析（Parsing）について学びます。

// あなたのタスク：
// `parse_expression` 関数は以下を受け取ります：
// - `tokens`: lesson_2_1で作成したトークンのリスト
// 以下を行う必要があります：
// 1. トークンから数式の構造を理解
// 2. 演算子の優先順位を考慮
// 3. 抽象構文木（AST）を構築
// 4. ASTノードを返す

// トークンを再利用（lesson_2_1から）
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
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

    // 数値または識別子を解析（最高優先度）
    fn parse_primary(&mut self) -> Result<Expr, String> {
        // ヒント：
        // 1. current_token()で現在のトークンを確認
        // 2. Number/Identifierの場合は対応するASTノードを作成
        // 3. advance()で次のトークンに進む
        // 4. その他のトークンの場合はエラーを返す
        match self.current_token() {
            Token::Number(n) => {
                let result = Ok(Expr::Number(*n));
                self.advance();
                result
            },
            Token::Identifier(s) => {
                let result = Ok(Expr::Identifier(s.clone()));
                self.advance();
                result
            },
            token => Err(format!("Expected number or identifier, found {:?}", token)),
        }
    }

    // 乗算・除算を解析（中優先度）
    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        // ヒント：
        // 1. parse_primary()を呼んで左辺を取得
        // 2. Star/Slashが続く限りループ
        // 3. 各演算子で右辺をparse_primary()で取得
        // 4. Binary ASTノードを作成
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
        // ヒント：
        // 1. parse_multiplicative()を呼んで左辺を取得
        // 2. Plus/Minusが続く限りループ
        // 3. 各演算子で右辺をparse_multiplicative()で取得  
        // 4. Binary ASTノードを作成
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
}

pub fn parse_expression(tokens: Vec<Token>) -> Result<Expr, String> {
    // ヒント：
    // 1. Parserインスタンスを作成
    // 2. parse_additive()を呼んで式全体を解析
    // 3. 結果を返す
    let mut parser = Parser::new(tokens);
    parser.parse_additive()
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let tokens = vec![Token::Number(42), Token::Eof];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Number(42));
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = vec![Token::Identifier("x".to_string()), Token::Eof];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Identifier("x".to_string()));
    }

    #[test]
    fn test_parse_addition() {
        let tokens = vec![
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Number(1)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Number(2)),
        });
    }

    #[test]
    fn test_parse_subtraction() {
        let tokens = vec![
            Token::Number(5),
            Token::Minus,
            Token::Number(3),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Number(5)),
            operator: BinaryOp::Subtract,
            right: Box::new(Expr::Number(3)),
        });
    }

    #[test]
    fn test_parse_multiplication() {
        let tokens = vec![
            Token::Number(2),
            Token::Star,
            Token::Number(3),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Number(2)),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Number(3)),
        });
    }

    #[test]
    fn test_parse_division() {
        let tokens = vec![
            Token::Number(8),
            Token::Slash,
            Token::Number(2),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Number(8)),
            operator: BinaryOp::Divide,
            right: Box::new(Expr::Number(2)),
        });
    }

    #[test]
    fn test_parse_operator_precedence() {
        // 1 + 2 * 3 should be parsed as 1 + (2 * 3)
        let tokens = vec![
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Star,
            Token::Number(3),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Number(1)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(2)),
                operator: BinaryOp::Multiply,
                right: Box::new(Expr::Number(3)),
            }),
        });
    }

    #[test]
    fn test_parse_left_associativity() {
        // 1 - 2 - 3 should be parsed as (1 - 2) - 3
        let tokens = vec![
            Token::Number(1),
            Token::Minus,
            Token::Number(2),
            Token::Minus,
            Token::Number(3),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(1)),
                operator: BinaryOp::Subtract,
                right: Box::new(Expr::Number(2)),
            }),
            operator: BinaryOp::Subtract,
            right: Box::new(Expr::Number(3)),
        });
    }

    #[test]
    fn test_parse_complex_expression() {
        // x + 1 * 2 - 3 / 4
        let tokens = vec![
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Number(1),
            Token::Star,
            Token::Number(2),
            Token::Minus,
            Token::Number(3),
            Token::Slash,
            Token::Number(4),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        // Should be parsed as: ((x + (1 * 2)) - (3 / 4))
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Number(1)),
                    operator: BinaryOp::Multiply,
                    right: Box::new(Expr::Number(2)),
                }),
            }),
            operator: BinaryOp::Subtract,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Number(3)),
                operator: BinaryOp::Divide,
                right: Box::new(Expr::Number(4)),
            }),
        });
    }

    #[test]
    fn test_parse_empty_tokens() {
        let tokens = vec![Token::Eof];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_token() {
        let tokens = vec![Token::Plus, Token::Eof];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }
}