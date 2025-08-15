# Lesson 4-3: 未使用関数検出

lesson_4_2で未使用インポート検出ができるようになりましたね。今度は、**未使用関数検出**を学びます。

## 🎯 なぜ未使用関数検出？

未使用関数検出は**rust-analyzerの実用診断機能の第3弾**です：

- **コードの整理**: デッドコードの除去で保守性向上
- **パフォーマンス**: 不要なコンパイルを回避
- **設計改善**: 使われていない機能の発見
- **理解しやすい**: lesson_4_1, 4_2の自然な発展

### 🔍 検出例

```rust
fn unused_function() -> i32 {  // ← 警告: 未使用
    42
}

fn used_function() -> i32 {    // ← 使用済み
    10
}

fn main() {
    let result = used_function();  // used_functionを使用
    println!("{}", result);
}
```

## 🏗️ 実装アーキテクチャ

### 📦 ASTの拡張

lesson_4_3では、関数定義と関数呼び出しを扱うためにASTを拡張します：

```rust
// 拡張された式（関数呼び出しを含む）
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier(String, Span),
    FunctionCall {           // 新規追加
        name: String,
        arguments: Vec<ExtendedExpr>,
        span: Span,
    },
}

// 拡張された文（関数定義を含む）
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedStmt {
    LetDeclaration { name: String, value: ExtendedExpr, span: Span },
    Expression(ExtendedExpr),
    FunctionDeclaration {    // 新規追加
        name: String,
        parameters: Vec<String>,
        body: Vec<ExtendedStmt>,
        span: Span,
    },
}
```

### 🔧 3つのフェーズ（lesson_4_1, 4_2と同じ構造）

```rust
impl UnusedFunctionChecker {
    pub fn check(&mut self, program: &ExtendedProgram) -> Vec<Diagnostic> {
        // Phase 1: 関数定義を収集
        self.collect_functions(program);
        
        // Phase 2: 関数使用を追跡  
        self.track_function_usage(program);
        
        // Phase 3: 未使用関数の診断を生成
        self.generate_unused_function_diagnostics();  // ← あなたが実装
    }
}
```

## 💡 実装のポイント

### 🎯 実装箇所: 診断生成

**場所**: `generate_unused_function_diagnostics()` メソッド

```rust
fn generate_unused_function_diagnostics(&mut self) {
    // todo!("未使用関数の診断を実装してください")
    // ヒント：
    // 1. self.functions をイテレート
    // 2. is_used が false の関数を見つける
    // 3. main関数は除外する（is_main フラグをチェック）
    // 4. 未使用関数の警告を生成
    // 5. self.diagnostics に追加
    // 6. DiagnosticCategory::UnusedVariable を使用（関数専用カテゴリがないため）
}
```

**考えるポイント**: 
- `function_info.is_main` による main関数の除外
- 適切なメッセージの作成（`"unused function `{}`"`）
- `DiagnosticCategory::UnusedVariable` の使用（関数も変数の一種として扱う）
- エラーコードの追加（`with_code("unused_function".to_string())`）

### 🚨 重要な特殊ケース: main関数

main関数は常に使用済みとして扱います：

```rust
impl FunctionInfo {
    pub fn new(name: String, definition_span: Span) -> Self {
        let is_main = name == "main";
        FunctionInfo {
            name,
            definition_span,
            is_used: is_main,  // main関数は最初から使用済み
            is_main,
        }
    }
}
```

### 🔍 関数呼び出しの検出

lesson_4_1, 4_2との最大の違いは、**ネストした関数呼び出し**の処理です：

```rust
fn track_usage_in_expr(&mut self, expr: &ExtendedExpr) {
    match expr {
        ExtendedExpr::FunctionCall { name, arguments, .. } => {
            // 関数呼び出しを検出
            if let Some(function_info) = self.functions.get_mut(name) {
                function_info.is_used = true;
            }
            // 引数内の関数呼び出しも再帰的に追跡
            for arg in arguments {
                self.track_usage_in_expr(arg);  // 再帰呼び出し
            }
        }
        // その他の式...
    }
}
```

