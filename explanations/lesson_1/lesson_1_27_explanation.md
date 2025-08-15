# Lesson 1-27: コードにヒントを表示しよう！ (LSPインレイヒント)

LSPのハイライト機能ができるようになりましたね。素晴らしいです！

次に、rust-analyzerの最も便利で人気な機能の一つである「**インレイヒント (Inlay Hints)**」について学びます。この機能は、コード中に型情報やパラメータ名などのヒントを表示することで、コードの理解を大幅に向上させます。

## インレイヒント (Inlay Hints) とは？

インレイヒント機能は、エディタがコード中に追加の情報を薄い色で表示する機能です。例えば：

```rust
let x = 10;        // x: i32 ← このようなヒントが表示される
let name = "Rust"; // name: &str ← 型情報が自動で表示される
```

これにより、明示的に型注釈を書かなくても、変数の型が一目で分かるようになります。rust-analyzerでは以下のようなヒントを表示できます：

* **型ヒント**: 変数の推論された型
* **パラメータヒント**: 関数呼び出し時のパラメータ名
* **チェーンヒント**: メソッドチェーンの中間型

エディタは、インレイヒントが必要になると、言語サーバーに `textDocument/inlayHint` リクエストを送ります。サーバーはこれに対して、ヒント情報のリスト（`InlayHint` オブジェクトの配列）を返します。

## `lsp-types::InlayHint` の構造

LSPのインレイヒントは `lsp_types::InlayHint` という構造体で表現されます。主なフィールドは以下の通りです。

*   `position`: ヒントを表示する位置。`Position` を使います。
*   `label`: ヒントとして表示するテキスト。`InlayHintLabel` を使います。
*   `kind`: ヒントの種類。`InlayHintKind` Enum（`Type`, `Parameter`など）を使います。
*   `tooltip`: オプションのフィールドで、ヒントにマウスオーバーした時の説明。

## 変数宣言を解析して型ヒントを作成する

今回のレッスンでは、シンプルな型推論を実装して、`let` 文の変数に型ヒントを表示します。

### Rustでの型判定方法

Rustでは、変数の型は値から推論できます。今回は以下のルールで型を判定します：

#### 1. 数値リテラル (`i32` 型)
```rust
let x = 10;     // 10 は整数リテラル → i32 型
let y = 42;     // 42 も整数リテラル → i32 型
```
**判定方法**: 値が数字のみで構成されている場合（例: `"10"`, `"42"`）

#### 2. 文字列リテラル (`&str` 型)
```rust
let s = "hello";    // "hello" は文字列リテラル → &str 型
let name = "Rust";  // "Rust" も文字列リテラル → &str 型
```
**判定方法**: 値が `"` で始まり `"` で終わる場合（例: `"\"hello\""`）

#### 3. 真偽値リテラル (`bool` 型)
```rust
let flag = true;    // true は真偽値リテラル → bool 型
let done = false;   // false も真偽値リテラル → bool 型
```
**判定方法**: 値が `"true"` または `"false"` の場合

### 文字列解析のテクニック

Rustで `let x = 10;` のような行から情報を抽出する方法：

```rust
let line = "let x = 10;";

// 1. "let " で始まるかチェック
if line.starts_with("let ") {
    // 2. "let " を削除
    let after_let = line.strip_prefix("let ").unwrap(); // "x = 10;"
    
    // 3. " = " で分割して変数名と値を取得
    if let Some(eq_pos) = after_let.find(" = ") {
        let var_name = &after_let[..eq_pos];                    // "x"
        let after_eq = &after_let[eq_pos + 3..];                // "10;"
        
        // 4. セミコロンを削除して値を取得
        let value = after_eq.trim_end_matches(';');             // "10"
        
        // 5. 値の種類に基づいて型を判定
        let type_name = if value.chars().all(|c| c.is_numeric()) {
            "i32"                           // 数値の場合
        } else if value.starts_with('"') && value.ends_with('"') {
            "&str"                          // 文字列の場合
        } else if value == "true" || value == "false" {
            "bool"                          // 真偽値の場合
        } else {
            return None;                    // その他は対象外
        };
    }
}
```

### 実装手順
1. ドキュメントの内容を指定された範囲で1行ずつ処理
2. 各行が `"let "` で始まるかチェック
3. `" = "` で分割して変数名と値を抽出
4. 値の文字列パターンから型を判定
5. 変数名の直後の位置に型ヒントを表示する `InlayHint` を作成

### 位置の計算方法
```rust
// "let x = 10;" の場合
let line = "let x = 10;";
let var_name = "x";

// 変数名の直後の位置を計算
let var_start = 4;                          // "let " の長さ
let var_end = var_start + var_name.len();   // 4 + 1 = 5

// InlayHint の position は var_end (Position::new(行番号, 5))
```

**ヒント:**
*   `line.find()` や `line.split()` を使って文字列を解析します
*   `char.is_numeric()` で数値かどうか判定できます
*   位置計算では文字数を正確にカウントすることが重要です

## やってみよう！

あなたの今回のミッションは、`get_inlay_hints` 関数を完成させることです。

1.  `document_store` から `file_uri` に対応するファイルの内容を取得します。見つからなければ空の `Vec<InlayHint>` を返します。
2.  指定された `range` 内の行のみを処理します。
3.  各行で `let 変数名 = 値;` のパターンを検索します。
4.  値の種類に基づいて型を推論します：
    *   数値（例: `10`, `42`）→ `"i32"`
    *   文字列（例: `"hello"`）→ `"&str"`
    *   真偽値（`true`, `false`）→ `"bool"`
5.  各変数に対して `InlayHint` オブジェクトを作成します：
    *   `position`: 変数名の直後の位置
    *   `label`: `": 型名"` の形式（例: `": i32"`）
    *   `kind`: `InlayHintKind::TYPE`
6.  見つかったすべての `InlayHint` を `Vec<InlayHint>` として返します。

`src/lessons/lesson_1_27.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！