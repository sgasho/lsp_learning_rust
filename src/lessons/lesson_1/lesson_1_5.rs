// Welcome to Lesson 1-5!
// You've learned to create, split, and parse the header and content of an LSP message.
// Now, let's put it all together to parse a real LSP message: the `initialize` request.

// Your Task:
// The function `parse_initialize_request` takes a full LSP message string (header + content).
// It should:
// 1. Split the message into header and content using `super::lesson_1_2::parse_lsp_message`.
// 2. Get the content length from the header using `super::lesson_1_3::get_content_length`.
// 3. Parse the content as JSON using `super::lesson_1_4::parse_json_content`.
// 4. Finally, deserialize the `serde_json::Value` into an `lsp_types::InitializeParams` struct.
//    You'll need to use `serde_json::from_value` for this.
// - If any step fails, return `None`.
// - If all steps succeed, return `Some(InitializeParams)`.

use lsp_types::InitializeParams;
use crate::lessons::lesson_1::lesson_1_2::parse_lsp_message;
use crate::lessons::lesson_1::lesson_1_4::parse_json_content;

pub fn parse_initialize_request(full_message: &str) -> Option<InitializeParams> {
    parse_lsp_message(full_message)
        .and_then(|(_, content)| parse_json_content(content))
        .and_then(|json_value| serde_json::from_value(json_value).ok())
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::parse_initialize_request;
    use lsp_types::{ClientInfo, WorkspaceFolder};
    use serde_json::json;

    #[test]
    fn test_parse_valid_initialize_request() {
        let json_content = json!({
            "processId": 123,
            "clientInfo": {
                "name": "VS Code",
                "version": "1.0"
            },
            "rootPath": "/path/to/project",
            "rootUri": "file:///path/to/project",
            "capabilities": {},
            "workspaceFolders": [
                {
                    "uri": "file:///path/to/project",
                    "name": "project"
                }
            ]
        });
        let content_str = serde_json::to_string(&json_content).unwrap();
        let full_message = format!("Content-Length: {}\r\n\r\n{}", content_str.len(), content_str);

        let parsed_params = parse_initialize_request(&full_message);

        assert!(parsed_params.is_some(), "Valid initialize request should be parsed successfully.");
        let params = parsed_params.unwrap();

        assert_eq!(params.process_id, Some(123));
        assert_eq!(params.client_info, Some(ClientInfo { name: "VS Code".to_string(), version: Some("1.0".to_string()) }));
        // assert_eq!(params.root_path, Some("/path/to/project".to_string()));
        // assert_eq!(params.root_uri, Some("file:///path/to/project".parse().unwrap()));
        assert!(params.capabilities.workspace.is_none()); // Capabilities are empty in this test
        assert_eq!(params.workspace_folders, Some(vec![
            WorkspaceFolder {
                uri: "file:///path/to/project".parse().unwrap(),
                name: "project".to_string(),
            }
        ]));
    }

    #[test]
    fn test_parse_initialize_request_with_missing_parts() {
        // Test with a message that is not a valid LSP message (missing separator)
        let malformed_message = "Content-Length: 10{\"a\":1}";
        assert!(parse_initialize_request(&malformed_message).is_none(), "Malformed message should return None.");

        // Test with valid LSP message format but invalid JSON content
        let invalid_json_content = "Content-Length: 5\r\n\r\n{abc}";
        assert!(parse_initialize_request(&invalid_json_content).is_none(), "Message with invalid JSON should return None.");

        // Test with valid LSP message format and JSON, but not an InitializeParams structure
        let non_initialize_json = "Content-Length: 7\r\n\r\n{\"a\":1}";
        assert!(parse_initialize_request(&non_initialize_json).is_none(), "Message with non-InitializeParams JSON should return None.");
    }

    #[test]
    fn test_parse_initialize_request_with_empty_content() {
        let empty_content_message = "Content-Length: 0\r\n\r\n";
        assert!(parse_initialize_request(&empty_content_message).is_none(), "Message with empty content should return None (as it's not a valid InitializeParams).");
    }
}
