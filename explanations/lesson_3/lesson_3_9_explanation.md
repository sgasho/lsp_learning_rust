# Lesson 3-9: 基本的な型システム

lesson_3_8で関数のスコープができるようになりましたね。今度は、**基本的な型システム**を学びます。

## 📚 型システムとは？

### 🤔 なぜ型が必要なのか？

**型システム**は、プログラムの値に「意味」を与えて正しさを保証する仕組みです。

```rust
// これらは全て数値に見えますが...
let age = 25;           // 年齢（整数）
let temperature = 36.5; // 体温（浮動小数点）
let is_adult = true;    // 成人かどうか（真偽値）

// 型がないと、こんな間違いが起きる
if age {               // Error: 数値を条件にできない！
    print("adult");
}

let result = is_adult + 10;  // Error: 真偽値に数値を足せない！
```

### 🎯 型システムの役割

型システムは「プログラムの安全性を守る番人」です：

1. **意味の明確化**: この値は何を表すのか？
2. **操作の制限**: この値にどんな操作ができるのか？
3. **エラーの防止**: 間違った操作を事前に止める

### rust-analyzerでの重要性

rust-analyzerは以下のために型システムが必要です：
- **リアルタイム型チェック**: タイピング中に型エラーを検出
- **型推論**: 明示的な型注釈がなくても型を推測
- **インテリセンス**: 型に基づいた適切なメソッド候補表示
- **リファクタリング支援**: 型安全な変数名変更や抽出

## 🎯 今回の目標（型システムの第一歩）

**入力**: 型注釈を含む簡単なプログラム
```rust
let x: i32 = 42;        // 型注釈: i32
let flag: bool = true;  // 型注釈: bool  
if flag {               // boolean型の条件チェック
    let result = x + 10; // i32 + i32 = i32の演算チェック
}
if x {                  // Error: i32は条件に使えない
    print("bad");
}
```

**出力**: 型チェック結果
```rust
// ✅ x: i32型として正常定義
// ✅ flag: bool型として正常定義  
// ✅ if flag: 条件がbool型なのでOK
// ✅ x + 10: i32同士の加算なのでOK
// ❌ if x: 条件がi32型なのでError!
```

### 🔍 今回学ぶ型チェックの種類

1. **変数定義での型チェック**: 値と型注釈の一致確認
2. **式の型推論**: 値から型を自動判定
3. **条件文の型チェック**: if/while文の条件はbool型のみ
4. **二項演算の型チェック**: 演算子に応じた型の組み合わせ確認

## 🏗️ 型システムの構造（段階的理解）

### 🎪 型の世界の住人たち

まず、プログラムで扱う「型」という概念を理解しましょう：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,    // 整数（例: 42, -10, 0）
    Boolean,    // 真偽値（例: true, false）
    String,     // 文字列（例: "hello", "world"）
    Function {  // 関数（例: fn add(a: i32, b: i32) -> i32）
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    Unknown,    // 型が不明（エラー処理用）
}
```

### 🏠 変数に「型という住所」を付ける

lesson_3_1〜3_8では、変数は「名前」だけを持っていました。今回は「型」という情報を追加します：

```rust
// Before (lesson_3_8まで)
pub struct Symbol {
    pub name: String,        // 変数名だけ
    pub scope_level: usize,  // スコープレベル
}

// After (lesson_3_9から)
pub struct Symbol {
    pub name: String,        // 変数名
    pub scope_level: usize,  // スコープレベル
    pub symbol_type: Type,   // ★型情報を追加！
}
```

この変更により、変数は「どこで定義されたか」だけでなく「何の型か」も覚えるようになります。

### 📝 ASTに型注釈を追加

プログラマが書く型注釈（`let x: i32 = 42;`の`: i32`部分）をASTで表現できるようにします：

```rust
// Before (lesson_3_8まで)
LetDeclaration {
    name: String,
    value: Expr,
}

