// Welcome to Lesson 1-31!
// You can now search symbols across the workspace.
// Let's learn about understanding code relationships: Call Hierarchy.

// Your Task:
// The function `call_hierarchy_incoming_calls` takes:
// - `target_function`: The name of the function for which to find incoming calls.
// - `document_store`: The `HashMap` containing all documents in the workspace.
// It should:
// 1. Search through all documents to find calls to the target function.
// 2. Look for patterns like "target_function(" in the code.
// 3. For each call found, determine which function contains that call.
// 4. Create `CallHierarchyIncomingCall` objects with:
//    - `from`: The calling function as a `CallHierarchyItem`
//    - `from_ranges`: The ranges where the calls occur
// 5. Return a `Vec<CallHierarchyIncomingCall>` containing all incoming calls.
// 6. If no calls are found, return an empty Vec.

use lsp_types::{CallHierarchyIncomingCall, CallHierarchyItem, Position, Range, SymbolKind, Url};
use std::collections::HashMap;

#[derive(Clone)]
struct FnRange {
    pub fn_name: String,
    pub url: Url,
    pub range: Range,
}

struct FnRangeMap {
    pub fn_range_map: HashMap<(String, Url), Vec<FnRange>>,
}

impl FnRangeMap {
    pub fn from(ranges: Vec<FnRange>) -> Self {
        let mut m: HashMap<(String, Url), Vec<FnRange>> = HashMap::new();
        ranges.iter().for_each(|range| {
            let current_ranges = m
                .entry((range.fn_name.clone(), range.clone().url))
                .or_insert_with(Vec::new);
            current_ranges.push(range.clone());
        });

        Self { fn_range_map: m }
    }

    pub fn ranges(&self, fn_name: &str, url: &Url) -> Option<Vec<Range>> {
        Some(
            self.fn_range_map
                .get(&(fn_name.to_string(), url.clone()))?
                .iter()
                .map(|fn_range| fn_range.range)
                .collect(),
        )
    }
}

