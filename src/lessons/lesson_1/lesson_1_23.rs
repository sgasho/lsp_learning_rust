// Welcome to Lesson 1-23!
// You can now provide document symbols.
// Let's add another interactive feature: Code Actions.

// Your Task:
// The function `get_code_actions` takes:
// - `file_uri`: The `Url` of the document.
// - `range`: The `Range` where the code action request was made.
// - `diagnostics`: A `Vec<Diagnostic>` relevant to the given range.
// It should:
// 1. Iterate through the `diagnostics`.
// 2. If a diagnostic's message is "Found a TODO item.", create a Code Action to fix it.
//    - The Code Action should have `title`: "Remove TODO item".
//    - The Code Action should have `kind`: `CodeActionKind::QuickFix`.
//    - The Code Action should have an `edit` field (type `WorkspaceEdit`).
//      - The `edit` should contain `changes` (type `HashMap<Url, Vec<TextEdit>>`).
//      - The `TextEdit` should replace the diagnostic's `range` with an empty string.
// 3. Return a `Vec<CodeAction>` containing all found code actions.

use lsp_types::{CodeAction, CodeActionKind, Diagnostic, Range, TextEdit, Url, WorkspaceEdit};
use std::collections::HashMap;

pub fn get_code_actions(file_uri: Url, _range: Range, diagnostics: Vec<Diagnostic>) -> Vec<CodeAction> {
    diagnostics
        .into_iter() // Vec<Diagnostic>を消費してイテレータにする
        .filter_map(|diagnostic| {
            if diagnostic.message == "Found a TODO item." {
                let mut changes = HashMap::new();
                changes.insert(
                    file_uri.clone(),
                    vec![TextEdit::new(diagnostic.range, "".to_string())],
                );

                Some(CodeAction {
                    title: "Remove TODO item".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: None, // 今回は診断をCodeActionに含めない
                    edit: Some(WorkspaceEdit { changes: Some(changes), document_changes: None, change_annotations: None }),
                    command: None,
                    is_preferred: None,
                    disabled: None,
                    data: None,
                })
            } else {
                None
            }
        })
        .collect()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_code_actions;
    use lsp_types::{CodeActionKind, Diagnostic, DiagnosticSeverity, Position, Range, Url};
    use std::str::FromStr;

    #[test]
    fn test_code_action_for_todo_diagnostic() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diagnostics = vec![
            Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 6)),
                Some(DiagnosticSeverity::WARNING),
                None,
                Some("toy-lang-server".to_string()),
                "Found a TODO item.".to_string(),
                None,
                None,
            )
        ];

        let actions = get_code_actions(uri.clone(), range, diagnostics);
        assert_eq!(actions.len(), 1, "Should generate one code action for TODO diagnostic.");

        let action = &actions[0];
        assert_eq!(action.title, "Remove TODO item");
        assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
        assert!(action.edit.is_some(), "Code action should have an edit.");

        let edit = action.edit.as_ref().unwrap();
        assert!(edit.changes.is_some(), "Edit should have changes.");

        let changes = edit.changes.as_ref().unwrap();
        assert_eq!(changes.len(), 1, "Should have one change entry.");
        assert!(changes.contains_key(&uri), "Changes should contain the file URI.");

        let text_edits = changes.get(&uri).unwrap();
        assert_eq!(text_edits.len(), 1, "Should have one text edit.");

        let text_edit = &text_edits[0];
        assert_eq!(text_edit.range, Range::new(Position::new(0, 0), Position::new(0, 6)));
        assert_eq!(text_edit.new_text, "");
    }

    #[test]
    fn test_no_code_action_for_other_diagnostic() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diagnostics = vec![
            Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 5)),
                Some(DiagnosticSeverity::ERROR),
                None,
                Some("toy-lang-server".to_string()),
                "Syntax error.".to_string(),
                None,
                None,
            )
        ];

        let actions = get_code_actions(uri, range, diagnostics);
        assert!(actions.is_empty(), "Should not generate code action for non-TODO diagnostic.");
    }

    #[test]
    fn test_no_code_action_for_no_diagnostics() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diagnostics = vec![];

        let actions = get_code_actions(uri, range, diagnostics);
        assert!(actions.is_empty(), "Should not generate code action for no diagnostics.");
    }
}
