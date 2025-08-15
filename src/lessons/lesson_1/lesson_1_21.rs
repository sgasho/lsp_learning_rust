// Welcome to Lesson 1-21!
// You can now provide Go to Definition.
// Let's add another powerful navigation feature: Find References.

// Your Task:
// The function `find_references` takes:
// - `file_uri`: The `Url` of the document where the request was made.
// - `position`: The `Position` where the request was made.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find the word at the given `position`.
// 3. If the word is "my_variable", find all occurrences of "my_variable" in the entire document.
//    For simplicity, assume all occurrences are in the same file.
// 4. Return a `Vec<Location>` for all found references.
// 5. For any other word, or if the document is not found, return an empty `Vec<Location>`.

use lsp_types::{Location, Position, Range, Url};
use std::collections::HashMap;

pub fn find_references(file_uri: &Url, position: Position, document_store: &HashMap<Url, String>) -> Vec<Location> {
    let locations: Option<Vec<Location>> = (|| {
        let (_, content)= document_store
            .get(file_uri)?
            .lines()
            .enumerate()
            .find(|(line_number, _)| *line_number == position.line as usize)?;

        let remaining_line = content.get(position.character as usize..)?;
        let keyword_end = remaining_line.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(remaining_line.len());
        let keyword = &remaining_line[..keyword_end];


        if keyword == "my_variable" {
            return Some(
                document_store
                .get(file_uri)?
                .lines()
                    .enumerate()
                    .filter_map(|(line_number, line)| {
                        if let Some(start_idx) = line.find("my_variable").map(|byte_start| line[..byte_start].chars().count()) {
                            return Some(
                                Location::new(
                                    file_uri.clone(),
                                    Range::new(
                                        Position::new(line_number as u32, start_idx as u32),
                                        Position::new(line_number as u32, start_idx as u32 + keyword.len() as u32),
                                    ),
                                )
                            )
                        }
                        None
                    })
                    .collect::<Vec<Location>>()
            );
        }
        Some(Vec::new())
    })();

    locations.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::find_references;
    use lsp_types::{Location, Position, Range, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_find_references_for_my_variable() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let my_variable = 10;\nmy_variable = 20;\nprintln!(\"{}\", my_variable);";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4); // Cursor on 'm' of my_variable

        let references = find_references(&uri, position, &store);
        assert_eq!(references.len(), 3, "Should find 3 references for my_variable.");

        let expected_locations = vec![
            Location {
                uri: uri.clone(),
                range: Range::new(Position::new(0, 4), Position::new(0, 15)),
            },
            Location {
                uri: uri.clone(),
                range: Range::new(Position::new(1, 0), Position::new(1, 11)),
            },
            Location {
                uri: uri.clone(),
                range: Range::new(Position::new(2, 15), Position::new(2, 26)),
            },
        ];

        // Sort both vectors to ensure order doesn't matter for comparison
        let mut sorted_references = references;
        sorted_references.sort_by(|a, b| a.range.start.line.cmp(&b.range.start.line));
        let mut sorted_expected_locations = expected_locations;
        sorted_expected_locations.sort_by(|a, b| a.range.start.line.cmp(&b.range.start.line));

        assert_eq!(sorted_references, sorted_expected_locations, "Found references should match expected locations.");
    }

    #[test]
    fn test_find_references_for_non_existent_variable() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let other_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);

        let references = find_references(&uri, position, &store);
        assert!(references.is_empty(), "Should return empty for non-existent variable.");
    }

    #[test]
    fn test_find_references_for_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "let my_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);

        let references = find_references(&non_existent_uri, position, &store);
        assert!(references.is_empty(), "Should return empty if document is not found.");
    }

    #[test]
    fn test_find_references_for_position_out_of_bounds() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let my_variable = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(99, 99); // Out of bounds line

        let references = find_references(&uri, position, &store);
        assert!(references.is_empty(), "Should return empty if position is out of bounds.");
    }
}