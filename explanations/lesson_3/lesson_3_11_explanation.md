# Lesson 3-11: エラー回復とエラー報告

lesson_3_10で型推論の高度化ができるようになりましたね。今度は、**エラー回復とエラー報告**を学びます。

## 📚 エラー回復とは？

### 🤔 なぜエラー回復が重要？

今までのlessonでは、1つのエラーが見つかるとすぐに解析を停止していました。しかし、実用的なIDEでは**複数のエラーを同時に表示**する必要があります：

```rust
// 従来（lesson_3_10まで）: 最初のエラーで停止
let x = undefined_var;  // Error: 解析停止 → 他のエラーは見えない
let y: i32 = true;      // このエラーは報告されない
let z = unknown_func(); // このエラーも報告されない

// 理想（lesson_3_11）: すべてのエラーを収集
let x = undefined_var;  // Error 1: 変数が未定義
let y: i32 = true;      // Error 2: 型が不一致  
let z = unknown_func(); // Error 3: 関数が未定義
```

### 🎯 エラー回復の役割

エラー回復は「事故が起きても運転を続ける自動車」のようなものです：

1. **継続性**: 1つのエラーで止まらず解析を続行
2. **完全性**: すべてのエラーを収集して報告
3. **有用性**: 開発者に最大限の情報を提供

### rust-analyzerでの重要性

rust-analyzerは以下のためにエラー回復が必要です：
- **開発体験**: すべてのエラーを一度に表示
- **効率性**: コンパイル前にすべての問題を発見
- **LSP対応**: エディタでリアルタイムにエラー表示
- **生産性**: 1つずつ修正→再実行のサイクルを削減

## 🎯 今回の目標（エラー回復システム）

**入力**: 複数のエラーを含むプログラム
```rust
let x = undefined_var;     // Error 1: 未定義変数
let y: i32 = true;         // Error 2: 型不一致
let x = 42;                // Error 3: 重複定義
if 42 {                    // Error 4: 条件が非boolean
    let z = unknown_func(); // Error 5: 未定義関数
}
```

**出力**: すべてのエラーを位置情報付きで収集
```rust
Diagnostics [
    Error { message: "Variable 'undefined_var' not defined", line: 1, column: 8 },
    Error { message: "Type mismatch: expected i32, found bool", line: 2, column: 13 },
    Error { message: "Variable 'x' already defined", line: 3, column: 4 },
    Error { message: "If condition must be boolean", line: 4, column: 3 },
    Error { message: "Function 'unknown_func' not defined", line: 5, column: 12 }
]
```

### 🔍 今回学ぶ新機能

1. **位置情報の管理**: エラーがどこで発生したかを記録
2. **診断システム**: エラー・警告・情報の分類
3. **エラー回復**: エラーでも解析を継続
4. **診断レポート**: 人間が読みやすい形式で出力

## 🏗️ エラー回復システムの構造

### 📍 位置情報の表現

ソースコード内の位置を正確に記録します：

```rust
// 1つの位置（行・列番号）
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,    // 0-indexed
    pub column: usize,  // 0-indexed
}

// 位置の範囲（開始〜終了）
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}
```

例：`let x = 42;` の場合
- `let`: Span { start: (0, 0), end: (0, 3) }
- `x`: Span { start: (0, 4), end: (0, 5) }
- `42`: Span { start: (0, 8), end: (0, 10) }

### 🩺 診断システム

エラー・警告・情報を統一的に管理します：

```rust
// エラーの重要度
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,   // 🔴 エラー（コンパイル不可）
    Warning, // 🟡 警告（コンパイル可能だが問題あり）
    Info,    // 🔵 情報（ヒントや提案）
}

// 診断情報
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub code: Option<String>, // エラーコード（例：E0001）
}
```

### 🔧 AST への位置情報追加

すべてのASTノードに位置情報を追加します：

```rust
// Before (lesson_3_10まで)
pub enum Expr {
    Number(i64),
    Identifier(String),
    // ...
}

// After (lesson_3_11から)
pub enum Expr {
    Number(i64, Span),      // 位置情報を追加
    Identifier(String, Span), // 位置情報を追加
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
        span: Span,           // 位置情報を追加
    },
    // ...
}
```

これにより、「どの式でエラーが発生したか」を正確に特定できます。

## 🔧 エラー回復の仕組み（詳細解説）

### 🚨 従来のエラー処理（停止型）

lesson_3_10までの方式：

```rust
fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
    match stmt {
        Stmt::LetDeclaration { name, value, .. } => {
            let value_type = self.infer_expression_type(value)?; // エラーで即停止
            self.symbol_table.define(name.clone(), value_type)?; // エラーで即停止
            Ok(())
        }
    }
}
```

**問題点**: 最初のエラーで`?`により関数が終了し、後続のエラーは発見されない。

