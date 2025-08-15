# 🔮 Issue #20264: `&field` missed inside macro

## 🎯 一言で説明すると

**マクロ内で`&field`の補完候補が欠落する問題**を解決します！マクロ展開時の補完文脈解析を改善して、マクロ内でも通常のコードと同じ補完体験を実現する挑戦的なissueです。

## 🔍 何が問題なの？

```rust
struct NamedField {
    out: Vec<u8>,
}

fn main() {
    let s = NamedField { out: Vec::new() };
    
    // ✅ 通常のコードでは期待通りに動作
    str::from_utf8(s.);  // → `&out` が補完候補に表示される
    
    // ❌ マクロ内では `&out` が補完候補に表示されない
    dbg!(s.);  // → `out` は表示されるが `&out` が欠落
}
```

### 🤔 なぜこれが問題なの？

- **🧩 マクロの普遍性**: `dbg!`, `println!`, `format!` など多くのマクロで発生
- **📚 開発体験の不整合**: マクロ内外で補完の挙動が異なる
- **⚡ 生産性の低下**: 手動で`&`を付ける必要があり効率が悪い
- **🔄 一貫性の欠如**: rust-analyzer の補完システムの不完全性を示す

## 🧬 マクロの基礎知識

### 🎭 マクロとは何か？

Rustのマクロは**コンパイル時にコードを生成する仕組み**です：

```rust
// マクロ呼び出し
dbg!(s.field);

// 展開後のコード（概念的）
{
    let val = &s.field;  // <- ここで &s.field が必要
    eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), "s.field", val);
    val
}
```

### 🔍 マクロの種類

1. **宣言的マクロ** (`macro_rules!`)
   ```rust
   macro_rules! my_macro {
       ($x:expr) => { println!("{}", $x); };
   }
   ```

2. **手続き的マクロ** (derive, attribute, function-like)
   ```rust
   #[derive(Debug)]  // derive macro
   struct Data { x: i32 }
   ```

### 🌊 マクロ展開のフロー

```
入力コード → トークン解析 → マクロ展開 → AST生成 → 型検査
    ↑                                              ↑
  ここで補完            理想的にはここでも補完が必要
```

## 🛠 rust-analyzerにおけるマクロ処理

### 📊 アーキテクチャ概要

```
User Input (マクロ内)
    ↓
TokenTree解析
    ↓
MacroExpansion Engine  ← 🎯 問題の発生箇所
    ↓
HirDef (High-level IR)
    ↓
Completion Engine      ← 🎯 修正対象
    ↓
LSP Response
```

### 🧩 主要コンポーネント

1. **hir_expand**: マクロ展開処理
2. **hir_def**: 定義解決
3. **ide_completion**: 補完エンジン
4. **syntax**: 構文解析

## 📁 ファイル構成

```
20264/
├── 📄 README.md                    👈 このファイル（全体概要）
├── 📄 overview.md                  👈 Issue の詳細技術説明
├── 📄 macro-deep-dive.md           👈 マクロ処理の詳細解説
├── 📄 completion-analysis.md       👈 補完システムの解析
├── 📄 code-analysis.md             👈 rust-analyzer コード解析
├── 📄 implementation-strategy.md   👈 実装方針・ステップ
├── 📄 verification.md              👈 動作確認方法
└── lessons/                        👈 学習教材
    ├── 📄 mod.rs                   👈 モジュール宣言
    ├── 📄 macro_basics.rs          👈 マクロ基礎学習
    ├── 📄 macro_expansion.rs       👈 マクロ展開メカニズム
    ├── 📄 completion_context.rs    👈 補完文脈解析
    └── 📄 token_tree_analysis.rs   👈 TokenTree解析
```

## 🎯 Quick Start

### 1. 📖 問題を深く理解する
```bash
# Issue の技術的詳細を把握
cat overview.md

# マクロ処理の深い理解
cat macro-deep-dive.md
```

### 2. 🔍 補完システムを解析する
```bash
# 補完エンジンの動作を理解
cat completion-analysis.md

# rust-analyzer の関連コードを理解
cat code-analysis.md
```

### 3. 🛠 実装戦略を確認する
```bash
# 段階的な実装ステップを確認
cat implementation-strategy.md
```

### 4. 🧪 検証方法を学ぶ
```bash
# テスト方法と確認手順を理解
cat verification.md
```

## 🎓 学習リソース

### 📚 基礎概念の学習（推奨順序）

1. **🧩 マクロ基礎**: `lessons/macro_basics.rs`
   - Rustマクロの基本概念
   - 宣言的マクロ vs 手続き的マクロ
   - TokenTreeの構造

