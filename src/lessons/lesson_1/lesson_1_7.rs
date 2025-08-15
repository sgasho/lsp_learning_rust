// Welcome to Lesson 1-7!
// You can now distinguish between requests and notifications.
// The next step is to understand *what* the message is asking for or reporting.
// This is done via the 'method' field.

// Your Task:
// The function `get_lsp_method` takes a `serde_json::Value` (the parsed JSON content of an LSP message).
// It should extract the 'method' field from the JSON and return it as a `String`.
// - If the 'method' field is missing or not a string, return `None`.

use serde_json::Value;

pub fn get_lsp_method(json_value: &Value) -> Option<String> {
    json_value
        .get("method")
        .and_then(|method| method.as_str())
        .map(|s| s.to_owned())
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_lsp_method;
    use serde_json::json;

    #[test]
    fn test_get_method_from_valid_request() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/didOpen",
            "params": {}
        });
        assert_eq!(get_lsp_method(&request_json), Some("textDocument/didOpen".to_string()));
    }

    #[test]
    fn test_get_method_from_valid_notification() {
        let notification_json = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {}
        });
        assert_eq!(get_lsp_method(&notification_json), Some("textDocument/didSave".to_string()));
    }

    #[test]
    fn test_get_method_missing_method_field() {
        let json_without_method = json!({
            "jsonrpc": "2.0",
            "id": 1
        });
        assert_eq!(get_lsp_method(&json_without_method), None, "Should return None if 'method' field is missing.");
    }

    #[test]
    fn test_get_method_non_string_method_field() {
        let json_with_non_string_method = json!({
            "jsonrpc": "2.0",
            "method": 123,
            "id": 1
        });
        assert_eq!(get_lsp_method(&json_with_non_string_method), None, "Should return None if 'method' field is not a string.");
    }

    #[test]
    fn test_get_method_empty_json() {
        let empty_json = json!({});
        assert_eq!(get_lsp_method(&empty_json), None, "Should return None for empty JSON.");
    }
}
