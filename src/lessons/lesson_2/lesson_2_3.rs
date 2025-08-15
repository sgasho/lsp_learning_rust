// Lesson 2-3へようこそ！
// lesson_2_2で構文解析を学びましたね。
// 今度は、括弧の処理とグループ化について学びます。

// あなたのタスク：
// 括弧を使った式の構文解析を実装してください。
// 例：(1 + 2) * 3, 1 + (2 * 3), ((1 + 2) * 3) + 4

// トークンを拡張（括弧を追加）
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,   // (
    RightParen,  // )
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
    Grouped(Box<Expr>),  // 括弧でグループ化された式
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
            Err(format!("Expected {:?}, found {:?}", expected, self.current_token()))
        }
    }

    // 数値、識別子、または括弧式を解析（最高優先度）
    fn parse_primary(&mut self) -> Result<Expr, String> {
        // ヒント：
        // 1. current_token()で現在のトークンを確認
        // 2. Number/Identifierの場合は対応するASTノードを作成
        // 3. LeftParenの場合は：
        //    - advance()で'('を消費
        //    - parse_expression()で括弧内の式を解析
        //    - expect(RightParen)で')'を確認
        //    - Grouped ASTノードを作成
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
            Token::LeftParen => {
                self.advance();
                let result = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::Grouped(Box::new(result)))
            },
            token => Err(format!("Expected number, identifier, or '(', found {:?}", token)),
        }
    }

    // 乗算・除算を解析（中優先度）
    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        // ヒント：lesson_2_2と同じ実装でOK
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
            }
        }

        Ok(left)
    }

    // 加算・減算を解析（低優先度）
    fn parse_additive(&mut self) -> Result<Expr, String> {
        // ヒント：lesson_2_2と同じ実装でOK
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
        Token::RightParen => Err("Unexpected closing parenthesis".to_string()),
        token => Err(format!("Unexpected token after expression: {:?}", token)),
    }
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_parentheses() {
        let tokens = vec![
            Token::LeftParen,
            Token::Number(42),
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Grouped(Box::new(Expr::Number(42))));
    }

    #[test]
    fn test_parse_parentheses_with_addition() {
        let tokens = vec![
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Grouped(Box::new(Expr::Binary {
            left: Box::new(Expr::Number(1)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Number(2)),
        })));
    }

    #[test]
    fn test_parse_parentheses_precedence() {
        // (1 + 2) * 3
        let tokens = vec![
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::RightParen,
            Token::Star,
            Token::Number(3),
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                left: Box::new(Expr::Number(1)),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Number(2)),
            }))),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Number(3)),
        });
    }

    #[test]
    fn test_parse_nested_parentheses() {
        // ((1 + 2) * 3)
        let tokens = vec![
            Token::LeftParen,
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::RightParen,
            Token::Star,
            Token::Number(3),
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Grouped(Box::new(Expr::Binary {
            left: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                left: Box::new(Expr::Number(1)),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Number(2)),
            }))),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Number(3)),
        })));
    }

    #[test]
    fn test_parse_mixed_precedence() {
        // 1 + (2 * 3)
        let tokens = vec![
            Token::Number(1),
            Token::Plus,
            Token::LeftParen,
            Token::Number(2),
            Token::Star,
            Token::Number(3),
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Number(1)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                left: Box::new(Expr::Number(2)),
                operator: BinaryOp::Multiply,
                right: Box::new(Expr::Number(3)),
            }))),
        });
    }

    #[test]
    fn test_parse_complex_expression() {
        // (1 + 2) * (3 - 4)
        let tokens = vec![
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::RightParen,
            Token::Star,
            Token::LeftParen,
            Token::Number(3),
            Token::Minus,
            Token::Number(4),
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                left: Box::new(Expr::Number(1)),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Number(2)),
            }))),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                left: Box::new(Expr::Number(3)),
                operator: BinaryOp::Subtract,
                right: Box::new(Expr::Number(4)),
            }))),
        });
    }

    #[test]
    fn test_parse_identifier_with_parentheses() {
        // x * (y + z)
        let tokens = vec![
            Token::Identifier("x".to_string()),
            Token::Star,
            Token::LeftParen,
            Token::Identifier("y".to_string()),
            Token::Plus,
            Token::Identifier("z".to_string()),
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Binary {
            left: Box::new(Expr::Identifier("x".to_string())),
            operator: BinaryOp::Multiply,
            right: Box::new(Expr::Grouped(Box::new(Expr::Binary {
                left: Box::new(Expr::Identifier("y".to_string())),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Identifier("z".to_string())),
            }))),
        });
    }

    #[test]
    fn test_parse_mismatched_parentheses() {
        // (1 + 2  (missing closing paren)
        let tokens = vec![
            Token::LeftParen,
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Eof
        ];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_extra_closing_paren() {
        // 1 + 2)  (extra closing paren)
        let tokens = vec![
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::RightParen,
            Token::Eof
        ];
        let result = parse_expression(tokens);
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_parentheses() {
        // ()
        let tokens = vec![
            Token::LeftParen,
            Token::RightParen,
            Token::Eof
        ];
        let result = parse_expression(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_nested_parentheses() {
        // (((1)))
        let tokens = vec![
            Token::LeftParen,
            Token::LeftParen,
            Token::LeftParen,
            Token::Number(1),
            Token::RightParen,
            Token::RightParen,
            Token::RightParen,
            Token::Eof
        ];
        let ast = parse_expression(tokens).unwrap();
        assert_eq!(ast, Expr::Grouped(Box::new(Expr::Grouped(Box::new(Expr::Grouped(Box::new(Expr::Number(1))))))));
    }
}