# Lesson 1-31: 関数の呼び出し関係を調べよう！ (LSPコールハイアラーキー)

LSPのワークスペースシンボル機能ができるようになりましたね。素晴らしいです！

次に、コードの構造と依存関係を理解するための重要な機能「**コールハイアラーキー（Call Hierarchy）**
」について学びます。この機能は、関数がどこから呼び出されているか、またはある関数がどの関数を呼び出しているかを階層的に表示する機能です。

## コールハイアラーキー (Call Hierarchy) とは？

コールハイアラーキー機能は、関数間の呼び出し関係を視覚化する機能です。2つの方向があります：

### 🔍 **今回実装**: Incoming Calls（被呼び出し）

**対象関数**を**どの関数が呼んでいるか**を見つける機能：

```rust
// 🎯 調べたい対象: "calculate" 関数
fn calculate(x: i32) -> i32 {    // ← この関数が対象
    x * 2
}

// 📍 結果: calculate を呼んでいる関数たち
fn main() {
    calculate(10);     // ✅ main が calculate を呼んでいる
}

fn process() {
    calculate(20);     // ✅ process が calculate を呼んでいる  
}

fn another() {
    let x = 5;        // ❌ calculate を呼んでいない
}
```

**結果として得られる情報**:

- `main` 関数が `calculate(10)` で呼び出し
- `process` 関数が `calculate(20)` で呼び出し

### 📖 参考: Outgoing Calls（呼び出し先）

**対象関数**が**どの関数を呼んでいるか**を見つける機能（今回は実装しません）：

```rust
// 🎯 調べたい対象: "main" 関数  
fn main() {                     // ← この関数が対象
    helper();                   // ✅ main が helper を呼んでいる
    calculate(10);              // ✅ main が calculate を呼んでいる
    println!("Done");           // ✅ main が println! を呼んでいる
}
```

この機能により、開発者は：

- **依存関係の把握**: 関数間の依存関係を明確に理解
- **影響範囲の分析**: 関数を変更した時の影響範囲を確認
- **リファクタリング支援**: 安全なコード変更の計画立案
- **デバッグ支援**: 呼び出し経路の追跡

## `lsp-types::CallHierarchyIncomingCall` の構造

LSPのコールハイアラーキーは複数の構造体で表現されます：

### `CallHierarchyIncomingCall`

```rust
pub struct CallHierarchyIncomingCall {
    pub from: CallHierarchyItem,        // 呼び出し元の関数情報
    pub from_ranges: Vec<Range>,        // 呼び出し箇所の範囲一覧
}
```

### `CallHierarchyItem`

```rust
pub struct CallHierarchyItem {
    pub name: String,                   // 関数名
    pub kind: SymbolKind,              // シンボルの種類（通常は FUNCTION）
    pub tags: Option<Vec<SymbolTag>>,   // 追加のタグ情報
    pub detail: Option<String>,         // 詳細情報（シグネチャなど）
    pub uri: Url,                      // ファイルのURI
    pub range: Range,                  // 関数全体の範囲
    pub selection_range: Range,        // 関数名の範囲
    pub data: Option<serde_json::Value>, // 追加データ
}
```

## 関数呼び出しの検出アルゴリズム

コールハイアラーキーを実装するには、以下の手順が必要です：

### 1. 関数呼び出しの検出

```rust
fn find_function_calls(content: &str, target_function: &str) -> Vec<(usize, usize)> {
    let mut calls = Vec::new();
    let pattern = format!("{}(", target_function);
    
    for (line_number, line) in content.lines().enumerate() {
        if let Some(column) = line.find(&pattern) {
            // 関数名の境界チェック（前後が識別子文字でないこと）
            if is_valid_function_call(line, column, target_function) {
                calls.push((line_number, column));
            }
        }
    }
    
    calls
}

fn is_valid_function_call(line: &str, start_pos: usize, function_name: &str) -> bool {
    // 前の文字が識別子文字でないことを確認
    if start_pos > 0 {
        let prev_char = line.chars().nth(start_pos - 1).unwrap_or(' ');
        if prev_char.is_alphanumeric() || prev_char == '_' {
            return false;
        }
    }
    
    // 後の文字チェック（関数名の直後に'('があることを確認済み）
    true
}
```

### 2. 呼び出し箇所を含む関数の特定

```rust
fn find_containing_function(content: &str, target_line: usize) -> Option<(String, Range)> {
    let lines: Vec<&str> = content.lines().collect();
    
    // target_line（呼び出し箇所）から逆方向に検索して、
    // その呼び出しがどの関数の中にあるかを見つける
    for line_number in (0..=target_line).rev() {
        let line = lines.get(line_number)?;
        
        if let Some(func_name) = extract_function_name(line) {
            // 関数の終了位置を見つける（簡略化: 次の関数定義まで）
            let end_line = find_function_end(content, line_number);
            
            let range = Range::new(
                Position::new(line_number as u32, 0),
                Position::new(end_line as u32, lines.get(end_line)?.len() as u32)
            );
            
            return Some((func_name, range));
        }
    }
    
    None
}

fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("fn ") {
        return None;
    }
    
    let after_fn = trimmed.strip_prefix("fn ")?;
    let end_pos = after_fn.find('(')?;
    Some(after_fn[..end_pos].trim().to_string())
}
```

**具体例**:

