# Lesson 1-30: プロジェクト全体から探そう！ (LSPワークスペースシンボル)

LSPのシグネチャヘルプ機能ができるようになりましたね。素晴らしいです！

次に、大規模なプロジェクトでのナビゲーションに欠かせない「**ワークスペースシンボル（Workspace Symbols）**
」機能について学びます。この機能は、プロジェクト全体から関数や型を名前で検索し、素早くジャンプできる強力な機能です。

## ワークスペースシンボル (Workspace Symbols) とは？

ワークスペースシンボル機能は、プロジェクト全体（ワークスペース）に含まれるすべてのファイルから、関数、構造体、列挙型、定数などのシンボルを検索する機能です。例えば：

```rust
// ユーザーが "calc" と検索すると以下が見つかる：

// src/mod
fn calculate(x: i32) -> i32 { ... }    // ← マッチ

// src/math.rs  
struct Calculator {
    ...
}              // ← マッチ

// src/utils.rs
fn recalculate() -> f64 { ... }       // ← マッチ
```

この機能により、開発者は：

- **素早いナビゲーション**: 大きなコードベースでも瞬時に目的の関数や型を発見
- **コードの理解**: プロジェクト全体の構造とシンボルの把握
- **リファクタリング支援**: 関数や型の影響範囲の確認

エディタは、ユーザーがシンボル検索を開始すると、言語サーバーに `workspace/symbol` リクエストを送ります。サーバーはこれに対して、マッチするシンボル情報のリスト（
`SymbolInformation` オブジェクトの配列）を返します。

## `lsp-types::SymbolInformation` の構造

LSPのワークスペースシンボルは `lsp_types::SymbolInformation` という構造体で表現されます。主なフィールドは以下の通りです。

* `name`: シンボルの名前（例: `"main"`, `"User"`, `"calculate"`）
* `kind`: シンボルの種類。`SymbolKind` Enum（`Function`, `Struct`, `Enum`, `Constant`など）を使います
* `location`: シンボルの場所情報（`Location`）
    - `uri`: ファイルのURI
    - `range`: シンボルの範囲（通常は定義行全体）
* `tags`: オプションの追加情報（`deprecated`など）
* `container_name`: オプションの親コンテナ名（例: モジュール名、クラス名）

## ワークスペース全体の検索アルゴリズム

ワークスペースシンボル検索は、複数のファイルを横断して行うため、効率的なアルゴリズムが重要です。

### 基本的な検索フロー

```rust
fn workspace_symbol_search(query: &str, workspace: &HashMap<Url, String>) -> Vec<SymbolInformation> {
    let mut results = Vec::new();
    
    // 1. 空のクエリは早期リターン
    if query.trim().is_empty() {
        return results;
    }
    
    // 2. ワークスペース内の全ファイルを反復処理
    for (file_uri, content) in workspace {
        // 3. 各ファイルからシンボルを抽出
        let file_symbols = extract_symbols_from_file(file_uri, content, query);
        results.extend(file_symbols);
    }
    
    // 4. 結果をソート（関連度順）
    results.sort_by(|a, b| relevance_score(&a.name, query).cmp(&relevance_score(&b.name, query)));
    
    results
}
```

### ファイルからのシンボル抽出

```rust
fn extract_symbols_from_file(uri: &Url, content: &str, query: &str) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let query_lower = query.to_lowercase();
    
    for (line_number, line) in content.lines().enumerate() {
        // 関数定義の検出
        if let Some(func_name) = extract_function_name(line) {
            if func_name.to_lowercase().contains(&query_lower) {
                symbols.push(create_symbol_info(
                    func_name,
                    SymbolKind::FUNCTION,
                    uri,
                    line_number,
                    line
                ));
            }
        }
        
        // 構造体定義の検出
        if let Some(struct_name) = extract_struct_name(line) {
            if struct_name.to_lowercase().contains(&query_lower) {
                symbols.push(create_symbol_info(
                    struct_name,
                    SymbolKind::STRUCT,
                    uri,
                    line_number,
                    line
                ));
            }
        }
    }
    
    symbols
}
```

