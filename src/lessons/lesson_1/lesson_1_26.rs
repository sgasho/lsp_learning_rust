// Welcome to Lesson 1-26!
// You can now prepare renames.
// Let's add another useful navigation feature: Document Highlight.

// Your Task:
// The function `get_document_highlights` takes:
// - `file_uri`: The `Url` of the document where the request was made.
// - `position`: The `Position` where the request was made.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find the word at the given `position`.
// 3. If the word is "my_variable", find all occurrences of "my_variable" in the entire document.
// 4. Return a `Vec<DocumentHighlight>` for all found occurrences.
//    - The `range` of the `DocumentHighlight` should cover the entire word.
//    - The `kind` can be `DocumentHighlightKind::Text` for simplicity.
// 5. For any other word, or if the document is not found, return an empty `Vec<DocumentHighlight>`.

use lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Range, Url};
use std::collections::HashMap;

pub fn get_document_highlights(file_uri: &Url, position: Position, document_store: &HashMap<Url, String>) -> Vec<DocumentHighlight> {
    let result: Option<Vec<DocumentHighlight>> = (|| {
        let (_, content) = document_store
            .get(file_uri)?
            .lines()
            .enumerate()
            .find(|(line_number, _)| *line_number as u32 == position.line)?;
        let remaining_line = content.get(position.character as usize..)?;
        let keyword_end = remaining_line.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(remaining_line.len());
        let keyword = &remaining_line[..keyword_end];

        if keyword != "my_variable" {
            return None;
        }

        Some(document_store
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
                             DocumentHighlight {
                                 range: Range::new(
                                     Position::new(line_number as u32, starts_at as u32),
                                     Position::new(line_number as u32, starts_at as u32 + keyword.len() as u32),
                                 ),
                                 kind: Some(DocumentHighlightKind::TEXT),
                             },
                    )
            })
            .collect::<Vec<DocumentHighlight>>())
    })();

    result.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_document_highlights;
    use lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Range, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_get_document_highlights_for_my_variable() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let my_variable = 10;\nmy_variable = 20;\nprintln!(\"{}\", my_variable);";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4); // Cursor on 'm' of my_variable

        let highlights = get_document_highlights(&uri, position, &store);
        assert_eq!(highlights.len(), 3, "Should find 3 highlights for my_variable.");

        let expected_highlights = vec![
            DocumentHighlight {
                range: Range::new(Position::new(0, 4), Position::new(0, 15)),
                kind: Some(DocumentHighlightKind::TEXT),
            },
            DocumentHighlight {
                range: Range::new(Position::new(1, 0), Position::new(1, 11)),
                kind: Some(DocumentHighlightKind::TEXT),
            },
            DocumentHighlight {
                range: Range::new(Position::new(2, 15), Position::new(2, 26)),
                kind: Some(DocumentHighlightKind::TEXT),
            },
        ];

        // Sort both vectors to ensure order doesn't matter for comparison
        let mut sorted_highlights = highlights;
        sorted_highlights.sort_by(|a, b| a.range.start.line.cmp(&b.range.start.line));
        let mut sorted_expected_highlights = expected_highlights;
        sorted_expected_highlights.sort_by(|a, b| a.range.start.line.cmp(&b.range.start.line));

        assert_eq!(sorted_highlights, sorted_expected_highlights, "Found highlights should match expected locations.");
    }

    #[test]
    fn test_get_document_highlights_for_non_existent_variable() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let other_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);

        let highlights = get_document_highlights(&uri, position, &store);
        assert!(highlights.is_empty(), "Should return empty for non-existent variable.");
    }

    #[test]
    fn test_get_document_highlights_for_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "let my_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);

        let highlights = get_document_highlights(&non_existent_uri, position, &store);
        assert!(highlights.is_empty(), "Should return empty if document is not found.");
    }

    #[test]
    fn test_get_document_highlights_for_position_out_of_bounds() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let my_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(99, 99); // Out of bounds line

        let highlights = get_document_highlights(&uri, position, &store);
        assert!(highlights.is_empty(), "Should return empty if position is out of bounds.");
    }
}