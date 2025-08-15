// Lesson 1-35へようこそ！
// Folding Range機能ができるようになりましたね。
// 今度は、コードの選択を賢くする機能：Selection Range（スマート選択）について学びます。

// あなたのタスク：
// `provide_selection_ranges` 関数は以下を受け取ります：
// - `content`: 解析対象のソースコード文字列
// - `positions`: 選択を拡張したい位置のリスト
// 以下を行う必要があります：
// 1. 各位置について段階的に選択範囲を拡張します：
//    - 単語 → 式 → 文 → ブロック → 関数全体
// 2. 各段階の選択範囲を計算します
// 3. `SelectionRange` オブジェクトを作成します
// 4. 階層的な選択範囲を返します

use lsp_types::{Position, Range, SelectionRange};

pub fn provide_selection_ranges(content: &str, positions: &[Position]) -> Vec<SelectionRange> {
    // ヒント：
    // 1. 各位置について現在の単語を検出
    // 2. 段階的に選択範囲を拡張
    // 3. SelectionRange の階層構造を作成
    positions
        .iter()
        .filter_map(|position| {
            provide_selection_range(content, *position)
        })
        .collect::<Vec<SelectionRange>>()
}

fn provide_selection_range(content: &str, position: Position) -> Option<SelectionRange> {
    let word_range = select_word_at_position(content, position)?;
    let statement_range = expand_to_statement(content, word_range)?;
    let block_range = expand_to_block(content, statement_range)?;

    create_selection_hierarchy(
        word_range,
        statement_range.into(),
        block_range.into(),
    )
}

// 指定位置の単語を選択する
fn select_word_at_position(content: &str, position: Position) -> Option<Range> {
    let line = content
        .lines()
        .nth(position.line as usize)?;

    let mut ends_at = 0;
    for pos in position.character..=line.chars().count() as u32 {
        let ch = line.chars().nth(pos as usize)?;
        if !(ch.is_alphanumeric() || ch == '_' || ch == '!') {
            ends_at = pos;
            break;
        }
    }

    let mut starts_at = 0;
    for pos in (0..position.character).rev() {
        let ch = line.chars().nth(pos as usize)?;
        if !(ch.is_alphanumeric() || ch == '_' || ch == '!') {
            starts_at = pos + 1;
            break;
        }
    }

    Some(Range::new(
        Position::new(position.line, starts_at),
        Position::new(position.line, ends_at),
    ))
}

// 単語選択から文選択に拡張する
fn expand_to_statement(content: &str, word_range: Range) -> Option<Range> {
    let line = content.lines().nth(word_range.start.line as usize)?;

    let mut expr_starts_at = 0;
    for pos in 0..line.chars().count() as u32 {
        let ch = line.chars().nth(pos as usize)?;
        if ch != ' ' {
            expr_starts_at = pos;
            break;
        }
    }

    let mut expr_ends_at = 0;
    for pos in expr_starts_at..line.chars().count() as u32 {
        let ch = line.chars().nth(pos as usize)?;
        if ch == ';' {
            expr_ends_at = pos;
            break;
        }
    }

    Some(Range::new(
        Position::new(word_range.start.line, expr_starts_at),
        Position::new(word_range.start.line, expr_ends_at),
    ))
}

// 文選択からブロック選択に拡張する
fn expand_to_block(content: &str, statement_range: Range) -> Option<Range> {
    let mut block_starts_at = Position::new(0, 0);
    for line_number in (0..statement_range.start.line).rev() {
        let lbrace_pos = content
            .lines()
            .nth(line_number as usize)?
            .find('{');
        if lbrace_pos.is_some() {
            block_starts_at = Position::new(line_number, lbrace_pos? as u32);
            break;
        }
    }

    let mut block_ends_at = Position::new(0, 0);
    for line_number in statement_range.start.line..content.lines().count() as u32 {
        let rbrace_pos = content
            .lines()
            .nth(line_number as usize)?
            .find('}');
        if rbrace_pos.is_some() {
            block_ends_at = Position::new(line_number, rbrace_pos? as u32);
            break;
        }
    }

    Some(Range::new(block_starts_at, block_ends_at))
}

