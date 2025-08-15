// Lesson 1-36へようこそ！
// Selection Range機能ができるようになりましたね。
// 今度は、コードに情報を表示する機能：Code Lens（コードレンズ）について学びます。

// あなたのタスク：
// `provide_code_lenses` 関数は以下を受け取ります：
// - `content`: 解析対象のソースコード文字列
// 以下を行う必要があります：
// 1. 関数定義を検出します
// 2. 各関数について実行可能なアクションを提案します：
//    - "Run test" （テスト関数の場合）
//    - "Run function" （main関数の場合）
//    - "Show references" （通常の関数の場合）
// 3. 関数の上にCodeLensを配置します
// 4. すべてのCodeLensを返します

use lsp_types::{CodeLens, Command, Position, Range};

pub fn provide_code_lenses(content: &str) -> Vec<CodeLens> {
    // ヒント：
    // 1. 各行を処理して関数定義を検出
    // 2. 関数の種類を判定（test, main, 通常）
    // 3. 適切なCommandを作成
    // 4. CodeLensオブジェクトを作成
    find_function_definitions(content)
}

// 関数定義行を検出する
fn find_function_definitions(content: &str) -> Vec<CodeLens> {
    // 戻り値: (行番号, 関数名, FunctionType) のタプルのベクター
    content
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            if line.starts_with("fn ") {
                let fn_name = extract_fn_name(line)?;

                if line_number >= 1 && content.lines().nth(line_number - 1)?.starts_with("#[test]")
                {
                    return Some(CodeLens {
                        range: fn_range(line_number, fn_name.len()),
                        command: create_command_for_function(FunctionType::Test, fn_name).into(),
                        data: None,
                    });
                }

                return if fn_name == "main" {
                    Some(CodeLens {
                        range: fn_range(line_number, fn_name.len()),
                        command: create_command_for_function(FunctionType::Main, fn_name).into(),
                        data: None,
                    })
                } else {
                    Some(CodeLens {
                        range: fn_range(line_number, fn_name.len()),
                        command: create_command_for_function(FunctionType::Regular, fn_name).into(),
                        data: None,
                    })
                };
            }
            None
        })
        .collect::<Vec<CodeLens>>()
}

fn fn_range(line_number: usize, fn_len: usize) -> Range {
    Range::new(
        Position::new(line_number as u32, 3),
        Position::new(line_number as u32, 3 + fn_len as u32),
    )
}

fn extract_fn_name(line: &str) -> Option<&str> {
    let remaining = line.trim_start_matches("fn ");
    let keyword_end = remaining.find(|c: char| !c.is_alphanumeric() && c != '_')?;
    remaining.get(..keyword_end)
}

// 関数タイプの列挙型
#[derive(Debug, PartialEq)]
enum FunctionType {
    Test,    // テスト関数
    Main,    // main関数
    Regular, // 通常の関数
}

// CodeLensのCommandを作成する
fn create_command_for_function(function_type: FunctionType, function_name: &str) -> Command {
    match function_type {
        FunctionType::Test => Command {
            title: "Run test".to_string(),
            command: "rust.test.run".to_string(),
            arguments: Some(vec![serde_json::Value::String(function_name.to_string())]),
        },
        FunctionType::Main => Command {
            title: "Run function".to_string(),
            command: "rust.main.run".to_string(),
            arguments: None,
        },
        FunctionType::Regular => Command {
            title: "Show references".to_string(),
            command: "rust.references.show".to_string(),
            arguments: Some(vec![serde_json::Value::String(function_name.to_string())]),
        },
    }
}

// --- テスト --- //

#[cfg(test)]
mod tests {
    use super::provide_code_lenses;

