# rust-analyzer アーキテクチャ完全ガイド

このガイドでは、rust-analyzerの公式アーキテクチャドキュメント（https://rust-analyzer.github.io/book/contributing/architecture.html）の内容を分かりやすく解説します。

## 🎯 rust-analyzerとは何か

rust-analyzerは**Rustの意味解析エンジン**です。ソースコードを受け取り、「すべての式に型がある、すべての参照が宣言にバインドされている」構造化された意味モデルを作成します。

```
Input: Rust Source Code
    ↓
rust-analyzer Processing
    ↓
Output: Structured Semantic Model
```

## 🏗️ 核となる設計哲学

### 1. **Incremental & On-Demand Computing**（増分・オンデマンド計算）

**なぜ重要？**

- 大規模なRustプロジェクトでも高速に動作
- ファイル1つを変更しても、全体を再解析する必要がない

**実装方法**：

- **Salsa Framework** を使用
- 依存関係を追跡し、変更があった部分のみ再計算
- 計算結果をメモ化（キャッシュ）

**具体例**：

```rust
// mod を編集
fn main() {
    println!("Hello, world!"); // この行を変更
}

// lib.rs は変更なし
pub fn calculate(x: i32) -> i32 {
    x * 2
}
```

→ rust-analyzerは `main.rs` の解析のみを再実行し、`lib.rs` はキャッシュされた結果を使用

### 2. **Resilient Design**（堅牢な設計）

**壊れたコードでも動作する**：

```rust
// こんなコードでも可能な限り解析を続行
fn broken_function( {  // 構文エラー
    let x = 42;
    x.  // 不完全な式
}

pub fn working_function() -> i32 {
    42  // この関数は正常に解析される
}
```

**エラーハンドリングの原則**：

- `Result<T, Error>` ではなく `(T, Vec<Error>)` を返す
- 一部のエラーで全体が止まらない設計
- パニックを避け、可能な限り結果を返す

### 3. **Memory-First Architecture**（メモリファースト設計）

**すべてのデータをメモリに保持**：

- ディスクI/Oを最小化
- 高速アクセスを実現
- ただし、メモリ使用量は多くなる

## 📦 クレート構成と責任分担

rust-analyzerは**30以上のクレート**で構成されています。主要な構成を理解しましょう：

### 🔤 構文解析レイヤー

#### `parser` クレート

**役割**: Rustコードをトークンに分解し、構文木を構築

```rust
// Input
"fn main() { println!(\"hello\"); }"

// Output (simplified)
FN_KW IDENT L_PAREN R_PAREN L_CURLY MACRO_CALL R_CURLY
```

#### `syntax` クレート

**役割**: CST（Concrete Syntax Tree）の表現と操作

- **Red-Green Tree**（rowan library）を使用
- 構文エラーがあっても完全な構文木を構築
- テキストの完全な再現性（whitespace、コメントも保持）

**重要な概念**：

- **SyntaxNode**: 構文木のノード
- **SyntaxToken**: トークン（キーワード、識別子など）
- **TextRange**: ソースコード内の位置情報

```rust
// AST例
fn main() {
    //└─ FN_DEF
    println!("hello");
    //  └─ EXPR_STMT
    //      └─ MACRO_CALL
}
```

### 🧠 意味解析レイヤー（HIR: High-level IR）

#### `hir-expand` クレート

**役割**: マクロの展開処理

```rust
// Before macro expansion
println!("Hello");

// After macro expansion
std::io::_print(std::fmt::Arguments::new_v1(&["Hello\n"], &[]));
```

#### `hir-def` クレート

**役割**: 名前解決とスコープ管理

- モジュールシステムの解決
- 識別子が何を指しているかの決定
- use文の処理

```rust
use std::collections::HashMap;
//   ↑ この `HashMap` が `std::collections::HashMap` であることを決定
```

#### `hir-ty` クレート

**役割**: 型推論とtype checking

- 式の型を推論
- trait実装の解決
- 型エラーの検出

```rust
let x = 42;        // i32と推論
let y = x + 1.0;   // 型エラーを検出
```

#### `hir` クレート

**役割**: 高レベルなオブジェクト指向API

- IDE機能で使いやすいAPIを提供
- 内部実装の複雑さを隠蔽

### 🎨 IDE機能レイヤー

#### `ide-db` クレート

**役割**: IDE機能の共通基盤

- 検索機能
- リファクタリング用のユーティリティ
- text edit operations