// SelectionRangeの階層構造を作成
fn create_selection_hierarchy(word: Range, statement: Option<Range>, block: Option<Range>) -> Option<SelectionRange> {
    SelectionRange {
        range: word,
        parent: Box::new(SelectionRange {
            range: statement?,
            parent: Box::new(SelectionRange {
                range: block?,
                parent: None,
            }).into(),
        }).into(),
    }.into()
}


// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::provide_selection_ranges;
    use lsp_types::{Position};

    #[test]
    fn test_word_selection() {
        let content = "let variable = 42;";
        let positions = vec![Position::new(0, 6)]; // "variable"の中
        let selections = provide_selection_ranges(content, &positions);
        
        assert_eq!(selections.len(), 1, "1つの選択範囲が返されるべきです");
        
        let selection = &selections[0];
        // 最小の選択範囲は単語"variable"
        assert_eq!(selection.range.start.character, 4, "単語の開始位置が正しいべきです");
        assert_eq!(selection.range.end.character, 12, "単語の終了位置が正しいべきです");
    }

    #[test]
    fn test_selection_hierarchy() {
        let content = "fn test() {\n    let x = 42;\n}";
        let positions = vec![Position::new(1, 8)]; // "x"の位置
        let selections = provide_selection_ranges(content, &positions);
        
        assert_eq!(selections.len(), 1, "1つの選択範囲が返されるべきです");
        
        let selection = &selections[0];
        
        // 階層構造があることを確認
        assert!(selection.parent.is_some(), "親の選択範囲があるべきです");
        
        // 親の選択範囲が文全体を含むことを確認
        let parent = selection.parent.as_ref().unwrap();
        assert!(parent.range.start.character <= 4, "親の範囲は文の開始を含むべきです");
        assert!(parent.range.end.character >= 14, "親の範囲は文の終了を含むべきです");
    }

    #[test]
    fn test_multiple_positions() {
        let content = "let a = 1;\nlet b = 2;";
        let positions = vec![
            Position::new(0, 4), // "a"の位置
            Position::new(1, 4), // "b"の位置
        ];
        let selections = provide_selection_ranges(content, &positions);
        
        assert_eq!(selections.len(), 2, "2つの選択範囲が返されるべきです");
        
        // 各選択範囲が異なることを確認
        assert_ne!(selections[0].range, selections[1].range, "異なる位置には異なる選択範囲があるべきです");
    }

    #[test]
    fn test_function_block_selection() {
        let content = "fn main() {\n    println!(\"Hello\");\n    let x = 1;\n}";
        let positions = vec![Position::new(1, 10)]; // println!の中
        let selections = provide_selection_ranges(content, &positions);
        
        assert_eq!(selections.len(), 1, "1つの選択範囲が返されるべきです");
        
        // 階層の最上位がブロック全体を含むことを確認
        let mut current = &selections[0];
        while let Some(parent) = &current.parent {
            current = parent;
        }
        
        // 最上位の選択範囲が関数全体を含む
        assert_eq!(current.range.start.line, 0, "最上位は関数の開始を含むべきです");
        assert_eq!(current.range.end.line, 3, "最上位は関数の終了を含むべきです");
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let positions = vec![Position::new(0, 0)];
        let selections = provide_selection_ranges(content, &positions);
        
        assert!(selections.is_empty(), "空のコンテンツでは空の結果を返すべきです");
    }

    #[test]
    fn test_invalid_position() {
        let content = "let x = 1;";
        let positions = vec![Position::new(10, 0)]; // 存在しない行
        let selections = provide_selection_ranges(content, &positions);
        
        assert!(selections.is_empty(), "無効な位置に対しては空の結果を返すべきです");
    }

    #[test]
    fn test_position_in_whitespace() {
        let content = "let x = 1;";
        let positions = vec![Position::new(0, 3)]; // スペースの位置
        let selections = provide_selection_ranges(content, &positions);
        
        // スペース位置でも何らかの選択範囲を提供するか、空を返すかは実装次第
        // ここでは実装をテストするためのプレースホルダー
        assert!(selections.len() <= 1, "スペース位置では最大1つの選択範囲");
    }
}