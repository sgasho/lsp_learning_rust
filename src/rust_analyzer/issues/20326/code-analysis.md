# 🔍 rust-analyzer コード解析

この機能を実装するために**どこのコードを見ればいいか**、**どんな仕組みを使えばいいか**を分かりやすく解説します。

## 🗺 全体マップ：どこに何があるか

```
🏠 rust-analyzer (巨大な家)
├── 🏢 crates/ide-assists/     👈 我々の作業場所
│   ├── 📁 src/handlers/       👈 ここに新機能を追加
│   │   ├── 📄 auto_import.rs  👈 use文の先輩機能 (参考になる!)
│   │   └── 📄 move_use_to_top.rs 👈 これを新しく作る
│   └── 📄 src/lib.rs         👈 新機能を登録する場所
└── 🏭 crates/ide-db/         👈 便利な道具箱
    └── 📁 src/imports/        👈 use文操作の魔法が詰まってる
```

## 📁 Assists の基本構造

### 🏗 ディレクトリ構成
```
rust-analyzer/
├── crates/
│   ├── ide-assists/
│   │   ├── src/
│   │   │   ├── handlers/         👈 アシスト実装の場所
│   │   │   │   ├── auto_import.rs
│   │   │   │   ├── extract_module.rs
│   │   │   │   └── move_use_to_top.rs  👈 新しく作るファイル
│   │   │   ├── utils.rs          👈 共通ユーティリティ
│   │   │   └── lib.rs            👈 アシスト登録
│   │   └── ide-db/
│   │       └── src/imports/      👈 import操作の核心部分
```

## 🧩 アシストの実装パターン詳細解説

### 📚 前提知識：基礎概念の定義

#### 🌳 AST（抽象構文木）とは
**AST（Abstract Syntax Tree）** = Rustコードを木構造で表現したもの

```
Rustコード                     AST（木構造）
─────────────                  ─────────────
fn test() {           →        FN
    let x = 1;        →        ├── BLOCK  
}                              │   └── LET_STMT
                               │       └── LITERAL(1)
```

#### 🧱 ノード（Node）とは
**ノード** = AST内の1つの要素。「関数」「変数」「文」などを表す

```
各ノードの例：
- ast::Fn     = 関数ノード      fn test() { ... }
- ast::Use    = use文ノード     use std::fmt;  
- ast::Let    = let文ノード     let x = 1;
```

#### 👨‍👩‍👧‍👦 親子関係とは
**親子関係** = ノード同士の階層関係

```
親ノード：関数         fn test() {
子ノード：use文            use std::fmt;    ← 関数の中にある
子ノード：let文            let x = 1;       ← 関数の中にある
                      }
```

#### 🎯 cast（キャスト）とは
**cast** = 「汎用ノード」を「特定の型のノード」に変換すること

```
SyntaxNode（汎用）  →  ast::Fn（関数専用）
   ↓ cast
「何かのノード」    →  「関数ノードだと確認済み」
```

#### 🔍 ancestors()とは
**ancestors** = 「祖先」という意味。自分の親、親の親、...を順番に返すイテレータ

```
fn test() {           ← 3番目: Fnノード（関数）
    {                 ← 2番目: Blockノード（ブロック）
        use std::fmt; ← 1番目: Useノード（自分）
    }
}
```

#### 🎯 find_map()とは
**find_map**: イテレータの各要素に関数を適用して、最初に成功したものを返す

```
動作の流れ：
1. Useノード → ast::Fn::cast → None（関数じゃない）
2. Blockノード → ast::Fn::cast → None（関数じゃない）  
3. Fnノード → ast::Fn::cast → Some(ast::Fn)（成功！）
```

### 🔧 Assists とは何か？
`Assists` は rust-analyzer におけるコード変換機能の**コンテナ**です。エディタが「何ができるか」を知るための**アクション一覧**を管理します。

```
🎬 実行の流れ
エディタ「何ができる？」 
    ↓
rust-analyzer「Assistsを確認中...」
    ↓  
各アシスト関数「私ができる！」→ acc.add()
    ↓
エディタ「このアクションが利用可能です」
```

### 🧭 AssistContext とは何か？
`AssistContext` は**コード分析の道具箱**です。カーソル位置、選択範囲、ASTノードなど、アシストが判断に必要な全情報を提供します。

```
📋 AssistContext の中身
┌─────────────────────────────┐
│ 🎯 カーソル位置情報          │
│ 📝 選択範囲                 │  
│ 🌳 構文木（AST）への参照      │
│ 📄 ファイル情報             │
│ ⚙️  設定情報                │
└─────────────────────────────┘
```