### 🛠️ エラー回復型の処理（継続型）

lesson_3_11の新方式：

```rust
fn check_statement(&mut self, stmt: &Stmt) {
    match stmt {
        Stmt::LetDeclaration { name, value, type_annotation, span } => {
            // Step 1: 値の型推論（エラーでも続行）
            let value_type = self.infer_expression_type(value);
            
            // Step 2: 型注釈チェック（エラーでも続行）
            if let Some(annotation) = type_annotation {
                if let Some(ref inferred) = value_type {
                    if inferred != annotation {
                        self.diagnostics.push(Diagnostic::error(
                            "Type mismatch".to_string(),
                            span.clone()
                        ));
                    }
                }
            }
            
            // Step 3: 変数定義（エラーでも続行）
            let final_type = value_type.unwrap_or(Type::Unknown);
            if let Err(diagnostic) = self.symbol_table.define(name.clone(), final_type, span.clone()) {
                self.diagnostics.push(diagnostic);
            }
        }
    }
}
```

**改善点**: エラーが発生しても処理を継続し、すべてのエラーを`diagnostics`に収集。

### 🔄 Option型を使った安全な継続

型推論でもエラー回復を適用：

```rust
fn infer_expression_type(&mut self, expr: &Expr) -> Option<Type> {
    match expr {
        Expr::Identifier(name, span) => {
            if let Some(symbol) = self.symbol_table.resolve(name) {
                Some(symbol.symbol_type.clone())  // 成功時
            } else {
                // エラーを記録するが、Noneを返して続行
                self.diagnostics.push(Diagnostic::error(
                    format!("Variable '{}' not defined", name),
                    span.clone()
                ));
                None
            }
        }
        Expr::Binary { left, right, operator, span } => {
            // 左右どちらがエラーでも可能な限り処理
            let left_type = self.infer_expression_type(left);
            let right_type = self.infer_expression_type(right);
            
            match (left_type, right_type) {
                (Some(l), Some(r)) => {
                    // 両方成功：通常の型チェック
                    self.check_binary_operation(l, r, operator, span)
                }
                _ => {
                    // どちらかが失敗：型チェックをスキップしてNone
                    None
                }
            }
        }
    }
}
```

## 🎬 実行例：エラー回復の流れ

### 📝 入力プログラム（複数エラー）

```rust
let x = undefined_var;  // Error 1: 未定義変数
let y: i32 = true;      // Error 2: 型不一致
let x = 42;             // Error 3: 重複定義
if x {                  // Error 4: 条件の型エラー  
    print("ok");
}
```

### 🔍 エラー回復プロセス

#### Step 1: `let x = undefined_var;`

```rust
💭 エラー回復型チェッカーの思考過程:
1. "値の型を推論しよう"
   → Identifier("undefined_var") 
   → resolve("undefined_var") → None
   → diagnostics.push(Error: "Variable 'undefined_var' not defined")
   → value_type = None

2. "型注釈はない → スキップ"

3. "変数を定義しよう"
   → final_type = Type::Unknown (デフォルト)
   → define("x", Type::Unknown) → OK

✅ 結果: エラー1個収集、解析は継続
```

#### Step 2: `let y: i32 = true;`

```rust
💭 エラー回復型チェッカーの思考過程:
1. "値の型を推論しよう"
   → Boolean(true) → Type::Boolean

2. "型注釈をチェックしよう"
   → annotation: Type::Integer
   → inferred: Type::Boolean
   → Integer != Boolean → diagnostics.push(Error: "Type mismatch")

3. "変数を定義しよう"
   → final_type = Type::Boolean (推論された型を使用)
   → define("y", Type::Boolean) → OK

✅ 結果: エラー1個追加収集、解析は継続
```

#### Step 3: `let x = 42;`

```rust
💭 エラー回復型チェッカーの思考過程:
1. "値の型を推論しよう"
   → Number(42) → Type::Integer

2. "型注釈はない → スキップ"

3. "変数を定義しよう"
   → define("x", Type::Integer) 
   → "x"は既に定義済み → diagnostics.push(Error: "already defined")

✅ 結果: エラー1個追加収集、解析は継続
```

#### Step 4: `if x { ... }`

```rust
💭 エラー回復型チェッカーの思考過程:
1. "条件の型を推論しよう"
   → Identifier("x") → resolve("x") → Type::Unknown (Step1で定義)

2. "if条件の型チェック"
   → Type::Unknown != Type::Boolean
   → diagnostics.push(Error: "If condition must be boolean")

3. "then_branchを処理しよう"
   → print("ok") を解析 (エラーなし)

✅ 結果: エラー1個追加収集、解析完了
```

### 📊 最終的な診断結果

