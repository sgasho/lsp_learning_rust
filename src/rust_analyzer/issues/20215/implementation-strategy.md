# Issue #20215 修正の進め方（初心者向け）

## まず理解すること

### 今のrust-analyzerのコード変換の仕組み
rust-analyzerは「PathTransform」という仕組みを使って、コード生成時に名前を正しいパスに変換しています。

```rust
// 変換前（元のトレイトのデフォルト実装）
match bound {
    Unbounded => true,      // 短い名前
    Included(x) => false,   // 短い名前
}

// 変換後（コピー先で使える形）
match bound {
    std::ops::Bound::Unbounded => true,      // フルパス
    std::ops::Bound::Included(x) => false,   // フルパス
}
```

### 何が問題なの？
現在のPathTransformは**「パス形式の名前」**しか変換できません：

- ✅ `Included(x)` → これは「関数呼び出し」っぽいのでパスとして認識される
- ❌ `Unbounded` → これは「単体の名前」なのでパスとして認識されない

## どこのコードを読めばいいの？

### 1. メインファイル：`crates/ide-db/src/path_transform.rs`
```bash
# このファイルを開いて読んでみよう
code crates/ide-db/src/path_transform.rs
```

### 2. 重要な関数たち：
- **`apply`メソッド（133行目）**: 外部から呼ばれるエントリーポイント
- **`transform_path`メソッド（317行目）**: パスを変換する処理
- **`transform_path_`メソッド（345行目）**: 実際の変換ロジック

### 3. 問題の箇所：`find_child_paths`関数（318行目）
```rust
fn find_child_paths(root_path: &SyntaxNode) -> Vec<ast::Path> {
    let mut result = Vec::new();
    for child in root_path.children() {
        if let Some(child_path) = ast::Path::cast(child.clone()) {
            result.push(child_path);  // ← ast::Path だけを収集
        } else {
            result.extend(find_child_paths(&child));
        }
    }
    result
}
```

**問題**: `ast::IdentPat`（パターン内の単体名前）を探していない！

## 修正の手順（ステップバイステップ）

### ステップ1: 現在のコードの流れを理解する
```rust
// apply メソッド（133行目）
pub fn apply(&self, syntax: &SyntaxNode) -> SyntaxNode {
    self.build_ctx().apply(syntax)  // Ctx::apply を呼び出し
}

// Ctx::apply メソッド（277行目）
fn apply(&self, item: &SyntaxNode) -> SyntaxNode {
    let item = self.transform_path(item).clone_subtree();  // パスを変換
    // ライフタイムも変換...
}

// transform_path メソッド（317行目）
fn transform_path(&self, path: &SyntaxNode) -> SyntaxNode {
    let result = find_child_paths(&root_path);  // ast::Path だけを探す
    // ast::Path を変換...
}
```

### ステップ2: 問題を特定する
`find_child_paths`関数は`ast::Path`だけを探しているが、`Unbounded`は`ast::IdentPat`なので見つからない。

### ステップ3: 修正アプローチ
`Ctx::apply`メソッドに`ast::IdentPat`の処理も追加する：

```rust
fn apply(&self, item: &SyntaxNode) -> SyntaxNode {
    let item = self.transform_path(item).clone_subtree();
    let mut editor = SyntaxEditor::new(item.clone());
    
    // 既存のライフタイム変換...
    preorder_rev(&item).filter_map(ast::Lifetime::cast).for_each(|lifetime| {
        // ...
    });
    
    // 新規追加：IdentPat の変換
    preorder_rev(&item).filter_map(ast::IdentPat::cast).for_each(|ident_pat| {
        if let Some(new_path) = self.transform_ident_pat(&ident_pat) {
            editor.replace(ident_pat.syntax(), new_path.syntax());
        }
    });

    editor.finish().new_root().clone()
}
```

### ステップ4: `transform_ident_pat` メソッドを実装
```rust
fn transform_ident_pat(&self, ident_pat: &ast::IdentPat) -> Option<ast::Path> {
    let name = ident_pat.name()?;
    
    // 名前からパスを作成（例: "Unbounded" -> ast::Path）
    let path = make::path_from_text(&name.text());
    
    // 既存の transform_path_ ロジックを再利用
    let mut editor = SyntaxEditor::new(path.syntax().clone());
    self.transform_path_(&mut editor, &path);
    
    // 変換されたパスを返す
    ast::Path::cast(editor.finish().new_root())
}
```

## 実際の作業の進め方

### 1. コードを読む
```bash
# rust-analyzerのリポジトリで
cd crates/ide-db/src/
less path_transform.rs  # ファイルを読む
```

### 2. 関連する型を理解する
```rust
// これらの型の違いを理解しよう
ast::Path      // パス形式：std::ops::Bound::Included
ast::IdentPat  // 単体名前：Unbounded
ast::PathPat   // パスのパターン：std::ops::Bound::Unbounded
```

### 3. 小さなテストから始める
```rust
#[test]
fn test_ident_pat_transform() {
    // 簡単なケースでテスト
    let input = "match x { Unbounded => true }";
    // 変換処理を実行
    let output = transform(input);
    // 期待される結果と比較
    assert_eq!(output, "match x { std::ops::Bound::Unbounded => true }");
}
```

## 困ったときは

### 1. 似たようなコードを探す
```bash
# "ast::IdentPat" を使っている他の場所を探す
rg "IdentPat" crates/
```

### 2. 既存のテストを見る
```bash
# path_transform のテストファイルを探す
find . -name "*test*" -exec grep -l "PathTransform" {} \;
```

### 3. rust-analyzerのドキュメントを読む
- [rust-analyzer guide](https://github.com/rust-lang/rust-analyzer/tree/master/docs)
- [AST nodes documentation](https://docs.rs/syntax/)

## 修正のゴール

**Before（現在の問題）:**
```rust
match bound {
    Unbounded => true,  // エラー：Unboundedが見つからない
}
```

**After（修正後）:**
```rust
match bound {
    std::ops::Bound::Unbounded => true,  // 正常動作
}
```

これで生成されたコードがコンパイルエラーにならなくなります！

## 最初の一歩

1. `crates/ide-db/src/path_transform.rs` を開く
2. `apply_to` メソッドを見つける
3. 「ast::Path を処理している部分」を理解する
4. 「ast::IdentPat も同じように処理する」コードを追加する

慣れてきたら、より詳細な実装に進んでいけます！