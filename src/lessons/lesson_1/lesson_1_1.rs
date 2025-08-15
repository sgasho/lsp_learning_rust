// Welcome to Lesson 1-1!
// This is your first step into the world of the Language Server Protocol (LSP).

// Your Task:
// The function `create_lsp_message` currently returns an empty string.
// Modify it to return a correctly formatted LSP message string.
// The message content should be a simple JSON object: `{"jsonrpc":"2.0"}`.
// The final string must have the LSP headers (`Content-Length`, etc.) prepended.

// For example: "Content-Length: 18\r\n\r\n{\"jsonrpc\":\"2.0\"}"

use std::ops::Add;

pub fn create_lsp_message() -> String {
    // FIXME: Return the correctly formatted LSP message string here.
    let mut s = String::new();
    let content = "{\"jsonrpc\":\"2.0\"}";
    s = s.add(format!("Content-Length: {}\r\n\r\n", content.len()).as_str());
    s = s.add(content);
    s
}


// --- Tests --- //
// The code below contains unit tests for your function.
// They will run when you type `cargo test`.
// You don't need to modify them. Each test checks one specific rule.

#[cfg(test)]
mod tests {
    use super::create_lsp_message;

    #[test]
    fn test_message_has_header_separator() {
        let message = create_lsp_message();
        assert!(message.contains("\r\n\r\n"), "Rule 1 Failed: The message must contain the '\r\n\r\n' separator that divides the header from the content.");
    }

    #[test]
    fn test_header_starts_with_content_length() {
        let message = create_lsp_message();
        let header = message.split("\r\n\r\n").next().unwrap_or("");
        assert!(header.starts_with("Content-Length:"), "Rule 2 Failed: The header part of the message must start with 'Content-Length:'.");
    }

    #[test]
    fn test_content_length_is_correct() {
        let message = create_lsp_message();
        if !message.contains("\r\n\r\n") {
            // This rule is checked in another test. We can skip here to avoid a crash.
            return;
        }
        let parts: Vec<&str> = message.splitn(2, "\r\n\r\n").collect();
        let header = parts[0];
        let content = parts[1];

        let length_str = header.strip_prefix("Content-Length:").unwrap_or("").trim();
        let length_from_header: usize = length_str.parse().unwrap_or(0);

        assert_eq!(length_from_header, content.len(), "Rule 3 Failed: The number in 'Content-Length' must be the exact length of the content part.");
    }

    #[test]
    fn test_content_is_correct_json() {
        let message = create_lsp_message();
        if !message.contains("\r\n\r\n") {
            // This rule is checked in another test. We can skip here.
            return;
        }
        let parts: Vec<&str> = message.splitn(2, "\r\n\r\n").collect();
        let content = parts[1];

        assert_eq!(content, "{\"jsonrpc\":\"2.0\"}", "Rule 4 Failed: The content part is not the expected JSON string.");
    }
}