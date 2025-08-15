# Lesson 3-10: 型推論の高度化

lesson_3_9で基本的な型システムができるようになりましたね。今度は、**型推論をより高度化**します。

## 📚 型推論の高度化とは？

### 🤔 なぜより高度な型推論が必要？

lesson_3_9では型注釈が必須でしたが、実際のプログラミングでは型注釈を省略することが多いです：

```rust
// lesson_3_9: 型注釈が必要
let x: i32 = 42;
let flag: bool = true;

// lesson_3_10: 型注釈なしでも推論
let x = 42;        // → i32と推論
let flag = true;   // → boolと推論
let result = x + 5; // → i32と推論（xがi32なので）
```

### 🎯 型推論の段階

型推論は「探偵の推理」のようなプロセスです：

1. **証拠収集**: 値やリテラルから初期の型情報を集める
2. **関係性分析**: 変数同士の関係を分析する
3. **制約解決**: 矛盾がないか確認し、最終的な型を決定

### rust-analyzerでの重要性

rust-analyzerは以下のために高度な型推論が必要です：
- **自然な開発体験**: 型注釈を書かずに済む
- **リアルタイム補完**: 推論した型に基づくメソッド提案
- **エラー検出**: 型注釈がなくても型エラーを検出
- **リファクタリング**: 型情報に基づく安全な変更

## 🎯 今回の目標（型推論の強化）

**入力**: 型注釈のないプログラム
```rust
let x = 42;           // 型注釈なし
let y = x + 10;       // 型注釈なし  
let flag = x > 5;     // 型注釈なし
if flag {             // 推論された型でチェック
    let z = y * 2;    // さらに推論
}
```

**出力**: 正確な型推論
```rust
// ✅ x: i32 (数値リテラルから推論)
// ✅ y: i32 (i32 + i32 = i32から推論)
// ✅ flag: bool (i32 > i32 = boolから推論)  
// ✅ z: i32 (i32 * i32 = i32から推論)
```

### 🔍 今回学ぶ高度な機能

1. **型注釈なしの変数定義**: `let x = value;`
2. **推論型の伝播**: 変数間での型情報の受け渡し
3. **代入式の型チェック**: `x = new_value;`
4. **段階的型解決**: 不明な型の後付け推論

## 🏗️ 型推論システムの拡張

### 🧩 推論型の表現

lesson_3_9の`Type`enumを拡張して、「推論中」の状態を表現します：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean, 
    String,
    Function { /* ... */ },
    Unknown,
    Inferred(Box<Type>),  // ★新規追加：推論された型
}

impl Type {
    // 推論チェーンを辿って最終的な型を取得
    pub fn resolve(&self) -> &Type {
        match self {
            Type::Inferred(inner) => inner.resolve(),
            other => other,
        }
    }
}
```

この`Inferred`型により、「推論はしたが、まだ確定していない」状態を表現できます。

### 🔄 型推論の段階的処理

高度な型推論は2段階で行います：

```rust
impl AdvancedTypeChecker {
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
        // Phase 1: 基本的な型推論
        for statement in &program.statements {
            self.check_statement(statement)?;
        }

        // Phase 2: 推論の解決
        self.resolve_type_inference()?;

