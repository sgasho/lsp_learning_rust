# Lesson 3-12: 構造体とフィールドアクセス

lesson_3_11でエラー回復システムができるようになりましたね。今度は、**構造体とフィールドアクセス**を学びます。

## 📚 構造体とは？

### 🤔 なぜ構造体が重要？

構造体は、**関連するデータをまとめて管理**する仕組みです。プログラムが複雑になると、個別の変数では管理が困難になります：

```rust
// 構造体なしの管理（煩雑）
let person_name = "Alice";
let person_age = 25;
let person_email = "alice@example.com";

let another_name = "Bob";
let another_age = 30;
let another_email = "bob@example.com";

// 構造体ありの管理（整理された）
struct Person {
    name: String,
    age: i32,
    email: String,
}

let alice = Person { name: "Alice", age: 25, email: "alice@example.com" };
let bob = Person { name: "Bob", age: 30, email: "bob@example.com" };
```

### 🎯 構造体の役割

構造体は「データの設計図」のようなものです：

1. **データの組織化**: 関連するフィールドをグループ化
2. **型安全性**: フィールドごとに型を指定
3. **再利用性**: 同じ構造を何度でも使用
4. **可読性**: コードの意図が明確になる

### rust-analyzerでの重要性

rust-analyzerは以下のために構造体解析が必要です：
- **フィールド補完**: `person.`を入力すると`name`, `age`, `email`を提案
- **型チェック**: `person.age = "hello"`のような型エラーを検出
- **参照検索**: フィールドの使用箇所を特定
- **リファクタリング**: フィールド名の一括変更

## 🎯 今回の目標（構造体システム）

**入力**: 構造体定義とその使用
```rust
struct Person {
    name: String,
    age: i32,
}

let alice = Person { name: "Alice", age: 25 };
let name = alice.name;     // フィールドアクセス
let invalid = alice.email; // Error: 存在しないフィールド
```

**出力**: 構造体とフィールドの型チェック
```rust
// ✅ Person構造体を定義
// ✅ alice: Person型として定義
// ✅ alice.name: String型として推論
// ❌ alice.email: エラー（フィールドが存在しない）
```

### 🔍 今回学ぶ新機能

1. **構造体定義**: `struct Name { fields... }`
2. **構造体コンストラクタ**: `Name { field: value, ... }`
3. **フィールドアクセス**: `object.field`
4. **フィールドの型チェック**: 存在確認と型推論

## 🏗️ 構造体システムの構造

### 🧩 構造体型の表現

lesson_3_11の`Type`enumを拡張して、構造体型を表現します：

```rust
// フィールドの定義
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,      // フィールド名
    pub field_type: Type,  // フィールドの型
    pub span: Span,        // 定義位置
}

// 型情報に構造体を追加
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Boolean,
    String,
    Function { /* ... */ },
    
    // ★新規追加：構造体型
    Struct {
        name: String,       // 構造体名
        fields: Vec<Field>, // フィールドリスト
    },
    
    Unknown,
    Inferred(Box<Type>),
}
```

この拡張により、`Person`のような構造体型を表現できます。

### 🔧 AST への構造体ノード追加

構造体定義とフィールドアクセスをASTで表現します：

```rust
// 文に構造体定義を追加
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { /* ... */ },
    // ... 他の文 ...
    
    // ★新規追加：構造体定義
    StructDeclaration {
        name: String,        // 構造体名
        fields: Vec<Field>,  // フィールドリスト
        span: Span,          // 定義位置
    },
}

// 式にフィールドアクセスと構造体コンストラクタを追加
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64, Span),
    // ... 他の式 ...
    
    // ★新規追加：フィールドアクセス
    FieldAccess {
        object: Box<Expr>,    // アクセス対象（例：person）
        field_name: String,   // フィールド名（例：name）
        span: Span,
    },
    
    // ★新規追加：構造体コンストラクタ
    StructConstructor {
        struct_name: String,              // 構造体名
        field_values: Vec<(String, Expr)>, // (フィールド名, 値)
        span: Span,
    },
}
```

### 🔍 フィールド検索機能

構造体型からフィールドを検索する機能を追加します：

```rust
impl Type {
    // 構造体のフィールドを検索
    pub fn get_field(&self, field_name: &str) -> Option<&Field> {
        match self.resolve() {
            Type::Struct { fields, .. } => {
                fields.iter().find(|field| field.name == field_name)
            }
            _ => None,
        }
    }
}
```

これにより、`person.name`のようなアクセスで`name`フィールドが存在するかチェックできます。

## 🔧 構造体解析の仕組み（詳細解説）

### 🏗️ 構造体定義の処理

構造体定義では以下をチェックします：

