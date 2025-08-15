# 🚀 Issue #20326: Move `use` statements to top-level

## 🎯 一言で説明すると

関数の中に書かれた `use` 文を、**ワンクリックでファイルの上部に移動**させるrust-analyzerの新機能を作ります！

## 🔍 何が問題なの？

```rust
// 😵 こんなコードを見たことありませんか？
fn calculate_stats() {
    use std::collections::HashMap;  // ← なぜここに？
    use serde::Serialize;           // ← 本来は上にあるべき
    
    let mut data = HashMap::new();
    // ...
}

fn read_file() {
    use std::fs::File;              // ← またここにも！
    use std::io::Read;
    
    let file = File::open("data.txt");
    // ...
}
```

### 🤔 この問題の何が悪いの？

- **📚 可読性が悪い**: import が散らばって依存関係が分からない
- **🔄 重複の可能性**: 同じ `use` が複数の関数に書かれがち  
- **🧹 保守性が低い**: import の管理が面倒

## 📁 ファイル構成

```
20326/
├── 📄 README.md                    👈 このファイル（全体概要）
├── 📄 overview.md                  👈 Issue の詳細説明
├── 📄 verification.md              👈 動作確認方法
├── 📄 code-analysis.md             👈 rust-analyzer コード解析
├── 📄 implementation-strategy.md   👈 実装方針・ステップ
└── lessons/                        👈 学習教材
    ├── 📄 README.md                👈 学習ガイド
    ├── 📄 mod.rs                   👈 モジュール宣言
    ├── 📄 ast_basics.rs            👈 AST基礎学習
    └── 📄 assist_context.rs        👈 AssistContext学習
```

## 🎯 Quick Start

### 1. 📖 Issue を理解する
```bash
# Issue の概要を把握
cat overview.md
```

### 2. 🔍 コードベースを理解する  
```bash
# rust-analyzer の関連コードを理解
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

### 📚 基礎概念の学習
```
lessons/
├── README.md            👈 学習の進め方ガイド  
├── mod.rs               👈 モジュール宣言
├── ast_basics.rs        👈 AST (構文木) の基礎
└── assist_context.rs    👈 AssistContext の使い方
```

### 🔗 外部リソース
- **Issue URL**: https://github.com/rust-lang/rust-analyzer/issues/20326
- **rust-analyzer 開発ガイド**: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/README.md
- **Assists 実装例**: `/crates/ide-assists/src/handlers/` ディレクトリ

## 🎯 学習の進め方

### Phase 1: 基礎理解 📖
1. `overview.md` で Issue の要件を理解
2. `lessons/README.md` で学習の進め方を確認
3. `lessons/ast_basics.rs` で AST の基礎を学習
4. `lessons/assist_context.rs` で AssistContext を学習

### Phase 2: コードベース理解 🔍
1. `code-analysis.md` で rust-analyzer の構造を理解
2. 実際の rust-analyzer リポジトリで関連コードを確認
3. 既存の assists 実装を参考にする

### Phase 3: 実装計画 🛠
1. `implementation-strategy.md` で実装ステップを確認
2. スモールステップでの実装計画を立てる
3. テストケースを先に設計する

### Phase 4: 実装・検証 🚀
1. Phase 1 から順次実装
2. `verification.md` の方法で動作確認
3. rust-analyzer のテストスイートで検証

## 💡 成功のコツ

### ✅ Do's
- **小さく始める**: 最小限の機能から実装
- **テスト駆動**: 実装前にテストケースを明確化  
- **既存コード活用**: ide-db の機能を最大限活用
- **段階的改善**: Phase 毎に動作確認

### ❌ Don'ts  
- 一度に全機能を実装しない
- テストなしで実装しない
- 既存のパターンを無視しない
- エラーハンドリングを後回しにしない

## 🎯 最終目標

```rust
// Before (関数内の use 文)
fn example() {
    use std::collections::HashMap;  // 👈 カーソルをここに置いて Code Action
    let map = HashMap::new();
}

// After (ファイル先頭に移動)
use std::collections::HashMap;

fn example() {
    let map = HashMap::new();
}
```

**rust-analyzer に新しい機能を貢献し、Rust 開発者の生産性向上に寄与しましょう！** 🎉

---

📞 **サポート**: 実装中に質問や問題があれば、各ファイルの詳細説明を参照するか、rust-analyzer コミュニティで相談してください。