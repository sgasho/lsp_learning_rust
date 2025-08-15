// Lesson 2-1へようこそ！
// lesson_1シリーズが完了しましたね。
// 今度は、rust-analyzerの核心部分：字句解析（Lexing）について学びます。

// あなたのタスク：
// `tokenize` 関数は以下を受け取ります：
// - `input`: 解析対象のRustコード文字列
// 以下を行う必要があります：
// 1. 文字列を字句（トークン）に分割
// 2. 空白文字をスキップ
// 3. トークンの種類を判定
// 4. トークンのリストを返す

// トークンの種類
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

pub fn tokenize(input: &str) -> Vec<Token> {
    // TODO: この関数を実装してください
    // ヒント：
    // 1. positionを使って文字列を1文字ずつ読む
    // 2. 空白はスキップ
    // 3. 数値、識別子、演算子を判定
    // 4. 最後にEofトークンを追加

    let mut pos = 0;
    let mut tokens = Vec::new();

    while pos < input.len() {
        let char = input.chars().nth(pos).unwrap();
        match char {
            '+' => {
                tokens.push(Token::Plus);
                pos += 1;
            },
            '-' => {
                tokens.push(Token::Minus);
                pos += 1;
            },
            '*' => {
                tokens.push(Token::Star);
                pos += 1;
            },
            '/' => {
                tokens.push(Token::Slash);
                pos += 1;
            },
            '1'..='9' => {
                let (num, next_pos) = read_number(input, pos).unwrap();
                tokens.push(Token::Number(num));
                pos = next_pos;
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let (identifier, next_pos) = read_identifier(input, pos).unwrap();
                tokens.push(Token::Identifier(identifier));
                pos = next_pos;
            },
            ' ' => {
                pos = skip_whitespace(input, pos).unwrap();
            },
            _ => break,
        }
    }

    tokens.push(Token::Eof);
    tokens
}

fn skip_whitespace(input: &str, current_pos: usize) -> Option<usize> {
    match input.get(current_pos..)?.chars().position(|c| !c.is_whitespace()) {
        Some(end_pos) => Some(current_pos + end_pos),
        None => Some(input.len()),
    }
}

fn read_number(input: &str, current_pos: usize) -> Option<(i64, usize)> {
    let end_pos = input.get(current_pos..)?.chars().position(|c| !c.is_numeric());
    match end_pos {
        Some(pos) => {
            Some((input.get(current_pos..current_pos + pos)?.parse::<i64>().ok()?, current_pos + pos))
        },
        None => Some((input.get(current_pos..)?.parse::<i64>().ok()?, input.len())),
    }
    
}

fn read_identifier(input: &str, current_pos: usize) -> Option<(String, usize)> {
    let end_pos = input.get(current_pos..)?.chars().position(|c| !c.is_alphanumeric());
    match end_pos {
        Some(pos) => {
            Some((input.get(current_pos..current_pos + pos)?.to_owned(), current_pos + pos))
        },
        None => {
            Some((input.get(current_pos..)?.to_owned(), input.len()))
        },
    }
}


// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_number() {
        let tokens = tokenize("42");
        assert_eq!(tokens, vec![Token::Number(42), Token::Eof]);
    }

    #[test]
    fn test_tokenize_identifier() {
        let tokens = tokenize("x");
        assert_eq!(tokens, vec![Token::Identifier("x".to_string()), Token::Eof]);
    }

    #[test]
    fn test_tokenize_plus() {
        let tokens = tokenize("+");
        assert_eq!(tokens, vec![Token::Plus, Token::Eof]);
    }

    #[test]
    fn test_tokenize_minus() {
        let tokens = tokenize("-");
        assert_eq!(tokens, vec![Token::Minus, Token::Eof]);
    }

    #[test]
    fn test_tokenize_star() {
        let tokens = tokenize("*");
        assert_eq!(tokens, vec![Token::Star, Token::Eof]);
    }

    #[test]
    fn test_tokenize_slash() {
        let tokens = tokenize("/");
        assert_eq!(tokens, vec![Token::Slash, Token::Eof]);
    }

    #[test]
    fn test_tokenize_simple_expression() {
        let tokens = tokenize("1 + 2");
        assert_eq!(tokens, vec![
            Token::Number(1),
            Token::Plus,
            Token::Number(2),
            Token::Eof
        ]);
    }

    #[test]
    fn test_tokenize_with_whitespace() {
        let tokens = tokenize("  42   +   x  ");
        assert_eq!(tokens, vec![
            Token::Number(42),
            Token::Plus,
            Token::Identifier("x".to_string()),
            Token::Eof
        ]);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokens = tokenize("");
        assert_eq!(tokens, vec![Token::Eof]);
    }

    #[test]
    fn test_tokenize_only_whitespace() {
        let tokens = tokenize("   ");
        assert_eq!(tokens, vec![Token::Eof]);
    }
}