### 🏗️ アシスト実装の基本テンプレート

```rust
pub(crate) fn move_use_to_top(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    // 1️⃣ 適用可能性チェック - 「このアシストが使えるか？」
    let use_item = ctx.find_node_at_offset::<ast::Use>()?;
    
    // 2️⃣ コンテキスト分析 - 「どんな状況か？」
    // use文から上に向かって親要素を辿り、最初に見つかった関数ノードを取得
    let parent_fn = use_item.syntax().ancestors()
        .find_map(ast::Fn::cast)?;
    
    // 3️⃣ アクション登録 - 「こんな変換ができます！」
    acc.add(
        AssistId("move_use_to_top", AssistKind::Refactor),
        "Move use statement to top-level",
        use_item.syntax().text_range(),
        |builder| {
            // 4️⃣ 実際の変換処理 - 「実行されたときの処理」
        },
    )
}
```

### 🎯 アシストの4段階詳細

#### 1️⃣ 適用可能性チェック段階
```
🔍 質問：「このアシストは今使える？」

┌─ カーソル位置チェック ─┐
│ use文の上にある？      │ → Yes: 続行 / No: None返却
└─────────────────────┘
```

#### 2️⃣ コンテキスト分析段階  
```
🧐 質問：「どんな状況で何をすべき？」

┌─ 状況分析 ─┐
│ 関数内？    │ → トップレベルへ移動が必要
│ impl内？    │ → モジュールトップへ移動
│ 既にトップ？ │ → 何もしない（適用不可）
└───────────┘
```

#### 3️⃣ アクション登録段階
```
📝 質問：「エディタに何を表示する？」

AssistId: move_use_to_top        // 内部識別子
AssistKind: Refactor            // カテゴリ（リファクタリング）
Label: "Move use to top"        // エディタに表示される文言
Target: use文の範囲             // ハイライト範囲
```

#### 4️⃣ 実行段階（builder内）
```
⚡ 質問：「実際にどう変換する？」

builder.delete(元のuse文の位置)
builder.insert(トップの位置, 新しいuse文)
```

### 🏛️ アシスト実装アーキテクチャ図

```
🎮 エディタ（VSCode等）
    ↓ 「カーソル位置でできることは？」
    
🧠 rust-analyzer メインエンジン
    ↓ 全アシスト関数を実行
    
┌─────────────────────────────────┐
│ 🔄 各アシスト関数                │
│ ┌─────────────────────────────┐ │
│ │ move_use_to_top(acc, ctx)   │ │
│ │ ┌─ 1️⃣チェック ──────────┐   │ │
│ │ │ ctx.find_node_at_offset │   │ │
│ │ │ "use文ある？"           │   │ │
│ │ └─────────────────────── ┘   │ │
│ │ ┌─ 2️⃣分析 ──────────────┐   │ │  
│ │ │ "関数内？トップレベル？" │   │ │
│ │ └─────────────────────── ┘   │ │
│ │ ┌─ 3️⃣登録 ──────────────┐   │ │
│ │ │ acc.add(...)           │   │ │ 
│ │ │ "このアクション可能！"   │   │ │
│ │ └─────────────────────── ┘   │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘
    ↓ Assistsコンテナに蓄積
    
📋 結果：利用可能アクション一覧
    ↓ エディタに返却
    
🎮 エディタ：ユーザーに選択肢を表示
```

## 🎯 重要なファイルと機能

### 📄 `/crates/ide-assists/src/handlers/auto_import.rs`
**なぜ重要**: use文の挿入とマージ処理のお手本
```rust
// 🔧 使える機能
use ide_db::imports::{
    insert_use::insert_use,          // use文の挿入
    merge_imports::try_merge_imports, // use文のマージ
    MergeBehavior,                   // グループ化戦略
};
```

### 📄 `/crates/ide-db/src/imports/`
**なぜ重要**: import操作の核心機能が集約されている
```
imports/
├── insert_use.rs     👈 use文挿入の処理
├── merge_imports.rs  👈 use文のマージ処理  
└── mod.rs           👈 import設定の定義
```

## 🎮 カーソル位置の検出

## 🧭 AssistContext の詳細解説

### AssistContext の役割
`AssistContext` は**アシストが判断に必要な全ての情報**を集約したコンテナです。カーソル位置、選択範囲、構文木、設定など、「今どんな状況か？」を知るための情報源です。

### 🏗️ AssistContext の内部構造

