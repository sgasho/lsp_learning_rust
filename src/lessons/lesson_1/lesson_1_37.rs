// Lesson 1-37へようこそ！
// Code Lens機能ができるようになりましたね。
// 今度は、関連する箇所を同時に編集する機能：Linked Editing（連動編集）について学びます。

// あなたのタスク：
// `provide_linked_editing_ranges` 関数は以下を受け取ります：
// - `content`: 解析対象のソースコード文字列
// - `position`: 編集したい位置
// 以下を行う必要があります：
// 1. 指定位置にある識別子を検出します
// 2. 同じスコープ内で同じ名前の識別子をすべて見つけます：
//    - 変数の宣言と使用箇所
//    - 関数名の定義と呼び出し箇所
//    - 構造体名の定義と使用箇所
// 3. 連動編集可能な範囲のリストを返します
// 4. 該当する識別子がない場合は None を返します

use lsp_types::{LinkedEditingRanges, Position, Range};

pub fn provide_linked_editing_ranges(content: &str, position: Position) -> Option<LinkedEditingRanges> {
    // ヒント：
    // 1. 指定位置の識別子を取得
    // 2. 同じ識別子の出現箇所をすべ検索
    // 3. LinkedEditingRanges を作成
    let ident = get_identifier_at_position(content, position)?;
    let ranges = find_all_occurrences(content, ident.as_str());

    Some(LinkedEditingRanges {
        ranges,
        word_pattern: ident.into()
    })
}

// 指定位置の識別子を取得する
fn get_identifier_at_position(content: &str, position: Position) -> Option<String> {
    let line = content.lines().nth(position.line as usize)?;
    let chars: Vec<char> = line.chars().collect();
    
    if position.character as usize >= chars.len() {
        return None;
    }
    
    let pos = position.character as usize;
    
    // 指定位置が識別子文字でない場合はNoneを返す
    if !is_identifier_char(chars[pos]) {
        return None;
    }
    
    // 識別子の開始位置を探す
    let mut start_pos = pos;
    while start_pos > 0 && is_identifier_char(chars[start_pos - 1]) {
        start_pos -= 1;
    }
    
    // 識別子の終了位置を探す
    let mut end_pos = pos;
    while end_pos < chars.len() && is_identifier_char(chars[end_pos]) {
        end_pos += 1;
    }
    
    let identifier = chars[start_pos..end_pos].iter().collect::<String>();
    
    // 識別子の開始文字が有効かチェック
    if !is_identifier_start(chars[start_pos]) {
        return None;
    }
    
    Some(identifier)
}

