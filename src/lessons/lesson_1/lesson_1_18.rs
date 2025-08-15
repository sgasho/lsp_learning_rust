// Welcome to Lesson 1-18!
// You can now handle `textDocument/didOpen` and `textDocument/didChange` notifications.
// Finally, let's learn how to handle `textDocument/didClose` notifications.

// Your Task:
// The function `handle_did_close_notification` takes:
// - `params_value`: The `serde_json::Value` from the `params` field of a `textDocument/didClose` notification.
// - `document_store`: A mutable `std::collections::HashMap<Url, String>` where the server stores document content.
// It should:
// 1. Extract the `uri` from the `params_value`.
//    - The `params_value` for `textDocument/didClose` typically looks like:
//      `{"textDocument": {"uri": "file:///a.rs"}}`
// 2. Remove the document from the `document_store` using the `uri` as the key.
// - If parsing `params_value` fails or required fields are missing, do nothing.

use serde_json::Value;
use lsp_types::Url;
use std::collections::HashMap;

pub fn handle_did_close_notification(params_value: &Value, document_store: &mut HashMap<Url, String>) {
    (|| {
        let uri = Url::parse(params_value.get("textDocument")?.get("uri")?.as_str()?).ok()?;
        document_store.remove(&uri)
    })();
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::handle_did_close_notification;
    use serde_json::{json, Value};
    use lsp_types::Url;
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create dummy didClose params
    fn dummy_did_close_params(uri_str: &str) -> Value {
        json!({
            "textDocument": {
                "uri": uri_str,
            }
        })
    }

    #[test]
    fn test_did_close_removes_document_from_store() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {}\n";

        // Simulate didOpen first
        store.insert(uri.clone(), content.to_string());
        assert!(store.contains_key(&uri), "Document should be in store initially.");

        let params = dummy_did_close_params(uri.as_str());
        handle_did_close_notification(&params, &mut store);

        assert!(!store.contains_key(&uri), "Document should be removed from store after didClose.");
    }

    #[test]
    fn test_did_close_with_non_existent_document() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///non_existent.rs").unwrap();

        assert!(!store.contains_key(&uri), "Document should not be in store initially.");

        let params = dummy_did_close_params(uri.as_str());
        handle_did_close_notification(&params, &mut store);

        assert!(!store.contains_key(&uri), "Store should remain unchanged if document was not present.");
    }

    #[test]
    fn test_did_close_with_malformed_params() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///malformed.rs").unwrap();
        store.insert(uri.clone(), "initial".to_string());

        let malformed_params = json!({
            "textDocument": {
                // Missing uri
            }
        });

        handle_did_close_notification(&malformed_params, &mut store);

        assert!(store.contains_key(&uri), "Document should not be removed for malformed params.");
    }

    #[test]
    fn test_did_close_with_invalid_uri() {
        let mut store = HashMap::new();
        let uri = Url::from_str("file:///invalid.rs").unwrap();
        store.insert(uri.clone(), "initial".to_string());

        let invalid_uri_params = json!({
            "textDocument": {
                "uri": "not a valid uri",
            }
        });

        handle_did_close_notification(&invalid_uri_params, &mut store);

        assert!(store.contains_key(&uri), "Document should not be removed for invalid URI.");
    }
}
