// Welcome to Lesson 1-22!
// You can now find references.
// Let's add another navigation feature: Document Symbols.

// Your Task:
// The function `get_document_symbols` takes:
// - `file_uri`: The `Url` of the document.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Analyze the content line by line.
// 3. If a line starts with "fn ", consider it a function definition.
//    - Extract the function name (the word after "fn ").
// 4. Create an `lsp_types::DocumentSymbol` for each found function.
//    - `name`: The extracted function name.
//    - `kind`: `SymbolKind::Function`.
//    - `range`: The full range of the line.
//    - `selection_range`: The range of the function name itself.
// 5. Return a `Vec<DocumentSymbol>` containing all found symbols.
// 6. If the document is not found, return an empty `Vec<DocumentSymbol>`.

use lsp_types::{DocumentSymbol, Range, SymbolKind, Url, Position};
use std::collections::HashMap;

pub fn get_document_symbols(file_uri: &Url, document_store: &HashMap<Url, String>) -> Vec<DocumentSymbol> {
    let result: Option<Vec<DocumentSymbol>> = (|| {
        Some(
            document_store
            .get(file_uri)?
            .lines()
            .enumerate()
            .filter_map(|(line_number, line)| {
                let section_length = line.len();
                if line.starts_with("fn") {
                    let remaining = line
                        .strip_prefix("fn ")?;
                    let keyword_ends_at = remaining.find(|c: char| !c.is_alphanumeric() && c != '_')?;
                    let keyword = &remaining[..keyword_ends_at];
                    return Some(DocumentSymbol {
                        name: keyword.to_string(),
                        detail: None,
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        range: Range::new(
                            Position::new(line_number as u32, 0),
                            Position::new(line_number as u32, section_length as u32),
                        ),
                        selection_range: Range::new(
                            Position::new(line_number as u32, "fn ".len() as u32),
                            Position::new(line_number as u32, "fn ".len() as u32 + keyword.len() as u32),
                        ),
                        children: None,
                    });
                }
                None
            })
            .collect::<Vec<DocumentSymbol>>()
        )
    })();

    result.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_document_symbols;
    use lsp_types::{Range, SymbolKind, Url, Position};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_get_document_symbols_for_functions() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {}\nfn helper() {}\nlet x = 10;";
        let store = create_dummy_store(uri.as_str(), content);

        let symbols = get_document_symbols(&uri, &store);
        assert_eq!(symbols.len(), 2, "Should find 2 function symbols.");

        let main_symbol = &symbols[0];
        assert_eq!(main_symbol.name, "main");
        assert_eq!(main_symbol.kind, SymbolKind::FUNCTION);
        assert_eq!(main_symbol.range, Range::new(Position::new(0, 0), Position::new(0, 12)));
        assert_eq!(main_symbol.selection_range, Range::new(Position::new(0, 3), Position::new(0, 7)));

        let helper_symbol = &symbols[1];
        assert_eq!(helper_symbol.name, "helper");
        assert_eq!(helper_symbol.kind, SymbolKind::FUNCTION);
        assert_eq!(helper_symbol.range, Range::new(Position::new(1, 0), Position::new(1, 14)));
        assert_eq!(helper_symbol.selection_range, Range::new(Position::new(1, 3), Position::new(1, 9)));
    }

    #[test]
    fn test_get_document_symbols_no_functions() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x = 10;\nconst Y = 20;";
        let store = create_dummy_store(uri.as_str(), content);

        let symbols = get_document_symbols(&uri, &store);
        assert!(symbols.is_empty(), "Should find no symbols for code without functions.");
    }

    #[test]
    fn test_get_document_symbols_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "fn main() {}";
        let store = create_dummy_store(uri.as_str(), content);

        let symbols = get_document_symbols(&non_existent_uri, &store);
        assert!(symbols.is_empty(), "Should return empty if document is not found.");
    }
}
