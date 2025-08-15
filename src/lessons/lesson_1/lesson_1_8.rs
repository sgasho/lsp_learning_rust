// Welcome to Lesson 1-8!
// You can now identify the method of an LSP message.
// Now, let's extract the actual data or arguments for that method: the 'params' field.

// Your Task:
// The function `get_lsp_params` takes a `serde_json::Value` (the parsed JSON content of an LSP message).
// It should extract the 'params' field from the JSON and return it as a `serde_json::Value`.
// - If the 'params' field is missing, return `None`.
// - Note: The 'params' field can be any valid JSON type (object, array, string, number, boolean, null).
//   So, you just need to get the value, not necessarily check its type beyond existence.

use serde_json::Value;

pub fn get_lsp_params(json_value: &Value) -> Option<Value> {
    json_value.get("params").map(|params| params.to_owned())
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_lsp_params;
    use serde_json::{json, Value};

    #[test]
    fn test_get_params_from_request_with_object_params() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///path/to/file.rs"
                }
            }
        });
        let expected_params = json!({
            "textDocument": {
                "uri": "file:///path/to/file.rs"
            }
        });
        assert_eq!(get_lsp_params(&request_json), Some(expected_params));
    }

    #[test]
    fn test_get_params_from_request_with_array_params() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "workspace/applyEdit",
            "params": [
                {
                    "changes": {}
                }
            ]
        });
        let expected_params = json!([
            {
                "changes": {}
            }
        ]);
        assert_eq!(get_lsp_params(&request_json), Some(expected_params));
    }

    #[test]
    fn test_get_params_from_request_with_null_params() {
        let request_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "window/logMessage",
            "params": null
        });
        assert_eq!(get_lsp_params(&request_json), Some(Value::Null));
    }

    #[test]
    fn test_get_params_missing_params_field() {
        let json_without_params = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "textDocument/didOpen"
        });
        assert_eq!(get_lsp_params(&json_without_params), None, "Should return None if 'params' field is missing.");
    }

    #[test]
    fn test_get_params_empty_json() {
        let empty_json = json!({});
        assert_eq!(get_lsp_params(&empty_json), None, "Should return None for empty JSON.");
    }
}
