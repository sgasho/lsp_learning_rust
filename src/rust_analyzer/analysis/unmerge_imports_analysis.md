# 🔧 unmerge_imports.rs 完全解析

rust-analyzer の `unmerge_imports` アシスト機能の実装を徹底的に解析し、コードの動作を詳しく解説します。

## 📚 概要

### 🎯 アシストの目的
**機能**: マージされたuse文から特定の項目を分離して独立したuse文にする

**変換例**:
```rust
// 変換前
use std::fmt::{Debug, Display};
//                     ↑ カーソル位置

// 変換後  
use std::fmt::{Debug};
use std::fmt::Display;
```

### 📁 ファイル構造
```
unmerge_imports.rs
├── imports（必要な依存関係）
├── unmerge_imports()（メイン関数）
├── resolve_full_path()（ヘルパー関数）
└── tests（テストモジュール）
```

## 🔍 imports（依存関係）の詳細解析

### syntax クレートからのインポート
```rust
use syntax::{
    AstNode, SyntaxKind,                    // AST操作の基本型
    ast::{
        self, HasAttrs, HasVisibility,      // AST特性
        edit::IndentLevel,                  // インデント処理
        edit_in_place::AttrsOwnerEdit,      // 属性編集
        make,                               // AST構築
        syntax_factory::SyntaxFactory,      // 構文ファクトリ
    },
    syntax_editor::{Element, Position, Removable}, // 構文編集
};
```

**各インポートの役割**:
- `AstNode`: AST操作の基本インターフェース
- `SyntaxKind`: 構文ノードの種類識別
- `HasAttrs`: 属性（`#[...]`）を持つノードの特性
- `HasVisibility`: 可視性（`pub`等）を持つノードの特性
- `IndentLevel`: コードのインデント処理
- `SyntaxFactory`: 新しいAST要素の生成

### crate 内部からのインポート
```rust
use crate::{
    AssistId,                               // アシスト識別子
    assist_context::{AssistContext, Assists}, // アシストコンテキスト
};
```

## 🏗️ メイン関数 `unmerge_imports()` の詳細解析

### 関数シグネチャ
```rust
pub(crate) fn unmerge_imports(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()>
```

**パラメータ解説**:
- `acc: &mut Assists`: アシスト一覧への可変参照（結果を追加する容器）
- `ctx: &AssistContext<'_>`: アシストコンテキスト（カーソル位置、AST等の情報）
- 戻り値: `Option<()>` アシスト適用可能なら`Some(())`、不可能なら`None`

### Step 1: カーソル位置のUseTreeを取得
```rust
let tree = ctx.find_node_at_offset::<ast::UseTree>()?;
```

**詳細動作**:
1. `ctx.find_node_at_offset::<ast::UseTree>()`: カーソル位置で`UseTree`ノードを検索
2. `UseTree`とは: use文内の個別項目（例: `Debug`, `Display`）
3. `?`演算子: 見つからない場合は`None`を返して関数終了

**AST構造例**:
```rust
use std::fmt::{Debug, Display};
//             ^^^^^  ^^^^^^^
//             UseTree UseTree
```

### Step 2: 親のUseTreeListを検証
```rust
let tree_list = tree.syntax().parent().and_then(ast::UseTreeList::cast)?;
if tree_list.use_trees().count() < 2 {
    cov_mark::hit!(skip_single_import);
    return None;
}
```

**詳細動作**:
1. `tree.syntax().parent()`: UseTreeの親ノードを取得
2. `ast::UseTreeList::cast`: 親が`UseTreeList`（`{...}`）かチェック
3. `use_trees().count() < 2`: リスト内の項目が2未満なら処理不要
4. `cov_mark::hit!()`: テストカバレッジマーク（テスト用）

**UseTreeListの構造**:
```rust
use std::fmt::{Debug, Display};
//           ^^^^^^^^^^^^^^^^
//           UseTreeList
//             ^^^^^  ^^^^^^^
//             UseTree UseTree
```

### Step 3: 祖先のUse文を取得
```rust
let use_ = tree_list.syntax().ancestors().find_map(ast::Use::cast)?;
```

**詳細動作**:
1. `tree_list.syntax().ancestors()`: UseTreeListから上位のノードを順次取得
2. `find_map(ast::Use::cast)`: 最初に見つかった`Use`ノード（use文全体）を取得

**AST階層構造**:
```
Use (use文全体)
└── UseTree
    └── UseTreeList {...}
        ├── UseTree (Debug)
        └── UseTree (Display)
```

### Step 4: フルパスの解決
```rust
let path = resolve_full_path(&tree)?;
```

**役割**: ネストしたuse文から完全なパスを構築
**例**: `foo::bar::{baz::qux}` → `foo::bar::baz::qux`

### Step 5: ユーザー向けラベルの生成
```rust
let label = match tree.path().and_then(|path| path.first_segment()) {
    Some(name) => format!("Unmerge use of `{name}`"),
    None => "Unmerge use".into(),
};
```

