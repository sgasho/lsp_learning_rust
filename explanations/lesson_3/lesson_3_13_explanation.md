# Lesson 3-13: ライフタイムの基本

lesson_3_12で構造体とフィールドアクセスができるようになりましたね。今度は、**ライフタイムの基本**を学びます。

## 📚 ライフタイムとは？

### 🤔 なぜライフタイムが重要？

ライフタイムは、**参照の安全性を保証する仕組み**です。プログラムでメモリが無効になった後にアクセスしようとすると、クラッシュやセキュリティ問題が発生します：

```rust
// 危険なコード（C言語の例）
int* dangling_pointer() {
    int x = 42;        // ローカル変数
    return &x;         // xのアドレスを返す（危険！）
}                      // xはここで破棄される

int main() {
    int* ptr = dangling_pointer();
    printf("%d", *ptr); // 無効なメモリアクセス → クラッシュ！
}

// Rustでの安全なコード
fn safe_reference() -> i32 {
    let x = 42;
    x                  // 値そのものを返す（安全！）
}
```

### 🎯 ライフタイムの役割

ライフタイムは「メモリの守護神」のようなものです：

1. **安全性**: ダングリング参照（無効な参照）を防止
2. **自動管理**: コンパイル時に参照の妥当性をチェック
3. **ゼロコスト**: ランタイムでのオーバーヘッドなし
4. **予測可能性**: メモリ関連のバグを事前に発見

### rust-analyzerでの重要性

rust-analyzerは以下のためにライフタイム解析が必要です：
- **参照エラーの検出**: ダングリング参照の早期発見
- **ライフタイム推論**: 明示的な注釈が不要な場合の自動推論
- **借用チェック**: 参照の競合状態の検出
- **エラーメッセージ**: 分かりやすいライフタイムエラーの説明

## 🎯 今回の目標（ライフタイムシステム）

**入力**: 参照とデリファレンスを含むプログラム
```rust
let x = 42;
let r = &x;           // xへの参照
let y = *r;           // 参照の中身を取得

// ダングリング参照（エラーケース）
let r = {
    let x = 42;
    &x                // Error: xのライフタイムが短すぎる
};
```

**出力**: ライフタイムとダングリング参照の検出
```rust
// ✅ r: &i32型として推論、ライフタイムも適切
// ✅ y: i32型として推論
// ❌ ダングリング参照エラー：xがスコープを外れた後にアクセス
```

### 🔍 今回学ぶ新機能

1. **参照型**: `&T`型の表現と推論
2. **参照演算子**: `&variable`式の処理
3. **デリファレンス**: `*reference`式の処理
4. **ライフタイム追跡**: 参照の生存期間の管理

## 🏗️ ライフタイムシステムの構造

### 🕰️ ライフタイムの表現

参照の生存期間を表現します：

```rust
// ライフタイム情報
#[derive(Debug, Clone, PartialEq)]
pub struct Lifetime {
    pub name: String,           // ライフタイム名（例："'a"）
    pub scope_level: usize,     // 生存するスコープレベル
    pub creation_span: Span,    // 作成された位置
}

// 参照型にライフタイム情報を追加
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    String,
    // ... 他の型 ...
    
    // ★新規追加：参照型
    Reference {
        inner_type: Box<Type>,        // 参照する型
        lifetime: Option<Lifetime>,   // ライフタイム
    },
    
    Unknown,
    Inferred(Box<Type>),
}
```

例：`&i32`型は`Type::Reference { inner_type: Integer, lifetime: Some(...) }`として表現されます。

### 🔧 AST への参照演算子追加

参照とデリファレンスをASTで表現します：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64, Span),
    // ... 他の式 ...
    
    // ★新規追加：参照演算子
    Reference {
        inner: Box<Expr>,    // 参照される式（例：x）
        span: Span,
    },
    
    // ★新規追加：デリファレンス演算子
    Dereference {
        inner: Box<Expr>,    // デリファレンスされる式（例：r）
        span: Span,
    },
}
```

これにより、`&x`や`*r`のような操作をASTで表現できます。

### 🔍 Type型のライフタイム関連メソッド

参照型を扱うための便利メソッドを追加します：

```rust
impl Type {
    // 参照型かどうかをチェック
    pub fn is_reference(&self) -> bool {
        matches!(self.resolve(), Type::Reference { .. })
    }

    // 参照の内部型を取得
    pub fn get_inner_type(&self) -> Option<&Type> {
        match self.resolve() {
            Type::Reference { inner_type, .. } => Some(inner_type),
            _ => None,
        }
    }

