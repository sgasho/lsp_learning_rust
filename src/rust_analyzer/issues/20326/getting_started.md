# 🚀 Getting Started: Issue #20326

## 📋 開発環境のセットアップ

rust-analyzer への貢献を始める前に、開発環境を整えましょう。

### 1. リポジトリのクローン
```bash
# rust-analyzer をクローン
git clone https://github.com/rust-lang/rust-analyzer.git
cd rust-analyzer

# フォークしてからクローンする場合 (推奨)
git clone https://github.com/YOUR_USERNAME/rust-analyzer.git
cd rust-analyzer
git remote add upstream https://github.com/rust-lang/rust-analyzer.git
```

### 2. ビルド環境の確認
```bash
# Rust の最新安定版が必要
rustc --version  # 1.70.0 以上推奨

# ビルドテスト
cargo check

# テスト実行 (時間がかかります)
cargo test --package ide-assists
```

### 3. 開発用ツールの準備
```bash
# rust-analyzer を開発モードでビルド
cargo build --release

# VS Code 拡張機能の開発版を使用する場合
cd editors/code
npm install
npm run package
# 生成された .vsix ファイルを VS Code にインストール
```

## 🎯 Issue #20326 の作業手順

### Phase 1: 理解とプランニング (1日目)

#### ✅ チェックリスト
- [ ] 既存の assists システムを理解する
- [ ] 関連コードを読む (`auto_import.rs`, `extract_module.rs`)
- [ ] AST と syntax tree の基本概念を学ぶ
- [ ] テストケースを設計する

#### 🔍 調査すべきファイル
```bash
# 既存の assists を確認
ls crates/ide-assists/src/handlers/

# 重要なファイルを読む
cat crates/ide-assists/src/handlers/auto_import.rs
cat crates/ide-db/src/imports/insert_use.rs
```

### Phase 2: 最小実装 (2-3日目)

#### 📝 作業内容
1. **ハンドラーファイルの作成**
   ```bash
   touch crates/ide-assists/src/handlers/move_use_to_top.rs
   ```

2. **基本構造の実装**
   - カーソル位置の `use` 文検出
   - 関数内にあることの確認
   - 最小限のテスト

3. **登録**
   ```rust
   // crates/ide-assists/src/lib.rs に追加
   mod handlers {
       // ...
       mod move_use_to_top;
   }
   
   pub fn all() -> &'static [Handler] {
       &[
           // ...
           move_use_to_top::move_use_to_top,
       ]
   }
   ```

### Phase 3: 機能実装 (4-5日目)

#### 🔧 実装する機能
1. **use 文の移動**
   - 現在位置からの削除
   - ファイル先頭への挿入

2. **既存 use 文との統合**
   - `ide-db::imports` の活用
   - グループ化とソート

3. **テストの充実**

### Phase 4: 品質向上 (6-7日目)

#### 🛡️ 改善項目
1. **エラーハンドリング**
2. **エッジケースのテスト**
3. **パフォーマンス最適化**
4. **ドキュメント作成**

## 🧪 開発中のテスト方法

### 単体テスト
```bash
# 特定のテストを実行
cargo test -p ide-assists move_use_to_top

# 全ての assists テストを実行
cargo test -p ide-assists
```

### 統合テスト (VS Code で確認)
```bash
# rust-analyzer を開発モードでビルド
cargo build --release

# VS Code で確認
# 1. Rust ファイルを開く
# 2. 関数内に use 文を書く
# 3. use 文にカーソルを置いて Ctrl+. を押す
# 4. "Move use statement to top-level" が表示されるかチェック
```

## 📚 学習リソース

### 必読ドキュメント
- [rust-analyzer Architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md)
- [Assists Guide](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/assists.md)

### 参考になる既存コード
```bash
# 最も参考になる assists
cat crates/ide-assists/src/handlers/auto_import.rs
cat crates/ide-assists/src/handlers/extract_module.rs
cat crates/ide-assists/src/handlers/move_bounds.rs

# テストの書き方を学ぶ
grep -r "check_assist" crates/ide-assists/src/handlers/
```

### AST 関連の学習
```bash
# syntax crate の理解
cat crates/syntax/src/ast/nodes.rs
cat crates/syntax/src/ast/traits.rs

# AST の構造を確認
cargo run --bin analysis-stats -- --help
```

## 🤝 コントリビューション手順

### 1. ブランチ作成
```bash
git checkout -b move-use-to-top-assist
```

### 2. 実装・テスト
```bash
# 実装
vim crates/ide-assists/src/handlers/move_use_to_top.rs

# テスト
cargo test -p ide-assists move_use_to_top
```

### 3. コミット
```bash
git add .
git commit -m "feat: add move use statement to top-level assist

Implements assist to move use statements from inside functions
to the top-level scope. Integrates with existing import merging
behavior.

Fixes #20326"
```

### 4. プルリクエスト
```bash
git push origin move-use-to-top-assist
# GitHub でプルリクエストを作成
```

## 💡 開発のコツ

### 🎯 小さく始める
- 最初は単純なケースだけ実装
- 複雑な統合機能は後回し
- テストを先に書いてから実装

### 🔍 既存コードを活用
- `ide-db::imports` の機能を最大限活用
- 車輪の再発明をしない
- 既存のテストパターンを真似る

### 🧪 テスト駆動開発
- 実装前にテストケースを明確にする
- 失敗ケースも必ずテストする
- エッジケースを見落とさない

### 📝 コードレビューを意識
- 読みやすいコードを心がける
- コメントで意図を説明
- 既存のコーディングスタイルに合わせる

## 🚨 よくある落とし穴

### ❌ 避けるべきこと
1. **AST 操作の誤解**: `syntax()` と `ast` の違いを理解する
2. **Text Range の計算ミス**: 削除・挿入位置の計算を慎重に
3. **テストの不備**: 成功ケースだけでなく失敗ケースもテスト
4. **既存機能の破壊**: 既存の import 機能との競合を避ける

### ✅ 確認すべきポイント
- [ ] カーソル位置の検出が正確
- [ ] トップレベルの use 文では動作しない
- [ ] 既存の use 文と適切にマージされる
- [ ] 元の use 文が確実に削除される
- [ ] インデントやフォーマットが保持される

これで Issue #20326 への取り組みを始められます！🎉