**詳細動作**:
1. `tree.path()`: UseTreeのパス部分を取得
2. `first_segment()`: パスの最初の部分（関数名等）を取得
3. ラベル生成: `"Unmerge use of `Display`"` のような説明文

### Step 6: アシストの登録と実行
```rust
let target = tree.syntax().text_range();
acc.add(AssistId::refactor_rewrite("unmerge_imports"), label, target, |builder| {
    // 実際の変換処理
})
```

**パラメータ解説**:
- `AssistId::refactor_rewrite("unmerge_imports")`: アシストID（リファクタリング種別）
- `label`: エディタに表示される説明文
- `target`: ハイライト範囲（UseTreeの範囲）
- `|builder|`: 実際の変換処理を行うクロージャ

## 🔧 変換処理の詳細（builderクロージャ内）

### Step 1: SyntaxFactoryの初期化
```rust
let make = SyntaxFactory::with_mappings();
```

**役割**: 新しいAST要素を生成するためのファクトリパターン
**mappings**: 元のノードと新しいノードの対応関係を記録

### Step 2: 新しいuse文の生成
```rust
let new_use = make.use_(
    use_.visibility(),                              // 可視性（pub等）
    make.use_tree(path, tree.use_tree_list(), tree.rename(), tree.star_token().is_some()),
);
```

**詳細パラメータ**:
1. `use_.visibility()`: 元のuse文の可視性（`pub`、`pub(crate)`等）を継承
2. `path`: `resolve_full_path()`で解決された完全パス
3. `tree.use_tree_list()`: ネストした子要素（存在する場合）
4. `tree.rename()`: `as`によるリネーム（存在する場合）
5. `tree.star_token().is_some()`: ワイルドカードインポート（`*`）かどうか

**生成例**:
```rust
// 元: use std::fmt::{Debug, Display};
// 生成: use std::fmt::Display;
```

### Step 3: 属性のコピー
```rust
use_.attrs().for_each(|attr| {
    new_use.add_attr(attr.clone_for_update());
});
```

**役割**: 元のuse文に付いている属性（`#[allow(...)]`等）を新しいuse文にもコピー

**例**:
```rust
// 元
#[allow(deprecated)]
use foo::{bar, baz};

// 生成後、両方に属性が付く
#[allow(deprecated)]
use foo::{bar};
#[allow(deprecated)]
use foo::baz;
```

### Step 4: 構文エディターの初期化
```rust
let mut editor = builder.make_editor(use_.syntax());
```

**役割**: use文を編集するためのエディターオブジェクトを作成

### Step 5: 元のUseTreeを削除
```rust
tree.remove(&mut editor);
```

**動作**: UseTreeList内から対象のUseTreeを削除
**結果**: `{Debug, Display}` → `{Debug}`

### Step 6: 新しいuse文の挿入
```rust
editor.insert_all(
    Position::after(use_.syntax()),
    vec![
        make.whitespace(&format!("\n{}", IndentLevel::from_node(use_.syntax())))
            .syntax_element(),
        new_use.syntax().syntax_element(),
    ],
);
```

**詳細動作**:
1. `Position::after(use_.syntax())`: 元のuse文の直後に挿入
2. `make.whitespace(...)`: 改行とインデントを生成
3. `IndentLevel::from_node(...)`: 元のuse文のインデントレベルを継承
4. `new_use.syntax().syntax_element()`: 新しいuse文を要素として追加

**挿入例**:
```rust
use std::fmt::{Debug};
use std::fmt::Display;  // ← ここに挿入される
```

### Step 7: マッピングの確定とファイル編集の登録
```rust
editor.add_mappings(make.finish_with_mappings());
builder.add_file_edits(ctx.vfs_file_id(), editor);
```

**動作**:
1. `make.finish_with_mappings()`: ファクトリで作成された要素のマッピングを取得
2. `editor.add_mappings()`: エディターにマッピング情報を登録
3. `builder.add_file_edits()`: 最終的な編集をbuilderに登録

## 🛠️ ヘルパー関数 `resolve_full_path()` の詳細解析

### 関数の目的
ネストしたuse文から完全なパスを解決する

### 実装の詳細
```rust
fn resolve_full_path(tree: &ast::UseTree) -> Option<ast::Path> {
    let paths = tree
        .syntax()
        .ancestors()                                    // 祖先ノードを取得
        .take_while(|n| n.kind() != SyntaxKind::USE)   // Use文まで（含まない）
        .filter_map(ast::UseTree::cast)                 // UseTreeのみ抽出
        .filter_map(|t| t.path());                      // パス部分のみ抽出
```

**詳細動作**:
1. `ancestors()`: tree から上位ノードを順次取得
2. `take_while(...)`: `USE`ノードに到達するまで続行
3. `filter_map(ast::UseTree::cast)`: UseTreeノードのみを抽出
4. `filter_map(|t| t.path())`: 各UseTreeからパス部分を抽出

### パスの連結
```rust
let final_path = paths.reduce(|prev, next| make::path_concat(next, prev))?;
```

**動作**: 複数のパス部分を連結して完全なパスを構築
**例**: `foo` + `bar` + `baz` → `foo::bar::baz`

