# Lesson 3-7: while文のスコープ

lesson_3_6でif文のスコープができるようになりましたね。今度は、**while文でのスコープ管理**を学びます。

## 📚 while文のスコープとは？

### 🤔 ループ文でのスコープ

**while文**は繰り返し処理を行いますが、ループ本体が独立したスコープを持ちます：

```rust
let i = 0;
while i < 10 {
    let temp = i * 2;    // ループスコープ内
    print(temp);         // OK: 同じスコープのtemp
    i = i + 1;           // OK: 外側のi
}
print(i);                // OK: 外側のi
print(temp);             // Error: tempはwhile文の外で見えない
```

### rust-analyzerでの重要性

rust-analyzerは以下のためにwhile文スコープが必要です：
- **ループ内変数管理**: ループ本体のスコープ
- **反復処理解析**: ループ変数の有効範囲
- **制御フロー**: break/continueでのスコープ管理
- **エラー検出**: ループスコープ外での未定義変数

## 🎯 今回の目標

**入力**: while文を含むAST
```rust
Program {
    statements: [
        LetDeclaration { name: "counter", value: Number(0) },
        WhileStatement {
            condition: Binary { left: Identifier("counter"), op: LessThan, right: Number(3) },
            body: Block {
                statements: [
                    LetDeclaration { name: "step", value: Number(1) },
                    Assignment { target: "counter", value: Binary { left: Identifier("counter"), op: Add, right: Identifier("step") } }
                ]
            }
        },
        Expression(Identifier("counter")),  // OK
        Expression(Identifier("step"))      // Error: stepは見えない
    ]
}
```

**出力**: 正確なwhile文スコープ解析
```rust
// グローバル: counter定義
// while文内: step定義、counter&step参照OK
// while文外: counter参照OK、step参照Error
```

## 🏗️ AST構造の拡張

### while文ノードの追加

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
    Block { statements: Vec<Stmt> },
    IfStatement {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    WhileStatement {                 // 新規追加
        condition: Expr,
        body: Box<Stmt>,
    },
}
```

### 比較演算子の追加

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,                        // 新規追加
}
```

## 🔧 while文のスコープ解析

### 1. while文解析の基本フロー

```rust
fn analyze_while_statement(&mut self, condition: &Expr, body: &Stmt) -> Result<(), String> {
    // 1. 条件式を解析（現在のスコープで）
    self.analyze_expression(condition)?;
    
    // 2. ループ本体を解析（新しいスコープで）
    self.analyze_statement(body)?;
    
    Ok(())
}
```

### 2. while文の特徴

1. **条件式**: 現在のスコープで評価
2. **ループ本体**: 独立したスコープ
3. **反復実行**: 同じスコープで複数回実行
4. **ループ後**: ループ内の変数は見えない

## 🎬 実行例：基本的なwhile文

### 入力コード

```rust
let count = 0;
while count < 3 {
    let message = "iteration";
    print(count);      // OK: 外側のcount
    print(message);    // OK: 同じスコープのmessage
    count = count + 1; // OK: 外側のcountを更新
}
print(count);          // OK: 外側のcount
print(message);        // Error: messageは見えない
```

### 解析プロセス

#### Step 1: let count = 0; (レベル0)
```rust
scope_level: 0
symbols: {"count": Symbol{scope_level: 0}}
```

#### Step 2: while count < 3 の条件解析
```rust
resolve("count"): 現在のスコープ（レベル0）で発見 → OK
```

#### Step 3: ループ本体開始 (レベル1)
```rust
// Blockの場合、enter_scope()が自動実行される
scope_level: 1
symbols: {}
parent: レベル0のスコープ
```

#### Step 4: let message = "iteration"; (レベル1)
```rust
scope_level: 1
symbols: {"message": Symbol{scope_level: 1}}
parent: レベル0のスコープ
```

#### Step 5: ループ内での変数参照
```rust
resolve("count"): レベル1→レベル0で発見 → OK
resolve("message"): レベル1で発見 → OK
```

#### Step 6: ループ本体終了
```rust
// Blockの場合、exit_scope()が自動実行される
scope_level: 0 (元に戻る)
```

#### Step 7: ループ外での変数参照
```rust
resolve("count"): レベル0で発見 → OK
resolve("message"): レベル0で見つからない → Error
```

## 🔍 while文の特殊な考慮事項

### 1. 条件式での変数更新

```rust
let i = 0;
while i < 5 {
    print(i);
    i = i + 1;  // 条件で使用される変数の更新
}
```

### 2. ループ内でのシャドウイング

```rust
let x = 1;
while x < 3 {
    let x = 10;  // 外側のxをシャドウ（条件では元のxが使われる）
    print(x);    // 10
}
```

### 3. 無限ループでのスコープ

```rust
while true {
    let temp = 42;
    if some_condition() {
        break;   // tempのスコープ終了
    }
}
// tempは見えない
```

## 💡 実装のポイント

### 1. while文解析の実装

```rust
Stmt::WhileStatement { condition, body } => {
    // TODO: 実装してください
    // ヒント：
    // 1. self.analyze_expression(condition)?; で条件解析
    // 2. self.analyze_statement(body)?; でループ本体解析
    // 3. Blockの場合は自動でスコープ管理される
    
    todo!("while文解析を実装してください")
}
```

### 2. LessThan演算子の追加

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    GreaterThan,
    LessThan,  // TODO: 追加してください
}
```

## 🐛 エラーケースの例

### 1. ループ内変数の外部参照

```rust
while true {
    let loop_var = 42;
}
print(loop_var);  // Error: loop_varは見えない
```

### 2. 条件式での未定義変数

```rust
while undefined_counter < 10 {  // Error: undefined_counter not defined
    print("loop");
}
```

### 3. ループ内での外側変数参照（OK）

```rust
let limit = 5;
while counter < limit {  // OK: 外側のlimitが見える
    counter = counter + 1;
}
```

## ✅ 実装の進め方

1. **AST拡張**: WhileStatementをStmtに追加
2. **演算子追加**: LessThanをBinaryOpに追加
3. **解析メソッド修正**: analyze_statementにWhileStatement処理を追加
4. **テストケース作成**: 基本的なwhile文スコープのテスト

**実行コマンド**: `cargo test lesson_3::lesson_3_7`

## 🎯 テストケース（3つ）

1. **基本while文**: ループ本体のスコープ管理
2. **ループ内外変数**: 外側変数へのアクセスと内側変数のスコープ制限
3. **条件エラー**: 条件式での未定義変数エラー

## 🔄 lesson_3_6からの進化

### 追加機能
- ✅ while文ノードの解析
- ✅ ループスコープ管理
- ✅ LessThan演算子

### 継承機能
- ✅ if文スコープ管理
- ✅ ネストしたスコープ管理
- ✅ シャドウイング処理

## 🎉 完了後の効果

lesson_3_7が完了すると：
- ループ構造の基本的な解析
- 反復処理でのスコープ管理
- **lesson_3_8**で関数スコープに進む準備完了

**while文はループ解析の基盤なので、しっかり実装しましょう！**