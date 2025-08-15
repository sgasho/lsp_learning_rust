# 🎯 Issue #20263: `ref` snippet shadowed by `ref` keyword in macro

## 🎯 一言で説明すると

マクロ内で`ref`キーワードが`ref`スニペットの補完を邪魔してしまう問題を解決します！補完システムの優先度調整により、開発者により良い補完体験を提供する実用的なissueです。

## 🔍 何が問題なの？

```rust
use std::convert::identity;

fn main() {
    let num = 42;
    println!("{}", identity(num.re|));  // ← カーソル位置
}
```

### 🤔 現在の動作 (問題)

- **`num.re`と入力**: `ref`スニペットが補完候補に表示される ✅
- **`num.ref`と完全入力**: `ref`スニペットが消えて、`ref`キーワードのみ表示される ❌

### ✅ 期待される動作

- **どのタイミングでも**: `ref`スニペット（`&num`に展開）が補完候補に表示されるべき
- **キーワードとの共存**: `ref`キーワードと`ref`スニペットが両方表示される

## 🧩 補完スニペットとキーワードの基礎知識

### 🎭 補完の種類

1. **キーワード補完**: `ref`, `mut`, `let` などの言語キーワード
2. **スニペット補完**: `ref` → `&$0` のようなコードテンプレート
3. **識別子補完**: 変数名、関数名、型名など
4. **パス補完**: モジュールパスや型パス

### 🌟 `ref` スニペットの実用性

```rust
// ref スニペットの展開例
let value = 42;

// 入力: value.ref
// 展開: &value
let reference = &value;

// より複雑な例
let data = vec![1, 2, 3];
// 入力: data.ref
// 展開: &data
some_function(&data);
```

### 🔍 問題が発生する特定条件

```rust
// ✅ 正常に動作するケース
fn normal_case() {
    let num = 42;
    dbg!(num.re);  // ref スニペットが表示される
}

// ❌ 問題が発生するケース  
fn problematic_case() {
    let num = 42;
    println!("{}", identity(num.re));  // ref スニペットが消える
}
```

**差異**: `println!` + `identity()` の組み合わせが問題を引き起こす

## 📁 ファイル構成

```
20263/
├── 📄 README.md                    👈 このファイル（全体概要）
├── 📄 overview.md                  👈 Issue の詳細技術分析
├── 📄 completion-deep-dive.md      👈 補完システムの詳細解説
├── 📄 code-analysis.md             👈 rust-analyzer コード解析
├── 📄 implementation-strategy.md   👈 実装方針・ステップ
├── 📄 verification.md              👈 動作確認方法
└── lessons/                        👈 学習教材
    ├── 📄 mod.rs                   👈 モジュール宣言
    ├── 📄 completion_basics.rs     👈 補完システムの基礎
    ├── 📄 snippet_system.rs        👈 スニペット機能の理解
    ├── 📄 keyword_handling.rs      👈 キーワード処理の仕組み
    └── 📄 priority_resolution.rs   👈 補完優先度の解決
```

## 🎯 Quick Start

### 1. 📖 問題を理解する
```bash
# Issue の詳細を把握
cat overview.md

# 補完システムの深い理解
cat completion-deep-dive.md
```

### 2. 🔍 コードベースを理解する
```bash
# rust-analyzer の補完関連コードを理解
cat code-analysis.md
```

### 3. 🛠 実装計画を確認する
```bash
# 段階的な実装ステップを確認
cat implementation-strategy.md
```

### 4. 🧪 動作確認方法を学ぶ
```bash
# テスト方法と確認手順を理解
cat verification.md
```

## 🎓 学習リソース

### 📚 基礎概念の学習（推奨順序）

1. **🎯 補完基礎**: `lessons/completion_basics.rs`
   - rust-analyzerの補完システム概要
   - CompletionContextの役割
   - 補完候補の生成プロセス

2. **📋 スニペット機能**: `lessons/snippet_system.rs`
   - スニペットの定義と展開
   - テンプレート変数（$0, $1など）
   - カスタムスニペットの作成

3. **🔤 キーワード処理**: `lessons/keyword_handling.rs`
   - Rustキーワードの補完処理
   - キーワードとコンテキストの関係
   - フィルタリングロジック

