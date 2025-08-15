// Welcome to Lesson 1-6!
// You can now parse an `initialize` request. 
// But not all LSP messages are requests. Some are notifications.
// Let's learn to distinguish between them and extract the request ID.

// Your Task:
// The function `is_request_and_get_id` takes a `serde_json::Value` (the parsed JSON content of an LSP message).
// It should determine if the message is an LSP Request and, if so, return its `id`.
// - An LSP Request has a "jsonrpc": "2.0", a "method": "...", and an "id": "..." (or number).
// - An LSP Notification has a "jsonrpc": "2.0", a "method": "...", but NO "id" field.
// - Return `Some(id)` if it's a request, where `id` is `serde_json::Value::Number` or `serde_json::Value::String`.
// - Return `None` if it's a notification or any other malformed JSON.

use serde_json::Value;

pub fn is_request_and_get_id(json_value: &Value) -> Option<Value> {
    // jsonrpcが"2.0"であることと、methodが存在することを確認
    let is_valid_lsp_message = json_value.get("jsonrpc")
                                         .and_then(|v| v.as_str())
                                         .map_or(false, |s| s == "2.0")
                                 && json_value.get("method").is_some();

    if !is_valid_lsp_message {
        return None;
    }

    // idが存在し、それが文字列か数値であることを確認して返す
    json_value.get("id")
              .filter(|v| v.is_string() || v.is_number())
              .map(|v| v.to_owned())
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::is_request_and_get_id;
    use serde_json::json;

    #[test]
    fn test_is_request_with_number_id() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/didOpen",
            "params": {}
        });
        assert_eq!(is_request_and_get_id(&request_json), Some(json!(1)));
    }

    #[test]
    fn test_is_request_with_string_id() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": "some-uuid",
            "method": "textDocument/didOpen",
            "params": {}
        });
        assert_eq!(is_request_and_get_id(&request_json), Some(json!("some-uuid")));
    }

    #[test]
    fn test_is_notification() {
        let notification_json = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {}
        });
        assert_eq!(is_request_and_get_id(&notification_json), None, "Notifications should return None.");
    }

    #[test]
    fn test_malformed_json_no_id() {
        let malformed_json = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen"
            // Missing ID
        });
        assert_eq!(is_request_and_get_id(&malformed_json), None, "Malformed JSON (missing ID) should return None.");
    }

    #[test]
    fn test_malformed_json_no_method() {
        let malformed_json = json!({
            "jsonrpc": "2.0",
            "id": 1
            // Missing method
        });
        assert_eq!(is_request_and_get_id(&malformed_json), None, "Malformed JSON (missing method) should return None.");
    }

    #[test]
    fn test_malformed_json_no_jsonrpc() {
        let malformed_json = json!({
            "id": 1,
            "method": "textDocument/didOpen"
            // Missing jsonrpc
        });
        assert_eq!(is_request_and_get_id(&malformed_json), None, "Malformed JSON (missing jsonrpc) should return None.");
    }

    #[test]
    fn test_empty_json() {
        let empty_json = json!({});
        assert_eq!(is_request_and_get_id(&empty_json), None, "Empty JSON should return None.");
    }
}
