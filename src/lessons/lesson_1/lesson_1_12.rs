// Welcome to Lesson 1-12!
// You can now parse and create various LSP messages.
// Now, let's understand the overall flow of an LSP session: the lifecycle.

// Your Task:
// The function `handle_lsp_lifecycle` simulates a simple LSP server's lifecycle handling.
// It takes an `LspMessage` (from Lesson 1-9) and a mutable boolean `initialized_state`.
// - If it's an `initialize` request, it should return a success response with `InitializeResult`.
//   (You can use `lsp_types::InitializeResult::default()` for simplicity).
//   Also, set `initialized_state` to `true` after sending the response.
// - If it's an `initialized` notification, it should do nothing (return `None`).
// - If it's a `shutdown` request, it should return a `null` result response.
//   (Do NOT set `initialized_state` to `false` here; that happens on `exit`).
// - If it's an `exit` notification, it should set `initialized_state` to `false` and return `None`.
// - For any other message, return `None`.

use crate::lessons::lesson_1::lesson_1_9::LspMessage;
use crate::lessons::lesson_1::lesson_1_10::create_lsp_response;

pub fn handle_lsp_lifecycle(message: LspMessage, initialized_state: &mut bool) -> Option<String> {
    match message {
        LspMessage::Request { id, method, .. } => { // paramsは使わないので`..`で無視
            match method.as_str() {
                "initialize" => {
                    *initialized_state = true;
                    // InitializeResult::default() を使用
                    let init_result = serde_json::to_value(lsp_types::InitializeResult::default()).ok()?;
                    Some(create_lsp_response(id, init_result))
                },
                "shutdown" => {
                    Some(create_lsp_response(id, serde_json::Value::Null))
                },
                _ => None, // &を削除
            }
        },
        LspMessage::Notification { method, .. } => { // paramsは使わないので`..`で無視
            match method.as_str() {
                "initialized" => {
                    None
                },
                "exit" => {
                    *initialized_state = false;
                    None
                },
                _ => None, // &を削除
            }
        }
    }
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::handle_lsp_lifecycle;
    use crate::lessons::lesson_1::lesson_1_9::LspMessage;
    use serde_json::{json, Value};

    // Helper to create a dummy InitializeParams for testing
    fn dummy_initialize_params() -> Value {
        json!({
            "processId": 123,
            "capabilities": {}
        })
    }

    #[test]
    fn test_handle_initialize_request() {
        let mut initialized = false;
        let id = json!(1);
        let message = LspMessage::Request {
            id: id.clone(),
            method: "initialize".to_string(),
            params: Some(dummy_initialize_params()),
        };

        let response = handle_lsp_lifecycle(message, &mut initialized);

        assert!(response.is_some(), "Initialize request should return a response.");
        let response_str = response.unwrap();
        // Basic check for response content
        assert!(response_str.contains("Content-Length:"));
        assert!(response_str.contains("\"id\":1"));
        assert!(response_str.contains("\"result\":"));
        assert!(initialized, "Initialized state should be true after initialize.");
    }

    #[test]
    fn test_handle_initialized_notification() {
        let mut initialized = false;
        let message = LspMessage::Notification {
            method: "initialized".to_string(),
            params: None,
        };

        let response = handle_lsp_lifecycle(message, &mut initialized);

        assert!(response.is_none(), "Initialized notification should not return a response.");
        assert!(!initialized, "Initialized state should not change on initialized notification.");
    }

    #[test]
    fn test_handle_shutdown_request() {
        let mut initialized = true;
        let id = json!(2);
        let message = LspMessage::Request {
            id: id.clone(),
            method: "shutdown".to_string(),
            params: None,
        };

        let response = handle_lsp_lifecycle(message, &mut initialized);

        assert!(response.is_some(), "Shutdown request should return a response.");
        let response_str = response.unwrap();
        // Basic check for response content
        assert!(response_str.contains("Content-Length:"));
        assert!(response_str.contains("\"id\":2"));
        assert!(response_str.contains("\"result\":null"));
        assert!(initialized, "Initialized state should not change on shutdown request.");
    }

    #[test]
    fn test_handle_exit_notification() {
        let mut initialized = true;
        let message = LspMessage::Notification {
            method: "exit".to_string(),
            params: None,
        };

        let response = handle_lsp_lifecycle(message, &mut initialized);

        assert!(response.is_none(), "Exit notification should not return a response.");
        assert!(!initialized, "Initialized state should be false after exit notification.");
    }

    #[test]
    fn test_handle_other_request() {
        let mut initialized = true;
        let id = json!(3);
        let message = LspMessage::Request {
            id: id.clone(),
            method: "textDocument/didOpen".to_string(),
            params: None,
        };

        let response = handle_lsp_lifecycle(message, &mut initialized);

        assert!(response.is_none(), "Other requests should not return a response in this handler.");
        assert!(initialized, "Initialized state should not change for other requests.");
    }

    #[test]
    fn test_handle_other_notification() {
        let mut initialized = true;
        let message = LspMessage::Notification {
            method: "textDocument/didChange".to_string(),
            params: None,
        };

        let response = handle_lsp_lifecycle(message, &mut initialized);

        assert!(response.is_none(), "Other notifications should not return a response in this handler.");
        assert!(initialized, "Initialized state should not change for other notifications.");
    }
}
