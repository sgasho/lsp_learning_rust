// Welcome to Lesson 1-14!
// You can now generate diagnostics for code.
// Now, let's learn how to send these diagnostics to the editor using `textDocument/publishDiagnostics`.

// Your Task:
// The function `create_publish_diagnostics_notification` takes a `file_uri` (as `Url`) and a `diagnostics` vector (as `Vec<Diagnostic>`).
// It should construct a valid `textDocument/publishDiagnostics` notification message string.
// The notification should be a JSON-RPC 2.0 notification.
// The final string must have the LSP headers (`Content-Length`, etc.) prepended.

// Example JSON content for a publishDiagnostics notification:
// {"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///a.rs","diagnostics":[]}}

use serde_json::json;
use lsp_types::{Diagnostic, Url};

pub fn create_publish_diagnostics_notification(file_uri: Url, diagnostics: Vec<Diagnostic>) -> String {
    let notification_content = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": file_uri.to_string(),
            "diagnostics": diagnostics,
        },
    });
    
    let notification_str = serde_json::to_string(&notification_content).unwrap();
    format!("Content-Length: {}\r\n\r\n{}", notification_str.len(), notification_str)
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::create_publish_diagnostics_notification;
    use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
    use serde_json::json;
    use std::str::FromStr;

    // Helper function to parse the full message and extract content for testing
    fn get_content_from_full_message(full_message: &str) -> Option<String> {
        full_message.splitn(2, "\r\n\r\n")
            .nth(1)
            .map(|s| s.to_string())
    }

    #[test]
    fn test_publish_diagnostics_no_diagnostics() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let diagnostics = vec![];
        let notification_message = create_publish_diagnostics_notification(uri.clone(), diagnostics);

        let content = get_content_from_full_message(&notification_message).expect("Failed to get content from notification.");
        let parsed_content: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse content as JSON.");

        assert_eq!(parsed_content["jsonrpc"], "2.0");
        assert_eq!(parsed_content["method"], "textDocument/publishDiagnostics");
        assert!(parsed_content["id"].is_null(), "ID field should not be present in a notification.");
        assert_eq!(parsed_content["params"]["uri"], uri.to_string());
        assert_eq!(parsed_content["params"]["diagnostics"], json!([]));
    }

    #[test]
    fn test_publish_diagnostics_with_single_diagnostic() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let diagnostics = vec![
            Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 0)),
                Some(DiagnosticSeverity::WARNING),
                None,
                Some("toy-lang-server".to_string()),
                "Found a TODO item.".to_string(),
                None,
                None,
            )
        ];
        let notification_message = create_publish_diagnostics_notification(uri.clone(), diagnostics.clone());

        let content = get_content_from_full_message(&notification_message).expect("Failed to get content from notification.");
        let parsed_content: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse content as JSON.");

        assert_eq!(parsed_content["jsonrpc"], "2.0");
        assert_eq!(parsed_content["method"], "textDocument/publishDiagnostics");
        assert_eq!(parsed_content["params"]["uri"], uri.to_string());
        // Note: Direct comparison of Diagnostic Value might be tricky due to default fields.
        // We'll check the length and a specific field.
        assert_eq!(parsed_content["params"]["diagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(parsed_content["params"]["diagnostics"][0]["message"], "Found a TODO item.");
    }

    #[test]
    fn test_publish_diagnostics_content_length_is_correct() {
        let uri = Url::from_str("file:///test.txt").unwrap();
        let diagnostics = vec![
            Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 0)),
                Some(DiagnosticSeverity::ERROR),
                None,
                Some("toy-lang-server".to_string()),
                "Syntax error.".to_string(),
                None,
                None,
            )
        ];
        let notification_message = create_publish_diagnostics_notification(uri, diagnostics);

        let parts: Vec<&str> = notification_message.splitn(2, "\r\n\r\n").collect();
        let header_part = parts[0];
        let content_part = parts[1];

        let length_str = header_part.strip_prefix("Content-Length:").unwrap_or("").trim();
        let length_from_header: usize = length_str.parse().expect("Content-Length should be a number.");

        assert_eq!(length_from_header, content_part.len(), "Content-Length must match the actual length of the content.");
    }
}