    // 参照のライフタイムを取得
    pub fn get_lifetime(&self) -> Option<&Lifetime> {
        match self.resolve() {
            Type::Reference { lifetime, .. } => lifetime.as_ref(),
            _ => None,
        }
    }
}
```

## 🔧 ライフタイム解析の仕組み（詳細解説）

### 🎯 参照演算子の処理

参照演算子`&x`では以下をチェックします：

```rust
Expr::Reference { inner, span } => {
    // Step 1: 内部式の型を推論
    if let Some(inner_type) = self.infer_expression_type(inner) {
        // Step 2: 参照される変数のライフタイムを取得
        if let Expr::Identifier(var_name, _) = inner.as_ref() {
            if let Some(lifetime) = self.create_lifetime_for_variable(var_name, span) {
                // Step 3: 参照型を作成
                Some(Type::Reference {
                    inner_type: Box::new(inner_type),
                    lifetime: Some(lifetime),
                })
            } else {
                None
            }
        } else {
            // 変数以外の参照（今回は簡略化）
            Some(Type::Reference {
                inner_type: Box::new(inner_type),
                lifetime: None,
            })
        }
    } else {
        None
    }
}
```

### 🔍 デリファレンス演算子の処理

デリファレンス演算子`*r`では以下をチェックします：

```rust
Expr::Dereference { inner, span } => {
    // Step 1: 内部式の型を推論
    if let Some(inner_type) = self.infer_expression_type(inner) {
        // Step 2: 参照型かチェック
        if let Some(dereferenced_type) = inner_type.get_inner_type() {
            // Step 3: 内部型を返す
            Some(dereferenced_type.clone())
        } else {
            // Step 4: 参照型でない場合のエラー
            self.diagnostics.push(Diagnostic::error(
                format!("Cannot dereference non-reference type {:?}", inner_type.resolve()),
                span.clone()
            ));
            None
        }
    } else {
        None
    }
}
```

### 🚨 ダングリング参照の検出

ライフタイムの妥当性をチェックします：

```rust
fn check_lifetime_validity(&mut self, reference_lifetime: &Lifetime, value_span: &Span) {
    // Step 1: 参照のライフタイムが現在のスコープレベル以下かチェック
    if reference_lifetime.scope_level > self.symbol_table.scope_level {
        // Step 2: ダングリング参照を検出
        self.diagnostics.push(Diagnostic::error(
            format!("This reference outlives the value it points to"),
            value_span.clone()
        ).with_code("E0009".to_string()));
    }
}
```

## 🎬 実行例：ライフタイム解析の流れ

### 📝 入力プログラム（正常ケース）

```rust
let x = 42;    // x: i32
let r = &x;    // r: &i32
let y = *r;    // y: i32
```

### 🔍 ライフタイム解析プロセス

#### Step 1: 変数x定義 `let x = 42;`

```rust
💭 ライフタイム対応型チェッカーの思考過程:
1. "値の型を推論しよう"
   → Number(42) → Type::Integer

2. "変数を定義しよう"
   → define("x", Type::Integer, scope_level=0) → OK

✅ 結果: x: Integer型として定義
```

#### Step 2: 参照作成 `let r = &x;`

```rust
💭 ライフタイム対応型チェッカーの思考過程:
1. "参照演算子を処理しよう"
   → Reference { inner: Identifier("x") }

2. "内部式の型を推論しよう"
   → Identifier("x") → resolve("x") → Type::Integer

3. "変数xのライフタイムを作成しよう"
   → create_lifetime_for_variable("x") → Lifetime { 
       name: "'x", 
       scope_level: 0,
       creation_span: ... 
     }

4. "参照型を作成しよう"
   → Type::Reference { 
       inner_type: Type::Integer, 
       lifetime: Some(Lifetime { scope_level: 0 }) 
     }

5. "変数rを定義しよう"
   → define("r", reference_type) → OK

✅ 結果: r: &i32型として定義
```

#### Step 3: デリファレンス `let y = *r;`

```rust
💭 ライフタイム対応型チェッカーの思考過程:
1. "デリファレンス演算子を処理しよう"
   → Dereference { inner: Identifier("r") }

2. "内部式の型を推論しよう"
   → Identifier("r") → resolve("r") → Type::Reference { inner: Integer, ... }

3. "参照型かチェックしよう"
   → is_reference() → true

4. "内部型を取得しよう"
   → get_inner_type() → Type::Integer

5. "変数yを定義しよう"
   → define("y", Type::Integer) → OK

✅ 結果: y: Integer型として定義
```

### 📝 入力プログラム（エラーケース）

```rust
let r = {
    let x = 42;
    &x              // Error: ダングリング参照
};
```

### 🔍 ダングリング参照検出プロセス

#### Step 1: ブロック内の処理

```rust
💭 ライフタイム対応型チェッカーの思考過程:
1. "ブロックに入るのでスコープレベルを上げよう"
   → enter_scope() → scope_level = 1

