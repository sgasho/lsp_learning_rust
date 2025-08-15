// Welcome to Lesson 1-9!
// You've mastered parsing individual parts of an LSP message.
// Now, let's combine all that knowledge to parse a complete LSP message into a structured form.

// Your Task:
// The function `parse_full_lsp_message` takes a full LSP message string (header + content).
// It should:
// 1. Use `super::lesson_1_2::parse_lsp_message` to split the message.
// 2. Use `super::lesson_1_4::parse_json_content` to parse the content into `serde_json::Value`.
// 3. Use `super::lesson_1_6::is_request_and_get_id` to check if it's a request and get its ID.
// 4. Use `super::lesson_1_7::get_lsp_method` to get the method.
// 5. Use `super::lesson_1_8::get_lsp_params` to get the params.
// 6. Return an `Option<LspMessage>` where `LspMessage` is an enum you define below.
//    - If any parsing step fails, return `None`.

use serde_json::Value;
use crate::lessons::lesson_1::lesson_1_2::parse_lsp_message;
use crate::lessons::lesson_1::lesson_1_4::parse_json_content;
use crate::lessons::lesson_1::lesson_1_6::is_request_and_get_id;
use crate::lessons::lesson_1::lesson_1_7::get_lsp_method;
use crate::lessons::lesson_1::lesson_1_8::get_lsp_params;

// Define an enum to represent the parsed LSP message.
// This will help you structure your output.
#[derive(Debug, PartialEq, Clone)]
pub enum LspMessage {
    Request {
        id: Value,
        method: String,
        params: Option<Value>,
    },
    Notification {
        method: String,
        params: Option<Value>,
    },
}

pub fn parse_full_lsp_message(full_message: &str) -> Option<LspMessage> {
    let json_content = parse_lsp_message(full_message)
        .and_then(|(_, content_str)| parse_json_content(content_str))?;

    // jsonrpcが"2.0"であることを確認
    let is_valid_lsp_jsonrpc = json_content.get("jsonrpc")
                                           .and_then(|v| v.as_str())
                                           .map_or(false, |s| s == "2.0");
    if !is_valid_lsp_jsonrpc {
        return None;
    }

    let method = get_lsp_method(&json_content)?; // メソッドはリクエスト/通知両方に必須
    let params = get_lsp_params(&json_content); // パラメータはOption<Value>なのでそのまま

    if let Some(id) = is_request_and_get_id(&json_content) {
        Some(LspMessage::Request { id, method, params })
    } else {
        Some(LspMessage::Notification { method, params })
    }
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::{parse_full_lsp_message, LspMessage};
    use serde_json::json;

    #[test]
    fn test_parse_valid_request_message() {
        let json_content = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/didOpen",
            "params": {"uri": "file:///a.rs"}
        });
        let content_str = serde_json::to_string(&json_content).unwrap();
        let full_message = format!("Content-Length: {}\r\n\r\n{}", content_str.len(), content_str);

        let parsed = parse_full_lsp_message(&full_message);
        assert_eq!(
            parsed,
            Some(LspMessage::Request {
                id: json!(1),
                method: "textDocument/didOpen".to_string(),
                params: Some(json!({"uri": "file:///a.rs"})),
            })
        );
    }

    #[test]
    fn test_parse_valid_notification_message() {
        let json_content = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {"uri": "file:///b.rs"}
        });
        let content_str = serde_json::to_string(&json_content).unwrap();
        let full_message = format!("Content-Length: {}\r\n\r\n{}", content_str.len(), content_str);

        let parsed = parse_full_lsp_message(&full_message);
        assert_eq!(
            parsed,
            Some(LspMessage::Notification {
                method: "textDocument/didSave".to_string(),
                params: Some(json!({"uri": "file:///b.rs"})),
            })
        );
    }

    #[test]
    fn test_parse_message_with_missing_jsonrpc() {
        let json_content = json!({
            "id": 1,
            "method": "test",
            "params": {}
        });
        let content_str = serde_json::to_string(&json_content).unwrap();
        let full_message = format!("Content-Length: {}\r\n\r\n{}", content_str.len(), content_str);
        assert_eq!(parse_full_lsp_message(&full_message), None, "Should return None if jsonrpc is missing.");
    }

    #[test]
    fn test_parse_message_with_invalid_json() {
        let full_message = "Content-Length: 5\r\n\r\n{abc}";
        assert_eq!(parse_full_lsp_message(&full_message), None, "Should return None if JSON content is invalid.");
    }

    #[test]
    fn test_parse_message_with_malformed_header() {
        let full_message = "Content-Length: 10{\"a\":1}";
        assert_eq!(parse_full_lsp_message(&full_message), None, "Should return None if header is malformed.");
    }

    #[test]
    fn test_parse_message_with_no_params() {
        let json_content = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/didOpen"
        });
        let content_str = serde_json::to_string(&json_content).unwrap();
        let full_message = format!("Content-Length: {}\r\n\r\n{}", content_str.len(), content_str);

        let parsed = parse_full_lsp_message(&full_message);
        assert_eq!(
            parsed,
            Some(LspMessage::Request {
                id: json!(1),
                method: "textDocument/didOpen".to_string(),
                params: None,
            })
        );
    }
}
