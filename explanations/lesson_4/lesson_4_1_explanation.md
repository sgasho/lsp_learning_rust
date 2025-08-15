# Lesson 4-1: 未使用変数検出

lesson_3でセマンティック解析ができるようになりましたね。今度は、**未使用変数検出**を学びます。

## 🎯 なぜ未使用変数検出？

未使用変数検出は**rust-analyzerで最も貢献しやすい機能**です：

- **頻繁なPR**: rust-analyzerのPRの約20%が未使用変数関連
- **理解しやすい**: アルゴリズムがシンプル
- **即効性**: ユーザーがすぐに恩恵を受ける

### 🔍 検出例

```rust
let unused_variable = 42;  // ← 警告: 未使用
let used_variable = 10;
println!("{}", used_variable);  // ← 使用済み
```

## 🏗️ 実装アーキテクチャ

### 📦 使用する共通モジュール

```rust
use super::common::{
    ast::{Program, Stmt, Expr},        // シンプルなAST
    diagnostic::Diagnostic,            // 診断システム
    span::{Position, Span},            // 位置情報
};
```

### 🔧 3つのフェーズ

```rust
impl UnusedVariableChecker {
    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        // Phase 1: 変数定義を収集
        self.collect_definitions(program);
        
        // Phase 2: 変数使用を追跡
        self.track_usage(program);
        
        // Phase 3: 未使用変数の診断を生成
        self.generate_unused_diagnostics();  // ← あなたが実装
    }
}
```

## 💡 実装のポイント

### 🎯 実装箇所: 診断生成

**場所**: `generate_unused_diagnostics()` メソッド

```rust
fn generate_unused_diagnostics(&mut self) {
    // todo!("未使用変数の診断を実装してください")
    // ヒント：
    // 1. self.symbols をイテレート
    // 2. is_used が false のシンボルを見つける
    // 3. 未使用変数の警告を生成
    // 4. self.diagnostics に追加
}
```

**考えるポイント**: 
- `Diagnostic::warning()` の使用
- 適切なメッセージの作成
- `DiagnosticCategory::UnusedVariable` の指定

## ✅ 実装手順

1. **lesson_4_1.rs** の `todo!()` を実装
2. **テスト実行**: `cargo test lesson_4::lesson_4_1`
3. **3つのテスト**をすべてパス

## 🎯 テストケース

1. **基本検出**: 未使用変数と使用済み変数の区別
2. **全て使用**: 診断が空になることを確認
3. **複数未使用**: 複数の未使用変数を検出

**この簡潔な実装で、rust-analyzerの核心機能を理解しましょう！**