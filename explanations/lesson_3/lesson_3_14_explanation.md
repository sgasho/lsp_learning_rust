# Lesson 3-14: 借用チェッカーの基本

lesson_3_13でライフタイムの基本ができるようになりましたね。今度は、**借用チェッカーの基本**を学びます。

## 📚 借用チェッカーとは？

### 🤔 なぜ借用チェッカーが重要？

借用チェッカーは、**メモリ安全性を保証するRustの核心的機能**です。複数の参照が同じデータにアクセスする際の競合状態を防ぎます：

```rust
// 危険なコード（Rustでは許されない）
let mut x = 42;
let r1 = &x;           // 不変借用
let r2 = &mut x;       // 可変借用 → コンパイルエラー！
println!("{}", *r1);   // r1を使用（データレースの可能性）
*r2 = 100;             // r2で変更（競合状態）

// 安全なコード
let mut x = 42;
let r1 = &x;           // 不変借用
// let r2 = &mut x;    // これはコンパイラが拒否
println!("{}", *r1);   // 安全に使用可能
```

### 🎯 借用ルール（Rust's Borrowing Rules）

Rustの借用システムには3つの基本ルールがあります：

1. **同時には、1つの可変借用 OR 複数の不変借用のみ**
2. **借用は、元のデータより長生きしてはいけない**
3. **借用されている間は、元のデータを変更できない**

### rust-analyzerでの重要性

rust-analyzerは以下のために借用チェックが必要です：
- **借用エラーの検出**: リアルタイムでの借用競合の警告
- **安全性保証**: メモリ安全でないコードの早期発見
- **IDE支援**: 借用関係の可視化と修正提案
- **高品質診断**: 分かりやすい借用エラーメッセージ

## 🎯 今回の目標（借用チェッカーシステム）

**入力**: 複数の借用を含むプログラム
```rust
let mut x = 42;
let r1 = &x;           // 不変借用
let r2 = &mut x;       // 可変借用（競合！）

// 複数の不変借用（OK な例）
let y = 10;
let s1 = &y;           // 不変借用1
let s2 = &y;           // 不変借用2 (複数OK)
```

**出力**: 借用競合の検出と型推論
```rust
// ❌ Error: cannot borrow `x` as mutable because it is also borrowed as immutable
// ✅ s1, s2: 複数の不変借用は許可
```

### 🔍 今回学ぶ新機能

1. **借用の追跡**: アクティブな借用の管理
2. **可変借用**: `&mut T`型の処理
3. **競合検出**: 借用ルールの違反チェック
4. **借用スコープ**: 借用の生存期間管理

## 🏗️ 借用チェッカーシステムの構造

### 📋 借用情報の表現

各借用の詳細情報を追跡します：

```rust
// 借用の種類
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowKind {
    Immutable,  // &T (不変借用)
    Mutable,    // &mut T (可変借用)
}

// 借用情報
#[derive(Debug, Clone, PartialEq)]
pub struct Borrow {
    pub variable: String,       // 借用される変数名
    pub kind: BorrowKind,       // 借用の種類
    pub lifetime: Lifetime,     // 借用のライフタイム
    pub creation_span: Span,    // 借用が作成された位置
}
```

### 🔧 型システムの拡張

参照型に可変性情報を追加します：

```rust
// Before (lesson_3_13)
Type::Reference {
    inner_type: Box<Type>,
    lifetime: Option<Lifetime>,
}

// After (lesson_3_14)
Type::Reference {
    inner_type: Box<Type>,
    lifetime: Option<Lifetime>,
    mutability: BorrowKind,     // 新規追加
}
```

### 🎯 ASTへの可変借用追加