// 識別子のすべての出現箇所を検索する
fn find_all_occurrences(content: &str, identifier: &str) -> Vec<Range> {
    let mut ranges = Vec::new();
    
    for (line_number, line) in content.lines().enumerate() {
        let mut start_pos = 0;
        
        // 各行で複数の出現箇所を検索
        while let Some(pos) = line[start_pos..].find(identifier) {
            let absolute_pos = start_pos + pos;
            
            // 単語境界をチェック
            let is_word_boundary = {
                let before_valid = absolute_pos == 0 || 
                    !is_identifier_char(line.chars().nth(absolute_pos - 1).unwrap_or(' '));
                let after_valid = absolute_pos + identifier.len() >= line.len() || 
                    !is_identifier_char(line.chars().nth(absolute_pos + identifier.len()).unwrap_or(' '));
                before_valid && after_valid
            };
            
            if is_word_boundary {
                ranges.push(Range::new(
                    Position::new(line_number as u32, absolute_pos as u32),
                    Position::new(line_number as u32, (absolute_pos + identifier.len()) as u32),
                ));
            }
            
            start_pos = absolute_pos + 1;
        }
    }
    
    ranges
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
    use super::provide_linked_editing_ranges;
    use lsp_types::Position;

    #[test]
    fn test_variable_linked_editing() {
        let content = "let variable = 42;\nlet result = variable + 1;\nprintln!(\"{}\", variable);";
        let position = Position::new(0, 6); // "variable"の中
        let ranges = provide_linked_editing_ranges(content, position);

        assert!(ranges.is_some(), "変数の連動編集範囲が返されるべきです");
        
        let linked_ranges = ranges.unwrap();
        assert_eq!(linked_ranges.ranges.len(), 3, "variableの3つの出現箇所が検出されるべきです");
        
        // 宣言箇所
        let declaration = linked_ranges.ranges.iter().find(|r| r.start.line == 0).unwrap();
        assert_eq!(declaration.start.character, 4, "宣言の開始位置が正しいべきです");
        assert_eq!(declaration.end.character, 12, "宣言の終了位置が正しいべきです");
        
        // 使用箇所
        let usage1 = linked_ranges.ranges.iter().find(|r| r.start.line == 1 && r.start.character == 13).unwrap();
        assert_eq!(usage1.end.character, 21, "使用箇所1の終了位置が正しいべきです");
        
        let usage2 = linked_ranges.ranges.iter().find(|r| r.start.line == 2).unwrap();
        assert_eq!(usage2.start.character, 15, "使用箇所2の開始位置が正しいべきです");
    }

    #[test]
    fn test_function_linked_editing() {
        let content = "fn calculate(x: i32) -> i32 {\n    x * 2\n}\n\nfn main() {\n    let result = calculate(10);\n}";
        let position = Position::new(0, 4); // "calculate"の中
        let ranges = provide_linked_editing_ranges(content, position);
        
        assert!(ranges.is_some(), "関数の連動編集範囲が返されるべきです");
        
        let linked_ranges = ranges.unwrap();
        assert_eq!(linked_ranges.ranges.len(), 2, "calculateの2つの出現箇所が検出されるべきです");
        
        // 定義箇所
        let definition = linked_ranges.ranges.iter().find(|r| r.start.line == 0).unwrap();
        assert_eq!(definition.start.character, 3, "定義の開始位置が正しいべきです");
        
        // 呼び出し箇所
        let call = linked_ranges.ranges.iter().find(|r| r.start.line == 5).unwrap();
        assert_eq!(call.start.character, 17, "呼び出しの開始位置が正しいべきです");
    }

    #[test]
    fn test_struct_linked_editing() {
        let content = "struct User {\n    name: String,\n}\n\nfn create_user() -> User {\n    User { name: \"test\".to_string() }\n}";
        let position = Position::new(0, 8); // "User"の中
        let ranges = provide_linked_editing_ranges(content, position);
        
        assert!(ranges.is_some(), "構造体の連動編集範囲が返されるべきです");
        
        let linked_ranges = ranges.unwrap();
        assert_eq!(linked_ranges.ranges.len(), 3, "Userの3つの出現箇所が検出されるべきです");
    }

    #[test]
    fn test_single_occurrence() {
        let content = "let unique_variable = 42;";
        let position = Position::new(0, 6); // "unique_variable"の中
        let ranges = provide_linked_editing_ranges(content, position);
        
        assert!(ranges.is_some(), "単一出現でも連動編集範囲が返されるべきです");
        
        let linked_ranges = ranges.unwrap();
        assert_eq!(linked_ranges.ranges.len(), 1, "1つの出現箇所が検出されるべきです");
    }

    #[test]
    fn test_no_identifier_at_position() {
        let content = "let x = 42;";
        let position = Position::new(0, 3); // スペースの位置
        let ranges = provide_linked_editing_ranges(content, position);
        
        assert!(ranges.is_none(), "識別子がない位置では None を返すべきです");
    }

    #[test]
    fn test_invalid_position() {
        let content = "let x = 42;";
        let position = Position::new(10, 0); // 存在しない行
        let ranges = provide_linked_editing_ranges(content, position);
        
        assert!(ranges.is_none(), "無効な位置では None を返すべきです");
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let position = Position::new(0, 0);
        let ranges = provide_linked_editing_ranges(content, position);
        
        assert!(ranges.is_none(), "空のコンテンツでは None を返すべきです");
    }
}