// After (lesson_3_9から)  
LetDeclaration {
    name: String,
    value: Expr,
    type_annotation: Option<Type>, // ★型注釈を追加！
}
```

`Option<Type>`なので、型注釈がある場合（`Some(Type::Integer)`）と、ない場合（`None`）の両方に対応できます。

## 🔧 型チェックの仕組み（詳細解説）

### 🕵️ 型推論：値から型を推測する探偵

型推論は「この式は何型？」を判定する機能です。まるで探偵のように、証拠（値）から真実（型）を見つけ出します：

```rust
fn infer_expression_type(&mut self, expr: &Expr) -> Result<Type, String> {
    match expr {
        // 🔢 数値リテラル: 見た目で判断
        Expr::Number(_) => Ok(Type::Integer),    // 42 → Integer型
        
        // ✅ 真偽値リテラル: 見た目で判断  
        Expr::Boolean(_) => Ok(Type::Boolean),  // true → Boolean型
        
        // 📝 文字列リテラル: 見た目で判断
        Expr::String(_) => Ok(Type::String),    // "hello" → String型
        
        // 🔍 変数参照: シンボルテーブルに聞く
        Expr::Identifier(name) => {
            if let Some(symbol) = self.symbol_table.resolve(name) {
                Ok(symbol.symbol_type.clone())  // 定義時の型を返す
            } else {
                Err(format!("Variable '{}' not defined", name))
            }
        }
        
        // ... 他の式も同様
    }
}
```

### 🧮 二項演算：型の相性をチェックする仲人

二項演算では「左の型」と「右の型」が「演算子」と相性が良いかをチェックします：

```rust
// 例: x + y の型チェック
Expr::Binary { left, operator, right } => {
    // Step 1: 左右の型を推論
    let left_type = self.infer_expression_type(left)?;   // x の型
    let right_type = self.infer_expression_type(right)?; // y の型

    // Step 2: 演算子に応じた相性チェック
    match operator {
        // 🧮 算術演算: 数値同士でないとダメ
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
            if left_type == Type::Integer && right_type == Type::Integer {
                Ok(Type::Integer)  // i32 + i32 = i32
            } else {
                Err("算術演算は整数同士でないとできません".to_string())
            }
        }
        
        // 📏 比較演算: 数値同士を比べて真偽値を返す
        BinaryOp::GreaterThan | BinaryOp::LessThan => {
            if left_type == Type::Integer && right_type == Type::Integer {
                Ok(Type::Boolean)  // i32 > i32 = bool
            } else {
                Err("比較は整数同士でないとできません".to_string())
            }
        }
        
        // ⚖️ 等価演算: 同じ型同士なら比較可能
        BinaryOp::Equal | BinaryOp::NotEqual => {
            if left_type == right_type {
                Ok(Type::Boolean)  // 同じ型 == 同じ型 = bool
            } else {
                Err("等価比較は同じ型同士でないとできません".to_string())
            }
        }
    }
}
```

### 🚦 条件文：真偽値の門番

if文やwhile文の条件は、必ずboolean型でないといけません：

```rust
// if文の条件チェック
Stmt::IfStatement { condition, .. } => {
    let condition_type = self.infer_expression_type(condition)?;
    
    if condition_type != Type::Boolean {
        return Err(format!(
            "if文の条件はbool型である必要があります。{:?}型が見つかりました",
            condition_type
        ));
    }
    
    // 条件がbool型なら、then_branchとelse_branchを処理
}
```

## 🎬 実行例：型チェックの流れ（ステップバイステップ）

### 📝 入力プログラム（わかりやすい例）

```rust
// プログラム: 年齢チェッカー
let age: i32 = 25;           // 年齢を整数で定義
let is_adult: bool = true;   // 成人かどうかを真偽値で定義

if is_adult {                // 条件: bool型 → OK
    let bonus = age + 5;     // 計算: i32 + i32 → i32型 → OK
    print(bonus);
}