        Ok(())
    }
}
```

**Phase 1**では「明らかな型」を推論し、**Phase 2**では「関係性から導かれる型」を解決します。

## 🔧 型推論の仕組み（詳細解説）

### 🕵️ Phase 1: 基本型推論

まず、リテラルや明確な型から推論します：

```rust
fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
    match stmt {
        Stmt::LetDeclaration { name, value, type_annotation } => {
            // Step 1: 値の型を推論
            let inferred_type = self.infer_expression_type(value)?;
            
            // Step 2: 型注釈の処理
            let final_type = if let Some(annotated_type) = type_annotation {
                // 型注釈がある場合：一致チェック
                if *inferred_type.resolve() != *annotated_type.resolve() {
                    return Err("Type mismatch".to_string());
                }
                annotated_type.clone()
            } else {
                // 型注釈がない場合：推論した型を使用
                inferred_type
            };
            
            // Step 3: シンボルテーブルに登録
            self.symbol_table.define(name.clone(), final_type)?;
            Ok(())
        }
        // ...
    }
}
```

### 🧮 式の型推論の強化

二項演算での型推論も、推論型を考慮して強化します：

```rust
fn infer_expression_type(&mut self, expr: &Expr) -> Result<Type, String> {
    match expr {
        Expr::Binary { left, operator, right } => {
            let left_type = self.infer_expression_type(left)?;
            let right_type = self.infer_expression_type(right)?;
            
            // 推論型を解決してから比較
            let resolved_left = left_type.resolve();
            let resolved_right = right_type.resolve();
            
            match operator {
                BinaryOp::Add | /* ... */ => {
                    if *resolved_left == Type::Integer && *resolved_right == Type::Integer {
                        Ok(Type::Integer)
                    } else {
                        Err("Arithmetic requires integers".to_string())
                    }
                }
                // ...
            }
        }
        // ...
    }
}
```

### 📝 代入式の追加

新しく代入式を追加して、既存変数への値代入をサポートします：

```rust
Expr::Assignment { name, value } => {
    // Step 1: 変数が定義されているかチェック
    let existing_symbol = self.symbol_table.resolve(name)
        .ok_or_else(|| format!("Variable '{}' not defined", name))?;
    
    // Step 2: 代入値の型を推論
    let value_type = self.infer_expression_type(value)?;
    
    // Step 3: 型の一致チェック
    if *existing_symbol.symbol_type.resolve() != *value_type.resolve() {
        return Err("Assignment type mismatch".to_string());
    }
    
    // Step 4: 代入値の型を返す
    Ok(value_type)
}
```

## 🎬 実行例：型推論の流れ（ステップバイステップ）

### 📝 入力プログラム（型注釈なし）

```rust
let x = 42;           // 型注釈なし
let y = x + 10;       // 型注釈なし
let flag = x > 5;     // 型注釈なし
x = 100;              // 代入
```

### 🔍 Phase 1: 基本型推論

#### Step 1: `let x = 42;`

```rust
💭 型チェッカーの思考過程:
1. "値の型を推論しよう"
   → Number(42) → Type::Integer

2. "型注釈はない"
   → None → 推論した型をそのまま使用

3. "変数を定義しよう"
   → Symbol { name: "x", symbol_type: Type::Integer }

✅ 結果: xはi32型として定義
```

#### Step 2: `let y = x + 10;`

```rust
💭 型チェッカーの思考過程:
1. "二項演算の型を推論しよう"
   → Binary { left: Identifier("x"), op: Add, right: Number(10) }

2. "左辺の型を推論"
   → Identifier("x") → resolve("x") → Type::Integer

3. "右辺の型を推論"
   → Number(10) → Type::Integer

4. "加算演算の型チェック"
   → Add(Type::Integer, Type::Integer) → Type::Integer

5. "変数を定義"
   → Symbol { name: "y", symbol_type: Type::Integer }

✅ 結果: yはi32型として定義
```

#### Step 3: `let flag = x > 5;`

```rust
💭 型チェッカーの思考過程:
1. "比較演算の型を推論しよう"
   → Binary { left: Identifier("x"), op: GreaterThan, right: Number(5) }

2. "左辺の型: x → Type::Integer"
3. "右辺の型: 5 → Type::Integer"
4. "比較演算の結果: Type::Boolean"

5. "変数を定義"
   → Symbol { name: "flag", symbol_type: Type::Boolean }

✅ 結果: flagはbool型として定義
```

#### Step 4: `x = 100;` (代入式)

```rust
💭 型チェッカーの思考過程:
1. "変数xが定義されているかチェック"
   → resolve("x") → Symbol { symbol_type: Type::Integer }

2. "代入値の型を推論"
   → Number(100) → Type::Integer

3. "型の一致チェック"
   → Type::Integer == Type::Integer → OK