```
🧭 AssistContext
┌─────────────────────────────────────┐
│ 📍 Position Information            │
│ ├─ offset: TextSize                │ ← カーソルの位置（文字数）
│ ├─ selection: TextRange            │ ← 選択範囲
│ └─ frange: FileRange               │ ← ファイル+範囲
│                                     │
│ 🌳 Syntax Tree Access             │  
│ ├─ source_file: SourceFile         │ ← 構文木のルート
│ ├─ db: &RootDatabase              │ ← 解析データベース
│ └─ semantic: Semantic              │ ← 意味解析情報
│                                     │
│ ⚙️ Configuration                   │
│ ├─ config: &AssistConfig          │ ← アシストの設定
│ └─ import_assets: ImportAssets     │ ← import関連設定
└─────────────────────────────────────┘
```

### 🎯 AssistContext の主要メソッド

#### 位置検出メソッド
```rust
impl AssistContext {
    // 🎯 カーソル位置の特定の型のASTノードを取得
    fn find_node_at_offset<N: AstNode>(&self) -> Option<N> {
        // 例: use文、関数、impl文などを取得
        self.source_file
            .syntax()
            .token_at_offset(self.offset)
            .find_map(|token| token.parent_ancestors().find_map(N::cast))
    }
    
    // 📍 カーソル位置を覆う要素（ノードまたはトークン）
    fn covering_element(&self) -> SyntaxElement {
        // カーソル位置の最小要素を取得
    }
    
    // 🎌 選択範囲内の特定型ノードを取得  
    fn find_node_in_selection<N: AstNode>(&self) -> Option<N>
    
    // ✅ 選択範囲が空かどうか
    fn has_empty_selection(&self) -> bool {
        self.selection.is_empty()
    }
}
```

#### 構文解析メソッド
```rust
impl AssistContext {
    // 🌳 構文木のルートファイル
    fn source_file(&self) -> &SourceFile
    
    // 🔍 意味解析（型情報、参照解決など）
    fn sema(&self) -> &Semantic<'_, RootDatabase>
    
    // 📄 現在のファイルID
    fn file_id(&self) -> FileId
}
```

### 🎯 実際の使用例

#### use文検出の例
```rust
// 1️⃣ カーソル位置のuse文を探す
let use_item = ctx.find_node_at_offset::<ast::Use>()?;

// 2️⃣ use文の詳細を分析
let use_tree = use_item.use_tree()?;
let path = use_tree.path()?;

// 3️⃣ use文の親要素を確認（関数内 vs トップレベル）
let parent = use_item.syntax().parent()?;
```

#### 状況分析の例  
```rust
// 🏠 どこにいるかを特定
let container = use_item.syntax().ancestors()
    .find_map(|node| {
        match_ast! {
            match node {
                ast::Fn(func) => Some(Container::Function(func)),
                ast::Impl(impl_) => Some(Container::Impl(impl_)),
                ast::Module(module) => Some(Container::Module(module)),
                _ => None,
            }
        }
    });

enum Container {
    Function(ast::Fn),    // 関数内 → トップレベルへ移動
    Impl(ast::Impl),      // impl内 → impl外へ移動  
    Module(ast::Module),  // モジュール内 → モジュール外へ移動
}
```


### 🔧 Assists の詳細解説

### Assists の役割
`Assists` は**利用可能なアクションを収集する容器**です。各アシスト関数が「私はこんなことができます！」と登録する場所です。

### 🏗️ Assists の内部構造
```
🔧 Assists
┌─────────────────────────────────────┐
│ Vec<Assist>                        │ ← 利用可能アクション一覧
│ ┌─────────────────────────────────┐ │
│ │ Assist {                        │ │
│ │   id: AssistId,                 │ │ ← 識別子("move_use_to_top")
│ │   label: String,                │ │ ← 表示名("Move use to top")
│ │   group: Option<GroupLabel>,    │ │ ← グループ化情報
│ │   target: TextRange,           │ │ ← ハイライト範囲
│ │   source_change: SourceChange  │ │ ← 実際の変更内容
│ │ }                               │ │
│ └─────────────────────────────────┘ │
│                                     │
│ config: &AssistConfig               │ ← アシストの設定
│ resolve: AssistResolveStrategy      │ ← 解決戦略（即座 or 遅延）
└─────────────────────────────────────┘
```

