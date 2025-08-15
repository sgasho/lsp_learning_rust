# Lesson 4-5: 変数リネーム機能

lesson_4_4で診断機能シリーズが完成しましたね。今度は、**変数リネーム機能**を学びます。

## 🎯 なぜ変数リネーム機能？

変数リネーム機能は**rust-analyzerの重要なIDE機能**です：

- **リファクタリング支援**: 安全な変数名変更
- **頻繁な使用**: 開発者が毎日使う機能
- **複雑性**: スコープと衝突を考慮した高度な処理
- **実用性**: IDEの価値を決める重要機能

### 🔍 機能例

```rust
fn example() {
    let old_name = 42;     // ← ここでリネーム要求
    let result = old_name + 10;
    println!("{}", old_name);
}

// リネーム後（"new_name"に変更）
fn example() {
    let new_name = 42;     // ← 自動変更
    let result = new_name + 10;  // ← 自動変更
    println!("{}", new_name);    // ← 自動変更
}
```

## 🏗️ 実装アーキテクチャ

### 📦 新しい概念: TextEdit

リファクタリング機能では**テキスト編集操作**を扱います：

```rust
// テキスト編集操作
#[derive(Debug, Clone, PartialEq)]
pub struct TextEdit {
    pub span: Span,        // 編集対象の範囲
    pub new_text: String,  // 新しいテキスト
}

// リネーム結果
#[derive(Debug, Clone, PartialEq)]
pub struct RenameResult {
    pub edits: Vec<TextEdit>,          // 実行すべき編集操作
    pub diagnostics: Vec<Diagnostic>,  // エラーがあれば診断情報
}
```

### 🔧 スコープ付きAST

変数のスコープを正確に追跡するため、ASTにスコープIDを追加：

```rust
// スコープ情報を含む拡張された式
#[derive(Debug, Clone, PartialEq)]
pub enum ScopedExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier {
        name: String,
        span: Span,
        scope_id: usize,  // 新規追加：スコープID
    },
}

// スコープ情報を含む拡張された文
#[derive(Debug, Clone, PartialEq)]
pub enum ScopedStmt {
    LetDeclaration {
        name: String,
        value: ScopedExpr,
        span: Span,
        scope_id: usize,  // 新規追加：スコープID
    },
    // その他...
}
```

### 🔧 4つのフェーズ（新しい構造）

```rust
impl VariableRenamer {
    pub fn rename_variable(&mut self, ...) -> RenameResult {
        // Phase 1: 変数定義と使用箇所を収集
        self.collect_variables(program);
        
        // Phase 2: リネーム対象の変数を見つける
        if let Some(target_var) = self.find_target_variable(&target_position) {
            // Phase 3: リネームが安全かチェック
            if let Some(conflict) = self.check_rename_conflicts(&target_var, &new_name) {
                return RenameResult::with_error(conflict, ...);
            }
            
            // Phase 4: リネーム操作を生成
            self.generate_rename_edits(&target_var, new_name)  // ← あなたが実装
        }
    }
}
```

## 💡 実装のポイント

### 🎯 実装箇所: 衝突チェック

**場所**: `check_rename_conflicts()` メソッド

```rust
fn check_rename_conflicts(
    &self,
    target_var: &VariableDefinition,
    new_name: &str,
) -> Option<String> {
    // todo!("リネームの衝突チェックを実装してください")
    // ヒント：
    // 1. 同じスコープに同じ名前の変数がないかチェック
    // 2. self.variables から new_name の変数定義を探す
    // 3. target_var.scope_id と同じスコープの定義があれば衝突
    // 4. 衝突がある場合は適切なエラーメッセージを返す
    // 5. 衝突がない場合は None を返す
}
```

**考えるポイント**: 
- `self.variables.get(new_name)`で新しい名前の変数を検索
- 同じ`scope_id`の定義があるかチェック
- 衝突がある場合は`Some("Variable 'new_name' already exists in this scope".to_string())`
- 衝突がない場合は`None`

### 🧠 変数の追跡システム

```rust
// 変数の定義情報
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub name: String,              // 変数名
    pub definition_span: Span,     // 定義箇所
    pub scope_id: usize,           // スコープID
    pub usages: Vec<Span>,         // 使用箇所一覧
}
```

