// Welcome to Lesson 1-2!
// Last time, you created a message. Now, let's learn how to read one.

// Your Task:
// The function `parse_lsp_message` takes a string slice representing a full LSP message.
// Your job is to split it into two parts: the header and the content.
// - If the message is valid (contains a "\r\n\r\n" separator), return `Some((header, content))`.
// - If the message is malformed (does not contain the separator), return `None`.

pub fn parse_lsp_message(message: &str) -> Option<(&str, &str)> {
    message.split_once("\r\n\r\n")
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::parse_lsp_message;

    #[test]
    fn test_parse_valid_message_returns_some() {
        let message = "Content-Length: 2\r\n\r\n{}";
        assert!(parse_lsp_message(message).is_some(), "A valid message should return Some.");
    }

    #[test]
    fn test_parse_valid_message_returns_correct_parts() {
        let message = "Content-Length: 2\r\n\r\n{}";
        if let Some((header, content)) = parse_lsp_message(message) {
            assert_eq!(header, "Content-Length: 2");
            assert_eq!(content, "{}");
        } else {
            panic!("Parsing failed for a valid message.");
        }
    }

    #[test]
    fn test_parse_message_without_separator_returns_none() {
        let message = "Content-Length: 2{}";
        assert!(parse_lsp_message(message).is_none(), "A message without a separator should return None.");
    }

    #[test]
    fn test_parse_empty_message_returns_none() {
        let message = "";
        assert!(parse_lsp_message(message).is_none(), "An empty message should return None.");
    }
}
