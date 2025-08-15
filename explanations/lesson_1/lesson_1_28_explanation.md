# Lesson 1-28: 入力をお手伝いしよう！ (LSPコンプリーション)

LSPのインレイヒント機能ができるようになりましたね。素晴らしいです！

次に、LSPの最も使用頻度が高く、開発者体験を大きく向上させる「**コンプリーション（Completion/オートコンプリート）**」機能について学びます。これはrust-analyzerの中核機能の一つで、開発者がコードを効率的に書くための必須機能です。

## コンプリーション (Completion) とは？

コンプリーション機能は、ユーザーが入力している途中の文字から、考えられる候補を自動的に提案する機能です。例えば：

```rust
fn main() {
    l|    // ← カーソル位置で "l" と入力すると...
}

// 候補として表示される：
// - let    (キーワード)
// - loop   (キーワード)
```

これにより、開発者は：
- **入力速度の向上**: タイピング量の削減
- **スペルミスの防止**: 正確なキーワード・関数名の入力
- **APIの発見**: 利用可能な関数やメソッドの学習

エディタは、ユーザーが入力を開始すると、言語サーバーに `textDocument/completion` リクエストを送ります。サーバーはこれに対して、候補のリスト（`CompletionItem` オブジェクトの配列）を返します。

## `lsp-types::CompletionItem` の構造

LSPのコンプリーション候補は `lsp_types::CompletionItem` という構造体で表現されます。主なフィールドは以下の通りです。

*   `label`: 候補として表示されるテキスト（例: `"let"`, `"fn"`）
*   `kind`: 候補の種類。`CompletionItemKind` Enum（`Keyword`, `Function`, `Variable`, `Type`など）を使います
*   `insert_text`: 実際に挿入されるテキスト（省略可、通常は`label`と同じ）
*   `detail`: 候補の詳細情報（例: `"Rust keyword"`）
*   `documentation`: 候補の説明文やドキュメント

## 部分的な単語の抽出方法

コンプリーション機能を実装するには、まずカーソル位置で「入力中の部分的な単語」を特定する必要があります。

### 部分単語抽出のアルゴリズム

```rust
// 例: "fn main() {\n    l" でカーソルが "l" の後にある場合
let line = "    l";
let cursor_pos = 5; // "l" の後

// 1. カーソル位置より前の文字列を取得
let before_cursor = &line[..cursor_pos]; // "    l"

// 2. 後ろから単語文字（英数字、アンダースコア）を辿る
let mut start = cursor_pos;
while start > 0 {
    let prev_char = before_cursor.chars().nth(start - 1).unwrap();
    if prev_char.is_alphanumeric() || prev_char == '_' {
        start -= 1;
    } else {
        break;
    }
}

// 3. 部分単語を抽出
let partial_word = &before_cursor[start..]; // "l"
```

### Rustでの実装例

```rust
fn extract_partial_word(line: &str, cursor_pos: usize) -> Option<String> {
    if cursor_pos > line.len() {
        return None;
    }
    
    let before_cursor = &line[..cursor_pos];
    let mut start = cursor_pos;
    
    // 後ろから英数字・アンダースコアを辿る
    for (i, ch) in before_cursor.char_indices().rev() {
        if ch.is_alphanumeric() || ch == '_' {
            start = i;
        } else {
            break;
        }
    }
    
    if start < cursor_pos {
        Some(before_cursor[start..].to_string())
    } else {
        None // 部分単語が見つからない
    }
}
```

## Rustキーワードとタイプの候補生成

今回のレッスンでは、以下のルールでRust候補を生成します：

### キーワード候補

```rust
fn generate_keyword_completions(partial: &str) -> Vec<CompletionItem> {
    let keywords = match partial.to_lowercase().as_str() {
        prefix if prefix.starts_with("l") => vec!["let", "loop"],
        prefix if prefix.starts_with("f") => vec!["fn", "for", "false"],
        prefix if prefix.starts_with("i") => vec!["if", "impl"],
        prefix if prefix.starts_with("t") => vec!["true", "type"],
        prefix if prefix.starts_with("s") => vec!["struct"],
        _ => vec![]
    };
    
    keywords.into_iter().map(|keyword| CompletionItem {
        label: keyword.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        insert_text: Some(keyword.to_string()),
        ..Default::default()
    }).collect()
}
```

### 型候補

```rust
fn generate_type_completions(partial: &str) -> Vec<CompletionItem> {
    let types = match partial.to_lowercase().as_str() {
        prefix if prefix.starts_with("i") => vec!["i32", "i64", "isize"],
        prefix if prefix.starts_with("s") => vec!["String", "str"],
        prefix if prefix.starts_with("u") => vec!["u32", "u64", "usize"],
        prefix if prefix.starts_with("b") => vec!["bool"],
        _ => vec![]
    };
    
    types.into_iter().map(|type_name| CompletionItem {
        label: type_name.to_string(),
        kind: Some(CompletionItemKind::TYPE_PARAMETER),
        insert_text: Some(type_name.to_string()),
        ..Default::default()
    }).collect()
}
```

## 実装のポイント

### 1. 大文字小文字の処理
```rust
// 大文字小文字を区別しない比較
let partial_lower = partial.to_lowercase();
if candidate.to_lowercase().starts_with(&partial_lower) {
    // マッチする候補
}
```

### 2. エラーハンドリング
```rust
// ドキュメントが見つからない場合
let content = document_store.get(file_uri)?;

// 位置が範囲外の場合
let lines: Vec<&str> = content.lines().collect();
if position.line as usize >= lines.len() {
    return Vec::new();
}
```

### 3. 効率的な候補フィルタリング
```rust
// 部分文字列マッチング
let candidates = ALL_KEYWORDS;
let matching: Vec<_> = candidates
    .iter()
    .filter(|&candidate| candidate.starts_with(partial))
    .map(|&candidate| create_completion_item(candidate))
    .collect();
```

## やってみよう！

あなたの今回のミッションは、`get_completion_items` 関数を完成させることです。

1.  `document_store` から `file_uri` に対応するファイルの内容を取得します。見つからなければ空の `Vec<CompletionItem>` を返します。
2.  `position` から該当する行を取得し、カーソル位置より前の部分文字列を抽出します。
3.  後ろから英数字・アンダースコアを辿って、入力中の部分単語を特定します。
4.  部分単語の最初の文字に基づいて、マッチするRustキーワードと型を検索します：
    *   `"l"` → `"let"`, `"loop"`
    *   `"f"` → `"fn"`, `"for"`, `"false"`
    *   `"i"` → `"i32"`, `"if"`, `"impl"`
    *   `"s"` → `"struct"`, `"String"`, `"str"`
    *   `"t"` → `"true"`, `"type"`
5.  各候補に対して `CompletionItem` オブジェクトを作成します：
    *   `label`: 候補のテキスト
    *   `kind`: `CompletionItemKind::KEYWORD` または `CompletionItemKind::TYPE_PARAMETER`
    *   `insert_text`: 挿入されるテキスト（省略可）
6.  見つかったすべての `CompletionItem` を `Vec<CompletionItem>` として返します。

`src/lessons/lesson_1_28.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！