```rust
Stmt::StructDeclaration { name, fields, span } => {
    // Step 1: フィールドの重複チェック
    let mut field_names = std::collections::HashSet::new();
    for field in fields {
        if !field_names.insert(&field.name) {
            // 重複フィールドエラー
            self.diagnostics.push(Diagnostic::error(
                format!("Field '{}' is defined multiple times", field.name),
                field.span.clone()
            ));
        }
    }
    
    // Step 2: 構造体型を作成
    let struct_type = Type::Struct {
        name: name.clone(),
        fields: fields.clone(),
    };
    
    // Step 3: 構造体をシンボルテーブルに定義
    if let Err(diagnostic) = self.symbol_table.define(name.clone(), struct_type, span.clone()) {
        self.diagnostics.push(diagnostic);
    }
}
```

### 🏭 構造体コンストラクタの処理

構造体の作成では以下をチェックします：

```rust
Expr::StructConstructor { struct_name, field_values, span } => {
    // Step 1: 構造体型が定義されているかチェック
    let struct_symbol = self.symbol_table.resolve(struct_name)?;
    
    if let Type::Struct { fields, .. } = &struct_symbol.symbol_type {
        // Step 2: 提供されたフィールドが全て存在するかチェック
        for (field_name, _value) in field_values {
            if !fields.iter().any(|f| f.name == *field_name) {
                self.diagnostics.push(Diagnostic::error(
                    format!("Struct '{}' has no field '{}'", struct_name, field_name),
                    span.clone()
                ));
            }
        }
        
        // Step 3: 各フィールドの値の型をチェック
        for (field_name, value) in field_values {
            if let Some(field) = fields.iter().find(|f| f.name == *field_name) {
                if let Some(value_type) = self.infer_expression_type(value) {
                    if *value_type.resolve() != *field.field_type.resolve() {
                        self.diagnostics.push(Diagnostic::error(
                            format!("Type mismatch for field '{}': expected {:?}, found {:?}",
                                field_name, field.field_type, value_type),
                            value.span().clone()
                        ));
                    }
                }
            }
        }
        
        // Step 4: 構造体型を返す
        Some(struct_symbol.symbol_type.clone())
    } else {
        self.diagnostics.push(Diagnostic::error(
            format!("'{}' is not a struct", struct_name),
            span.clone()
        ));
        None
    }
}
```

### 🎯 フィールドアクセスの処理

フィールドアクセスでは以下をチェックします：

```rust
Expr::FieldAccess { object, field_name, span } => {
    // Step 1: オブジェクトの型を推論
    if let Some(object_type) = self.infer_expression_type(object) {
        // Step 2: 構造体型かチェック
        if let Some(field) = object_type.get_field(field_name) {
            // Step 3: フィールドの型を返す
            Some(field.field_type.clone())
        } else {
            // Step 4: フィールドが見つからない場合
            match object_type.resolve() {
                Type::Struct { name, .. } => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("Struct '{}' has no field '{}'", name, field_name),
                        span.clone()
                    ));
                }
                _ => {
                    self.diagnostics.push(Diagnostic::error(
                        format!("Cannot access field '{}' on non-struct type {:?}", 
                            field_name, object_type.resolve()),
                        span.clone()
                    ));
                }
            }
            None
        }
    } else {
        None
    }
}
```

## 🎬 実行例：構造体システムの流れ

### 📝 入力プログラム（構造体の定義と使用）

```rust
struct Person {
    name: String,
    age: i32,
}

let alice = Person { name: "Alice", age: 25 };
let alice_name = alice.name;
let invalid = alice.email;  // Error: 存在しないフィールド
```

### 🔍 構造体解析プロセス

#### Step 1: 構造体定義 `struct Person { ... }`

```rust
💭 構造体対応型チェッカーの思考過程:
1. "フィールドの重複をチェックしよう"
   → ["name", "age"] → 重複なし → OK

2. "構造体型を作成しよう"
   → Type::Struct {
       name: "Person",
       fields: [
         Field { name: "name", field_type: Type::String },
         Field { name: "age", field_type: Type::Integer }
       ]
     }

3. "構造体をシンボルテーブルに定義しよう"
   → define("Person", struct_type) → OK

✅ 結果: Person構造体が定義される
```

#### Step 2: 構造体コンストラクタ `Person { name: "Alice", age: 25 }`

```rust
💭 構造体対応型チェッカーの思考過程:
1. "Person構造体が定義されているかチェック"
   → resolve("Person") → Type::Struct { ... } → OK

2. "提供されたフィールドが存在するかチェック"
   → "name" → fields.find("name") → OK
   → "age" → fields.find("age") → OK

3. "各フィールドの値の型をチェック"
   → "name": String("Alice") → Type::String → 期待型Type::String → 一致 → OK
   → "age": Number(25) → Type::Integer → 期待型Type::Integer → 一致 → OK

4. "構造体型を返す"
   → Type::Struct { name: "Person", ... }

✅ 結果: alice: Person型として定義
```

#### Step 3: フィールドアクセス `alice.name`

