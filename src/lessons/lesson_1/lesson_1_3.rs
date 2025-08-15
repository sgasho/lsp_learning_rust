// Welcome to Lesson 1-3!
// You can now split a message into a header and content. 
// Let's extract the most important information from the header: the content length.

// Your Task:
// The function `get_content_length` receives a header string slice.
// It should parse this header and return the numerical value of the `Content-Length`.
// - If the header is valid (e.g., "Content-Length: 123"), return `Some(123)`.
// - If the header is malformed (e.g., doesn't start with "Content-Length:", or the value is not a number), return `None`.

pub fn get_content_length(header: &str) -> Option<usize> {
    header.strip_prefix("Content-Length: ")
          .map(|s| s.trim())
          .and_then(|s| s.parse::<usize>().ok())
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_content_length;

    #[test]
    fn test_get_length_from_valid_header() {
        let header = "Content-Length: 42";
        assert_eq!(get_content_length(header), Some(42));
    }

    #[test]
    fn test_header_with_extra_whitespace() {
        let header = "Content-Length:   123  ";
        assert_eq!(get_content_length(header), Some(123), "Should handle extra whitespace after the colon and at the end.");
    }

    #[test]
    fn test_header_is_not_content_length() {
        let header = "Content-Type: application/json";
        assert_eq!(get_content_length(header), None, "Should return None if the header is not Content-Length.");
    }

    #[test]
    fn test_header_with_non_numeric_value() {
        let header = "Content-Length: abc";
        assert_eq!(get_content_length(header), None, "Should return None if the value is not a number.");
    }

    #[test]
    fn test_empty_header() {
        let header = "";
        assert_eq!(get_content_length(header), None, "Should return None for an empty header.");
    }
}