#### `ide-assists` クレート

**役割**: コードアクション（リファクタリング）

```rust
// Before assist
if let Some(x) = option {
    x
} else {
    return;
}

// After "convert if-let to match" assist
match option {
    Some(x) => x,
    None => return,
}
```

#### `ide-completion` クレート

**役割**: コード補完

- コンテキストに応じた候補の提供
- 型に基づく補完
- モジュール内の項目補完

#### `ide-diagnostics` クレート

**役割**: エラーと警告の生成

- コンパイルエラーの検出
- lint警告
- 型エラーのユーザーフレンドリーな表示

#### `ide` クレート

**役割**: 統合されたIDE機能の提供

- 各機能を組み合わせ
- LSPに対する統一されたAPI

### 🌐 LSPレイヤー

#### `rust-analyzer` クレート

**役割**: Language Server Protocol の実装

- エディタとの通信
- リクエストのルーティング
- プロトコル変換

## 🔄 データフローの理解

rust-analyzerでのデータ処理の流れ：

```
1. Source Code (text files)
   ↓
2. VFS (Virtual File System) - ファイル変更の追跡
   ↓
3. parser → syntax - 構文解析
   ↓
4. hir-expand - マクロ展開
   ↓
5. hir-def - 名前解決
   ↓
6. hir-ty - 型推論
   ↓
7. hir - 高レベルAPI
   ↓
8. ide-* - IDE機能（completion, diagnostics, assists）
   ↓
9. rust-analyzer - LSP応答
   ↓
10. Editor (VS Code, vim, etc.)
```

## 🚀 Salsa Framework の仕組み

### Query-Based Architecture

rust-analyzerの核となるのは**Query System**です：

```rust
// 例：ファイルの構文木を取得するクエリ
#[salsa::query]
fn parse_query(db: &impl Database, file_id: FileId) -> Parse<SourceFile> {
    // ファイルの内容をパースして構文木を返す
}
```

### 依存関係の追跡

```
FileText → ParseQuery → HIRQuery → TypeQuery → DiagnosticsQuery
```

ファイルが変更されると：

1. `FileText` が無効化される
2. 依存する `ParseQuery` も無効化される
3. さらに依存する `HIRQuery`, `TypeQuery` も無効化される
4. 次回アクセス時に再計算される

### メモ化の利点

```rust
// 最初の呼び出し
let parse_result = db.parse_query(file_id); // 計算実行

// 二回目の呼び出し（ファイルが変更されていない場合）
let parse_result = db.parse_query(file_id); // キャッシュから返す（高速）
```

## 🛠️ 開発者向けのポイント

### 1. **Snapshot-based Testing**

- テストは「入力」→「期待される出力」の形式
- エディタでの実際の操作をシミュレート

### 2. **エラー処理のパターン**

```rust
// Good: 部分的な結果でも返す
fn analyze_function(code: &str) -> (Option<Function>, Vec<Error>) {
    // 可能な限り解析を続行
}

// Avoid: エラーで全体を諦める
fn analyze_function(code: &str) -> Result<Function, Error> {
    // 小さなエラーで全体が失敗
}
```

### 3. **性能重視の設計**

- 最初から最適化を考慮
- プロファイリング機能が充実
- メモリ使用量の監視

## 🎯 貢献時に知っておくべきこと

### クレート選択の指針

- **構文に関する変更**: `syntax`, `parser`
- **意味解析の改善**: `hir-*`
- **IDE機能の追加**: `ide-*`
- **LSP拡張**: `rust-analyzer`

### テスト戦略

- 各クレートは独立してテスト可能
- 統合テストは `ide` レベルで実行
- 性能テストも重要視される

### 設計原則の遵守

- 増分計算を前提とした設計
- エラー時の部分的な結果返却
- メモリ効率の考慮

## 📚 さらなる学習リソース

1. **Salsa Book**: https://salsa-rs.github.io/salsa/
2. **rust-analyzer dev docs**: https://rust-analyzer.github.io/book/
3. **Rowan (red-green tree)**: https://github.com/rust-analyzer/rowan
4. **Language Server Protocol**: https://microsoft.github.io/language-server-protocol/

---

このアーキテクチャ理解により、rust-analyzerの各部分がどう協調して動作するかが把握できるはずです。lesson_4で学んだAST操作、診断システム、リファクタリング機能は、すべてこの大きな枠組みの中で動作していることがわかります。