2. **🔄 マクロ展開**: `lessons/macro_expansion.rs`
   - rust-analyzerでのマクロ展開プロセス
   - HirExpanderの役割
   - MacroDefとMacroCall

3. **💭 補完文脈**: `lessons/completion_context.rs`
   - CompletionContextの仕組み
   - SyntaxNodeとTokenの関係
   - ExpectedTypeの推論

4. **🌳 TokenTree解析**: `lessons/token_tree_analysis.rs`
   - TokenTreeの走査方法
   - Spanとオリジナル位置の対応
   - マクロ内での位置追跡

### 🔗 重要な外部リソース

- **Issue URL**: https://github.com/rust-lang/rust-analyzer/issues/20264
- **マクロ展開コード**: `/crates/hir-expand/src/`
- **補完エンジン**: `/crates/ide-completion/src/`
- **Rustマクロブック**: https://veykril.github.io/tlborm/

## 🚀 実装アプローチ

### 🎯 Core Problem

```rust
// 問題の核心：マクロ展開時の型情報継承
dbg!(s.field)
//   ^ ここでの補完で &field が欠落
//     なぜなら、展開後の文脈で期待される型が
//     正しく推論されていないため
```

### 🔧 Technical Challenges

1. **🌊 Span Mapping**: オリジナルコードと展開後コードの対応
2. **📍 Context Preservation**: マクロ展開前後での型情報保持
3. **🎭 Multiple Expansions**: ネストしたマクロでの処理
4. **⚡ Performance**: 展開処理のオーバーヘッド最小化

## 🎯 学習の進め方

### Phase 1: マクロ基礎理解 🧩
1. `overview.md` で問題の技術的詳細を理解
2. `macro-deep-dive.md` でマクロ処理の深層を学習
3. `lessons/macro_basics.rs` でRustマクロの基礎を実践
4. `lessons/macro_expansion.rs` で展開メカニズムを学習

### Phase 2: 補完システム理解 💭
1. `completion-analysis.md` で補完エンジンを理解
2. `lessons/completion_context.rs` で補完文脈を学習
3. `lessons/token_tree_analysis.rs` でTokenTree解析を学習
4. 実際のrust-analyzerコードで動作を確認

### Phase 3: コードベース解析 🔍
1. `code-analysis.md` で関連コードを特定
2. hir-expand crate の詳細調査
3. ide-completion crate の補完ロジック理解
4. 既存のマクロ関連バグ修正を参考にする

### Phase 4: 実装・検証 🚀
1. `implementation-strategy.md` で実装ステップを確認
2. 最小限のプロトタイプから開始
3. `verification.md` の方法で動作確認
4. 包括的なテストケースで検証

## 💡 成功のコツ

### ✅ Do's
- **🔬 段階的理解**: マクロ → 展開 → 補完の順で学習
- **🧪 実験重視**: 小さなテストケースで動作確認
- **📊 ログ活用**: デバッグ出力で展開過程を可視化
- **🤝 既存パターン活用**: 類似のマクロ処理を参考にする

### ❌ Don'ts
- マクロ展開の複雑さを過小評価しない
- 補完文脈の微細な差異を見逃さない
- パフォーマンスへの影響を無視しない
- エッジケース（ネストマクロ等）を軽視しない

## 🎯 最終目標

```rust
struct Data { field: String }

fn test() {
    let data = Data { field: "hello".to_string() };
    
    // ✅ 修正後：マクロ内でも &field が補完候補に表示される
    dbg!(data.);  // -> `field`, `&field` 両方が候補に
    println!("{}", data.);  // -> `field`, `&field` 両方が候補に
    format!("{}", data.);   // -> `field`, `&field` 両方が候補に
}
```

**🌟 rust-analyzerのマクロ補完を改善し、すべてのRust開発者により一貫した開発体験を提供しましょう！**

## 🚨 注意事項

### ⚠️ 技術的な複雑さ
- マクロ展開は rust-analyzer の最も複雑な部分の一つ
- span mapping やトークン位置の追跡が困難
- 他の機能への影響を慎重に評価する必要

### 🎯 スコープの明確化
- 最初は `dbg!` マクロに限定して実装
- 動作確認後に他のマクロに拡張
- エラーハンドリングを最初から組み込む

---

📞 **サポート**: マクロは複雑な分野です。実装中に詰まったら、各ファイルの詳細説明を参照するか、rust-analyzer コミュニティで積極的に質問してください。一歩一歩着実に進めることが成功の鍵です！