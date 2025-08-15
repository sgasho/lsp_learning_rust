// Welcome to Lesson 1-16!
// You can now generate completion items.
// Now, let's make our server aware of the files opened in the editor.
// This is handled by the `textDocument/didOpen` notification.

// Your Task:
// The function `handle_did_open_notification` takes:
// - `params_value`: The `serde_json::Value` from the `params` field of a `textDocument/didOpen` notification.
// - `document_store`: A mutable `std::collections::HashMap<Url, String>` where the server stores document content.
// It should:
// 1. Extract the `uri` and `text` from the `params_value`.
//    - The `params_value` for `textDocument/didOpen` typically looks like:
//      `{"textDocument": {"uri": "file:///a.rs", "languageId": "rust", "version": 1, "text": "fn main() {}\n"}}`
// 2. Store the `text` in the `document_store` using the `uri` as the key.
// 3. Generate diagnostics for the opened document using `super::lesson_1_13::generate_diagnostics`.
// 4. Return the generated `Vec<Diagnostic>`.
// - If parsing `params_value` fails or required fields are missing, return an empty `Vec<Diagnostic>`.

use serde_json::Value;
use lsp_types::{Diagnostic, Url};
use std::collections::HashMap;
use crate::lessons::lesson_1::lesson_1_13::generate_diagnostics;

pub fn handle_did_open_notification(params_value: &Value, document_store: &mut HashMap<Url, String>) -> Vec<Diagnostic> {
    // このクロージャの中で ? 演算子を安全に使います。
    // クロージャがNoneを返した場合、最終的にVec::new()が返されます。
    let result: Option<Vec<Diagnostic>> = (|| {
        let text_document_obj = params_value.get("textDocument")?.as_object()?;
        let uri_str = text_document_obj.get("uri")?.as_str()?;
        let text_content = text_document_obj.get("text")?.as_str()?;
        let uri = Url::parse(uri_str).ok()?;

        document_store.insert(uri.clone(), text_content.to_string());
        Some(generate_diagnostics(uri, text_content))
    })(); // ここでクロージャをすぐに実行します

    result.unwrap_or_default() // クロージャがNoneを返したらVec::new()を返す
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::handle_did_open_notification;
    use serde_json::{json, Value};
    use lsp_types::Url;
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create dummy didOpen params
    fn dummy_did_open_params(uri_str: &str, content: &str) -> Value {
        json!({
            "textDocument": {
                "uri": uri_str,
                "languageId": "rust",
                "version": 1,
                "text": content,
            }
        })
    }

    #[test]
    fn test_did_open_stores_document_and_generates_diagnostics() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {\n// TODO: Implement\n}";
        let params = dummy_did_open_params(uri.as_str(), content);

        let diagnostics = handle_did_open_notification(&params, &mut store);

        // Check if document is stored
        assert_eq!(store.get(&uri), Some(&content.to_string()), "Document content should be stored in the HashMap.");

        // Check generated diagnostics
        assert_eq!(diagnostics.len(), 1, "Should generate one diagnostic for TODO.");
        assert_eq!(diagnostics[0].message, "Found a TODO item.");
        assert_eq!(diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn test_did_open_with_clean_document() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///clean.rs").unwrap();
        let content = "fn main() {}\n";
        let params = dummy_did_open_params(uri.as_str(), content);

        let diagnostics = handle_did_open_notification(&params, &mut store);

        // Check if document is stored
        assert_eq!(store.get(&uri), Some(&content.to_string()), "Document content should be stored in the HashMap.");

        // Check generated diagnostics
        assert!(diagnostics.is_empty(), "Should generate no diagnostics for clean code.");
    }

    #[test]
    fn test_did_open_with_malformed_params_returns_empty_diagnostics() {
        let mut store = HashMap::new();
        let malformed_params = json!({
            "textDocument": {
                "uri": "file:///bad.rs",
                // Missing text field
            }
        });

        let diagnostics = handle_did_open_notification(&malformed_params, &mut store);

        assert!(store.is_empty(), "No document should be stored for malformed params.");
        assert!(diagnostics.is_empty(), "Should return empty diagnostics for malformed params.");
    }

    #[test]
    fn test_did_open_with_invalid_uri_returns_empty_diagnostics() {
        let mut store = HashMap::new();
        let invalid_uri_params = json!({
            "textDocument": {
                "uri": "not a valid uri",
                "languageId": "rust",
                "version": 1,
                "text": "some code",
            }
        });

        let diagnostics = handle_did_open_notification(&invalid_uri_params, &mut store);

        assert!(store.is_empty(), "No document should be stored for invalid URI.");
        assert!(diagnostics.is_empty(), "Should return empty diagnostics for invalid URI.");
    }
}