可変借用式を追加：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ... 既存の式 ...
    
    Reference {
        inner: Box<Expr>,    // &variable
        span: Span,
    },
    
    // ★新規追加：可変借用
    MutableReference {
        inner: Box<Expr>,    // &mut variable
        span: Span,
    },
    
    Dereference {
        inner: Box<Expr>,
        span: Span,
    },
}
```

### 📊 借用チェッカーの構造

借用の状態を追跡する仕組み：

```rust
#[derive(Debug)]
pub struct BorrowChecker {
    symbol_table: SymbolTable,
    diagnostics: Vec<Diagnostic>,
    active_borrows: Vec<Borrow>,  // 現在アクティブな借用のリスト
}
```

## 🔧 借用チェックの仕組み（詳細解説）

### 🔒 不変借用の処理

不変借用`&x`では以下をチェックします：

```rust
Expr::Reference { inner, span } => {
    // Step 1: 内部式の型を推論
    if let Some(inner_type) = self.infer_expression_type(inner) {
        if let Expr::Identifier(var_name, _) = inner.as_ref() {
            // Step 2: 借用情報を作成
            if let Some(new_borrow) = self.create_borrow(
                var_name.clone(), 
                BorrowKind::Immutable, 
                span
            ) {
                // Step 3: 借用競合をチェック
                self.check_borrow_conflicts(&new_borrow);
                
                // Step 4: 借用を記録
                self.active_borrows.push(new_borrow.clone());
                
                // Step 5: 参照型を返す
                Some(Type::Reference {
                    inner_type: Box::new(inner_type),
                    lifetime: Some(new_borrow.lifetime),
                    mutability: BorrowKind::Immutable,
                })
            } else {
                None
            }
        } else {
            // 変数以外の参照（簡略化）
            Some(Type::Reference {
                inner_type: Box::new(inner_type),
                lifetime: None,
                mutability: BorrowKind::Immutable,
            })
        }
    } else {
        None
    }
}
```

### 🔧 可変借用の処理

可変借用`&mut x`では同様ですが、より厳しいチェックを行います：

```rust
Expr::MutableReference { inner, span } => {
    // Step 1: 内部式の型を推論
    if let Some(inner_type) = self.infer_expression_type(inner) {
        if let Expr::Identifier(var_name, _) = inner.as_ref() {
            // Step 2: 可変借用情報を作成
            if let Some(new_borrow) = self.create_borrow(
                var_name.clone(), 
                BorrowKind::Mutable, 
                span
            ) {
                // Step 3: より厳しい借用競合チェック
                self.check_borrow_conflicts(&new_borrow);
                
                // Step 4: 借用を記録
                self.active_borrows.push(new_borrow.clone());
                
                // Step 5: 可変参照型を返す
                Some(Type::Reference {
                    inner_type: Box::new(inner_type),
                    lifetime: Some(new_borrow.lifetime),
                    mutability: BorrowKind::Mutable,
                })
            } else {
                None
            }
        } else {
            Some(Type::Reference {
                inner_type: Box::new(inner_type),
                lifetime: None,
                mutability: BorrowKind::Mutable,
            })
        }
    } else {
        None
    }
}
```

### ⚖️ 借用競合チェック

借用ルールを適用して競合を検出します：

```rust
fn check_borrow_conflicts(&mut self, new_borrow: &Borrow) {
    for existing_borrow in &self.active_borrows {
        // 同じ変数への借用かチェック
        if existing_borrow.variable == new_borrow.variable {
            match (&existing_borrow.kind, &new_borrow.kind) {
                // Rule 1: 可変借用は排他的
                (BorrowKind::Mutable, _) | (_, BorrowKind::Mutable) => {
                    self.diagnostics.push(Diagnostic::error(
                        format!(
                            "Cannot borrow `{}` as {:?} because it is already borrowed as {:?}",
                            new_borrow.variable,
                            new_borrow.kind,
                            existing_borrow.kind
                        ),
                        new_borrow.creation_span.clone()
                    ).with_code("E0502".to_string()));
                }
                // Rule 2: 複数の不変借用は OK
                (BorrowKind::Immutable, BorrowKind::Immutable) => {
                    // 競合なし
                }
            }
        }
    }
}
```

## 🎬 実行例：借用チェックの流れ

### 📝 入力プログラム（競合ケース）

```rust
let mut x = 42;    // 可変変数
let r1 = &x;       // 不変借用
let r2 = &mut x;   // 可変借用（競合！）
```

### 🔍 借用チェックプロセス

#### Step 1: 変数x定義 `let mut x = 42;`

```rust
💭 借用チェッカーの思考過程:
1. "変数xを定義しよう"
   → define("x", Type::Integer) → OK

✅ 結果: x: Integer型として定義
```

#### Step 2: 不変借用作成 `let r1 = &x;`

```rust
💭 借用チェッカーの思考過程:
1. "不変借用を処理しよう"
   → Reference { inner: Identifier("x") }

2. "借用情報を作成しよう"
   → Borrow { 
       variable: "x", 
       kind: Immutable, 
       lifetime: scope_level=0 
     }

3. "借用競合をチェックしよう"
   → active_borrows = [] → 競合なし

4. "借用を記録しよう"
   → active_borrows.push(borrow) → [Borrow(x, Immutable)]

5. "参照型を返そう"
   → Type::Reference { 
       inner: Integer, 
       mutability: Immutable 
     }

✅ 結果: r1: &i32型として定義、xに不変借用を記録
```

#### Step 3: 可変借用作成 `let r2 = &mut x;` （競合検出）

```rust
💭 借用チェッカーの思考過程:
1. "可変借用を処理しよう"
   → MutableReference { inner: Identifier("x") }

2. "借用情報を作成しよう"
   → Borrow { 
       variable: "x", 
       kind: Mutable, 
       lifetime: scope_level=0 
     }

3. "借用競合をチェックしよう"
   → active_borrows = [Borrow(x, Immutable)]
   → 同じ変数"x"への既存借用を発見
   → (Immutable, Mutable) → 競合！

4. "エラーを報告しよう"
   → diagnostics.push(Error: "Cannot borrow `x` as Mutable because it is already borrowed as Immutable")

