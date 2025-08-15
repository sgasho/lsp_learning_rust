// Welcome to Lesson 1-29!
// You can now provide auto-completion.
// Let's learn about another helpful LSP feature: Signature Help.

// Your Task:
// The function `get_signature_help` takes:
// - `file_uri`: The `Url` of the document where signature help was requested.
// - `position`: The `Position` where signature help was triggered.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find if the cursor is inside a function call (look for "function_name(" pattern).
// 3. For known built-in functions, provide signature information:
//    - "println!": signature "println!(format: &str, ...)"
//    - "format!": signature "format!(format: &str, ...) -> String"
//    - "vec!": signature "vec![element, ...] -> Vec<T>"
//    - "assert_eq!": signature "assert_eq!(left: T, right: T)"
// 4. Determine the active parameter by counting commas before cursor position.
// 5. Create a `SignatureHelp` object with:
//    - `signatures`: Vec containing the function signature
//    - `active_signature`: Index of active signature (0 for single signature)
//    - `active_parameter`: Index of the parameter currently being typed
// 6. Return `Some(SignatureHelp)` if inside a known function call, `None` otherwise.

use lsp_types::{ParameterInformation, Position, SignatureHelp, SignatureInformation, Url};
use std::collections::HashMap;

fn find_function_call(line: &str, cursor_pos: usize) -> Option<(String, String)> {
    let before_cursor = &line[..cursor_pos];

    // 後ろから括弧を探す
    let mut paren_count = 0;
    let mut func_end = None;

    for (i, ch) in before_cursor.char_indices().rev() {
        match ch {
            ')' | ']' => paren_count += 1,
            '(' | '[' => {
                if paren_count == 0 {
                    func_end = Some(i);
                    break;
                } else {
                    paren_count -= 1;
                }
            }
            _ => {}
        }
    }

    let func_end = func_end?;

    // 関数名を後ろから抽出
    let before_paren = &before_cursor[..func_end];
    let mut func_start = func_end;

    for (i, ch) in before_paren.char_indices().rev() {
        if ch.is_alphanumeric() || ch == '_' || ch == '!' {
            func_start = i;
        } else {
            break;
        }
    }

    if func_start < func_end {
        Some((before_paren[func_start..].to_string(), before_cursor.get(func_end..)?.to_string()))
    } else {
        None
    }
}