### self キーワードの処理
```rust
if final_path.segment().is_some_and(|it| it.self_token().is_some()) {
    final_path.qualifier()
} else {
    Some(final_path)
}
```

**目的**: `self`キーワードの特別処理
**例**: `std::process::{Command, self}` の`self`を`std::process`に変換

## 🧪 テストケースの詳細分析

### 1. 単一要素の場合（処理対象外）
```rust
#[test]
fn skip_single_import() {
    check_assist_not_applicable(
        unmerge_imports,
        r"use std::fmt::Debug$0;",  // ← 単独のuse文
    );
}
```

**理由**: 分離する対象がないため処理不要

### 2. 基本的な分離処理
```rust
#[test]
fn unmerge_import() {
    check_assist(
        unmerge_imports,
        r"use std::fmt::{Debug, Display$0};",
        r"use std::fmt::{Debug};
use std::fmt::Display;",
    );
}
```

**検証内容**: 最も基本的な分離が正しく動作することを確認

### 3. ワイルドカードインポート
```rust
#[test]
fn unmerge_glob_import() {
    check_assist(
        unmerge_imports,
        r"use std::fmt::{*$0, Display};",
        r"use std::fmt::{Display};
use std::fmt::*;",
    );
}
```

**特徴**: `*`（ワイルドカード）も正しく分離される

### 4. リネームの処理
```rust
#[test]
fn unmerge_renamed_import() {
    check_assist(
        unmerge_imports,
        r"use std::fmt::{Debug, Display as Disp$0};",
        r"use std::fmt::{Debug};
use std::fmt::Display as Disp;",
    );
}
```

**検証**: `as`によるリネームが保持される

### 5. インデント処理
```rust
#[test]
fn unmerge_indented_import() {
    check_assist(
        unmerge_imports,
        r"mod format {
    use std::fmt::{Debug, Display$0 as Disp, format};
}",
        r"mod format {
    use std::fmt::{Debug, format};
    use std::fmt::Display as Disp;
}",
    );
}
```

**検証**: ネストしたモジュール内でも正しいインデントが適用される

### 6. ネストした構造
```rust
#[test]
fn unmerge_nested_import() {
    check_assist(
        unmerge_imports,
        r"use foo::bar::{baz::{qux$0, foobar}, barbaz};",
        r"use foo::bar::{baz::{foobar}, barbaz};
use foo::bar::baz::qux;",
    );
}
```

**検証**: 深くネストした構造でも`resolve_full_path()`が正しく動作

### 7. 可視性の継承
```rust
#[test]
fn unmerge_import_with_visibility() {
    check_assist(
        unmerge_imports,
        r"pub use std::fmt::{Debug, Display$0};",
        r"pub use std::fmt::{Debug};
pub use std::fmt::Display;",
    );
}
```

**検証**: `pub`等の可視性修飾子が新しいuse文にも適用される

### 8. self キーワード
```rust
#[test]
fn unmerge_import_on_self() {
    check_assist(
        unmerge_imports,
        r"use std::process::{Command, self$0};",
        r"use std::process::{Command};
use std::process;",
    );
}
```

**検証**: `self`が正しくモジュール名に変換される

### 9. 属性の継承
```rust
#[test]
fn unmerge_import_with_attributes() {
    check_assist(
        unmerge_imports,
        r"#[allow(deprecated)]
use foo::{bar, baz$0};",
        r"#[allow(deprecated)]
use foo::{bar};
#[allow(deprecated)]
use foo::baz;",
    );
}
```

**検証**: 属性（`#[...]`）が両方のuse文に正しくコピーされる

## 🎯 エラーハンドリングとエッジケース

### 1. カバレッジマーク
```rust
cov_mark::hit!(skip_single_import);
```

**目的**: テストカバレッジを確実にするためのマーク
**動作**: 特定のコードパスが実行されたことを記録

### 2. Option型の活用
- 各ステップで`?`演算子を使用してearly returnを実現
- 処理不可能な場合は`None`を返して静かに失敗

### 3. 構文エラーの回避
- AST操作前に必要な要素の存在確認
- 型安全なAST操作によりランタイムエラーを防止

## 📊 パフォーマンス考慮

### 1. 遅延評価
- アシスト登録時は実際の変換処理を行わない
- ユーザーがアシストを実行した時点で初めて変換を実行

### 2. メモリ効率
- 不要なクローンを避けて参照を多用
- SyntaxFactoryのmappingsで効率的なノード管理

### 3. AST操作の最適化
- syntax_editorを使用した効率的な編集
- バッチ処理による複数変更の一括適用

## 🔗 まとめ

`unmerge_imports`アシストは以下の特徴を持つ：

1. **堅牢性**: 豊富なエラーハンドリングとエッジケース対応
2. **柔軟性**: 様々なuse文形式（ネスト、リネーム、属性等）に対応
3. **保守性**: テストケースによる動作保証
4. **効率性**: AST操作とメモリ使用の最適化

この実装は、rust-analyzerのアシスト機能の典型的な設計パターンを示しており、他のアシスト開発の良い参考例となっています。