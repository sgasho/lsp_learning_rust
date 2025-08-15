// Lesson 1-34へようこそ！
// Document Formatting機能ができるようになりましたね。
// 今度は、コードを見やすく整理する機能：Folding Range（コード折りたたみ）について学びます。

// あなたのタスク：
// `provide_folding_ranges` 関数は以下を受け取ります：
// - `content`: 解析対象のソースコード文字列
// 以下を行う必要があります：
// 1. コード内の折りたたみ可能な範囲を検出します：
//    - 関数ブロック（`fn` から `}`まで）
//    - 構造体定義（`struct` から `}`まで）
//    - if文ブロック（`if` から `}`まで）
// 2. 各範囲の開始行と終了行を特定します
// 3. `FoldingRange` オブジェクトを作成します
// 4. すべての折りたたみ範囲を返します

use std::str::Lines;
use lsp_types::{FoldingRange, FoldingRangeKind};

pub fn provide_folding_ranges(content: &str) -> Vec<FoldingRange> {
    // ヒント：
    // 1. 各行を処理してブロックの開始を検出
    // 2. 対応する終了ブレースを見つける
    // 3. FoldingRange オブジェクトを作成
    let mut current_line_number = 0;
    let mut folding_ranges = vec![];

    while current_line_number < content.lines().count() {
        let line = content.lines().nth(current_line_number).unwrap_or_default();
        if line.contains('{') {
            let rbrace_line_number  = find_matching_brace(content.lines(), current_line_number).unwrap_or_default();
            if rbrace_line_number == current_line_number {
                current_line_number += 1;
                continue;
            }
            folding_ranges.push(
                FoldingRange {
                    start_line: current_line_number as u32,
                    start_character: Some(line.find('{').unwrap_or_default() as u32),
                    end_line: rbrace_line_number as u32,
                    end_character: Some(content.lines().nth(rbrace_line_number).unwrap_or_default().find('}').unwrap_or_default() as u32),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                }
            );
        }
        current_line_number += 1;
    }
    
    folding_ranges
}

// 対応する閉じブレースを見つける
fn find_matching_brace(lines: Lines, start_line: usize) -> Option<usize> {
    let mut depth = 0;
    for line_number in start_line..=lines.clone().count() {
        let line = lines.clone().nth(line_number).unwrap_or_default();
        if line.contains('{') {
            depth += 1;
        } else if line.contains('}') {
            depth -= 1;
            if depth == 0 {
                return Some(line_number);
            }
        }
    }
    None
}


// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::provide_folding_ranges;
    use lsp_types::FoldingRangeKind;

    #[test]
    fn test_function_folding() {
        let content = "fn main() {\n    let x = 1;\n    println!(\"Hello\");\n}";
        let ranges = provide_folding_ranges(content);
        
        assert_eq!(ranges.len(), 1, "関数の折りたたみ範囲が1つ検出されるべきです");
        
        let func_range = &ranges[0];
        assert_eq!(func_range.start_line, 0, "関数の開始行は0行目です");
        assert_eq!(func_range.end_line, 3, "関数の終了行は3行目です");
        assert_eq!(func_range.kind, Some(FoldingRangeKind::Region), "適切な種類が設定されるべきです");
    }

    #[test]
    fn test_struct_folding() {
        let content = "struct User {\n    name: String,\n    age: u32,\n}";
        let ranges = provide_folding_ranges(content);
        
        assert_eq!(ranges.len(), 1, "構造体の折りたたみ範囲が1つ検出されるべきです");
        
        let struct_range = &ranges[0];
        assert_eq!(struct_range.start_line, 0, "構造体の開始行は0行目です");
        assert_eq!(struct_range.end_line, 3, "構造体の終了行は3行目です");
    }

    #[test]
    fn test_nested_blocks() {
        let content = "fn test() {\n    if true {\n        let x = 1;\n    }\n}";
        let ranges = provide_folding_ranges(content);
        
        assert_eq!(ranges.len(), 2, "ネストしたブロックで2つの範囲が検出されるべきです");
        
        // 外側の関数ブロック
        let outer_range = ranges.iter().find(|r| r.start_line == 0).unwrap();
        assert_eq!(outer_range.end_line, 4, "外側のブロックは4行目で終了します");
        
        // 内側のifブロック
        let inner_range = ranges.iter().find(|r| r.start_line == 1).unwrap();
        assert_eq!(inner_range.end_line, 3, "内側のブロックは3行目で終了します");
    }

    #[test]
    fn test_multiple_functions() {
        let content = "fn first() {\n    println!(\"1\");\n}\n\nfn second() {\n    println!(\"2\");\n}";
        let ranges = provide_folding_ranges(content);
        
        assert_eq!(ranges.len(), 2, "2つの関数で2つの範囲が検出されるべきです");
        
        let first_func = ranges.iter().find(|r| r.start_line == 0).unwrap();
        assert_eq!(first_func.end_line, 2, "最初の関数は2行目で終了します");
        
        let second_func = ranges.iter().find(|r| r.start_line == 4).unwrap();
        assert_eq!(second_func.end_line, 6, "2番目の関数は6行目で終了します");
    }

    #[test]
    fn test_no_folding_ranges() {
        let content = "let x = 1;\nlet y = 2;\nprintln!(\"Hello\");";
        let ranges = provide_folding_ranges(content);
        
        assert!(ranges.is_empty(), "ブロックがないコードでは折りたたみ範囲がないべきです");
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let ranges = provide_folding_ranges(content);
        
        assert!(ranges.is_empty(), "空のコンテンツでは空の結果を返すべきです");
    }

    #[test]
    fn test_single_line_block() {
        let content = "fn test() { println!(\"Hello\"); }";
        let ranges = provide_folding_ranges(content);
        
        // 1行のブロックは折りたたみ対象外
        assert!(ranges.is_empty(), "1行のブロックは折りたたみ範囲に含まれないべきです");
    }
}