## 🔍 lesson_4_1, 4_2からの進化

### 共通パターン（変わらない部分）
- ✅ **3フェーズ構造**: 収集 → 追跡 → 診断
- ✅ **使用状況追跡**: `is_used` フラグパターン
- ✅ **HashMap管理**: 名前をキーとした効率的な検索
- ✅ **診断生成**: 同じパターンの警告作成

### 新しい要素（追加された部分）
- 🔄 **ASTの拡張**: 関数定義と関数呼び出しの追加
- 🔄 **再帰的追跡**: 関数本体内と引数内の関数呼び出し
- 🔄 **特殊ケース**: main関数の特別扱い
- 🔄 **ネストした構造**: 関数本体内のstatement処理

### 実装の違い

```rust
// lesson_4_1: 変数使用
Expr::Identifier(name, _) => {
    if let Some(symbol) = self.symbols.get_mut(name) {
        symbol.is_used = true;
    }
}

// lesson_4_2: インポート使用
Expr::Identifier(name, _) => {
    if let Some(import_info) = self.imports.get_mut(name) {
        import_info.is_used = true;
    }
}

// lesson_4_3: 関数使用（新しい複雑さ）
ExtendedExpr::FunctionCall { name, arguments, .. } => {
    if let Some(function_info) = self.functions.get_mut(name) {
        function_info.is_used = true;
    }
    // 引数も再帰的に処理
    for arg in arguments {
        self.track_usage_in_expr(arg);
    }
}
```

## ✅ 実装手順

1. **lesson_4_3.rs** の `todo!()` を実装
2. **テスト実行**: `cargo test lesson_4::lesson_4_3`
3. **4つのテスト**をすべてパス

## 🎯 テストケース

1. **基本検出**: 未使用関数と使用済み関数の区別、main関数の除外
2. **全て使用**: 診断が空になることを確認
3. **複数未使用**: 複数の未使用関数を検出
4. **ネスト呼び出し**: 関数が他の関数を呼ぶ複雑なケース

## 🔄 学習効果の積み重ね

### lesson_4_1（変数）→ lesson_4_2（インポート）→ lesson_4_3（関数）

```
複雑さの進化:
単純な識別子 → 単純な識別子 → 関数呼び出し + 再帰

lesson_4_1: Expr::Identifier(name)
    ↓ 同じパターン
lesson_4_2: Expr::Identifier(name) 
    ↓ 複雑化
lesson_4_3: ExtendedExpr::FunctionCall { name, arguments }
```

### パターンの理解度向上

- ✅ **基本パターン**: 3フェーズ診断システム
- ✅ **応用力**: 異なる対象への同じアルゴリズム適用
- ✅ **拡張性**: ASTの拡張と処理の追加
- ✅ **実用性**: 実際のrust-analyzerに近い複雑さ

## 📚 実際のrust-analyzerでの例

```rust
// よくある未使用関数のケース

// ケース1: 開発中に作ったが使わなかった
fn calculate_complex_value() -> f64 {  // ← 未使用警告
    // 複雑な計算...
    42.0
}

// ケース2: テスト用だが#[cfg(test)]を忘れた
fn test_helper() {  // ← 未使用警告
    // テスト用の処理...
}

// ケース3: API設計で用意したが実際には使用されていない
pub fn public_but_unused() {  // ← 未使用警告（publicでも未使用なら警告）
    println!("This function is never called");
}

fn main() {
    println!("Hello, world!");
    // calculate_complex_value, test_helper, public_but_unused は呼ばれない
}
```

## 🎉 完了後の効果

lesson_4_3が完了すると：
- **複雑なAST処理**: 再帰的な構造の処理スキル
- **実用的診断**: より高度な診断機能の理解
- **rust-analyzer準備**: 実際のコードベースに近い複雑さの経験

**lesson_4_1-4_3で、rust-analyzerの診断機能の核心が完成します！**