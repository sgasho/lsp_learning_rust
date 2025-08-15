// Lesson 1-32へようこそ！
// Call Hierarchy機能ができるようになりましたね。
// 今度は、コードを美しくハイライトする機能：Semantic Tokens（セマンティックトークン）について学びます。

// あなたのタスク：
// `provide_semantic_tokens` 関数は以下を受け取ります：
// - `content`: 解析対象のソースコード文字列
// 以下を行う必要があります：
// 1. コード内のトークン（キーワード、識別子、文字列など）を解析します
// 2. 各トークンの種類を特定します（例：keyword, function, variable, string）
// 3. 各トークンの位置情報（行、列、長さ）を計算します
// 4. `SemanticToken` オブジェクトのリストを作成します：
//    - `delta_line`: 前のトークンからの行の差分
//    - `delta_start`: 前のトークンからの列の差分（同じ行の場合）
//    - `length`: トークンの長さ
//    - `token_type`: トークンの種類のインデックス
//    - `token_modifiers_bitset`: トークンの修飾子（今回は0）
// 5. すべてのセマンティックトークンを含む `SemanticTokens` を返します
// 6. トークンが見つからない場合は、空のトークンリストを返します

use lsp_types::{SemanticTokens};

pub fn provide_semantic_tokens(content: &str) -> SemanticTokens {
    let mut tokens = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    let mut prev_token_literal = String::new();
    
    for (line_number, line) in content.lines().enumerate() {
        let mut char_indices = line.char_indices().peekable();
        
        while let Some((start_pos, ch)) = char_indices.next() {
            match ch {
                // 文字列リテラルの処理
                '"' => {
                    let mut end_pos = start_pos;
                    let mut escaped = false;
                    
                    // 文字列の終端を見つける
                    while let Some((pos, next_ch)) = char_indices.next() {
                        end_pos = pos;
                        if escaped {
                            escaped = false;
                            continue;
                        }
                        if next_ch == '\\' {
                            escaped = true;
                        } else if next_ch == '"' {
                            break;
                        }
                    }
                    
                    let length = end_pos - start_pos + 1;
                    let (delta_line, delta_start) = calculate_delta(
                        line_number as u32, start_pos as u32, prev_line, prev_start
                    );
                    
                    tokens.push(lsp_types::SemanticToken {
                        delta_line,
                        delta_start,
                        length: length as u32,
                        token_type: 3, // STRING
                        token_modifiers_bitset: 0,
                    });
                    
                    prev_line = line_number as u32;
                    prev_start = start_pos as u32;
                }
                
                // 数値リテラルの処理
                c if c.is_ascii_digit() => {
                    let mut end_pos = start_pos;
                    
                    // 数値の終端を見つける
                    while let Some((pos, next_ch)) = char_indices.peek() {
                        if next_ch.is_ascii_digit() {
                            end_pos = *pos;
                            char_indices.next();
                        } else {
                            break;
                        }
                    }
                    
                    let length = end_pos - start_pos + 1;
                    let (delta_line, delta_start) = calculate_delta(
                        line_number as u32, start_pos as u32, prev_line, prev_start
                    );
                    
                    tokens.push(lsp_types::SemanticToken {
                        delta_line,
                        delta_start,
                        length: length as u32,
                        token_type: 4, // NUMBER
                        token_modifiers_bitset: 0,
                    });
                    
                    prev_line = line_number as u32;
                    prev_start = start_pos as u32;
                }
                
                // 識別子（キーワード、関数名、変数名）の処理
                c if is_identifier_start(c) => {
                    let mut end_pos = start_pos;
                    
                    // 識別子の終端を見つける
                    while let Some((pos, next_ch)) = char_indices.peek() {
                        if is_identifier_char(*next_ch) {
                            end_pos = *pos;
                            char_indices.next();
                        } else {
                            break;
                        }
                    }
                    
                    let token_text = &line[start_pos..=end_pos];

                    if let Some((token_type, token_literal)) = get_token_type(token_text, prev_token_literal.as_str()) {
                        prev_token_literal = token_literal;
                        let length = end_pos - start_pos + 1;
                        let (delta_line, delta_start) = calculate_delta(
                            line_number as u32, start_pos as u32, prev_line, prev_start
                        );
                        
                        tokens.push(lsp_types::SemanticToken {
                            delta_line,
                            delta_start,
                            length: length as u32,
                            token_type,
                            token_modifiers_bitset: 0,
                        });
                        
                        prev_line = line_number as u32;
                        prev_start = start_pos as u32;
                    }
                }
                
                // その他の文字はスキップ
                _ => continue,
            }
        }
    }
    
    SemanticTokens {
        result_id: None,
        data: tokens,
    }
}

