// Welcome to Lesson 1-27!
// You can now highlight document symbols.
// Let's learn about a very useful rust-analyzer feature: Inlay Hints.

// Your Task:
// The function `get_inlay_hints` takes:
// - `file_uri`: The `Url` of the document where the request was made.
// - `range`: The `Range` for which inlay hints are requested.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Look for variable declarations like "let variable_name = value;" within the given range.
// 3. For each variable declaration found, determine the type based on the value:
//    - If value is a number (like "10"), the type is "i32"
//    - If value is a string (like "\"hello\""), the type is "&str"
//    - If value is "true" or "false", the type is "bool"
// 4. Create an `InlayHint` for each variable with:
//    - `position`: Right after the variable name
//    - `label`: The inferred type (like ": i32")
//    - `kind`: `InlayHintKind::Type`
// 5. Return a `Vec<InlayHint>` containing all found hints.
// 6. If the document is not found, return an empty `Vec<InlayHint>`.

use lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position, Range, Url};
use std::collections::HashMap;

pub fn get_inlay_hints(file_uri: &Url, range: Range, document_store: &HashMap<Url, String>) -> Vec<InlayHint> {
    // Hint:
    // 1. Get document content from document_store
    // 2. Iterate through lines within the given range
    // 3. Look for lines that match "let variable_name = value;"
    // 4. Extract variable name and value, infer type from value
    // 5. Create InlayHint with position after variable name and type label

    let result: Option<Vec<InlayHint>> = (|| {
        Some(document_store
            .get(file_uri)?
            .lines()
            .enumerate()
            .filter_map(|(line_number, line)| {
                // 範囲チェック：指定された範囲内の行のみ処理
                let out_of_range = line_number < range.start.line as usize || line_number > range.end.line as usize;
                // let文のチェック：変数宣言行のみ対象
                let non_var_def_line = !line.starts_with("let ");
                if out_of_range || non_var_def_line {
                    return None;
                }

                let after_let = line.strip_prefix("let ").unwrap(); // "x = 10;"

                if let Some(eq_pos) = after_let.find(" = ") {
                    let var_name = &after_let[..eq_pos];
                    let after_eq = &after_let[eq_pos + 3..];
                    let value = after_eq.trim_end_matches(';');

                    let label = if value.chars().all(|c| c.is_numeric()) {
                        ": i32"                           // 数値の場合
                    } else if value.starts_with('"') && value.ends_with('"') {
                        ": &str"                          // 文字列の場合
                    } else if value == "true" || value == "false" {
                        ": bool"                          // 真偽値の場合
                    } else {
                        return None;                    // その他は対象外
                    };

                    return Some(InlayHint {
                        // 変数名の直後の正確な位置を計算
                        position: Position::new(line_number as u32, (4 + var_name.len()) as u32),
                        label: InlayHintLabel::String(label.to_string()),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: None,
                        padding_left: None,
                        padding_right: None,
                        data: None,
                    });
                };
                None
            })
            .collect::<Vec<InlayHint>>()
        )
    })();

    result.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_inlay_hints;
    use lsp_types::{InlayHintKind, InlayHintLabel, Position, Range, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_get_inlay_hints_for_basic_types() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x = 10;\nlet y = \"hello\";\nlet z = true;";
        let store = create_dummy_store(uri.as_str(), content);
        let range = Range::new(Position::new(0, 0), Position::new(2, 20));

        let hints = get_inlay_hints(&uri, range, &store);
        assert_eq!(hints.len(), 3, "Should find 3 inlay hints.");

        // Check first hint (x: i32)
        let x_hint = &hints[0];
        assert_eq!(x_hint.position, Position::new(0, 5)); // After "let x"
        if let InlayHintLabel::String(ref label) = x_hint.label {
            assert_eq!(label, ": i32");
        } else {
            panic!("Expected String label");
        }
        assert_eq!(x_hint.kind, Some(InlayHintKind::TYPE));

        // Check second hint (y: &str)
        let y_hint = &hints[1];
        assert_eq!(y_hint.position, Position::new(1, 5)); // After "let y"
        if let InlayHintLabel::String(ref label) = y_hint.label {
            assert_eq!(label, ": &str");
        } else {
            panic!("Expected String label");
        }
        assert_eq!(y_hint.kind, Some(InlayHintKind::TYPE));

        // Check third hint (z: bool)
        let z_hint = &hints[2];
        assert_eq!(z_hint.position, Position::new(2, 5)); // After "let z"
        if let InlayHintLabel::String(ref label) = z_hint.label {
            assert_eq!(label, ": bool");
        } else {
            panic!("Expected String label");
        }
        assert_eq!(z_hint.kind, Some(InlayHintKind::TYPE));
    }

    #[test]
    fn test_get_inlay_hints_no_variable_declarations() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {\n    println!(\"Hello\");\n}";
        let store = create_dummy_store(uri.as_str(), content);
        let range = Range::new(Position::new(0, 0), Position::new(2, 10));

        let hints = get_inlay_hints(&uri, range, &store);
        assert!(hints.is_empty(), "Should return empty vector when no variable declarations found.");
    }

    #[test]
    fn test_get_inlay_hints_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "let x = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let range = Range::new(Position::new(0, 0), Position::new(0, 20));

        let hints = get_inlay_hints(&non_existent_uri, range, &store);
        assert!(hints.is_empty(), "Should return empty vector if document is not found.");
    }

    #[test]
    fn test_get_inlay_hints_range_filtering() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x = 10;\nlet y = \"hello\";\nlet z = true;";
        let store = create_dummy_store(uri.as_str(), content);
        // Only include the first line in range
        let range = Range::new(Position::new(0, 0), Position::new(0, 20));

        let hints = get_inlay_hints(&uri, range, &store);
        assert_eq!(hints.len(), 1, "Should find only 1 hint within the specified range.");
        assert_eq!(hints[0].position, Position::new(0, 5));
        if let InlayHintLabel::String(ref label) = hints[0].label {
            assert_eq!(label, ": i32");
        } else {
            panic!("Expected String label");
        }
    }

    #[test]
    fn test_get_inlay_hints_complex_values() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let a = 42;\nlet b = false;\nlet c = \"world\";";
        let store = create_dummy_store(uri.as_str(), content);
        let range = Range::new(Position::new(0, 0), Position::new(2, 20));

        let hints = get_inlay_hints(&uri, range, &store);
        assert_eq!(hints.len(), 3, "Should find 3 inlay hints.");
        
        if let InlayHintLabel::String(ref label) = hints[0].label {
            assert_eq!(label, ": i32");
        } else {
            panic!("Expected String label");
        }
        
        if let InlayHintLabel::String(ref label) = hints[1].label {
            assert_eq!(label, ": bool");
        } else {
            panic!("Expected String label");
        }
        
        if let InlayHintLabel::String(ref label) = hints[2].label {
            assert_eq!(label, ": &str");
        } else {
            panic!("Expected String label");
        }
    }
}