## シンボル名の抽出方法

### 関数名の抽出

```rust
fn extract_function_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("fn ") {
        return None;
    }
    
    // "fn " の後から関数名を抽出
    let after_fn = trimmed.strip_prefix("fn ")?;
    
    // 関数名は `(` まで
    let end_pos = after_fn.find('(')?;
    let func_name = after_fn[..end_pos].trim();
    
    // ジェネリクスがある場合（例: "my_func<T>"）
    let clean_name = if let Some(generic_pos) = func_name.find('<') {
        &func_name[..generic_pos]
    } else {
        func_name
    };
    
    Some(clean_name.to_string())
}
```

### 構造体名の抽出

```rust
fn extract_struct_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("struct ") {
        return None;
    }
    
    // "struct " の後から構造体名を抽出
    let after_struct = trimmed.strip_prefix("struct ")?;
    
    // 構造体名は空白、`{`、`<` まで
    let mut end_pos = after_struct.len();
    for (i, ch) in after_struct.char_indices() {
        if ch.is_whitespace() || ch == '{' || ch == '<' {
            end_pos = i;
            break;
        }
    }
    
    let struct_name = &after_struct[..end_pos];
    Some(struct_name.to_string())
}
```

## 位置情報の作成

```rust
fn create_symbol_info(
    name: String,
    kind: SymbolKind,
    uri: &Url,
    line_number: usize,
    line_content: &str
) -> SymbolInformation {
    let start_pos = Position::new(line_number as u32, 0);
    let end_pos = Position::new(line_number as u32, line_content.len() as u32);
    
    SymbolInformation {
        name,
        kind,
        tags: None,
        deprecated: None,
        location: Location::new(
            uri.clone(),
            Range::new(start_pos, end_pos)
        ),
        container_name: None,
    }
}
```

## 実装のポイント

### 1. 大文字小文字を区別しない検索

```rust
// クエリとシンボル名の両方を小文字に変換
let query_lower = query.to_lowercase();
let symbol_lower = symbol_name.to_lowercase();
if symbol_lower.contains(&query_lower) {
    // マッチ
}
```

### 2. 部分マッチのサポート

```rust
// "calc" で "calculate" や "Calculator" にマッチ
if symbol_name.to_lowercase().contains(&query.to_lowercase()) {
    // 部分マッチでも結果に含める
}
```

### 3. パフォーマンスの考慮

```rust
// 空クエリの早期リターン
if query.trim().is_empty() {
    return Vec::new();
}

// 結果数の制限（大きなプロジェクト対応）
const MAX_RESULTS: usize = 100;
if results.len() >= MAX_RESULTS {
    break;
}
```

### 4. エラーハンドリング

```rust
// ファイル解析エラーの無視（他のファイルは続行）
for (uri, content) in workspace {
    if let Ok(symbols) = extract_symbols_safely(uri, content, query) {
        results.extend(symbols);
    }
    // エラーがあっても他のファイルの処理は続行
}
```

## やってみよう！

あなたの今回のミッションは、`workspace_symbol` 関数を完成させることです。

1. クエリが空の場合は、空の `Vec<SymbolInformation>` を返します。
2. `document_store` 内の全ファイルを反復処理します。
3. 各ファイルで以下を検索します：
    * `"fn "` で始まる行から関数名を抽出
    * `"struct "` で始まる行から構造体名を抽出
4. 抽出したシンボル名がクエリを含むかチェックします（大文字小文字を区別しない）。
5. マッチするシンボルに対して `SymbolInformation` オブジェクトを作成します：
    * `name`: シンボル名
    * `kind`: `SymbolKind::FUNCTION` または `SymbolKind::STRUCT`
    * `location`: ファイルURIと行の範囲を含む `Location`
6. 見つかったすべてのシンボルを `Vec<SymbolInformation>` として返します。

`src/lessons/lesson_1_30.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！