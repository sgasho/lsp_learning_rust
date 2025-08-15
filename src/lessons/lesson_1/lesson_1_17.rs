// Welcome to Lesson 1-17!
// You can now handle `textDocument/didOpen` notifications.
// Now, let's make our server react to changes in the document content.
// This is handled by the `textDocument/didChange` notification.

// Your Task:
// The function `handle_did_change_notification` takes:
// - `params_value`: The `serde_json::Value` from the `params` field of a `textDocument/didChange` notification.
// - `document_store`: A mutable `std::collections::HashMap<Url, String>` where the server stores document content.
// It should:
// 1. Extract the `uri` and `text` (the full new content) from the `params_value`.
//    - The `params_value` for `textDocument/didChange` typically looks like:
//      `{"textDocument": {"uri": "file:///a.rs", "version": 2}, "contentChanges": [{"text": "new content"}]}`
//      Note: For simplicity, we assume `contentChanges` always contains one item with the full new text.
// 2. Update the `text` in the `document_store` for the given `uri`.
// 3. Generate diagnostics for the updated document using `super::lesson_1_13::generate_diagnostics`.
// 4. Return the generated `Vec<Diagnostic>`.
// - If parsing `params_value` fails or required fields are missing, return an empty `Vec<Diagnostic>`.

use serde_json::Value;
use lsp_types::{Diagnostic, Url};
use std::collections::HashMap;
use crate::lessons::lesson_1::lesson_1_13::generate_diagnostics;

pub fn handle_did_change_notification(params_value: &Value, document_store: &mut HashMap<Url, String>) -> Vec<Diagnostic> {

    let result: Option<Vec<Diagnostic>> = (|| {
        let uri = Url::parse(params_value.get("textDocument")?.get("uri")?.as_str()?).ok()?;
        let file_content = params_value.get("contentChanges")?.as_array()?.first()?.get("text")?.as_str()?;
        document_store.insert(uri.clone(), file_content.to_string());
        Some(generate_diagnostics(uri, file_content))
    })();

    result.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::handle_did_change_notification;
    use serde_json::{json, Value};
    use lsp_types::Url;
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create dummy didChange params
    fn dummy_did_change_params(uri_str: &str, new_content: &str) -> Value {
        json!({
            "textDocument": {
                "uri": uri_str,
                "version": 2
            },
            "contentChanges": [
                {
                    "text": new_content
                }
            ]
        })
    }

    #[test]
    fn test_did_change_updates_document_and_generates_diagnostics() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///test.rs").unwrap();
        let initial_content = "fn main() {}\n";
        let updated_content = "fn main() {\n// TODO: Updated\n}";

        // Simulate didOpen first
        store.insert(uri.clone(), initial_content.to_string());

        let params = dummy_did_change_params(uri.as_str(), updated_content);
        let diagnostics = handle_did_change_notification(&params, &mut store);

        // Check if document is updated
        assert_eq!(store.get(&uri), Some(&updated_content.to_string()), "Document content should be updated in the HashMap.");

        // Check generated diagnostics for updated content
        assert_eq!(diagnostics.len(), 1, "Should generate one diagnostic for TODO in updated content.");
        assert_eq!(diagnostics[0].message, "Found a TODO item.");
        assert_eq!(diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn test_did_change_with_clean_update() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///clean.rs").unwrap();
        let initial_content = "// TODO: Old\n";
        let updated_content = "fn main() {}\n";

        // Simulate didOpen first
        store.insert(uri.clone(), initial_content.to_string());

        let params = dummy_did_change_params(uri.as_str(), updated_content);
        let diagnostics = handle_did_change_notification(&params, &mut store);

        // Check if document is updated
        assert_eq!(store.get(&uri), Some(&updated_content.to_string()), "Document content should be updated in the HashMap.");

        // Check generated diagnostics
        assert!(diagnostics.is_empty(), "Should generate no diagnostics for clean updated code.");
    }

    #[test]
    fn test_did_change_with_malformed_params_returns_empty_diagnostics() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///malformed.rs").unwrap();
        store.insert(uri.clone(), "initial".to_string());

        let malformed_params = json!({
            "textDocument": {
                "uri": "file:///malformed.rs",
            },
            // Missing contentChanges
        });

        let diagnostics = handle_did_change_notification(&malformed_params, &mut store);

        assert_eq!(store.get(&uri), Some(&"initial".to_string()), "Document should not be updated for malformed params.");
        assert!(diagnostics.is_empty(), "Should return empty diagnostics for malformed params.");
    }

    #[test]
    fn test_did_change_with_invalid_uri_returns_empty_diagnostics() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///invalid.rs").unwrap();
        store.insert(uri.clone(), "initial".to_string());

        let invalid_uri_params = json!({
            "textDocument": {
                "uri": "not a valid uri",
            },
            "contentChanges": [
                {
                    "text": "new content"
                }
            ]
        });

        let diagnostics = handle_did_change_notification(&invalid_uri_params, &mut store);

        assert_eq!(store.get(&uri), Some(&"initial".to_string()), "Document should not be updated for invalid URI.");
        assert!(diagnostics.is_empty(), "Should return empty diagnostics for invalid URI.");
    }
}
