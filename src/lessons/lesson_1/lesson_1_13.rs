// Welcome to Lesson 1-13!
// You can now handle the basic LSP lifecycle.
// Now, let's make our server useful by providing diagnostics (warnings/errors) for code.

// Your Task:
// The function `generate_diagnostics` takes a `file_uri` (as `Url`) and `file_content` (as `&str`).
// It should analyze the `file_content` for specific patterns and generate `lsp_types::Diagnostic` messages.
// For this lesson, we will look for the string "TODO" (case-insensitive).
// If "TODO" is found, create a `Diagnostic` with:
// - `range`: The range where "TODO" is found (you can simplify to the start of the line for now).
// - `severity`: `DiagnosticSeverity::Warning`.
// - `message`: "Found a TODO item."
// - `source`: "toy-lang-server"
// Return a `Vec<Diagnostic>` containing all found diagnostics.

use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

pub fn generate_diagnostics(_file_uri: Url, file_content: &str) -> Vec<Diagnostic> {
    file_content
        .lines()
        .enumerate() // (行番号, 行の内容) のイテレータ
        .filter_map(|(row_number, line)| { // 各行をフィルタリングし、Diagnosticを生成
            if line.to_lowercase().contains("todo") {
                Some(Diagnostic::new(
                    Range::new(Position::new(row_number as u32, 0), Position::new(row_number as u32, 0)),
                    Some(DiagnosticSeverity::WARNING),
                    None, // code
                    Some("toy-lang-server".to_string()), // source
                    "Found a TODO item.".to_string(), // message
                    None, // tags
                    None, // related_information
                ))
            } else {
                None // "TODO" が含まれていなければNoneを返し、filter_mapでスキップ
            }
        })
        .collect() // 結果をVec<Diagnostic>に収集
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::generate_diagnostics;
    use lsp_types::{Range, Url, Position};
    use std::str::FromStr;

    #[test]
    fn test_no_diagnostics_for_clean_code() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let content = "fn main() {\n  println!(\"Hello\");\n}";
        let diagnostics = generate_diagnostics(uri, content);
        assert!(diagnostics.is_empty(), "Should not generate diagnostics for clean code.");
    }

    #[test]
    fn test_single_todo_diagnostic() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let content = "// TODO: Implement this\nfn main() {}";
        let diagnostics = generate_diagnostics(uri, content);
        assert_eq!(diagnostics.len(), 1, "Should generate one diagnostic for a single TODO.");
        let diag = &diagnostics[0];
        assert_eq!(diag.range, Range::new(Position::new(0, 0), Position::new(0, 0)), "Range should be at the start of the line.");
        assert_eq!(diag.severity, Some(lsp_types::DiagnosticSeverity::WARNING));
        assert_eq!(diag.message, "Found a TODO item.");
        assert_eq!(diag.source, Some("toy-lang-server".to_string()));
    }

    #[test]
    fn test_multiple_todo_diagnostics() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let content = "// TODO: First\n// todo: Second\n// TODO: Third";
        let diagnostics = generate_diagnostics(uri, content);
        assert_eq!(diagnostics.len(), 3, "Should generate three diagnostics for multiple TODOs.");
        // Check ranges for each
        assert_eq!(diagnostics[0].range.start.line, 0);
        assert_eq!(diagnostics[1].range.start.line, 1);
        assert_eq!(diagnostics[2].range.start.line, 2);
    }

    #[test]
    fn test_todo_case_insensitivity() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let content = "// todo\n// ToDo\n// tOdO";
        let diagnostics = generate_diagnostics(uri, content);
        assert_eq!(diagnostics.len(), 3, "Should be case-insensitive for TODO.");
    }

    #[test]
    fn test_todo_in_different_lines() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let content = "Line 1\nLine 2 TODO\nLine 3";
        let diagnostics = generate_diagnostics(uri, content);
        assert_eq!(diagnostics.len(), 1, "Should find TODO on the correct line.");
        assert_eq!(diagnostics[0].range.start.line, 1);
    }
}