### 🎬 acc.add() の動作詳細
```rust
// アクション登録の詳細
acc.add(
    AssistId("move_use_to_top", AssistKind::Refactor),  // 1️⃣ 識別情報
    "Move use statement to top-level",                   // 2️⃣ ユーザー向け表示
    target_range,                                        // 3️⃣ ハイライト範囲
    |builder| {                                         // 4️⃣ 変更処理
        // この時点で実際の編集が記録される
        builder.delete(old_range);
        builder.insert(new_position, new_text);
    },
)
```

#### 処理の流れ
```
🎬 acc.add() の実行フロー

1️⃣ 基本情報設定
   ├─ AssistId: 内部識別子
   ├─ AssistKind: カテゴリ（Refactor, QuickFix等）
   └─ Label: エディタ表示用文字列

2️⃣ ターゲット設定  
   └─ target_range: ハイライトする範囲

3️⃣ 変更処理記録
   ├─ builder.delete() → 削除操作を記録
   ├─ builder.insert() → 挿入操作を記録
   └─ SourceChange に変換して保存

4️⃣ Assistsコンテナに追加
   └─ Vec<Assist>に新しいAssistを追加
```

### 実際の検出パターン
```rust
// use文を検出
let use_item = ctx.find_node_at_offset::<ast::Use>()?;

// 親の関数を検出（トップレベルでないことを確認）
let parent_fn = use_item.syntax().ancestors()
    .find_map(ast::Fn::cast)?;
```

## 🌐 依存関係とモジュール構造詳細

### 🏗️ rust-analyzer アーキテクチャ概要
```
🏛️ rust-analyzer 全体構造
┌─────────────────────────────────────────────────────────────┐
│ 🎮 エディタ（VSCode, Neovim等）                              │
│ ↕️ LSP Protocol                                              │
└─────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────┐
│ 🧠 rust-analyzer メインエンジン                              │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🎯 IDE Layer (crates/ide/)                             │ │
│ │ ├─ completion: 自動補完                                 │ │  
│ │ ├─ hover: ホバー情報                                    │ │
│ │ ├─ goto_definition: 定義ジャンプ                        │ │
│ │ └─ assists: コード変換（我々の作業領域！）               │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 📊 Analysis Layer (crates/ide-db/)                     │ │
│ │ ├─ RootDatabase: 全データの中央管理                     │ │
│ │ ├─ imports/: import操作の核心部分                       │ │
│ │ ├─ search/: コード検索                                  │ │
│ │ └─ symbols/: シンボル管理                               │ │
│ └─────────────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🌳 Base Layer (crates/hir*, syntax/)                   │ │
│ │ ├─ syntax: 構文解析（AST）                              │ │
│ │ ├─ hir: 高レベル中間表現                                │ │
│ │ └─ parser: Rustコードの解析                             │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 🎯 ide-assists の依存関係マップ
```
📁 crates/ide-assists/ の依存関係
┌─────────────────────────────────────────────────┐
│ 🎯 ide-assists (我々の作業場所)                  │
│                                                 │
│ 📥 依存している外部クレート                      │
│ ├─ 🌳 syntax (AST操作)                         │
│ │  ├─ ast::Use, ast::Fn等のノード型             │
│ │  ├─ SyntaxNode, SyntaxElement                │
│ │  └─ TextRange, TextSize                      │
│ │                                              │
│ ├─ 📊 ide-db (解析・操作ライブラリ)              │
│ │  ├─ imports/: use文操作の核心                │
│ │  │  ├─ insert_use::insert_use               │
│ │  │  ├─ merge_imports::try_merge_imports     │
│ │  │  └─ MergeBehavior, InsertUseConfig      │
│ │  ├─ RootDatabase: データアクセス             │
│ │  └─ search/: コード検索機能                  │
│ │                                              │
│ ├─ 🧠 hir (意味解析)                           │
│ │  ├─ Module, Function等の意味情報             │
│ │  └─ 型情報、スコープ解析                      │
│ │                                              │
│ └─ 🔧 stdx (ユーティリティ)                     │
│    └─ 共通的なヘルパー関数                      │
│                                                 │
│ 📤 提供している機能                             │
│ └─ Assists: エディタ向けアクション一覧           │
└─────────────────────────────────────────────────┘
```

### 🎯 move_use_to_top 実装で使う依存関係
```
🎯 move_use_to_top の実装依存関係
┌─────────────────────────────────────────────────┐
│ move_use_to_top(acc, ctx)                       │
│ ┌─────────────────────────────────────────────┐ │
│ │ 使用する主要な型とメソッド                   │ │
│ │                                             │ │
│ │ 📍 AssistContext から                       │ │
│ │ ├─ find_node_at_offset::<ast::Use>()       │ │
│ │ ├─ source_file()                           │ │
│ │ └─ sema() // 意味解析アクセス               │ │
│ │                                             │ │
│ │ 🌳 syntax クレートから                      │ │
│ │ ├─ ast::Use, ast::Fn                       │ │
│ │ ├─ SyntaxNode::ancestors()                 │ │
│ │ ├─ TextRange                               │ │
│ │ └─ match_ast! マクロ                        │ │
│ │                                             │ │
│ │ 📊 ide-db::imports から                    │ │
│ │ ├─ insert_use::insert_use()                │ │
│ │ ├─ InsertUseConfig                         │ │
│ │ └─ MergeBehavior                           │ │
│ │                                             │ │
│ │ 🔧 Assists へ登録                          │ │  
│ │ └─ acc.add() でアクション追加               │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### 🔄 実行時の情報フロー
```
🔄 move_use_to_top 実行時の情報フロー

1️⃣ エディタからの要求
   🎮 VSCode「カーソル位置: line 5, col 10」
   ↓ LSP Protocol
   🧠 rust-analyzer「アシスト確認開始」

2️⃣ AssistContext 構築
   📋 AssistContext {
   ├─ offset: 95 (文字位置)
   ├─ source_file: 構文木
   ├─ db: RootDatabase参照
   └─ config: アシスト設定
   }

3️⃣ AST解析
   🌳 syntax::ast::Use ← find_node_at_offset()
   ├─ use_tree: "std::collections::HashMap"
   ├─ parent: ast::Fn (関数内use文と判明)
   └─ ancestors: [Use, Fn, SourceFile]

4️⃣ 変換処理計画
   📊 ide-db::imports ← 操作方法を決定
   ├─ 削除: 元のuse文位置
   ├─ 挿入: ファイル先頭付近
   └─ マージ: 既存use文との統合

5️⃣ アクション登録
   🔧 Assists::add() ← エディタ向け情報
   ├─ AssistId: "move_use_to_top"
   ├─ Label: "Move use statement to top-level"  
   ├─ target: use文の範囲
   └─ builder: 実際の編集処理

6️⃣ エディタへ返却
   🎮 VSCode「💡 Move use statement to top-level」
```

