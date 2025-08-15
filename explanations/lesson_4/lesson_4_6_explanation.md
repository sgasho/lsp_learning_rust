# Lesson 4-6: 関数抽出機能

lesson_4_5で変数リネーム機能ができるようになりましたね。今度は、**関数抽出機能**を学びます。

## 🎯 なぜ関数抽出機能？

関数抽出機能は**rust-analyzerの最重要リファクタリング機能**です：

- **最頻使用**: 開発者が最も使うリファクタリング機能
- **複雑性**: 変数スコープ、パラメータ、戻り値の高度な解析
- **実用性**: コードの可読性と保守性を劇的に改善
- **rust-analyzer貢献**: この機能の理解で実際のPRが書けるレベルに

### 🔍 機能例

```rust
// 抽出前のコード
fn main() {
    let a = 10;
    let b = 20;
    
    // この部分を選択して関数抽出
    let result = a + b;
    let doubled = result * 2;
    println!("Result: {}", doubled);
}

// 抽出後のコード  
fn main() {
    let a = 10;
    let b = 20;
    
    calculate_and_print(a, b);  // ← 抽出された関数呼び出し
}

fn calculate_and_print(a: i32, b: i32) {  // ← 新しく生成された関数
    let result = a + b;
    let doubled = result * 2;
    println!("Result: {}", doubled);
}
```

## 🏗️ 実装アーキテクチャ

### 📦 新しい概念: 変数使用解析

関数抽出では**変数の読み取り・書き込みパターン**を詳細に分析します：

```rust
// 変数の使用情報
#[derive(Debug, Clone)]
pub struct VariableUsage {
    pub name: String,
    pub is_read: bool,      // 変数が読み取られるか
    pub is_written: bool,   // 変数が書き込まれるか
    pub first_use_span: Span,
}

// 使用パターンの分類:
// 1. 読み取り専用 (is_read=true, is_written=false) → 関数パラメータ
// 2. 書き込み専用 (is_read=false, is_written=true) → 戻り値
// 3. 読み書き両方 (is_read=true, is_written=true) → パラメータ + 戻り値
```

### 🔧 5つのフェーズ（最も複雑な処理）

```rust
impl FunctionExtractor {
    pub fn extract_function(&mut self, ...) -> ExtractResult {
        // Phase 1: 変数使用パターンを分析
        self.analyze_variable_usage(code_block);
        
        // Phase 2: 抽出可能性をチェック
        if let Some(error) = self.check_extractability(code_block) {
            return ExtractResult::with_error(error, ...);
        }
        
        // Phase 3: 関数シグネチャを生成
        let signature = self.generate_function_signature(&function_name);
        
        // Phase 4: 関数本体を生成
        let body = self.generate_function_body(code_block);
        
        // Phase 5: リファクタリング操作を生成  ← あなたが実装
        self.generate_extract_edits(...)
    }
}
```

## 💡 実装のポイント

### 🎯 実装箇所: 編集操作生成

**場所**: `generate_extract_edits()` メソッド

```rust
fn generate_extract_edits(
    &self,
    code_block: &CodeBlock,
    signature: String,
    body: String,
    function_name: String,
) -> ExtractResult {
    // todo!("関数抽出の編集操作を実装してください")
    // ヒント：
    // 1. 抽出された関数の定義を生成
    // 2. 元のコードブロックを関数呼び出しに置換
    // 3. 必要な引数と戻り値の処理
    // 4. TextEditを作成してExtractResultに追加
    // 5. 生成された関数名を設定
}
```

**実装すべき内容**: 
1. **新しい関数定義**をコードの適切な位置に挿入
2. **元のコードブロック**を関数呼び出しに置換
3. **引数**と**戻り値**を適切に処理

## 🧠 変数使用解析の仕組み

### 🔍 パターン分析例

```rust
// 例1: 読み取り専用変数
let external = 42;
let result = external + 10;  // external は読み取り専用 → パラメータ

// 例2: 書き込み変数
let mut calculated = 0;
calculated = a + b;  // calculated は書き込み → 戻り値

// 例3: 読み書き両方
let mut counter = 0;
counter = counter + 1;  // counter は読み書き両方 → パラメータ + 戻り値
```

