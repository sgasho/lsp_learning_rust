// Welcome to Lesson 1-30!
// You can now provide signature help.
// Let's learn about workspace-wide navigation: Workspace Symbols.

// Your Task:
// The function `workspace_symbol` takes:
// - `query`: The search query string (e.g., "main", "calc", "User").
// - `document_store`: The `HashMap` containing all documents in the workspace.
// It should:
// 1. Search through all documents in the workspace for symbols matching the query.
// 2. Look for function definitions (lines starting with "fn ") and struct definitions (lines starting with "struct ").
// 3. Extract the symbol name and check if it contains the query string (case-insensitive).
// 4. For each matching symbol, create a `SymbolInformation` object with:
//    - `name`: The symbol name (e.g., "main", "calculate", "User")
//    - `kind`: `SymbolKind::FUNCTION` for functions, `SymbolKind::STRUCT` for structs
//    - `location`: The location of the symbol (file URI and range)
// 5. Return a `Vec<SymbolInformation>` containing all matching symbols.
// 6. If no symbols match or the query is empty, return an empty Vec.

use lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};
use std::collections::HashMap;

fn extract_fn_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("fn ") {
        return None;
    }

    let remaining = trimmed.strip_prefix("fn ")?;
    let keyword_end = remaining
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(remaining.len());

    if keyword_end > 0 {
        Some(remaining[..keyword_end].to_string())
    } else {
        None
    }
}

fn extract_struct_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("struct ") {
        return None;
    }

    let remaining = trimmed.strip_prefix("struct ")?;
    let keyword_end = remaining
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(remaining.len());

    if keyword_end > 0 {
        Some(remaining[..keyword_end].to_string())
    } else {
        None
    }
}

fn create_symbol_info(
    name: String,
    kind: SymbolKind,
    uri: &Url,
    line_number: u32,
    line_content: &str,
) -> SymbolInformation {
    SymbolInformation {
        name,
        kind,
        tags: None,
        deprecated: None,
        location: Location {
            uri: uri.clone(),
            range: Range::new(
                Position::new(line_number, 0),
                Position::new(line_number, line_content.len() as u32),
            ),
        },
        container_name: None,
    }
}

pub fn workspace_symbol(
    query: &str,
    document_store: &HashMap<Url, String>,
) -> Vec<SymbolInformation> {
    // 早期リターン: 空のクエリ
    if query.is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // ワークスペース内の全ファイルを反復処理
    for (uri, content) in document_store {
        // ファイル全体の事前チェック（パフォーマンス最適化）
        if !content.to_lowercase().contains(&query_lower) {
            continue;
        }

        // 各行を処理（行番号付き）
        for (line_number, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();

            // 効率的な早期フィルタリング
            if !(trimmed.starts_with("fn ") || trimmed.starts_with("struct ")) {
                continue;
            }

            // 関数定義の処理
            if let Some(fn_name) = extract_fn_name(line) {
                if fn_name.to_lowercase().contains(&query_lower) {
                    results.push(create_symbol_info(
                        fn_name,
                        SymbolKind::FUNCTION,
                        uri,
                        line_number as u32,
                        line,
                    ));
                }
            }

            // 構造体定義の処理
            if let Some(struct_name) = extract_struct_name(line) {
                if struct_name.to_lowercase().contains(&query_lower) {
                    results.push(create_symbol_info(
                        struct_name,
                        SymbolKind::STRUCT,
                        uri,
                        line_number as u32,
                        line,
                    ));
                }
            }
        }
    }

    results
}

// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::workspace_symbol;
    use lsp_types::{Position, SymbolKind, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a workspace with multiple files
    fn create_workspace() -> HashMap<Url, String> {
        let mut workspace = HashMap::new();

        // mod
        workspace.insert(
            Url::from_str("file:///src/mod").unwrap(),
            "fn main() {\n    println!(\"Hello World\");\n}\n\nfn calculate(x: i32) -> i32 {\n    x * 2\n}".to_string()
        );

        // user.rs
        workspace.insert(
            Url::from_str("file:///src/user.rs").unwrap(),
            "struct User {\n    name: String,\n    age: u32,\n}\n\nfn create_user() -> User {\n    User { name: \"test\".to_string(), age: 25 }\n}".to_string()
        );

        // utils.rs
        workspace.insert(
            Url::from_str("file:///src/utils.rs").unwrap(),
            "struct Calculator {\n    value: f64,\n}\n\nfn add_numbers(a: i32, b: i32) -> i32 {\n    a + b\n}".to_string()
        );

        workspace
    }

    #[test]
    fn test_workspace_symbol_search_functions() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("main", &workspace);

        assert_eq!(symbols.len(), 1, "Should find 1 symbol for 'main'");
        let main_symbol = &symbols[0];
        assert_eq!(main_symbol.name, "main");
        assert_eq!(main_symbol.kind, SymbolKind::FUNCTION);
        assert_eq!(main_symbol.location.uri.as_str(), "file:///src/mod");
        assert_eq!(main_symbol.location.range.start, Position::new(0, 0));
    }

    #[test]
    fn test_workspace_symbol_search_structs() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("User", &workspace);

        // Should find both "User" struct and "create_user" function
        assert_eq!(
            symbols.len(),
            2,
            "Should find 2 symbols for 'User' (struct + function)"
        );

        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(symbol_names.contains(&"User"), "Should find 'User' struct");
        assert!(
            symbol_names.contains(&"create_user"),
            "Should find 'create_user' function"
        );

        // Check that we have both a struct and a function
        let has_struct = symbols.iter().any(|s| s.kind == SymbolKind::STRUCT);
        let has_function = symbols.iter().any(|s| s.kind == SymbolKind::FUNCTION);
        assert!(has_struct, "Should find at least one struct");
        assert!(has_function, "Should find at least one function");
    }

    #[test]
    fn test_workspace_symbol_case_insensitive() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("CALC", &workspace);

        // Should find both "calculate" function and "Calculator" struct
        assert_eq!(
            symbols.len(),
            2,
            "Should find 2 symbols for 'CALC' (case-insensitive)"
        );

        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            symbol_names.contains(&"calculate"),
            "Should find 'calculate' function"
        );
        assert!(
            symbol_names.contains(&"Calculator"),
            "Should find 'Calculator' struct"
        );
    }

    #[test]
    fn test_workspace_symbol_partial_match() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("user", &workspace);

        // Should find both "User" struct and "create_user" function
        assert_eq!(symbols.len(), 2, "Should find 2 symbols containing 'user'");

        let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(symbol_names.contains(&"User"), "Should find 'User' struct");
        assert!(
            symbol_names.contains(&"create_user"),
            "Should find 'create_user' function"
        );
    }

    #[test]
    fn test_workspace_symbol_no_matches() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("nonexistent", &workspace);

        assert!(
            symbols.is_empty(),
            "Should return empty for non-matching query"
        );
    }

    #[test]
    fn test_workspace_symbol_empty_query() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("", &workspace);

        assert!(symbols.is_empty(), "Should return empty for empty query");
    }

    #[test]
    fn test_workspace_symbol_all_types() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("a", &workspace); // Should match several symbols

        // Should find: main, calculate, add_numbers, Calculator
        assert!(
            symbols.len() >= 3,
            "Should find multiple symbols containing 'a'"
        );

        // Check that we have both functions and structs
        let has_function = symbols.iter().any(|s| s.kind == SymbolKind::FUNCTION);
        let has_struct = symbols.iter().any(|s| s.kind == SymbolKind::STRUCT);
        assert!(has_function, "Should find at least one function");
        assert!(has_struct, "Should find at least one struct");
    }

    #[test]
    fn test_workspace_symbol_correct_locations() {
        let workspace = create_workspace();
        let symbols = workspace_symbol("Calculator", &workspace);

        assert_eq!(symbols.len(), 1);
        let calc_symbol = &symbols[0];

        // Should be in utils.rs at line 0
        assert_eq!(calc_symbol.location.uri.as_str(), "file:///src/utils.rs");
        assert_eq!(calc_symbol.location.range.start.line, 0);
        assert_eq!(calc_symbol.location.range.start.character, 0);
        // Range should cover the entire line
        assert!(calc_symbol.location.range.end.character > 0);
    }

    #[test]
    fn test_workspace_symbol_empty_workspace() {
        let empty_workspace = HashMap::new();
        let symbols = workspace_symbol("anything", &empty_workspace);

        assert!(
            symbols.is_empty(),
            "Should return empty for empty workspace"
        );
    }
}
