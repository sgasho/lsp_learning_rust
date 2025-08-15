# 🚀 実装方針

issue #20326を**段階的に**、**確実に**実装するための戦略を説明します。初心者でも迷わないよう、スモールステップで進めます。

## 🎯 全体の進め方

```
🔥 Phase 1: とりあえず動く状態を作る
    ↓
⚡ Phase 2: 基本的な移動機能を実装
    ↓  
🧩 Phase 3: 既存のuse文との統合
    ↓
🛡 Phase 4: エラーハンドリングで完璧に
```

## 📋 実装ステップ（スモールステップ）

### Phase 1: 基本構造の構築 🏗
```
🎯 Goal: 最小限の動作する実装を作る

📝 Tasks:
1. ハンドラーファイルの作成
2. 基本的なカーソル検出の実装
3. 簡単なテストケースの作成
4. lib.rs への登録

⏱ 期間: 1-2日
✅ 成功基準: カーソルが use 文にあるときアシストが表示される
```

### Phase 2: 移動機能の実装 🔄
```
🎯 Goal: use 文の移動機能を実装

📝 Tasks:
1. use 文の削除処理
2. ファイル先頭への挿入処理
3. 基本的な移動テストの作成

⏱ 期間: 2-3日
✅ 成功基準: 単純な use 文がファイル先頭に移動される
```

### Phase 3: 統合・グループ化機能 🧩
```
🎯 Goal: 既存の use 文との統合

📝 Tasks:
1. 既存 use 文の検出
2. MergeBehavior を使った統合
3. グループ化のテスト

⏱ 期間: 2-3日  
✅ 成功基準: 既存の use 文と適切にマージされる
```

### Phase 4: エラーハンドリング・エッジケース 🛡
```
🎯 Goal: 堅牢性の向上

📝 Tasks:
1. トップレベル use 文での無効化
2. 複雑なケースのテスト（ネスト関数、モジュール内等）
3. エラーケースの充実

⏱ 期間: 1-2日
✅ 成功基準: 全てのエッジケースで適切に動作
```

## 🛠 詳細実装計画

### 1. ファイル構成
```
📁 /crates/ide-assists/src/handlers/
├── move_use_to_top.rs        👈 メイン実装
└── tests/
    └── move_use_to_top.rs    👈 テスト（必要に応じて）
```

### 2. コア実装の設計

#### 🎯 main function signature
```rust
pub(crate) fn move_use_to_top(acc: &mut Assists, ctx: &AssistContext) -> Option<()>
```

#### 🔍 detection logic
```rust
// Step 1: use 文の検出
let use_item = ctx.find_node_at_offset::<ast::Use>()?;

// Step 2: 関数内かチェック（トップレベルでない）
let parent_fn = use_item.syntax().ancestors()
    .find_map(ast::Fn::cast)?;

// Step 3: モジュール内の場合も考慮
let in_module = use_item.syntax().ancestors()
    .any(|node| ast::Module::can_cast(node.kind()));
```

#### 🔄 transformation logic
```rust
acc.add(
    AssistId("move_use_to_top", AssistKind::Refactor),
    "Move use statement to top-level",
    use_item.syntax().text_range(),
    |builder| {
        // 1. 現在位置から削除
        builder.delete(use_item.syntax().text_range());
        
        // 2. ファイル先頭に挿入（ide-db の機能を使用）
        let source_file = ctx.sema.parse(ctx.file_id());
        insert_use(&source_file, use_item.clone(), &ctx.config.insert_use);
    },
)
```

## 🧪 テスト戦略

### Priority 1: 基本機能テスト
```rust
#[test] fn test_move_simple_use_from_function()
#[test] fn test_not_applicable_for_top_level_use()
#[test] fn test_cursor_not_on_use_statement()
```

### Priority 2: 統合テスト  
```rust
#[test] fn test_merge_with_existing_uses()
#[test] fn test_group_with_same_crate_imports()
#[test] fn test_preserve_use_statement_formatting()
```

### Priority 3: エッジケーステスト
```rust
#[test] fn test_nested_function_use()
#[test] fn test_module_inner_function_use()
#[test] fn test_multiple_use_statements_in_function()
```

## 🎯 成功指標

### ✅ 最小限成功（MVP）
- [ ] カーソルが関数内の use 文にあるときアシストが提案される
- [ ] use 文がファイル先頭に移動される
- [ ] 元の use 文が削除される

### ✅ 完全成功（Full Feature）
- [ ] 既存の use 文と適切にマージされる
- [ ] グループ化戦略（クレート別、モジュール別）が動作する
- [ ] トップレベルの use 文では提案されない
- [ ] 複雑なネスト構造でも動作する

## 🚧 実装時の注意点

### ⚠️ 落とし穴
1. **Text Range の計算**: use 文の正確な範囲を取得
2. **Insert Position**: ファイル先頭の適切な位置を特定
3. **Merge Behavior**: 既存の設定に従ったマージ処理
4. **Syntax Tree**: AST の変更時の整合性

### 💡 ベストプラクティス
1. **小さく始める**: 最小限の機能から実装
2. **テスト駆動**: 実装前にテストケースを明確化
3. **既存コード活用**: ide-db の機能を最大限活用
4. **エラーハンドリング**: 適用不可能なケースを明確化

## 📚 学習リソース

### 事前学習が必要な概念
1. **AST (Abstract Syntax Tree)**: Rust コードの構文木
2. **LSP Assists**: Language Server Protocol のアシスト機能
3. **Text Editing**: エディタでのテキスト操作
4. **Import Merging**: use 文のマージ戦略

### 参考にすべき既存コード
1. `auto_import.rs` - use 文の挿入処理
2. `extract_module.rs` - 複数ステップの変換処理
3. `merge_imports.rs` - use 文のマージロジック