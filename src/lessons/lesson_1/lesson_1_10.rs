// Welcome to Lesson 1-10!
// You can now parse incoming LSP messages. 
// Now, let's learn how to send responses back to the editor.

// Your Task:
// The function `create_lsp_response` takes a request `id` (as `serde_json::Value`) and a `result` (as `serde_json::Value`).
// It should construct a valid LSP response message string.
// The response should be a JSON-RPC 2.0 success response.
// The final string must have the LSP headers (`Content-Length`, etc.) prepended.

// Example JSON content for a success response:
// {"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}

use serde_json::{json, Value};

pub fn create_lsp_response(id: Value, result: Value) -> String {
    let response_content = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });

    let response_str = serde_json::to_string(&response_content).ok().unwrap();
    format!("Content-Length: {}\r\n\r\n{}", response_str.len(), response_str)
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::create_lsp_response;
    use serde_json::json;

    // Helper function to parse the full message and extract content for testing
    fn get_content_from_full_message(full_message: &str) -> Option<String> {
        full_message.splitn(2, "\r\n\r\n")
            .nth(1)
            .map(|s| s.to_string())
    }

    #[test]
    fn test_create_simple_response() {
        let id = json!(1);
        let result = json!("hello");
        let response_message = create_lsp_response(id.clone(), result.clone());

        let content = get_content_from_full_message(&response_message).expect("Failed to get content from response.");
        let parsed_content: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse content as JSON.");

        assert_eq!(parsed_content["jsonrpc"], "2.0");
        assert_eq!(parsed_content["id"], id);
        assert_eq!(parsed_content["result"], result);
        assert!(parsed_content["error"].is_null(), "Error field should not be present in a success response.");
    }

    #[test]
    fn test_response_with_object_result() {
        let id = json!("req-123");
        let result = json!({
            "capabilities": {
                "textDocumentSync": 1
            }
        });
        let response_message = create_lsp_response(id.clone(), result.clone());

        let content = get_content_from_full_message(&response_message).expect("Failed to get content from response.");
        let parsed_content: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse content as JSON.");

        assert_eq!(parsed_content["jsonrpc"], "2.0");
        assert_eq!(parsed_content["id"], id);
        assert_eq!(parsed_content["result"], result);
    }

    #[test]
    fn test_response_content_length_is_correct() {
        let id = json!(2);
        let result = json!(["item1", "item2"]);
        let response_message = create_lsp_response(id, result);

        let parts: Vec<&str> = response_message.splitn(2, "\r\n\r\n").collect();
        let header_part = parts[0];
        let content_part = parts[1];

        let length_str = header_part.strip_prefix("Content-Length:").unwrap_or("").trim();
        let length_from_header: usize = length_str.parse().expect("Content-Length should be a number.");

        assert_eq!(length_from_header, content_part.len(), "Content-Length must match the actual length of the content.");
    }
}
