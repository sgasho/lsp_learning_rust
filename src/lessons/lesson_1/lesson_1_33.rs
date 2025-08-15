// Lesson 1-33へようこそ！
// Semantic Tokens機能ができるようになりましたね。
// 今度は、コードを整理する基本機能：Document Formatting（基本的なインデント修正）について学びます。

// あなたのタスク：
// `format_document` 関数は以下を受け取ります：
// - `content`: フォーマット対象のソースコード文字列
// 以下を行う必要があります：
// 1. 各行のインデントを正しく調整します
// 2. `{` の後は次の行のインデントを1レベル増加
// 3. `}` の行は前の行よりインデントを1レベル減少
// 4. 1つのインデントレベル = 4つのスペース
// 5. 空行のインデントは変更しません
// 6. フォーマット済みのコードを文字列として返します


pub fn format_document(content: &str) -> String {
    // ヒント：
    // 1. 各行を処理してインデントレベルを計算
    // 2. `{` と `}` でインデントレベルを調整
    // 3. 正しいインデントで各行を再構築
    let mut next_level = 0;

    let result = content
        .lines()
        .map(|line| {
            if line.is_empty() {
                return line.to_string();
            }
            
            let current_level = next_level;
            if line.contains('{') {
                next_level += 1;
                format!("{}{}", create_indent(current_level), line.trim_start())
            } else if line.contains('}') {
                next_level -= 1;
                format!("{}{}", create_indent(next_level), line.trim_start())
            } else {
                format!("{}{}", create_indent(current_level), line.trim_start())
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    
    result
}

// 指定されたレベルでインデント文字列を作成
fn create_indent(level: i32) -> String {
    " ".repeat((level * 4).max(0) as usize)
}


// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::format_document;

    #[test]
    fn test_format_simple_function() {
        let input = "fn main() {\nlet x = 1;\n}";
        let expected = "fn main() {\n    let x = 1;\n}";
        let result = format_document(input);
        
        assert_eq!(result, expected, "関数内のインデントが正しく調整されるべきです");
    }

    #[test]
    fn test_format_nested_blocks() {
        let input = "fn test() {\nif true {\nlet x = 1;\n}\n}";
        let expected = "fn test() {\n    if true {\n        let x = 1;\n    }\n}";
        let result = format_document(input);
        
        assert_eq!(result, expected, "ネストしたブロックのインデントが正しく調整されるべきです");
    }

    #[test]
    fn test_format_already_correct() {
        let input = "fn main() {\n    let x = 1;\n}";
        let expected = "fn main() {\n    let x = 1;\n}";
        let result = format_document(input);
        
        assert_eq!(result, expected, "既に正しいインデントは変更されないべきです");
    }

    #[test]
    fn test_format_empty_lines() {
        let input = "fn main() {\n\n    let x = 1;\n\n}";
        let expected = "fn main() {\n\n    let x = 1;\n\n}";
        let result = format_document(input);
        
        assert_eq!(result, expected, "空行のインデントは変更されないべきです");
    }

    #[test]
    fn test_format_struct() {
        let input = "struct User {\nname: String,\nage: u32,\n}";
        let expected = "struct User {\n    name: String,\n    age: u32,\n}";
        let result = format_document(input);
        
        assert_eq!(result, expected, "構造体のフィールドのインデントが正しく調整されるべきです");
    }

    #[test]
    fn test_format_no_braces() {
        let input = "let x = 1;\nlet y = 2;";
        let expected = "let x = 1;\nlet y = 2;";
        let result = format_document(input);
        
        assert_eq!(result, expected, "ブレースがない場合はインデントを変更しないべきです");
    }

    #[test]
    fn test_format_empty_content() {
        let input = "";
        let expected = "";
        let result = format_document(input);
        
        assert_eq!(result, expected, "空のコンテンツは空文字列を返すべきです");
    }
}