if age {                     // 条件: i32型 → Error!
    print("これはエラー");
}
```

### 🔍 型チェックの詳細プロセス

#### Step 1: `let age: i32 = 25;` の型チェック

```rust
💭 型チェッカーの思考過程:
1. 「右辺の型推論をしよう」
   → Number(25) を見る → "これは数値リテラルだ" → Type::Integer

2. 「型注釈と一致するかチェックしよう」  
   → 型注釈: Some(Type::Integer)
   → 右辺の型: Type::Integer
   → "一致している！OK"

3. 「変数を定義しよう」
   → Symbol { name: "age", symbol_type: Type::Integer }
   → シンボルテーブルに追加

✅ 結果: ageはi32型として正常定義
```

#### Step 2: `let is_adult: bool = true;` の型チェック

```rust
💭 型チェッカーの思考過程:
1. 「右辺の型推論をしよう」
   → Boolean(true) を見る → "これは真偽値リテラルだ" → Type::Boolean

2. 「型注釈と一致するかチェックしよう」
   → 型注釈: Some(Type::Boolean)  
   → 右辺の型: Type::Boolean
   → "一致している！OK"

3. 「変数を定義しよう」
   → Symbol { name: "is_adult", symbol_type: Type::Boolean }
   → シンボルテーブルに追加

✅ 結果: is_adultはbool型として正常定義
```

#### Step 3: `if is_adult { ... }` の条件チェック

```rust
💭 型チェッカーの思考過程:
1. 「条件の型を推論しよう」
   → Identifier("is_adult") を見る
   → シンボルテーブルを検索: is_adult → Type::Boolean

2. 「if文の条件はbool型でないといけない」
   → 条件の型: Type::Boolean
   → 要求される型: Type::Boolean  
   → "一致している！OK"

3. 「then_branchを型チェックしよう」
   → ブロック内の文を順番に処理

✅ 結果: if文の条件は正常
```

#### Step 4: `let bonus = age + 5;` の型チェック

```rust
💭 型チェッカーの思考過程:
1. 「右辺の二項演算を型チェックしよう」
   → Binary { left: Identifier("age"), op: Add, right: Number(5) }

2. 「左辺の型を推論しよう」
   → Identifier("age") → シンボルテーブル検索 → Type::Integer

3. 「右辺の型を推論しよう」  
   → Number(5) → Type::Integer

4. 「加算演算の型チェックをしよう」
   → Add(Type::Integer, Type::Integer) → "整数同士の加算だ" → Type::Integer

5. 「変数を定義しよう」
   → Symbol { name: "bonus", symbol_type: Type::Integer }

✅ 結果: bonusはi32型として正常定義
```

#### Step 5: `if age { ... }` の条件チェック（エラーケース）

```rust
💭 型チェッカーの思考過程:
1. 「条件の型を推論しよう」
   → Identifier("age") を見る
   → シンボルテーブルを検索: age → Type::Integer

2. 「if文の条件はbool型でないといけない」
   → 条件の型: Type::Integer
   → 要求される型: Type::Boolean
   → "型が一致しない！エラーだ！"

❌ 結果: Error: "If condition must be boolean, found Integer"
```

### 📊 最終的なシンボルテーブル

```rust
symbols: {
    "age" → Symbol { name: "age", symbol_type: Type::Integer },
    "is_adult" → Symbol { name: "is_adult", symbol_type: Type::Boolean },
    "bonus" → Symbol { name: "bonus", symbol_type: Type::Integer }
}
```

## 🔍 関数型の扱い

### 1. 関数定義の型チェック

```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

型情報として：
```rust
Type::Function {
    parameters: vec![Type::Integer, Type::Integer],
    return_type: Box::new(Type::Integer),
}
```

### 2. 関数呼び出しの型チェック