fn create_signature_help(label: &str, params: Vec<(&str, Option<&str>)>, inside_call: &str) -> SignatureHelp {
    let parameters = params.into_iter()
        .map(|(param, doc)| ParameterInformation {
            label: lsp_types::ParameterLabel::Simple(param.to_string()),
            documentation: doc.map(|s| lsp_types::Documentation::String(s.to_string())),
        })
        .collect();

    SignatureHelp {
        signatures: vec![SignatureInformation {
            label: label.to_string(),
            documentation: None,
            parameters: Some(parameters),
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: count_active_parameter(inside_call).into(),
    }
}

fn count_active_parameter(inside_call: &str) -> u32 {
    let mut paren_count = 0;
    let mut comma_count = 0;

    for ch in inside_call.chars() {
        match ch {
            '(' | '[' => paren_count += 1,
            ')' | ']' => paren_count -= 1,
            ',' if paren_count == 1 => comma_count += 1, // 同じレベルのカンマのみカウント
            _ => {}
        }
    }

    comma_count // 0番目のパラメータから始まるため、カンマの数 = パラメータインデックス
}

pub fn get_signature_help(file_uri: &Url, position: Position, document_store: &HashMap<Url, String>) -> Option<SignatureHelp> {
    // エラーハンドリングの改善
    let content = document_store.get(file_uri)?;
    let line = content.lines().nth(position.line as usize)?;

    let (fn_name, inside_call) = find_function_call(line, position.character as usize)?;

    match fn_name.as_str() {
        "println!" => Some(create_signature_help(
            "println!(format: &str, ...)",
            vec![("format: &str", Some("Format string")), ("...", Some("Arguments for formatting"))],
            &inside_call
        )),
        "format!" => Some(create_signature_help(
            "format!(format: &str, ...) -> String", 
            vec![("format: &str", Some("Format string")), ("...", Some("Arguments for formatting"))],
            &inside_call
        )),
        "vec!" => Some(create_signature_help(
            "vec![element, ...] -> Vec<T>",
            vec![("element", Some("Vector element")), ("...", Some("Additional elements"))],
            &inside_call
        )),
        "assert_eq!" => Some(create_signature_help(
            "assert_eq!(left: T, right: T)",
            vec![("left: T", Some("Left side of assertion")), ("right: T", Some("Right side of assertion"))],
            &inside_call
        )),
        _ => None
    }
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_signature_help;
    use lsp_types::{Position, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_signature_help_for_println() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {\n    println!(\"Hello, {}\", \n}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(1, 23); // After the comma in println!

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_some(), "Should provide signature help for println!");
        
        let help = help.unwrap();
        assert_eq!(help.signatures.len(), 1, "Should have one signature");
        assert_eq!(help.active_signature, Some(0), "Should have active signature at index 0");
        assert_eq!(help.active_parameter, Some(1), "Should be on parameter 1 (after comma)");
        
        let signature = &help.signatures[0];
        assert!(signature.label.contains("println!"), "Signature should contain function name");
        assert!(signature.label.contains("format"), "Signature should show format parameter");
    }

    #[test]
    fn test_signature_help_for_format() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let s = format!(\"Value: {}\",";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 28); // After the comma in format!

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_some(), "Should provide signature help for format!");
        
        let help = help.unwrap();
        assert_eq!(help.signatures.len(), 1, "Should have one signature");
        assert_eq!(help.active_parameter, Some(1), "Should be on parameter 1");
        
        let signature = &help.signatures[0];
        assert!(signature.label.contains("format!"), "Signature should contain function name");
        assert!(signature.label.contains("String"), "Signature should show return type");
    }

    #[test]
    fn test_signature_help_for_vec() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let v = vec![1, 2,";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 18); // After "2,"

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_some(), "Should provide signature help for vec!");
        
        let help = help.unwrap();
        assert_eq!(help.active_parameter, Some(2), "Should be on parameter 2 (third element)");
        
        let signature = &help.signatures[0];
        assert!(signature.label.contains("vec!"), "Signature should contain function name");
    }

    #[test]
    fn test_signature_help_for_assert_eq() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "assert_eq!(actual,";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 18); // After "actual,"

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_some(), "Should provide signature help for assert_eq!");
        
        let help = help.unwrap();
        assert_eq!(help.active_parameter, Some(1), "Should be on parameter 1 (right side)");
        
        let signature = &help.signatures[0];
        assert!(signature.label.contains("assert_eq!"), "Signature should contain function name");
        assert!(signature.label.contains("left"), "Signature should show left parameter");
        assert!(signature.label.contains("right"), "Signature should show right parameter");
    }

    #[test]
    fn test_signature_help_first_parameter() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "println!(";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 9); // Right after opening parenthesis

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_some(), "Should provide signature help for println!");
        
        let help = help.unwrap();
        assert_eq!(help.active_parameter, Some(0), "Should be on parameter 0 (first parameter)");
    }

    #[test]
    fn test_signature_help_not_in_function_call() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x = 42;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 10); // At end of statement

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_none(), "Should not provide signature help outside function calls");
    }

    #[test]
    fn test_signature_help_unknown_function() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "unknown_func(arg1,";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 18); // After comma

        let help = get_signature_help(&uri, position, &store);
        assert!(help.is_none(), "Should not provide signature help for unknown functions");
    }

    #[test]
    fn test_signature_help_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "println!(\"test\",";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 15);

        let help = get_signature_help(&non_existent_uri, position, &store);
        assert!(help.is_none(), "Should return None when document not found");
    }
}