## 🔄 use文の操作メソッド

### MergeBehavior（グループ化戦略）
```rust
pub enum MergeBehavior {
    Crate,   // クレート毎にグループ化
    Module,  // モジュール毎にグループ化  
    One,     // 1つにまとめる
}
```

### InsertUseConfig（挿入設定）
```rust
pub struct InsertUseConfig {
    pub merge: Option<MergeBehavior>,  // マージ戦略
    pub prefix_kind: PrefixKind,       // プレフィックス形式
    pub group: bool,                   // グループ化するか
}
```

## 🛠 実装で参考になるコード

### 1. `/crates/ide-assists/src/handlers/extract_module.rs`
**参考ポイント**: 複数ステップの変換処理
```rust
acc.add(
    AssistId("extract_module", AssistKind::RefactorExtract),
    "Extract Module",
    target.text_range(),
    |builder| {
        // 複数の編集操作
        builder.edit_file(file_id);
        builder.delete(range);
        builder.insert(offset, text);
    },
)
```

### 2. Text編集のユーティリティ
```rust
// 🗑 削除
builder.delete(use_item.syntax().text_range());

// ➕ 挿入  
builder.insert(offset, new_text);

// 📝 複数ファイル編集
builder.edit_file(file_id);
```

## 🧪 テストパターン

### 基本テストの書き方
```rust
#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};
    
    #[test]
    fn test_basic_move() {
        check_assist(
            move_use_to_top,
            r#"
fn test() {
    use std::collections::HashMap$0;  // $0 = カーソル位置
    let map = HashMap::new();
}
"#,
            r#"
use std::collections::HashMap;

fn test() {
    let map = HashMap::new();
}
"#,
        );
    }
    
    #[test]
    fn test_not_applicable_for_top_level() {
        check_assist_not_applicable(
            move_use_to_top,
            r#"
use std::collections::HashMap$0;  // トップレベル = 適用不可
"#,
        );
    }
}
```

## 📝 実装の流れ

```
1. ファイル作成
   📄 /handlers/move_use_to_top.rs

2. 基本構造実装
   🎯 カーソル検出 → コンテキスト確認 → 変換処理

3. テスト作成
   🧪 正常ケース + エラーケース

4. 登録
   📋 lib.rs に追加

5. 動作確認
   🚀 cargo test で確認
```