### 📊 関数シグネチャ生成

```rust
fn generate_function_signature(&self, function_name: &str) -> String {
    let mut params = Vec::new();
    let mut return_vars = Vec::new();

    for (var_name, usage) in &self.variable_usages {
        if usage.is_read && !usage.is_written {
            // 読み取り専用 → パラメータ
            params.push(format!("{}: i32", var_name));
        } else if usage.is_written {
            // 書き込み → 戻り値候補
            return_vars.push(var_name.clone());
        }
    }

    // 例: fn calculate(a: i32, b: i32) -> i32
    // 例: fn process(input: i32) -> (i32, i32)  // 複数戻り値
}
```

## 🔄 lesson_4_5との比較

### 共通点（継承された概念）
- ✅ **TextEdit**: コード変更操作の表現
- ✅ **結果構造**: 成功・失敗の統合処理
- ✅ **位置ベース処理**: Spanを使った精密な位置管理

### 新しい複雑さ
- 🔄 **多段階解析**: 5つのフェーズによる段階的処理
- 🔄 **変数フロー解析**: 読み取り・書き込みパターンの追跡
- 🔄 **コード生成**: 新しい関数の完全な生成
- 🔄 **複数編集**: 2箇所の同時編集（関数定義 + 呼び出し）

### 実装の違い

```rust
// lesson_4_5: 単純な置換
old_text → new_text

// lesson_4_6: 複雑な変換
selected_code_block → {
    1. new_function_definition,
    2. function_call_with_params_and_return
}
```

## 📚 実際のrust-analyzerでの例

### 🎯 よくある抽出パターン

```rust
// パターン1: 計算処理の抽出
fn main() {
    let width = 10;
    let height = 20;
    
    // 選択範囲
    let area = width * height;
    let perimeter = 2 * (width + height);
    println!("Area: {}, Perimeter: {}", area, perimeter);
    // 選択範囲終了
}

// 抽出後
fn main() {
    let width = 10; 
    let height = 20;
    
    calculate_geometry(width, height);
}

fn calculate_geometry(width: i32, height: i32) {
    let area = width * height;
    let perimeter = 2 * (width + height);
    println!("Area: {}, Perimeter: {}", area, perimeter);
}
```

```rust
// パターン2: 戻り値がある抽出
fn process_data() {
    let input = get_input();
    
    // 選択範囲
    let processed = input * 2 + 5;
    let result = processed.min(100);
    // 選択範囲終了
    
    save_result(result);
}

// 抽出後
fn process_data() {
    let input = get_input();
    
    let result = transform_input(input);
    
    save_result(result);
}

fn transform_input(input: i32) -> i32 {
    let processed = input * 2 + 5;
    let result = processed.min(100);
    result
}
```

## ✅ 実装手順

1. **lesson_4_6.rs** の `todo!()` を実装
2. **テスト実行**: `cargo test lesson_4::lesson_4_6`
3. **4つのテスト**をすべてパス

## 🎯 テストケース

1. **単純な式抽出**: 基本的な計算処理の抽出
2. **複数文抽出**: 複数の文を含む複雑な処理
3. **空ブロック**: エラーハンドリングの確認
4. **変数使用解析**: 読み取り・書き込みパターンの検証

## 🎉 完了後の効果

lesson_4_6が完了すると：
- **高度なリファクタリング**: 実用的な関数抽出機能の理解
- **コード生成**: 新しいコードの自動生成スキル
- **変数フロー解析**: 複雑な変数依存関係の処理
- **rust-analyzer準備**: 実際のPRが書けるレベルに到達

## 🔄 学習の完成

```
lesson_4シリーズの集大成:

診断機能（lesson_4_1-4_4）:
問題の発見と警告
    +
リファクタリング機能（lesson_4_5-4_6）:
コードの改善と変更
    =
完全なIDE機能理解  ← rust-analyzer貢献準備完了
```

**lesson_4_6完成により、あなたはrust-analyzerのコア機能を完全に理解し、実際に貢献できるレベルに到達します！**