```rust
fn main() {           // ← 5行目: main関数の開始
    let x = 10;       // ← 6行目
    calculate(x);     // ← 7行目: calculate呼び出し発見！
    println!("OK");   // ← 8行目
}                     // ← 9行目: main関数の終了

fn process() {        // ← 11行目: process関数の開始
    calculate(20);    // ← 12行目: calculate呼び出し発見！
}                     // ← 13行目: process関数の終了
```

- 7行目で `calculate` 呼び出し発見 → 逆方向検索 → 5行目で `fn main` 発見 → "main関数の中"
- 12行目で `calculate` 呼び出し発見 → 逆方向検索 → 11行目で `fn process` 発見 → "process関数の中"

### 3. 呼び出し情報の統合

```rust
fn group_calls_by_function(
    calls: Vec<(Url, usize, usize)>, 
    document_store: &HashMap<Url, String>
) -> Vec<CallHierarchyIncomingCall> {
    let mut grouped: HashMap<String, (CallHierarchyItem, Vec<Range>)> = HashMap::new();
    
    for (uri, line_number, column) in calls {
        let content = document_store.get(&uri)?;
        
        if let Some((func_name, func_range)) = find_containing_function(content, line_number) {
            let call_range = Range::new(
                Position::new(line_number as u32, column as u32),
                Position::new(line_number as u32, (column + target_function.len()) as u32)
            );
            
            // 同じ関数からの複数呼び出しをグループ化
            let key = format!("{}::{}", uri, func_name);
            
            match grouped.get_mut(&key) {
                Some((_, ranges)) => {
                    ranges.push(call_range);
                }
                None => {
                    let hierarchy_item = CallHierarchyItem {
                        name: func_name.clone(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: None,
                        uri: uri.clone(),
                        range: func_range.clone(),
                        selection_range: create_selection_range(&func_range, &func_name),
                        data: None,
                    };
                    grouped.insert(key, (hierarchy_item, vec![call_range]));
                }
            }
        }
    }
    
    // HashMap から Vec への変換
    grouped.into_values()
        .map(|(from, from_ranges)| CallHierarchyIncomingCall { from, from_ranges })
        .collect()
}
```

## 実装のポイント

### 1. 正確な関数呼び出し検出

```rust
// 単純な文字列検索では不十分
// ❌ "calculate_result()" で "calculate(" を誤検出
// ✅ 境界チェックで正確な検出

if start_pos == 0 || !line.chars().nth(start_pos - 1).unwrap_or(' ').is_alphanumeric() {
    // 正確な関数呼び出し
}
```

### 2. ネストした関数の処理

```rust
// 関数内の関数（クロージャなど）への対応
fn outer() {
    let closure = |x| {
        calculate(x); // この呼び出しは outer から？ closure から？
    };
}
```

### 3. コメント内の呼び出し除外

```rust
// コメント内の関数名は除外
fn process() {
    // calculate(10); ← これは実際の呼び出しではない
    let result = calculate(5); // ← これは実際の呼び出し
}
```

### 4. 効率的な検索

```rust
// 大きなファイルでのパフォーマンス考慮
// 1. ファイル全体の事前フィルタリング
if !content.contains(target_function) {
    continue;
}

// 2. 行レベルでの早期スキップ
if !line.contains(target_function) {
    continue;
}
```

## やってみよう！

あなたの今回のミッションは、`call_hierarchy_incoming_calls` 関数を完成させることです。

### 🎯 目標: "calculate" を呼んでいる関数を全て見つける

例えば、`target_function = "calculate"` の場合：

```rust
// 📂 mod
fn main() {
    calculate(10);        // ✅ 発見: main が calculate を呼んでいる
}

fn another_func() {
    calculate(20);        // ✅ 発見: another_func が calculate を呼んでいる  
}

// 📂 utils.rs  
fn process_data() {
    let result = calculate(5);  // ✅ 発見: process_data が calculate を呼んでいる
}
```

### 🔧 実装手順

1. **📋 STEP 1**: ワークスペース内の全ファイルで `target_function` **への呼び出し箇所**を検索
    - `"calculate("` のパターンを探す
    - 呼び出し箇所の行番号と列番号を記録

2. **🔍 STEP 2**: 各呼び出し箇所について**どの関数の中で呼ばれているか**を特定
    - `find_containing_function` ヘルパー関数を使用
    - 呼び出し行から逆方向に検索して最寄りの `fn` 定義を見つける

3. **📦 STEP 3**: 同じ関数からの複数呼び出しをグループ化
    - 1つの関数が対象関数を複数回呼んでいる場合をまとめる

4. **🏗️ STEP 4**: 各**呼び出し元関数**に対して `CallHierarchyIncomingCall` オブジェクトを作成
    - `from`: **呼び出し元関数**の `CallHierarchyItem`（main, process_data など）
    - `from_ranges`: その関数内での**呼び出し箇所**一覧

5. **📤 STEP 5**: すべての呼び出し情報を `Vec<CallHierarchyIncomingCall>` として返す

### 🚨 重要な理解ポイント

- **`target_function`**: 調べたい対象の関数（例: "calculate"）
- **`from`**: その対象関数を**呼んでいる関数**（例: "main", "process_data"）
- **`from_ranges`**: **呼び出している箇所**の位置情報

**ヒント**: `find_containing_function` ヘルパー関数も実装する必要があります。

`src/lessons/lesson_1_31.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！