// Delta エンコーディングの計算
fn calculate_delta(line: u32, start: u32, prev_line: u32, prev_start: u32) -> (u32, u32) {
    let delta_line = line - prev_line;
    let delta_start = if delta_line == 0 {
        start - prev_start
    } else {
        start
    };
    (delta_line, delta_start)
}

// トークンの種類を判定するヘルパー関数
fn get_token_type(token: &str, prev_token_literal: &str) -> Option<(u32, String)> {
    match token {
        // Rustキーワード (インデックス 0)
        "fn" | "let" | "mut" | "if" | "else" | "for" | "while" | "match" 
        | "struct" | "enum" | "impl" | "trait" | "use" | "pub" | "return"
        | "const" | "static" | "mod" | "as" | "where" | "self" | "Self" => Some((0, token.to_string())),
        
        // Rust基本型 (インデックス 5)
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" 
        | "f32" | "f64" | "bool" | "char" | "str" | "String" => Some((5, token.to_string())),
        
        // その他の識別子は文脈によって判定（デフォルトは変数）
        _ => {
            if token.chars().all(|c| c.is_alphanumeric() || c == '_') 
                && token.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) {
                if prev_token_literal == "fn" {
                    Some((1, token.to_string()))
                } else {
                    Some((2, token.to_string())) // VARIABLE として扱う
                }
            } else {
                None
            }
        }
    }
}

// 文字が識別子に使用できるかチェック
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// 文字が識別子の開始文字として有効かチェック  
fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}


// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::provide_semantic_tokens;

    // 基本的なRustコードのサンプルを作成
    fn create_sample_rust_code() -> String {
        "fn main() {\n    let x = 42;\n    println!(\"Hello {}\", x);\n}".to_string()
    }

    #[test]
    fn test_semantic_tokens_basic_function() {
        let code = create_sample_rust_code();
        let tokens = provide_semantic_tokens(&code);
        
        // トークンが検出されることを確認
        assert!(!tokens.data.is_empty(), "セマンティックトークンが検出されるべきです");
        
        // 最初のトークンは 'fn' キーワードであることを確認
        let first_token = &tokens.data[0];
        assert_eq!(first_token.delta_line, 0, "最初のトークンは0行目にあるべきです");
        assert_eq!(first_token.delta_start, 0, "最初のトークンは行の先頭にあるべきです"); 
        assert_eq!(first_token.length, 2, "'fn' の長さは2文字です");
        assert_eq!(first_token.token_type, 0, "'fn' はキーワード（インデックス0）です");
    }

    #[test]
    fn test_semantic_tokens_function_name() {
        let code = "fn calculate() {}".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // 少なくとも2つのトークンが検出されることを確認（fn + calculate）
        assert!(tokens.data.len() >= 2, "'fn' と 'calculate' の2つのトークンが検出されるべきです");
        
        // 2番目のトークンは関数名 'calculate' であることを確認
        let function_token = &tokens.data[1];
        assert_eq!(function_token.length, 9, "'calculate' の長さは9文字です");
        assert_eq!(function_token.token_type, 1, "'calculate' は関数名（インデックス1）です");
    }

    #[test]
    fn test_semantic_tokens_variables() {
        let code = "let mut count = 10;".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // キーワードと変数名が検出されることを確認
        assert!(tokens.data.len() >= 3, "'let', 'mut', 'count' のトークンが検出されるべきです");
        
        // 'let' キーワードの確認
        let let_token = &tokens.data[0];
        assert_eq!(let_token.token_type, 0, "'let' はキーワード（インデックス0）です");
        
        // 'mut' キーワードの確認
        let mut_token = &tokens.data[1];
        assert_eq!(mut_token.token_type, 0, "'mut' はキーワード（インデックス0）です");
        
        // 'count' 変数名の確認
        let var_token = &tokens.data[2];
        assert_eq!(var_token.token_type, 2, "'count' は変数名（インデックス2）です");
    }

    #[test]
    fn test_semantic_tokens_string_literals() {
        let code = "println!(\"Hello World\");".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // 文字列リテラルが検出されることを確認
        let string_tokens: Vec<_> = tokens.data.iter()
            .filter(|token| token.token_type == 3) // STRING type
            .collect();
        
        assert!(!string_tokens.is_empty(), "文字列リテラルが検出されるべきです");
        
        let string_token = string_tokens[0];
        assert_eq!(string_token.length, 13, "\"Hello World\" の長さは13文字です");
    }

    #[test]
    fn test_semantic_tokens_numbers() {
        let code = "let value = 42;".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // 数値リテラルが検出されることを確認
        let number_tokens: Vec<_> = tokens.data.iter()
            .filter(|token| token.token_type == 4) // NUMBER type
            .collect();
        
        assert!(!number_tokens.is_empty(), "数値リテラルが検出されるべきです");
        
        let number_token = number_tokens[0];
        assert_eq!(number_token.length, 2, "'42' の長さは2文字です");
    }

    #[test]
    fn test_semantic_tokens_types() {
        let code = "let x: i32 = 0;".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // 型名が検出されることを確認
        let type_tokens: Vec<_> = tokens.data.iter()
            .filter(|token| token.token_type == 5) // TYPE
            .collect();
        
        assert!(!type_tokens.is_empty(), "型名が検出されるべきです");
        
        let type_token = type_tokens[0];
        assert_eq!(type_token.length, 3, "'i32' の長さは3文字です");
    }

    #[test]
    fn test_semantic_tokens_delta_encoding() {
        let code = "fn test() {\n    let x = 1;\n}".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // Delta エンコーディングが正しく動作することを確認
        assert!(!tokens.data.is_empty(), "トークンが検出されるべきです");
        
        // 最初のトークンは delta_line = 0, delta_start = 0
        let first_token = &tokens.data[0];
        assert_eq!(first_token.delta_line, 0);
        assert_eq!(first_token.delta_start, 0);
        
        // 2行目のトークンは delta_line > 0 を持つはず
        let second_line_tokens: Vec<_> = tokens.data.iter()
            .filter(|token| token.delta_line > 0)
            .collect();
        
        assert!(!second_line_tokens.is_empty(), "2行目にトークンが検出されるべきです");
    }

    #[test]
    fn test_semantic_tokens_empty_code() {
        let code = "".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // 空のコードは空のトークンリストを返すべき
        assert!(tokens.data.is_empty(), "空のコードは空のトークンリストを返すべきです");
    }

    #[test]
    fn test_semantic_tokens_complex_code() {
        let code = "struct User {\n    name: String,\n    age: u32,\n}\n\nfn create_user() -> User {\n    User {\n        name: \"Alice\".to_string(),\n        age: 30,\n    }\n}".to_string();
        let tokens = provide_semantic_tokens(&code);
        
        // 複雑なコードでも適切にトークンが検出されることを確認
        assert!(tokens.data.len() > 10, "複雑なコードは多数のトークンを生成するべきです");
        
        // 様々なトークンタイプが検出されることを確認
        let token_types: std::collections::HashSet<u32> = tokens.data.iter()
            .map(|token| token.token_type)
            .collect();
        
        assert!(token_types.len() >= 3, "少なくとも3種類のトークンタイプが検出されるべきです");
    }
}