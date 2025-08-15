// Welcome to Lesson 1-28!
// You can now provide inlay hints.
// Let's learn about one of the most important LSP features: Completion (Auto-complete).

// Your Task:
// The function `get_completion_items` takes:
// - `file_uri`: The `Url` of the document where completion was requested.
// - `position`: The `Position` where completion was triggered.
// - `document_store`: The `HashMap` containing document content.
// It should:
// 1. Retrieve the content of `file_uri` from the `document_store`.
// 2. Find the word being typed at the given `position` (partial word before cursor).
// 3. Based on the partial word, suggest relevant Rust keywords and built-in types:
//    - If partial starts with "l": suggest "let", "loop"
//    - If partial starts with "f": suggest "fn", "for", "false"
//    - If partial starts with "i": suggest "i32", "if", "impl"
//    - If partial starts with "s": suggest "struct", "String", "str"
//    - If partial starts with "t": suggest "true", "type"
// 4. Create `CompletionItem` objects with:
//    - `label`: The completion text (e.g., "let")
//    - `kind`: Appropriate `CompletionItemKind` (Keyword, Type, etc.)
//    - `insert_text`: Optional text to insert (same as label for now)
// 5. Return a `Vec<CompletionItem>` containing all matching suggestions.
// 6. If no partial word is found or document doesn't exist, return empty Vec.

use lsp_types::{CompletionItem, CompletionItemKind, Position, Url};
use std::collections::HashMap;

fn new_completion_item(label: &str, kind: CompletionItemKind) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(kind),
        ..Default::default()
    }
}

fn get_keyword_completions(prefix: &str) -> Vec<CompletionItem> {
    let keywords = match prefix {
        p if p.starts_with("l") => vec!["let", "loop"],
        p if p.starts_with("f") => vec!["fn", "for", "false"],
        p if p.starts_with("i") => vec!["if", "impl"],
        p if p.starts_with("s") => vec!["struct"],
        p if p.starts_with("t") => vec!["true", "type"],
        _ => vec![]
    };
    
    keywords.into_iter()
        .map(|keyword| new_completion_item(keyword, CompletionItemKind::KEYWORD))
        .collect()
}

fn get_type_completions(prefix: &str) -> Vec<CompletionItem> {
    let types = match prefix {
        p if p.starts_with("i") => vec!["i32"],
        p if p.starts_with("s") => vec!["String", "str"],
        _ => vec![]
    };
    
    types.into_iter()
        .map(|type_name| new_completion_item(type_name, CompletionItemKind::TYPE_PARAMETER))
        .collect()
}