❌ 結果: Error: 借用競合エラー
```

### 📝 入力プログラム（正常ケース）

```rust
let y = 10;        // 不変変数
let s1 = &y;       // 不変借用1
let s2 = &y;       // 不変借用2
```

### 🔍 複数不変借用プロセス

#### Step 1-2: 変数定義と最初の不変借用

```rust
💭 借用チェッカーの思考過程:
1. "変数yを定義" → OK
2. "不変借用s1を作成" → active_borrows = [Borrow(y, Immutable)]
```

#### Step 3: 2つ目の不変借用 `let s2 = &y;`

```rust
💭 借用チェッカーの思考過程:
1. "不変借用を処理しよう"
   → Reference { inner: Identifier("y") }

2. "借用競合をチェックしよう"
   → active_borrows = [Borrow(y, Immutable)]
   → 同じ変数"y"への既存借用を発見
   → (Immutable, Immutable) → 競合なし！

3. "借用を記録しよう"
   → active_borrows.push(borrow) → [Borrow(y, Immutable), Borrow(y, Immutable)]

✅ 結果: s2: &i32型として定義、複数の不変借用OK
```

## 💡 実装のポイント（あなたが書く部分）

今回実装する箇所は**4つ**です。借用チェッカーの考え方を理解して実装してください。

### 🎯 実装箇所1: 不変借用の処理

**場所**: `infer_expression_type()` メソッド内の `Expr::Reference` 部分

```rust
Expr::Reference { inner, span } => {
    // ヒント：
    // 1. 内部式の型を推論
    // 2. 変数のライフタイムを作成
    // 3. 不変借用を記録
    // 4. 借用の競合をチェック
    // 5. 参照型を返す

    todo!("不変借用の処理を実装してください")
}
```

**考えるポイント**: 借用情報の作成と競合チェックの呼び出し。

### 🎯 実装箇所2: 可変借用の処理

**場所**: `infer_expression_type()` メソッド内の `Expr::MutableReference` 部分

```rust
Expr::MutableReference { inner, span } => {
    // ヒント：
    // 1. 内部式の型を推論
    // 2. 変数のライフタイムを作成
    // 3. 可変借用を記録
    // 4. 借用の競合をチェック（より厳しい）
    // 5. 可変参照型を返す

    todo!("可変借用の処理を実装してください")
}
```

**考えるポイント**: 不変借用との違いは`BorrowKind::Mutable`のみ。

### 🎯 実装箇所3: 借用競合チェック

**場所**: `check_borrow_conflicts()` メソッド

```rust
fn check_borrow_conflicts(&mut self, new_borrow: &Borrow) {
    // ヒント：
    // 1. 同じ変数の既存借用をチェック
    // 2. 可変借用の排他性ルールを適用
    // 3. 不変借用と可変借用の競合を検出
    // 4. エラーを診断情報に追加

    todo!("借用競合チェックを実装してください")
}
```

**考えるポイント**: 借用ルールの正確な実装。

### 🎯 実装箇所4: 借用情報作成

**場所**: `create_borrow()` メソッド

```rust
fn create_borrow(&self, variable: String, kind: BorrowKind, span: &Span) -> Option<Borrow> {
    // ヒント：
    // 1. 変数のシンボルを解決
    // 2. ライフタイムを作成
    // 3. 借用情報を作成

    todo!("借用情報作成を実装してください")
}
```

**考えるポイント**: `create_lifetime_for_variable`の活用。

## ✅ 実装の進め方

### Step 1: TODOを探す 🔍

lesson_3_14.rsファイルで `todo!(\"...\")` の4箇所を探してください。

### Step 2: 借用チェッカー実装 ✏️

不変借用、可変借用、競合チェック、借用作成の4つの機能を実装してください。

### Step 3: テスト実行 🧪

```bash
cargo test lesson_3::lesson_3_14
```

## 🎯 テストケース（4つ）

1. **`test_immutable_borrow`**: 基本的な不変借用
2. **`test_mutable_borrow`**: 基本的な可変借用
3. **`test_borrow_conflict`**: 借用競合の検出
4. **`test_multiple_immutable_borrows`**: 複数不変借用の許可

## 🔄 lesson_3_13からの進化

### lesson_3_13でできたこと
- ✅ ライフタイムの基本概念
- ✅ 参照の作成と型推論
- ✅ 基本的なダングリング参照検出

### lesson_3_14で新しく追加されること
- ✅ **借用の追跡**: アクティブな借用の管理
- ✅ **可変借用**: `&mut T`型の処理
- ✅ **競合検出**: 借用ルールの違反チェック
- ✅ **借用スコープ**: スコープによる借用の無効化

### 変更されないもの
- ✅ 基本的なライフタイムシステム
- ✅ 構造体とフィールドアクセス
- ✅ エラー回復システム

## 🎉 完了後の効果

lesson_3_14が完了すると：
- **借用安全性**: メモリ安全でないコードの検出
- **Rustの核心理解**: 借用システムの深い理解
- **rust-analyzer貢献準備**: 実用的な借用エラー検出

**次のステップ**: lesson_3_15でライフタイム推論を学習し、より実用的な借用システムに進みます！

**借用チェッカーはRustの最も重要な機能の一つです。丁寧に実装して理解を深めましょう！**