4. **⭐ 優先度解決**: `lessons/priority_resolution.rs`
   - 補完候補の優先度システム
   - 候補の重複排除
   - ユーザー体験の最適化

### 🔗 重要な外部リソース

- **Issue URL**: https://github.com/rust-lang/rust-analyzer/issues/20263
- **補完エンジン**: `/crates/ide-completion/src/`
- **スニペット定義**: `/crates/ide-completion/src/snippet.rs`
- **LSP specification**: https://microsoft.github.io/language-server-protocol/

## 🚀 実装アプローチ

### 🎯 Core Problem

```rust
// 問題の核心：補完候補の優先度とフィルタリング
println!("{}", identity(num.ref|));
//                          ^^^^ この位置での補完

// 現在の動作:
// 1. "ref" 文字列完全一致でキーワードフィルタが強く働く
// 2. スニペット候補が除外される
// 3. キーワードのみが表示される

// 期待される動作:
// 1. キーワードとスニペットが共存
// 2. スニペットの方が実用的なので優先度を高く
// 3. 文脈に応じて適切な候補を提示
```

### 🔧 Technical Challenges

1. **🎭 Context Analysis**: マクロ内での補完文脈の正確な判定
2. **⭐ Priority System**: キーワードvsスニペットの優先度調整
3. **🔍 Filtering Logic**: 文字列マッチングでの候補絞り込み改善
4. **🚀 Performance**: 補完速度への影響最小化

## 🎯 学習の進め方

### Phase 1: 補完システム理解 📖
1. `overview.md` で Issue の技術的詳細を理解
2. `completion-deep-dive.md` で補完システムの深層を学習
3. `lessons/completion_basics.rs` で基本概念を実践
4. `lessons/snippet_system.rs` でスニペット機能を学習

### Phase 2: 問題分析 🔍
1. `lessons/keyword_handling.rs` でキーワード処理を理解
2. `lessons/priority_resolution.rs` で優先度システムを学習
3. 実際の問題を再現して動作を確認
4. `code-analysis.md` で関連コードを特定

### Phase 3: 実装設計 🛠
1. `implementation-strategy.md` で実装アプローチを確認
2. 最小限の修正箇所を特定
3. テストケースを事前に設計
4. 既存の補完テストを参考にする

### Phase 4: 実装・検証 🚀
1. 優先度調整ロジックを実装
2. `verification.md` の方法で動作確認
3. エッジケースのテスト
4. パフォーマンスへの影響確認

## 💡 成功のコツ

### ✅ Do's
- **🔬 問題を再現**: まず確実に問題を再現する
- **📊 既存動作を理解**: 現在の補完ロジックを詳しく調べる
- **🎯 最小限の変更**: 既存システムへの影響を最小化
- **🧪 テスト重視**: 補完の微細な動作変化をテストで捕捉

### ❌ Don'ts
- 補完システム全体を変更しない
- パフォーマンスへの影響を軽視しない
- エッジケースのテストを怠らない
- 他の補完機能への副作用を見逃さない

## 🎯 最終目標

```rust
use std::convert::identity;

fn test() {
    let num = 42;
    
    // ✅ 修正後：どの段階でも ref スニペットが利用可能
    println!("{}", identity(num.re|));   // ref スニペット表示
    println!("{}", identity(num.ref|));  // ref スニペット表示（改善）
}
```

**期待される補完候補:**
- 🥇 `ref` スニペット (`&num`) - 最も実用的
- 🥈 `ref` キーワード - 言語仕様として必要
- 🥉 その他の候補 - フィールドやメソッドなど

## 🚨 重要な学習ポイント

### ⚠️ 補完システムの複雑さ
- 複数の候補源（キーワード、スニペット、識別子）の統合
- 文脈依存の候補フィルタリング
- ユーザー体験と性能のバランス

### 🎯 実用性の考慮
- 開発者が最も使いたい候補を優先
- キーワード補完 vs 実用的なスニペット
- マクロ内外での一貫した体験

### 🔧 テスト戦略
- 基本的なスニペット補完
- キーワードとの共存
- マクロ内での特殊条件
- パフォーマンスリグレッション

---

📞 **サポート**: 補完システムは rust-analyzer の核心機能です。実装中に詰まったら、各ファイルの詳細説明を参照するか、既存の補完関連PRを参考にしてください。小さな改善でも大きな開発体験向上に繋がります！