pub fn get_completion_items(file_uri: &Url, position: Position, document_store: &HashMap<Url, String>) -> Vec<CompletionItem> {
    // Hint:
    // 1. Get document content from document_store
    // 2. Extract the line at the given position
    // 3. Find the partial word before the cursor position
    // 4. Match the partial word against predefined keywords/types
    // 5. Create CompletionItem objects for matching suggestions
    let content: Option<&str> = (|| {
        let (_, content) = document_store
            .get(file_uri)?
            .lines()
            .enumerate()
            .find(|(line_number, _)| *line_number == position.line as usize )?;
        Some(content)
    })();

    let completion_items: Option<Vec<CompletionItem>> = (|| {
        let before_cursor = content?.get(..position.character as usize)?;
        let mut start = position.character as usize;

        // 後ろから英数字・アンダースコアを辿る
        for (i, ch) in before_cursor.char_indices().rev() {
            if ch.is_alphanumeric() || ch == '_' {
                start = i;
            } else {
                break;
            }
        }

        let partial = if start < position.character as usize {
            Some(before_cursor[start..].to_string())
        } else {
            None // 部分単語が見つからない
        };

        let prefix = partial?.to_lowercase();
        let mut completions = Vec::new();
        
        // キーワード候補を追加
        completions.extend(get_keyword_completions(&prefix));
        
        // 型候補を追加
        completions.extend(get_type_completions(&prefix));
        
        Some(completions)
    })();

    completion_items.unwrap_or_default()
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::get_completion_items;
    use lsp_types::{CompletionItemKind, Position, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a dummy document store
    fn create_dummy_store(uri_str: &str, content: &str) -> HashMap<Url, String> {
        let mut store = HashMap::new();
        store.insert(Url::from_str(uri_str).unwrap(), content.to_string());
        store
    }

    #[test]
    fn test_completion_for_let_keywords() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {\n    l\n}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(1, 5); // After "l"

        let items = get_completion_items(&uri, position, &store);
        assert!(!items.is_empty(), "Should provide completions for 'l'");
        
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        assert!(labels.contains(&"let"), "Should suggest 'let'");
        assert!(labels.contains(&"loop"), "Should suggest 'loop'");
        
        // Check that "let" has the correct kind
        let let_item = items.iter().find(|item| item.label == "let").unwrap();
        assert_eq!(let_item.kind, Some(CompletionItemKind::KEYWORD));
    }

    #[test]
    fn test_completion_for_function_keywords() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "f";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1); // After "f"

        let items = get_completion_items(&uri, position, &store);
        assert!(!items.is_empty(), "Should provide completions for 'f'");
        
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        assert!(labels.contains(&"fn"), "Should suggest 'fn'");
        assert!(labels.contains(&"for"), "Should suggest 'for'");
        assert!(labels.contains(&"false"), "Should suggest 'false'");
    }

    #[test]
    fn test_completion_for_types() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x: i";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 8); // After "i"

        let items = get_completion_items(&uri, position, &store);
        assert!(!items.is_empty(), "Should provide completions for 'i'");
        
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        assert!(labels.contains(&"i32"), "Should suggest 'i32'");
        assert!(labels.contains(&"if"), "Should suggest 'if'");
        assert!(labels.contains(&"impl"), "Should suggest 'impl'");
        
        // Check that "i32" has the correct kind
        let i32_item = items.iter().find(|item| item.label == "i32").unwrap();
        assert_eq!(i32_item.kind, Some(CompletionItemKind::TYPE_PARAMETER));
    }

    #[test]
    fn test_completion_for_struct_keywords() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "s";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1); // After "s"

        let items = get_completion_items(&uri, position, &store);
        assert!(!items.is_empty(), "Should provide completions for 's'");
        
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        assert!(labels.contains(&"struct"), "Should suggest 'struct'");
        assert!(labels.contains(&"String"), "Should suggest 'String'");
        assert!(labels.contains(&"str"), "Should suggest 'str'");
    }

    #[test]
    fn test_completion_no_partial_word() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "fn main() {\n    \n}";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(1, 4); // At whitespace

        let items = get_completion_items(&uri, position, &store);
        assert!(items.is_empty(), "Should return empty when no partial word found");
    }

    #[test]
    fn test_completion_document_not_found() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let non_existent_uri = Url::from_str("file:///non_existent.rs").unwrap();
        let content = "l";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1);

        let items = get_completion_items(&non_existent_uri, position, &store);
        assert!(items.is_empty(), "Should return empty when document not found");
    }

    #[test]
    fn test_completion_position_out_of_bounds() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "let x = 10;";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(10, 10); // Out of bounds

        let items = get_completion_items(&uri, position, &store);
        assert!(items.is_empty(), "Should return empty when position is out of bounds");
    }

    #[test]
    fn test_completion_case_insensitive() {
        let uri = Url::from_str("file:///test.rs").unwrap();
        let content = "L";
        let store = create_dummy_store(uri.as_str(), content);
        let position = Position::new(0, 1); // After "L"

        let items = get_completion_items(&uri, position, &store);
        // Should work with uppercase as well
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        // Note: We'll implement case-insensitive matching
        assert!(labels.contains(&"let") || labels.contains(&"loop"), "Should handle uppercase input");
    }
}