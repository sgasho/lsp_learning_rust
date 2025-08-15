// Welcome to Lesson 1-20!
// You can now provide hover information.
// Let's add another core feature: Go to Definition.

// Your Task:
// The function `get_definition_location` takes:
// - `file_uri`: The `Url` of the document where the request was made.
// - `position`: The `Position` where the request was made.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find the word at the given `position`.
// 3. If the word is "my_function", return a `Location` pointing to its definition.
//    For simplicity, assume "my_function" is always defined at line 0, character 0 in the same file.
// 4. For any other word or if the document is not found, return `None`.

use lsp_types::{Location, Position, Range, Url};
use std::collections::HashMap;

pub fn get_definition_location(file_uri: &Url, position: Position, document_store: &HashMap<Url, String>) -> Option<Location> {
    let (_, content) = document_store
        .get(file_uri)?
        .lines()
        .enumerate()
        .find(|(line_number, _)| *line_number == position.line as usize)?;
    let remaining_line = content.get(position.character as usize - 1..)?;
    let keyword_end = remaining_line.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(remaining_line.len());
    let keyword = &remaining_line[..keyword_end];
    
    if keyword == "my_function" {
        Some(Location {
            uri: file_uri.clone(),
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 0),
            }
        })
    } else {
        None
    }
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_definition_location;
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
    fn test_go_to_definition_for_my_function() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn my_function() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4); // Cursor on 'f' of my_function

        let location = get_definition_location(&uri, position, &store);
        assert!(location.is_some());
        assert_eq!(
            location.unwrap(),
            Location {
                uri: uri.clone(),
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            }
        );
    }

    #[test]
    fn test_go_to_definition_for_non_existent_function() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn other_function() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);

        let location = get_definition_location(&uri, position, &store);
        assert!(location.is_none(), "Should return None for non-existent function.");
    }

    #[test]
    fn test_go_to_definition_for_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "fn my_function() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 4);

        let location = get_definition_location(&non_existent_uri, position, &store);
        assert!(location.is_none(), "Should return None if document is not found.");
    }

    #[test]
    fn test_go_to_definition_for_position_out_of_bounds() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn my_function() {}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(99, 99); // Out of bounds line

        let location = get_definition_location(&uri, position, &store);
        assert!(location.is_none(), "Should return None if position is out of bounds.");

        let position_col_out_of_bounds = Position::new(0, 99); // Out of bounds column
        let location_col = get_definition_location(&uri, position_col_out_of_bounds, &store);
        assert!(location_col.is_none(), "Should return None if column position is out of bounds.");
    }
}
