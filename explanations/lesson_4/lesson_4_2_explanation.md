# Lesson 4-2: 未使用インポート検出

lesson_4_1で未使用変数検出ができるようになりましたね。今度は、**未使用インポート検出**を学びます。

## 🎯 なぜ未使用インポート検出？

未使用インポート検出は**rust-analyzerで2番目によく貢献される機能**です：

- **頻繁なPR**: rust-analyzerのPRの約15%が未使用インポート関連
- **コードの整理**: 不要なインポートを除去してクリーンなコードに
- **コンパイル高速化**: 不要なインポートはコンパイル時間に影響
- **理解しやすい**: lesson_4_1の自然な拡張

### 🔍 検出例

```rust
use std::collections::HashMap;  // ← 警告: 未使用
use std::vec::Vec;              // ← 使用済み

fn main() {
    let data = Vec::new();      // Vec を使用
    // HashMap は使用されていない
}
```

## 🏗️ 実装アーキテクチャ

### 📦 拡張されたAST

```rust
// 新規追加：インポート文
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module_name: String,     // "std::collections"
    pub imported_name: String,   // "HashMap"
    pub span: Span,              // 位置情報
}

// 拡張されたプログラム
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramWithImports {
    pub imports: Vec<Import>,     // インポート一覧
    pub statements: Vec<Stmt>,    // 従来の文
}
```

### 🔧 3つのフェーズ（lesson_4_1と同じ構造）

```rust
impl UnusedImportChecker {
    pub fn check(&mut self, program: &ProgramWithImports) -> Vec<Diagnostic> {
        // Phase 1: インポートを収集
        self.collect_imports(program);
        
        // Phase 2: インポートの使用を追跡
        self.track_import_usage(program);
        
        // Phase 3: 未使用インポートの診断を生成
        self.generate_unused_import_diagnostics();  // ← あなたが実装
    }
}
```

## 💡 実装のポイント

### 🎯 実装箇所: 診断生成

**場所**: `generate_unused_import_diagnostics()` メソッド

```rust
fn generate_unused_import_diagnostics(&mut self) {
    // todo!("未使用インポートの診断を実装してください")
    // ヒント：
    // 1. self.imports をイテレート
    // 2. is_used が false のインポートを見つける
    // 3. 未使用インポートの警告を生成
    // 4. self.diagnostics に追加
    // 5. DiagnosticCategory::UnusedImport を使用
}
```

**考えるポイント**: 
- `Diagnostic::warning()` の使用
- 適切なメッセージの作成（`"unused import `{}`"`）
- `DiagnosticCategory::UnusedImport` の指定
- `import.span` の位置情報使用

## 🔍 lesson_4_1からの違い

### 類似点（同じ3フェーズ構造）
- ✅ **Phase 1**: 定義収集（変数 → インポート）
- ✅ **Phase 2**: 使用追跡（変数使用 → インポート使用）  
- ✅ **Phase 3**: 診断生成（未使用警告）

### 相違点（対象が変わる）
- 🔄 **対象**: 変数定義 → インポート文
- 🔄 **使用検出**: 変数名 → インポート名
- 🔄 **データ構造**: `Symbol` → `ImportInfo`
- 🔄 **診断カテゴリ**: `UnusedVariable` → `UnusedImport`

### 実装の変化

```rust
// lesson_4_1: 変数収集
Stmt::LetDeclaration { name, .. } => {
    let symbol = Symbol::new(name.clone(), span.clone());
    self.symbols.insert(name.clone(), symbol);
}

// lesson_4_2: インポート収集  
for import in &program.imports {
    let import_info = ImportInfo::new(import.clone());
    self.imports.insert(import.imported_name.clone(), import_info);
}
```

```rust
// lesson_4_1: 変数使用追跡
Expr::Identifier(name, _) => {
    if let Some(symbol) = self.symbols.get_mut(name) {
        symbol.is_used = true;
    }
}

// lesson_4_2: インポート使用追跡
Expr::Identifier(name, _) => {
    if let Some(import_info) = self.imports.get_mut(name) {
        import_info.is_used = true;
    }
}
```

## ✅ 実装手順

1. **lesson_4_2.rs** の `todo!()` を実装
2. **テスト実行**: `cargo test lesson_4::lesson_4_2`
3. **3つのテスト**をすべてパス

## 🎯 テストケース

1. **基本検出**: 未使用インポートと使用済みインポートの区別
2. **全て使用**: 診断が空になることを確認
3. **複数未使用**: 複数の未使用インポートを検出

## 📚 実際のrust-analyzerでの例

```rust
// よくある未使用インポートのケース

// ケース1: 開発中に追加したが結局使わなかった
use std::collections::HashMap;  // ← 未使用警告

// ケース2: リファクタリングで不要になった
use serde::{Deserialize, Serialize};  // ← 未使用警告

// ケース3: 条件付きコンパイルで一部のみ使用
#[cfg(test)]
use mockall::predicate::*;     // ← testビルドでのみ使用

fn main() {
    // HashMap も serde も使用されていない
    println!("Hello, world!");
}
```

## 🔄 lesson_4_1との学習効果

### 共通パターンの理解
- ✅ **3フェーズ構造**: 収集 → 追跡 → 診断
- ✅ **使用状況追跡**: `is_used` フラグパターン
- ✅ **HashMap管理**: 名前をキーとした効率的な検索

### 応用力の向上
- ✅ **AST拡張**: 新しい文法要素の追加方法
- ✅ **診断カテゴリ**: 機能別の診断分類
- ✅ **パターン適用**: 同じアルゴリズムの異なる対象への適用

## 🎉 完了後の効果

lesson_4_2が完了すると：
- **応用力**: 同様の診断機能を自分で設計できる
- **rust-analyzer準備**: 実際のPRで貢献しやすくなる
- **コード品質**: 不要なインポートを自動検出する重要性の理解

**lesson_4_1と合わせて、rust-analyzerの診断機能の基礎が完成します！**