✅ 結果: 代入成功
```

### 📊 最終的なシンボルテーブル

```rust
symbols: {
    "x" → Symbol { name: "x", symbol_type: Type::Integer },
    "y" → Symbol { name: "y", symbol_type: Type::Integer },
    "flag" → Symbol { name: "flag", symbol_type: Type::Boolean }
}
```

## 💡 実装のポイント（あなたが書く部分）

今回実装する箇所は**3つ**です。lesson_3_9での学習を活かして、より考えて実装してください。

### 🎯 実装箇所1: 型注釈なしの変数定義

**場所**: `check_statement()` メソッド内の `Stmt::LetDeclaration` 部分

```rust
Stmt::LetDeclaration { name, value, type_annotation } => {
    // ヒント：
    // 1. 値の型を推論
    // 2. 型注釈がある場合は一致確認  
    // 3. 型注釈がない場合は推論された型を使用
    // 4. 推論された型がUnknownの場合は後で解決するためマーク
    
    todo!("型注釈なしの変数定義を実装してください")
}
```

**考えるポイント**: lesson_3_9との違いは、型注釈がない場合の処理です。

### 🎯 実装箇所2: 高度な二項演算の型推論

**場所**: `infer_expression_type()` メソッド内の `Expr::Binary` 部分

```rust
Expr::Binary { left, operator, right } => {
    // ヒント：
    // 1. 左右の型を推論
    // 2. 推論された型を解決してから比較
    // 3. オペレーターに応じた型チェック
    
    todo!("高度な二項演算の型推論を実装してください")
}
```

**考えるポイント**: `Type::resolve()`を使って推論型を解決してから比較する必要があります。

### 🎯 実装箇所3: 代入式の型チェック

**場所**: `infer_expression_type()` メソッド内の `Expr::Assignment` 部分

```rust
Expr::Assignment { name, value } => {
    // ヒント：
    // 1. 変数が定義されているかチェック
    // 2. 値の型を推論
    // 3. 既存の変数の型と一致するかチェック
    // 4. 代入される値の型を返す
    
    todo!("代入式の型チェックを実装してください")
}
```

**考えるポイント**: 新しい機能なので、どういう処理が必要か考えてみてください。

## ✅ 実装の進め方（シンプル3ステップ）

### Step 1: TODOを探す 🔍

lesson_3_10.rsファイルで `todo!("...")` の3箇所を探してください。

### Step 2: 実装 ✏️

今回はヒントがより抽象的です。lesson_3_9の知識を活かして考えて実装してください。

### Step 3: テスト実行 🧪

```bash
cargo test lesson_3::lesson_3_10
```

## 🎯 テストケース（3つ）

1. **`test_type_inference_without_annotation`**: 型注釈なしの推論
2. **`test_boolean_inference`**: 真偽値の推論
3. **`test_assignment_type_check`**: 代入式の型チェック

## 🔄 lesson_3_9からの進化

### lesson_3_9でできたこと
- ✅ 基本的な型システム
- ✅ 型注釈の一致チェック
- ✅ 型エラー検出

### lesson_3_10で新しく追加されること
- ✅ **型注釈なしの変数定義**: 推論だけで型決定
- ✅ **推論型の管理**: `Type::Inferred`で推論状態を表現
- ✅ **代入式のサポート**: `x = value;`の型チェック
- ✅ **段階的型解決**: より複雑な推論処理

### 変更されないもの
- ✅ 基本的なスコープ管理
- ✅ 関数管理
- ✅ エラー処理

## 🎉 完了後の効果

lesson_3_10が完了すると：
- **自然な型推論**: 型注釈なしでも安全なプログラミング
- **より実用的**: 実際のプログラミングに近い体験
- **rust-analyzerの核心**: 型推論エンジンの基本理解

**次のステップ**: lesson_3_11でさらに高度な機能（ジェネリクス等）に進みます！

**型推論は複雑ですが、段階的に理解していけば大丈夫です。がんばってください！**