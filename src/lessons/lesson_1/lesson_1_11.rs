// Welcome to Lesson 1-11!
// You can now create success responses for LSP requests.
// But what happens when something goes wrong? Let's learn to send error responses.

// Your Task:
// The function `create_lsp_error_response` takes a request `id` (as `serde_json::Value`), 
// an `error_code` (as `i32`), an `error_message` (as `String`), and optional `error_data` (as `Option<Value>`).
// It should construct a valid LSP error response message string.
// The response should be a JSON-RPC 2.0 error response.
// The final string must have the LSP headers (`Content-Length`, etc.) prepended.

// Example JSON content for an error response:
// {"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}

use serde_json::{json, Value};

pub fn create_lsp_error_response(id: Value, error_code: i32, error_message: String, error_data: Option<Value>) -> String {
    let error_content = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": error_code,
            "message": error_message,
            "data": error_data,
        }
    }).to_string();
    let content_length = error_content.len();
    format!("Content-Length: {}\r\n\r\n{}", content_length, error_content)
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::create_lsp_error_response;
    use serde_json::json;

    // Helper function to parse the full message and extract content for testing
    fn get_content_from_full_message(full_message: &str) -> Option<String> {
        full_message.splitn(2, "\r\n\r\n")
            .nth(1)
            .map(|s| s.to_string())
    }

    #[test]
    fn test_create_simple_error_response() {
        let id = json!(1);
        let code = -32601; // Method not found
        let message = "Method 'unknownMethod' not found.".to_string();
        let data = None;
        let response_message = create_lsp_error_response(id.clone(), code, message.clone(), data.clone());

        let content = get_content_from_full_message(&response_message).expect("Failed to get content from response.");
        let parsed_content: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse content as JSON.");

        assert_eq!(parsed_content["jsonrpc"], "2.0");
        assert_eq!(parsed_content["id"], id);
        assert!(parsed_content["result"].is_null(), "Result field should not be present in an error response.");
        assert_eq!(parsed_content["error"]["code"], code);
        assert_eq!(parsed_content["error"]["message"], message);
        assert!(parsed_content["error"]["data"].is_null(), "Data field should not be present if None is provided.");
    }

    #[test]
    fn test_create_error_response_with_data() {
        let id = json!("req-error");
        let code = -32000; // Server error
        let message = "Internal server error.".to_string();
        let data = Some(json!({ "details": "Something went wrong." }));
        let response_message = create_lsp_error_response(id.clone(), code, message.clone(), data.clone());

        let content = get_content_from_full_message(&response_message).expect("Failed to get content from response.");
        let parsed_content: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse content as JSON.");

        assert_eq!(parsed_content["jsonrpc"], "2.0");
        assert_eq!(parsed_content["id"], id);
        assert_eq!(parsed_content["error"]["code"], code);
        assert_eq!(parsed_content["error"]["message"], message);
        assert_eq!(parsed_content["error"]["data"], data.unwrap());
    }

    #[test]
    fn test_error_response_content_length_is_correct() {
        let id = json!(3);
        let code = -32602; // Invalid params
        let message = "Invalid parameters provided.".to_string();
        let data = None;
        let response_message = create_lsp_error_response(id, code, message, data);

        let parts: Vec<&str> = response_message.splitn(2, "\r\n\r\n").collect();
        let header_part = parts[0];
        let content_part = parts[1];

        let length_str = header_part.strip_prefix("Content-Length:").unwrap_or("").trim();
        let length_from_header: usize = length_str.parse().expect("Content-Length should be a number.");

        assert_eq!(length_from_header, content_part.len(), "Content-Length must match the actual length of the content.");
    }
}