    #[test]
    fn test_main_function_code_lens() {
        let content = "fn main() {\n    println!(\"Hello World\");\n}";
        let lenses = provide_code_lenses(content);

        assert_eq!(lenses.len(), 1, "main関数のCodeLensが1つ検出されるべきです");

        let lens = &lenses[0];
        assert_eq!(
            lens.range.start.line, 0,
            "CodeLensは関数定義行に配置されるべきです"
        );

        let command = lens.command.as_ref().unwrap();
        assert_eq!(
            command.title, "Run function",
            "main関数には'Run function'コマンドが設定されるべきです"
        );
        assert_eq!(
            command.command, "rust.main.run",
            "適切なコマンドIDが設定されるべきです"
        );
    }

    #[test]
    fn test_test_function_code_lens() {
        let content = "#[test]\nfn test_example() {\n    assert_eq!(1, 1);\n}";
        let lenses = provide_code_lenses(content);

        assert_eq!(
            lenses.len(),
            1,
            "テスト関数のCodeLensが1つ検出されるべきです"
        );

        let lens = &lenses[0];
        assert_eq!(
            lens.range.start.line, 1,
            "CodeLensはtest関数定義行に配置されるべきです"
        );

        let command = lens.command.as_ref().unwrap();
        assert_eq!(
            command.title, "Run test",
            "テスト関数には'Run test'コマンドが設定されるべきです"
        );
        assert_eq!(
            command.command, "rust.test.run",
            "適切なコマンドIDが設定されるべきです"
        );
    }

    #[test]
    fn test_regular_function_code_lens() {
        let content = "fn calculate(x: i32) -> i32 {\n    x * 2\n}";
        let lenses = provide_code_lenses(content);

        assert_eq!(lenses.len(), 1, "通常関数のCodeLensが1つ検出されるべきです");

        let lens = &lenses[0];
        let command = lens.command.as_ref().unwrap();
        assert_eq!(
            command.title, "Show references",
            "通常関数には'Show references'コマンドが設定されるべきです"
        );
        assert_eq!(
            command.command, "rust.references.show",
            "適切なコマンドIDが設定されるべきです"
        );
    }

    #[test]
    fn test_multiple_functions() {
        let content =
            "fn main() {\n    calculate(42);\n}\n\nfn calculate(x: i32) -> i32 {\n    x * 2\n}";
        let lenses = provide_code_lenses(content);

        assert_eq!(
            lenses.len(),
            2,
            "2つの関数で2つのCodeLensが検出されるべきです"
        );

        // main関数のCodeLens
        let main_lens = lenses
            .iter()
            .find(|lens| lens.range.start.line == 0)
            .unwrap();
        assert_eq!(main_lens.command.as_ref().unwrap().title, "Run function");

        // calculate関数のCodeLens
        let calc_lens = lenses
            .iter()
            .find(|lens| lens.range.start.line == 4)
            .unwrap();
        assert_eq!(calc_lens.command.as_ref().unwrap().title, "Show references");
    }

    #[test]
    fn test_no_functions() {
        let content = "let x = 42;\nprintln!(\"Hello\");";
        let lenses = provide_code_lenses(content);

        assert!(
            lenses.is_empty(),
            "関数がないコードではCodeLensがないべきです"
        );
    }

    #[test]
    fn test_empty_content() {
        let content = "";
        let lenses = provide_code_lenses(content);

        assert!(
            lenses.is_empty(),
            "空のコンテンツではCodeLensがないべきです"
        );
    }

    #[test]
    fn test_code_lens_range() {
        let content = "fn example() {\n    println!(\"test\");\n}";
        let lenses = provide_code_lenses(content);

        assert_eq!(lenses.len(), 1);
        let lens = &lenses[0];

        // CodeLensの範囲が関数名にかかることを確認
        assert_eq!(lens.range.start.line, 0, "開始行は関数定義行");
        assert_eq!(lens.range.start.character, 3, "開始文字位置は'fn 'の後");
        assert!(
            lens.range.end.character > lens.range.start.character,
            "終了位置は開始位置より後"
        );
    }
}
