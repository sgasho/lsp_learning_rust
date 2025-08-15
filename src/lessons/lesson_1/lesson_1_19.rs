// Welcome to Lesson 1-19!
// You can now manage document lifecycle.
// Let's add another useful feature: providing hover information.

// Your Task:
// The function `get_hover_info` takes:
// - `file_uri`: The `Url` of the document.
// - `position`: The `Position` where the hover request was made.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find the word at the given `position`.
// 3. If the word is "fn", return a Hover with content "Keyword: Function definition".
// 4. If the word is "let", return a Hover with content "Keyword: Variable declaration".
// 5. If the word is "struct", return a Hover with content "Keyword: Structure definition".
// 6. For any other word or if the document is not found, return `None`.

use lsp_types::{Hover, MarkupContent, MarkupKind, Position, Url};
use std::collections::HashMap;

pub fn get_hover_info(file_uri: &Url, position: Position, document_store: &HashMap<Url, String>) -> Option<Hover> {
    let (_, content) = document_store
        .get(file_uri)?
        .lines()
        .enumerate()
        .find(|(line_number, _)| *line_number == position.line as usize)?;
    let remaining_line = content.get(position.character as usize - 1..)?;
    let keyword_end = remaining_line.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(remaining_line.len());
    let keyword = &remaining_line[..keyword_end];

    match keyword {
        "fn" => {
            Some(Hover {
                contents: lsp_types::HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Keyword: Function definition".to_string(),
                }),
                range: None,
            })
        },
        "let" => {
            Some(Hover {
                contents: lsp_types::HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Keyword: Variable declaration".to_string(),
                }),
                range: None,
            })
        },
        "struct" => {
            Some(Hover {
                contents: lsp_types::HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Keyword: Structure definition".to_string(),
                }),
                range: None,
            })
        },
        _ => None
    }
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_hover_info;
    use lsp_types::{MarkupContent, MarkupKind, Position, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_hover_for_fn_keyword() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1);

        let hover = get_hover_info(&uri, position, &store);
        assert!(hover.is_some());
        assert_eq!(
            hover.unwrap().contents,
            lsp_types::HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Keyword: Function definition".to_string(),
            })
        );
    }

    #[test]
    fn test_hover_for_let_keyword() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1);

        let hover = get_hover_info(&uri, position, &store);
        assert!(hover.is_some());
        assert_eq!(
            hover.unwrap().contents,
            lsp_types::HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Keyword: Variable declaration".to_string(),
            })
        );
    }

    #[test]
    fn test_hover_for_struct_keyword() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "struct MyStruct {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1);

        let hover = get_hover_info(&uri, position, &store);
        assert!(hover.is_some());
        assert_eq!(
            hover.unwrap().contents,
            lsp_types::HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Keyword: Structure definition".to_string(),
            })
        );
    }

    #[test]
    fn test_hover_for_non_keyword() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "some_variable";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 5);

        let hover = get_hover_info(&uri, position, &store);
        assert!(hover.is_none(), "Should return None for non-keyword.");
    }

    #[test]
    fn test_hover_for_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "fn main() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1);

        let hover = get_hover_info(&non_existent_uri, position, &store);
        assert!(hover.is_none(), "Should return None if document is not found.");
    }

    #[test]
    fn test_hover_for_position_out_of_bounds() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(99, 99); // Out of bounds line

        let hover = get_hover_info(&uri, position, &store);
        assert!(hover.is_none(), "Should return None if position is out of bounds.");

        let position_col_out_of_bounds = Position::new(0, 99); // Out of bounds column
        let hover_col = get_hover_info(&uri, position_col_out_of_bounds, &store);
        assert!(hover_col.is_none(), "Should return None if column position is out of bounds.");
    }
}