```rust
let result = add(5, 10);  // add: (i32, i32) -> i32
```

チェック過程：
```rust
// 1. 関数名解決: "add" → Type::Function { parameters: [Integer, Integer], return_type: Integer }
// 2. 引数数確認: 2個の引数 == 2個のパラメータ → OK
// 3. 引数型確認: [Number(5)→Integer, Number(10)→Integer] == [Integer, Integer] → OK
// 4. 戻り値型: Type::Integer
```

## 💡 実装のポイント（あなたが書く部分）

今回実装する箇所は**3つ**だけです。すでにヒントコメントが書いてあるので、それに従って実装してください。

### 🎯 実装箇所1: 変数定義での型チェック

**場所**: `check_statement()` メソッド内の `Stmt::LetDeclaration` 部分

```rust
Stmt::LetDeclaration { name, value, type_annotation } => {
    // TODO: 以下のステップで実装してください
    
    // Step 1: 値の型を推論する
    let value_type = self.infer_expression_type(value)?;
    
    // Step 2: 型注釈がある場合は一致確認
    let final_type = if let Some(annotated_type) = type_annotation {
        if &value_type != annotated_type {
            return Err(format!(
                "Type mismatch: expected {:?}, found {:?}",
                annotated_type, value_type
            ));
        }
        annotated_type.clone()
    } else {
        value_type  // 型注釈がない場合は推論した型を使用
    };

    // Step 3: 変数を定義（今回は型情報付き）
    self.symbol_table.define(name.clone(), final_type)
}
```

**何をしているか**: `let x: i32 = 42;` のような文で、「42の型」と「i32という注釈」が一致するかチェックしています。

### 🎯 実装箇所2: 二項演算の型チェック

**場所**: `infer_expression_type()` メソッド内の `Expr::Binary` 部分

```rust
Expr::Binary { left, operator, right } => {
    // TODO: 以下のステップで実装してください
    
    // Step 1: 左右の式の型を推論
    let left_type = self.infer_expression_type(left)?;
    let right_type = self.infer_expression_type(right)?;

    // Step 2: 演算子ごとに型チェック
    match operator {
        // 算術演算: 整数 + 整数 = 整数
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
            if left_type == Type::Integer && right_type == Type::Integer {
                Ok(Type::Integer)
            } else {
                Err(format!(
                    "Arithmetic operation requires integers, found {:?} and {:?}",
                    left_type, right_type
                ))
            }
        }
        
        // 比較演算: 整数 > 整数 = 真偽値
        BinaryOp::GreaterThan | BinaryOp::LessThan => {
            if left_type == Type::Integer && right_type == Type::Integer {
                Ok(Type::Boolean)
            } else {
                Err(format!(
                    "Comparison requires integers, found {:?} and {:?}",
                    left_type, right_type
                ))
            }
        }
        
        // 等価演算: 同じ型 == 同じ型 = 真偽値
        BinaryOp::Equal | BinaryOp::NotEqual => {
            if left_type == right_type {
                Ok(Type::Boolean)
            } else {
                Err(format!(
                    "Equality comparison requires same types, found {:?} and {:?}",
                    left_type, right_type
                ))
            }
        }
    }
}
```

**何をしているか**: `x + y` や `a > b` のような演算で、左右の型が演算子と相性が良いかチェックしています。

### 🎯 実装箇所3: 関数呼び出しの型チェック

**場所**: `infer_expression_type()` メソッド内の `Expr::FunctionCall` 部分