```rust
💭 構造体対応型チェッカーの思考過程:
1. "オブジェクトの型を推論しよう"
   → Identifier("alice") → resolve("alice") → Type::Struct { name: "Person", ... }

2. "nameフィールドが存在するかチェック"
   → Type::Struct.get_field("name") → Field { field_type: Type::String } → OK

3. "フィールドの型を返す"
   → Type::String

✅ 結果: alice_name: String型として推論
```

#### Step 4: 無効なフィールドアクセス `alice.email` （エラーケース）

```rust
💭 構造体対応型チェッカーの思考過程:
1. "オブジェクトの型を推論しよう"
   → Identifier("alice") → Type::Struct { name: "Person", ... }

2. "emailフィールドが存在するかチェック"
   → Type::Struct.get_field("email") → None

3. "エラーを報告"
   → diagnostics.push(Error: "Struct 'Person' has no field 'email'")

❌ 結果: Error: "Struct 'Person' has no field 'email'"
```

### 📊 最終的なシンボルテーブル

```rust
symbols: {
    "Person" → Symbol { 
        symbol_type: Type::Struct { 
            name: "Person", 
            fields: [
                Field { name: "name", field_type: Type::String },
                Field { name: "age", field_type: Type::Integer }
            ]
        }
    },
    "alice" → Symbol { symbol_type: Type::Struct { name: "Person", ... } },
    "alice_name" → Symbol { symbol_type: Type::String }
}
```

## 💡 実装のポイント（あなたが書く部分）

今回実装する箇所は**3つ**です。構造体システムの考え方を理解して実装してください。

### 🎯 実装箇所1: 構造体定義の処理

**場所**: `check_statement()` メソッド内の `Stmt::StructDeclaration` 部分

```rust
Stmt::StructDeclaration { name, fields, span } => {
    // ヒント：
    // 1. 構造体型を作成
    // 2. フィールドの重複チェック
    // 3. 構造体をシンボルテーブルに定義

    todo!("構造体定義の処理を実装してください")
}
```

**考えるポイント**: フィールドの重複チェックと構造体型の作成。

### 🎯 実装箇所2: フィールドアクセスの処理

**場所**: `infer_expression_type()` メソッド内の `Expr::FieldAccess` 部分

```rust
Expr::FieldAccess { object, field_name, span } => {
    // ヒント：
    // 1. オブジェクトの型を推論
    // 2. 構造体型かチェック
    // 3. フィールドが存在するかチェック
    // 4. フィールドの型を返す

    todo!("フィールドアクセスの処理を実装してください")
}
```

**考えるポイント**: `Type::get_field()`メソッドを活用する。

### 🎯 実装箇所3: 構造体コンストラクタの処理

**場所**: `infer_expression_type()` メソッド内の `Expr::StructConstructor` 部分

```rust
Expr::StructConstructor { struct_name, field_values, span } => {
    // ヒント：
    // 1. 構造体型が定義されているかチェック
    // 2. 提供されたフィールドが全て存在するかチェック
    // 3. 各フィールドの値の型をチェック
    // 4. 構造体型を返す

    todo!("構造体コンストラクタの処理を実装してください")
}
```

**考えるポイント**: フィールドの存在確認と型の一致チェック。

## ✅ 実装の進め方

### Step 1: TODOを探す 🔍

lesson_3_12.rsファイルで `todo!("...")` の3箇所を探してください。

### Step 2: 構造体システム実装 ✏️

構造体の定義、作成、フィールドアクセスの3つの機能を実装してください。

### Step 3: テスト実行 🧪

```bash
cargo test lesson_3::lesson_3_12
```

## 🎯 テストケース（3つ）

1. **`test_basic_struct_definition`**: 基本的な構造体定義と作成
2. **`test_field_access`**: フィールドアクセスの正常動作
3. **`test_field_access_errors`**: フィールドアクセスのエラーケース

## 🔄 lesson_3_11からの進化

### lesson_3_11でできたこと
- ✅ エラー回復システム
- ✅ 位置情報付きエラー報告
- ✅ 複数エラーの収集

### lesson_3_12で新しく追加されること
- ✅ **構造体型**: `Type::Struct`で構造体を表現
- ✅ **構造体定義**: `struct Name { fields... }`の解析
- ✅ **フィールドアクセス**: `object.field`の型チェック
- ✅ **構造体コンストラクタ**: `Name { field: value }`の型チェック

### 変更されないもの
- ✅ エラー回復システム
- ✅ 基本的な型システム
- ✅ スコープ管理

## 🎉 完了後の効果

lesson_3_12が完了すると：
- **データ構造の表現**: 複雑なデータをまとめて管理
- **型安全なフィールドアクセス**: 存在しないフィールドへのアクセスを防止
- **ライフタイムの準備**: lesson_3_13で参照とライフタイムを学ぶ基盤完成

**次のステップ**: lesson_3_13でライフタイムの基本を学習し、あなたが興味を持っている分野に進みます！

**構造体はデータ管理の基本です。しっかり実装しましょう！**