pub fn call_hierarchy_incoming_calls(
    target_function: &str,
    document_store: &HashMap<Url, String>,
) -> Vec<CallHierarchyIncomingCall> {
    // Hint:
    // 1. Search through all files for calls to target_function
    // 2. For each call found, determine the containing function
    // 3. Group calls by their containing function
    // 4. Create CallHierarchyIncomingCall objects for each calling function
    let search_pattern = format!("{}(", target_function);

    let fn_ranges = document_store
        .iter()
        .filter_map(|(url, content)| {
            Some(
                content
                    .lines()
                    .enumerate()
                    .filter_map(|(line_number, line)| {
                        // より正確な関数呼び出し検出
                        if line.trim_start().starts_with("fn ") {
                            return None; // 関数定義行は除外
                        }

                        if let Some(pos) = line.find(&search_pattern) {
                            // 関数名の境界チェック
                            let is_valid_call = pos == 0
                                || !line.chars().nth(pos - 1).unwrap_or(' ').is_alphanumeric();

                            if is_valid_call {
                                return find_containing_function(url, &content, line_number);
                            }
                        }
                        None
                    })
                    .collect::<Vec<FnRange>>(),
            )
        })
        .flatten()
        .collect::<Vec<FnRange>>();

    let fn_range_map = FnRangeMap::from(fn_ranges);

    fn_range_map
        .fn_range_map
        .iter()
        .filter_map(|((fn_name, url), fn_ranges)| {
            let first_range = fn_ranges.first()?.range;

            Some(CallHierarchyIncomingCall {
                from: CallHierarchyItem {
                    name: fn_name.clone(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    detail: None,
                    uri: url.clone(),
                    range: first_range,
                    selection_range: Range::new(
                        first_range.start,
                        Position::new(
                            first_range.start.line,
                            first_range.start.character + fn_name.len() as u32,
                        ),
                    ),
                    data: None,
                },
                from_ranges: fn_range_map.ranges(fn_name, url)?,
            })
        })
        .collect::<Vec<CallHierarchyIncomingCall>>()
}

// Helper function to find the containing function for a given line
fn find_containing_function(url: &Url, content: &str, target_line: usize) -> Option<FnRange> {
    let lines: Vec<&str> = content.lines().collect();

    for line_number in (0..=target_line).rev() {
        let line = lines.get(line_number)?;

        if !line.starts_with("fn") {
            continue;
        }

        let fn_name = extract_fn_name(line)?;
        return Some(FnRange {
            fn_name: fn_name.to_string(),
            url: url.clone(),
            range: Range::new(
                Position::new(line_number as u32, line.find(fn_name)? as u32),
                Position::new(find_fn_end(content, line_number)? as u32, 0),
            ),
        });
    }

    None
}

fn extract_fn_name(line: &str) -> Option<&str> {
    let remaining = line.trim_start_matches("fn ");
    let fn_def_ends_at = remaining.find('(')?;
    remaining.get(..fn_def_ends_at)
}

fn find_fn_end(content: &str, start_line_number: usize) -> Option<usize> {
    let lines: Vec<&str> = content.lines().collect();

    // 次の関数定義を探す（現在の関数の後から）
    for line_number in (start_line_number + 1)..lines.len() {
        if let Some(line) = lines.get(line_number) {
            if line.trim_start().starts_with("fn ") {
                return Some(line_number - 1); // 前の行が現在の関数の終了
            }
        }
    }

    // 次の関数が見つからない場合はファイル終端
    Some(lines.len().saturating_sub(1))
}

// --- Tests --- //

#[cfg(test)]
mod tests {
    use super::call_hierarchy_incoming_calls;
    use lsp_types::{SymbolKind, Url};
    use std::collections::HashMap;
    use std::str::FromStr;

    // Helper to create a workspace with function calls
    fn create_call_hierarchy_workspace() -> HashMap<Url, String> {
        let mut workspace = HashMap::new();

        // mod - calls helper and calculate
        workspace.insert(
            Url::from_str("file:///src/mod").unwrap(),
            "fn main() {\n    let result = calculate(10);\n    helper();\n    println!(\"Result: {}\", result);\n}\n\nfn another_func() {\n    calculate(20);\n}".to_string()
        );

        // utils.rs - has calculate function and calls helper
        workspace.insert(
            Url::from_str("file:///src/utils.rs").unwrap(),
            "fn calculate(x: i32) -> i32 {\n    helper();\n    x * 2\n}\n\nfn helper() {\n    println!(\"Helper called\");\n}".to_string()
        );

        // lib.rs - calls calculate from different functions
        workspace.insert(
            Url::from_str("file:///src/lib.rs").unwrap(),
            "fn process_data() {\n    let value = calculate(5);\n    println!(\"Processed: {}\", value);\n}\n\nfn batch_process() {\n    for i in 0..10 {\n        calculate(i);\n    }\n}".to_string()
        );

        workspace
    }

    #[test]
    fn test_call_hierarchy_calculate_function() {
        let workspace = create_call_hierarchy_workspace();
        let calls = call_hierarchy_incoming_calls("calculate", &workspace);

        // calculate is called from: main, another_func, process_data, batch_process
        assert_eq!(
            calls.len(),
            4,
            "Should find 4 functions calling 'calculate'"
        );

        let calling_functions: Vec<&str> =
            calls.iter().map(|call| call.from.name.as_str()).collect();
        assert!(
            calling_functions.contains(&"main"),
            "Should find call from 'main'"
        );
        assert!(
            calling_functions.contains(&"another_func"),
            "Should find call from 'another_func'"
        );
        assert!(
            calling_functions.contains(&"process_data"),
            "Should find call from 'process_data'"
        );
        assert!(
            calling_functions.contains(&"batch_process"),
            "Should find call from 'batch_process'"
        );
    }

    #[test]
    fn test_call_hierarchy_helper_function() {
        let workspace = create_call_hierarchy_workspace();
        let calls = call_hierarchy_incoming_calls("helper", &workspace);

        // helper is called from: main, calculate
        assert_eq!(calls.len(), 2, "Should find 2 functions calling 'helper'");

        let calling_functions: Vec<&str> =
            calls.iter().map(|call| call.from.name.as_str()).collect();
        assert!(
            calling_functions.contains(&"main"),
            "Should find call from 'main'"
        );
        assert!(
            calling_functions.contains(&"calculate"),
            "Should find call from 'calculate'"
        );
    }

    #[test]
    fn test_call_hierarchy_no_calls() {
        let workspace = create_call_hierarchy_workspace();
        let calls = call_hierarchy_incoming_calls("nonexistent_function", &workspace);

        assert!(
            calls.is_empty(),
            "Should return empty for function with no calls"
        );
    }

    #[test]
    fn test_call_hierarchy_call_ranges() {
        let workspace = create_call_hierarchy_workspace();
        let calls = call_hierarchy_incoming_calls("calculate", &workspace);

        // Find the call from main function
        let main_call = calls.iter().find(|call| call.from.name == "main").unwrap();
        assert!(
            !main_call.from_ranges.is_empty(),
            "Should have at least one call range"
        );

        let call_range = &main_call.from_ranges[0];
        assert_eq!(call_range.start.line, 0, "Call should be on line 0");
        assert!(
            call_range.start.character > 0,
            "Call should have valid character position"
        );
    }

    #[test]
    fn test_call_hierarchy_multiple_calls_same_function() {
        let workspace = create_call_hierarchy_workspace();
        let calls = call_hierarchy_incoming_calls("calculate", &workspace);

        // Find the call from batch_process which has calculate in a loop
        let batch_call = calls
            .iter()
            .find(|call| call.from.name == "batch_process")
            .unwrap();
        assert_eq!(
            batch_call.from_ranges.len(),
            1,
            "Should find 1 call range even with loop"
        );
    }

    #[test]
    fn test_call_hierarchy_call_hierarchy_item_structure() {
        let workspace = create_call_hierarchy_workspace();
        let calls = call_hierarchy_incoming_calls("helper", &workspace);

        let main_call = calls.iter().find(|call| call.from.name == "main").unwrap();

        // Check CallHierarchyItem structure
        assert_eq!(main_call.from.name, "main");
        assert_eq!(main_call.from.kind, SymbolKind::FUNCTION);
        assert_eq!(main_call.from.uri.as_str(), "file:///src/mod");
        assert_eq!(main_call.from.range.start.line, 0); // main function starts at line 0
        assert_eq!(main_call.from.selection_range.start.line, 0);
    }

    #[test]
    fn test_call_hierarchy_empty_workspace() {
        let empty_workspace = HashMap::new();
        let calls = call_hierarchy_incoming_calls("any_function", &empty_workspace);

        assert!(calls.is_empty(), "Should return empty for empty workspace");
    }

    #[test]
    fn test_call_hierarchy_function_calls_itself() {
        let mut workspace = HashMap::new();
        workspace.insert(
            Url::from_str("file:///src/recursive.rs").unwrap(),
            "fn factorial(n: u32) -> u32 {\n    if n <= 1 {\n        1\n    } else {\n        n * factorial(n - 1)\n    }\n}".to_string()
        );

        let calls = call_hierarchy_incoming_calls("factorial", &workspace);

        // factorial calls itself recursively
        assert_eq!(
            calls.len(),
            1,
            "Should find 1 function calling 'factorial' (itself)"
        );
        assert_eq!(
            calls[0].from.name, "factorial",
            "Should find recursive call from 'factorial'"
        );
    }
}