```rust
Expr::FunctionCall { name, arguments } => {
    // TODO: 以下のステップで実装してください
    
    // Step 1: 関数の型情報を取得（借用問題回避）
    let function_type = if let Some(symbol) = self.symbol_table.resolve(name) {
        symbol.symbol_type.clone()
    } else {
        return Err(format!("Function '{}' not defined", name));
    };

    // Step 2: 関数型かどうか確認
    if let Type::Function { parameters, return_type } = function_type {
        // Step 3: 引数の数をチェック
        if arguments.len() != parameters.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, found {}",
                name, parameters.len(), arguments.len()
            ));
        }

        // Step 4: 各引数の型をチェック
        for (arg, expected_type) in arguments.iter().zip(parameters.iter()) {
            let arg_type = self.infer_expression_type(arg)?;
            if arg_type != *expected_type && *expected_type != Type::Unknown {
                return Err(format!(
                    "Argument type mismatch: expected {:?}, found {:?}",
                    expected_type, arg_type
                ));
            }
        }

        // Step 5: 戻り値の型を返す
        Ok(*return_type)
    } else {
        Err(format!("'{}' is not a function", name))
    }
}
```

**何をしているか**: `add(x, y)` のような関数呼び出しで、関数が存在し、引数の型が正しいかチェックしています。

## 🐛 エラーケースの例

### 1. 型の不一致

```rust
let x: i32 = true;  // Error: Type mismatch: expected Integer, found Boolean
```

### 2. 条件文の型エラー

```rust
if 42 {  // Error: If condition must be boolean, found Integer
    print("bad");
}
```

### 3. 算術演算の型エラー

```rust
let result = true + 5;  // Error: Arithmetic operation requires integers, found Boolean and Integer
```

### 4. 関数呼び出しの型エラー

```rust
fn add(a: i32, b: i32) -> i32 { return a + b; }
let result = add(true, 5);  // Error: Argument type mismatch: expected Integer, found Boolean
```

## ✅ 実装の進め方（シンプル3ステップ）

**重要**: 既に構造体や基本的なコードは実装済みです。あなたがやることは**TODOコメントがある3箇所を埋める**だけです！

### Step 1: TODOを探す 🔍

lesson_3_9.rsファイルで `// ヒント:` から始まるコメントを探してください。

### Step 2: 3箇所を実装 ✏️

1. `check_statement()` の `Stmt::LetDeclaration` 部分
2. `infer_expression_type()` の `Expr::Binary` 部分
3. `infer_expression_type()` の `Expr::FunctionCall` 部分

### Step 3: テスト実行 🧪

```bash
cargo test lesson_3::lesson_3_9
```

**実行コマンド**: `cargo test lesson_3::lesson_3_9`

## 🎯 テストケース（3つ、すべて実装済み）

1. **`test_basic_type_checking`**: 正常な型チェック
   - `let x: i32 = 42;` → OK
   - `let flag: bool = true;` → OK

2. **`test_type_mismatch_error`**: 型不一致エラー
   - `let x: i32 = true;` → Error

3. **`test_if_condition_type_check`**: 条件文エラー
   - `if 42 { ... }` → Error（条件は数値型）

## 🔄 lesson_3_8からの進化

### lesson_3_8でできたこと
- ✅ 関数スコープ管理
- ✅ パラメータ管理
- ✅ 関数呼び出しチェック

### lesson_3_9で新しく追加されること
- ✅ **型システム**: 変数に型情報を付与
- ✅ **型推論**: 値から型を自動判定
- ✅ **型チェック**: 型の不一致を検出
- ✅ **条件文の型制約**: if/while文の条件はbool型のみ

### 変更されないもの
- ✅ スコープ管理（lesson_3_8と同じ）
- ✅ 変数定義・参照（lesson_3_8と同じ）
- ✅ 関数管理（lesson_3_8と同じ）

## 🎉 完了後の効果

lesson_3_9が完了すると：
- **型安全性**: 間違った型の使用を事前に防げる
- **開発体験**: タイピング中に型エラーを発見
- **rust-analyzerの基盤**: 型に基づく高度な機能の準備完了

**次のステップ**: lesson_3_10で型推論をさらに高度化し、rust-analyzerにより近づけます！

**型システムは複雑に見えますが、今回は基本的な部分だけです。リラックスして取り組んでください！**