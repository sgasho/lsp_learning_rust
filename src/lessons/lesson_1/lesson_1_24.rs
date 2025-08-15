// Welcome to Lesson 1-24!
// You can now provide code actions.
// Let's add another useful feature: Code Formatting.

// Your Task:
// The function `format_document` takes:
// - `file_uri`: The `Url` of the document.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. For each line in the document, if it has leading or trailing whitespace,
//    create a `TextEdit` to trim that whitespace.
//    - The `range` of the `TextEdit` should cover the entire line.
//    - The `new_text` should be the trimmed version of the line.
// 3. Return a `Vec<TextEdit>` containing all necessary edits to format the document.
// 4. If the document is not found, return an `empty Vec<TextEdit>`.

use lsp_types::{Position, Range, TextEdit, Url};
use std::collections::HashMap;

pub fn format_document(file_uri: &Url, document_store: &HashMap<Url, String>) -> Vec<TextEdit> {
    let result: Option<Vec<TextEdit>> = (|| {
        Some(document_store
            .get(file_uri)?
            .lines()
            .enumerate()
            .filter_map(|(line_number, line)| {
                let trimmed = line.trim_ascii_start().trim_ascii_end();
                if line != trimmed {
                    return Some(TextEdit {
                        range: Range::new(
                            Position::new(line_number as u32, 0),
                            Position::new(line_number as u32, line.len() as u32),
                        ),
                        new_text: trimmed.to_string(),
                    });
                }
                None
            })
            .collect::<Vec<TextEdit>>())
    })();

    result.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::format_document;
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
    fn test_format_document_trims_whitespace() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "  fn main() {\n    println!(\"Hello\");  \n }\n";
        let store = create_dummy_store(uri.as_str(), content);

        let edits = format_document(&uri, &store);
        assert_eq!(edits.len(), 3, "Should generate 3 edits for 3 lines.");

        // Expected edits
        let expected_edits = vec![
            TextEdit::new(Range::new(Position::new(0, 0), Position::new(0, 13)), "fn main() {".to_string()),
            TextEdit::new(Range::new(Position::new(1, 0), Position::new(1, 24)), "println!(\"Hello\");".to_string()),
            TextEdit::new(Range::new(Position::new(2, 0), Position::new(2, 2)), "}".to_string()),
        ];

        assert_eq!(edits, expected_edits, "Generated edits should correctly trim whitespace.");
    }

    #[test]
    fn test_format_document_no_changes_for_clean_code() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {\nprintln!(\"Hello\");\n}";
        let store = create_dummy_store(uri.as_str(), content);

        let edits = format_document(&uri, &store);
        assert!(edits.is_empty(), "Should generate no edits for already clean code.");
    }

    #[test]
    fn test_format_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "fn main() {}";
        let store = create_dummy_store(uri.as_str(), content);

        let edits = format_document(&non_existent_uri, &store);
        assert!(edits.is_empty(), "Should return empty if document is not found.");
    }
}