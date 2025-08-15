// Welcome to Lesson 1-4!
// You can now extract the content from an LSP message. 
// Now, let's learn how to understand what's inside that content: JSON.

// Your Task:
// The function `parse_json_content` takes a string slice that is expected to be a JSON string.
// It should parse this JSON string into a `serde_json::Value`.
// - If the string is valid JSON, return `Some(Value)`.
// - If the string is not valid JSON, return `None`.

use serde_json::{self, Value};

pub fn parse_json_content(json_str: &str) -> Option<Value> {
    serde_json::from_str(json_str).ok()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::parse_json_content;
    use serde_json::json;

    #[test]
    fn test_parse_valid_json() {
        let json_str = r#"{"method":"initialize","params":{}}"#;
        let expected_value = json!({
            "method": "initialize",
            "params": {}
        });
        assert_eq!(parse_json_content(json_str), Some(expected_value));
    }

    #[test]
    fn test_parse_empty_object() {
        let json_str = r#"{}"#;
        let expected_value = json!({});
        assert_eq!(parse_json_content(json_str), Some(expected_value));
    }

    #[test]
    fn test_parse_invalid_json() {
        let json_str = r#"{"method":"initialize""#;
        assert_eq!(parse_json_content(json_str), None, "Invalid JSON should return None.");
    }

    #[test]
    fn test_parse_empty_string() {
        let json_str = "";
        assert_eq!(parse_json_content(json_str), None, "Empty string should return None.");
    }

    #[test]
    fn test_parse_json_with_array() {
        let json_str = r#"[1, 2, 3]"#;
        let expected_value = json!([1, 2, 3]);
        assert_eq!(parse_json_content(json_str), Some(expected_value));
    }
}
