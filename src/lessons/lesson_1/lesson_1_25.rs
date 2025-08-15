// Welcome to Lesson 1-25!
// You can now format code.
// Let's add another powerful refactoring feature: Rename.

// Your Task:
// The function `prepare_rename` takes:
// - `file_uri`: The `Url` of the document where the request was made.
// - `position`: The `Position` where the request was made.
// - `new_name`: The new name for the symbol.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find the word at the given `position`.
// 3. If the word is "my_variable", find all occurrences of "my_variable" in the entire document.
// 4. Create a `WorkspaceEdit` that renames all occurrences of "my_variable" to `new_name`.
//    - The `WorkspaceEdit` should contain `changes` (type `HashMap<Url, Vec<TextEdit>>`).
//    - Each `TextEdit` should replace the range of "my_variable" with `new_name`.
// 5. For any other word, or if the document is not found, return `None`.

use lsp_types::{Position, Range, TextEdit, Url, WorkspaceEdit};
use std::collections::HashMap;

pub fn prepare_rename(file_uri: &Url, position: Position, new_name: String, document_store: &HashMap<Url, String>) -> Option<WorkspaceEdit> {
    let (_, content) = document_store
        .get(file_uri)?
        .lines()
        .enumerate()
        .find(|(line_number, _)| *line_number == position.line as usize)?;
    let remaining_line = content.get(position.character as usize..)?;
    let keyword_end = remaining_line.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(remaining_line.len());
    let keyword = &remaining_line[..keyword_end];

    if keyword != "my_variable" {
        return None;
    }

    let mut changes_hash = HashMap::new();
    let changes = document_store
        .get(file_uri)?
        .lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            if !line.contains(keyword) {
                return None;
            }

            line
                .find(keyword)
                .map(|starts_at|
                    TextEdit::new(
                        Range::new(
                            Position::new(line_number as u32, starts_at as u32),
                            Position::new(line_number as u32, starts_at as u32 + keyword.len() as u32),
                        ),
                        new_name.clone(),
                    ),
                )
        })
        .collect::<Vec<TextEdit>>();

    changes_hash.insert(file_uri.clone(), changes);
    Some(WorkspaceEdit::new(changes_hash))
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::prepare_rename;
    use lsp_types::{Position, Range, TextEdit, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_prepare_rename_for_my_variable() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let my_variable = 10;\nmy_variable = 20;\nprintln!(\"{}\", my_variable);";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4); // Cursor on 'm' of my_variable
        let new_name = "new_variable".to_string();

        let edit = prepare_rename(&uri, position, new_name.clone(), &store);
        println!("{:?}", edit);
        assert!(edit.is_some(), "Should return a WorkspaceEdit.");

        let expected_changes: HashMap<Url, Vec<TextEdit>> = [
            (uri.clone(), vec![
                TextEdit::new(Range::new(Position::new(0, 4), Position::new(0, 15)), new_name.clone()),
                TextEdit::new(Range::new(Position::new(1, 0), Position::new(1, 11)), new_name.clone()),
                TextEdit::new(Range::new(Position::new(2, 15), Position::new(2, 26)), new_name.clone()),
            ])
        ].iter().cloned().collect();

        assert_eq!(edit.unwrap().changes, Some(expected_changes), "Generated WorkspaceEdit should contain correct changes.");
    }

    #[test]
    fn test_prepare_rename_for_non_existent_variable() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let other_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);
        let new_name = "new_variable".to_string();

        let edit = prepare_rename(&uri, position, new_name, &store);
        assert!(edit.is_none(), "Should return None for non-existent variable.");
    }

    #[test]
    fn test_prepare_rename_for_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "let my_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);
        let new_name = "new_variable".to_string();

        let edit = prepare_rename(&non_existent_uri, position, new_name, &store);
        assert!(edit.is_none(), "Should return None if document is not found.");
    }

    #[test]
    fn test_prepare_rename_for_position_out_of_bounds() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let my_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(99, 99); // Out of bounds line
        let new_name = "new_variable".to_string();

        let edit = prepare_rename(&uri, position, new_name, &store);
        assert!(edit.is_none(), "Should return None if position is out of bounds.");
    }
}