```rust
Diagnostics [
    Error { 
        message: "Variable 'undefined_var' not defined", 
        span: Span { start: (0, 8), end: (0, 21) },
        code: "E0004"
    },
    Error { 
        message: "Type mismatch: expected Integer, found Boolean", 
        span: Span { start: (1, 13), end: (1, 17) },
        code: "E0002"
    },
    Error { 
        message: "Variable 'x' already defined in this scope", 
        span: Span { start: (2, 4), end: (2, 5) },
        code: "E0001"
    },
    Error { 
        message: "If condition must be boolean, found Unknown", 
        span: Span { start: (3, 3), end: (3, 4) },
        code: "E0003"
    }
]
```

## 💡 実装のポイント（あなたが書く部分）

今回実装する箇所は**4つ**です。エラー回復の考え方を理解して実装してください。

### 🎯 実装箇所1: エラー回復型の変数定義チェック

**場所**: `check_statement()` メソッド内の `Stmt::LetDeclaration` 部分

```rust
Stmt::LetDeclaration { name, value, type_annotation, span } => {
    // ヒント：
    // 1. 値の型を推論（エラーでも続行）
    // 2. 型注釈との一致チェック（エラーでも続行）
    // 3. 変数定義（重複チェック）
    // 4. すべてのエラーをdiagnosticsに追加

    todo!("エラー回復型の変数定義チェックを実装してください")
}
```

**考えるポイント**: `?`を使わず、エラーを収集しながら処理を続行する。

### 🎯 実装箇所2: エラー回復型のif文チェック

**場所**: `check_statement()` メソッド内の `Stmt::IfStatement` 部分

```rust
Stmt::IfStatement { condition, then_branch, else_branch, .. } => {
    // ヒント：
    // 1. 条件の型チェック（エラーでも続行）
    // 2. then_branchのチェック
    // 3. else_branchのチェック（もしあれば）

    todo!("エラー回復型のif文チェックを実装してください")
}
```

**考えるポイント**: 条件でエラーが出ても、branchの解析は続行する。

### 🎯 実装箇所3: エラー回復型の二項演算チェック

**場所**: `infer_expression_type()` メソッド内の `Expr::Binary` 部分

```rust
Expr::Binary { left, operator, right, span } => {
    // ヒント：
    // 1. 左右の型を推論（どちらがエラーでも続行）
    // 2. 両方の型が取得できた場合のみ演算子チェック
    // 3. エラーはdiagnosticsに追加

    todo!("エラー回復型の二項演算チェックを実装してください")
}
```

**考えるポイント**: 左の式でエラーが出ても、右の式の解析は実行する。

### 🎯 実装箇所4: エラー回復型の代入チェック

**場所**: `infer_expression_type()` メソッド内の `Expr::Assignment` 部分

```rust
Expr::Assignment { name, value, span } => {
    // ヒント：
    // 1. 変数の存在チェック
    // 2. 値の型推論
    // 3. 型の一致チェック
    // 4. エラーはdiagnosticsに追加、戻り値はOption<Type>

    todo!("エラー回復型の代入チェックを実装してください")
}
```

**考えるポイント**: 戻り値が`Option<Type>`になったことに注意。

## ✅ 実装の進め方

### Step 1: TODOを探す 🔍

lesson_3_11.rsファイルで `todo!("...")` の4箇所を探してください。

### Step 2: エラー回復型実装 ✏️

従来の`Result<(), String>`ではなく、エラーを収集しながら処理を続行する方式で実装してください。

### Step 3: テスト実行 🧪

```bash
cargo test lesson_3::lesson_3_11
```

## 🎯 テストケース（3つ）

1. **`test_multiple_errors_collection`**: 複数エラーの収集
2. **`test_error_recovery_in_expressions`**: 式でのエラー回復
3. **`test_if_condition_error_recovery`**: if文でのエラー回復

## 🔄 lesson_3_10からの進化

### lesson_3_10でできたこと
- ✅ 型推論の高度化
- ✅ 型注釈なしの変数定義
- ✅ 代入式のサポート

### lesson_3_11で新しく追加されること
- ✅ **位置情報管理**: エラーの正確な位置を記録
- ✅ **診断システム**: エラー・警告・情報の分類
- ✅ **エラー回復**: 1つのエラーで停止せず継続
- ✅ **複数エラー収集**: すべてのエラーを一度に報告

### 変更されないもの
- ✅ 基本的な型システム
- ✅ スコープ管理
- ✅ 関数管理

## 🎉 完了後の効果

lesson_3_11が完了すると：
- **実用的なIDE機能**: 複数エラーの同時表示
- **開発体験の向上**: すべての問題を一度に発見
- **rust-analyzerの基盤**: エラー報告システムの理解

**次のステップ**: lesson_3_12で構造体とフィールドアクセスを学習し、より実用的なコード解析に進みます！

**エラー回復は複雑ですが、実用的なIDEには必須の機能です。頑張ってください！**