2. "変数xを定義しよう"
   → define("x", Type::Integer, scope_level=1) → OK

3. "参照演算子を処理しよう"
   → Reference { inner: Identifier("x") }

4. "変数xのライフタイムを作成しよう"
   → Lifetime { scope_level: 1, ... }

5. "参照型を作成しよう"
   → Type::Reference { inner: Integer, lifetime: scope_level=1 }

6. "ブロックを出るのでスコープレベルを下げよう"
   → exit_scope() → scope_level = 0

7. "ライフタイムの妥当性をチェックしよう"
   → reference_lifetime.scope_level (1) > current_scope_level (0)
   → ダングリング参照を検出！

❌ 結果: Error: "This reference outlives the value it points to"
```

## 💡 実装のポイント（あなたが書く部分）

今回実装する箇所は**4つ**です。ライフタイムの考え方を理解して実装してください。

### 🎯 実装箇所1: 参照演算子の処理

**場所**: `infer_expression_type()` メソッド内の `Expr::Reference` 部分

```rust
Expr::Reference { inner, span } => {
    // ヒント：
    // 1. 内部式の型を推論
    // 2. 参照される変数のスコープレベルを取得
    // 3. ライフタイムを作成
    // 4. 参照型を返す

    todo!("参照演算子の処理を実装してください")
}
```

**考えるポイント**: 変数のライフタイムをどうやって取得するか。

### 🎯 実装箇所2: デリファレンス演算子の処理

**場所**: `infer_expression_type()` メソッド内の `Expr::Dereference` 部分

```rust
Expr::Dereference { inner, span } => {
    // ヒント：
    // 1. 内部式の型を推論
    // 2. 参照型かチェック
    // 3. 内部型を返す
    // 4. エラーケースの処理

    todo!("デリファレンス演算子の処理を実装してください")
}
```

**考えるポイント**: `Type::get_inner_type()`メソッドを活用する。

### 🎯 実装箇所3: ライフタイムの妥当性チェック

**場所**: `check_lifetime_validity()` メソッド

```rust
fn check_lifetime_validity(&mut self, reference_lifetime: &Lifetime, value_span: &Span) {
    // ヒント：
    // 1. 参照のライフタイムが現在のスコープレベル以下かチェック
    // 2. ダングリング参照を検出
    // 3. エラーを診断情報に追加

    todo!("ライフタイムの妥当性チェックを実装してください")
}
```

**考えるポイント**: スコープレベルの比較によるダングリング参照の検出。

### 🎯 実装箇所4: 変数のライフタイム作成

**場所**: `create_lifetime_for_variable()` メソッド

```rust
fn create_lifetime_for_variable(&self, var_name: &str, span: &Span) -> Option<Lifetime> {
    // ヒント：
    // 1. 変数のシンボルを解決
    // 2. 変数のスコープレベルを取得
    // 3. ライフタイムを作成

    todo!("変数のライフタイム作成を実装してください")
}
```

**考えるポイント**: シンボルテーブルから変数の情報を取得する。

## ✅ 実装の進め方

### Step 1: TODOを探す 🔍

lesson_3_13.rsファイルで `todo!(\"...\")` の4箇所を探してください。

### Step 2: ライフタイムシステム実装 ✏️

参照の作成、デリファレンス、ライフタイム追跡の4つの機能を実装してください。

### Step 3: テスト実行 🧪

```bash
cargo test lesson_3::lesson_3_13
```

## 🎯 テストケース（3つ）

1. **`test_basic_reference`**: 基本的な参照の作成と使用
2. **`test_dereference`**: デリファレンス演算子の正常動作
3. **`test_dangling_reference`**: ダングリング参照の検出

## 🔄 lesson_3_12からの進化

### lesson_3_12でできたこと
- ✅ 構造体の定義と作成
- ✅ フィールドアクセスの型チェック
- ✅ 構造体コンストラクタの検証

### lesson_3_13で新しく追加されること
- ✅ **参照型**: `&T`型の表現と推論
- ✅ **参照演算子**: `&variable`式の処理
- ✅ **デリファレンス**: `*reference`式の処理
- ✅ **ライフタイム追跡**: 参照の生存期間の管理

### 変更されないもの
- ✅ 構造体システム
- ✅ エラー回復システム
- ✅ 基本的な型システム

## 🎉 完了後の効果

lesson_3_13が完了すると：
- **メモリ安全性**: ダングリング参照の自動検出
- **参照システム**: Rustの参照モデルの理解
- **ライフタイム追跡**: 参照の生存期間の管理

**次のステップ**: あなたが興味を持っているライフタイムの更なる発展や、他の高度な機能に進むことができます！

**ライフタイムはRustの核心的な機能です。丁寧に実装して理解を深めましょう！**