// Welcome to Lesson 1-15!
// You can now provide diagnostics for code.
// Now, let's make our server even smarter by offering code completion suggestions.

// Your Task:
// The function `generate_completions` takes `file_content` (as `&str`) and `position` (as `Position`).
// It should generate a list of `lsp_types::CompletionItem` based on simple rules.
// For this lesson, we will always suggest "fn", "let", and "struct" as completion items,
// regardless of the content or position.
// Return a `Vec<CompletionItem>`.

use lsp_types::{CompletionItem, CompletionItemKind, Position};

pub fn generate_completions(_file_content: &str, _position: Position) -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "fn".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "let".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        },
        CompletionItem {
            label: "struct".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        }
    ]
}


// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::generate_completions;
    use lsp_types::{CompletionItem, CompletionItemKind, Position};

    #[test]
    fn test_always_suggests_basic_keywords() {
        let content = "";
        let position = Position::new(0, 0);
        let completions = generate_completions(content, position);

        assert_eq!(completions.len(), 3, "Should suggest 3 items.");

        let expected_items = vec![
            CompletionItem {
                label: "fn".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            },
            CompletionItem {
                label: "struct".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            },
        ];

        // Sort both vectors to ensure order doesn't matter for comparison
        let mut sorted_completions = completions;
        sorted_completions.sort_by(|a, b| a.label.cmp(&b.label));
        let mut sorted_expected_items = expected_items;
        sorted_expected_items.sort_by(|a, b| a.label.cmp(&b.label));

        assert_eq!(sorted_completions, sorted_expected_items, "Suggested items should match expected keywords.");
    }

    #[test]
    fn test_completions_are_always_the_same_regardless_of_input() {
        let content1 = "some code";
        let position1 = Position::new(1, 5);
        let completions1 = generate_completions(content1, position1);

        let content2 = "another line";
        let position2 = Position::new(0, 0);
        let completions2 = generate_completions(content2, position2);

        assert_eq!(completions1, completions2, "Completions should be consistent regardless of input for this lesson.");
    }
}