これにより、変数の**定義**と**全ての使用箇所**を一元管理できます。

## 🔍 スコープベースの衝突検出

### 🎯 衝突するケース

```rust
fn example() {
    let existing = 1;      // scope_id: 0
    let target = 2;        // scope_id: 0  ← これを"existing"にリネーム
    // 衝突！同じスコープに"existing"が既にある
}
```

### ✅ 衝突しないケース

```rust
fn example() {
    let target = 1;        // scope_id: 0
    {
        let existing = 2;  // scope_id: 1  ← 異なるスコープ
    }
    // 衝突なし！異なるスコープなので安全
}
```

## 🔄 lesson_4_1-4_4からの大きな変化

### 診断機能との違い

```
診断機能（lesson_4_1-4_4）:
- 目的: 問題の検出と警告
- 出力: Diagnostic（警告メッセージ）
- 処理: 静的解析のみ

リファクタリング機能（lesson_4_5-）:
- 目的: コードの変更と改善
- 出力: TextEdit（実際の編集操作）
- 処理: 静的解析 + コード変更
```

### 新しい複雑さ

- ✅ **位置ベース検索**: カーソル位置から変数を特定
- ✅ **衝突検出**: スコープを考慮した名前衝突チェック
- ✅ **編集操作生成**: 実際のテキスト変更操作の作成
- ✅ **結果の構造化**: 成功時の編集と失敗時の診断を統合

## 📊 Position-basedな検索

```rust
// カーソル位置から変数を見つける
fn find_target_variable(&self, target_position: &Position) -> Option<&VariableDefinition> {
    for var_defs in self.variables.values() {
        for var_def in var_defs {
            // 定義箇所がターゲット位置にあるかチェック
            if self.position_in_span(target_position, &var_def.definition_span) {
                return Some(var_def);
            }
            // 使用箇所がターゲット位置にあるかチェック
            for usage_span in &var_def.usages {
                if self.position_in_span(target_position, usage_span) {
                    return Some(var_def);
                }
            }
        }
    }
    None
}
```

この機能により、IDEでユーザーがクリックした位置から正確に変数を特定できます。

## ✅ 実装手順

1. **lesson_4_5.rs** の `todo!()` を実装
2. **テスト実行**: `cargo test lesson_4::lesson_4_5`
3. **4つのテスト**をすべてパス

## 🎯 テストケース

1. **単純なリネーム**: 基本的な変数リネーム
2. **衝突検出**: 同じスコープでの名前衝突
3. **複数使用**: 複数箇所で使用される変数のリネーム
4. **変数未発見**: 指定位置に変数がない場合

## 📚 実際のrust-analyzerでの例

```rust
// よくあるリネームのケース

fn process_data() {
    let data = vec![1, 2, 3];    // ← "data"を"items"にリネーム
    
    for item in data.iter() {    // ← 自動で"items.iter()"に変更
        println!("{}", item);
    }
    
    let len = data.len();        // ← 自動で"items.len()"に変更
}

// 衝突するケース
fn example() {
    let items = vec![1, 2, 3];
    let data = vec![4, 5, 6];    // ← これを"items"にリネームしようとする
    // エラー：'items' already exists in this scope
}

// スコープが異なる安全なケース  
fn example() {
    let data = 42;               // ← これを"temp"にリネーム
    {
        let temp = 100;          // 異なるスコープなので衝突しない
        println!("{}", temp);
    }
    println!("{}", data);        // 自動で"temp"に変更
}
```

## 🎉 完了後の効果

lesson_4_5が完了すると：
- **リファクタリング支援**: IDEの重要機能を理解
- **位置ベース検索**: カーソル位置からの要素特定
- **編集操作生成**: 実際のコード変更の作成
- **スコープ管理**: より複雑な変数スコープ処理

## 🔄 学習の進化

```
診断機能シリーズ（lesson_4_1-4_4）:
問題の検出と警告生成
    ↓
リファクタリング支援シリーズ（lesson_4_5-）:
コードの改善と変更支援  ← より実用的で複雑
```

**rust-analyzerの利用者が最も頻繁に使う機能の実装を学びます！**