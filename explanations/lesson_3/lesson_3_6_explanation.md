# Lesson 3-6: if文のスコープ

lesson_3_5でシャドウイングができるようになりましたね。今度は、**if文でのスコープ管理**を学びます。

## 📚 if文のスコープとは？

### 🤔 条件分岐でのスコープ

**if文**は条件分岐を行いますが、それぞれの分岐（then/else）が独立したスコープを持ちます：

```rust
let x = 1;
if true {
    let y = 2;       // then分岐のスコープ
    print(x + y);    // OK: x=1, y=2
}
print(x);            // OK: x=1
print(y);            // Error: yはif文の外で見えない
```

### rust-analyzerでの重要性

rust-analyzerは以下のためにif文スコープが必要です：
- **条件分岐の正確な解析**: then/elseの独立スコープ
- **変数の有効範囲**: 分岐内で定義された変数の管理
- **コード補完**: 分岐ごとの適切な候補表示
- **エラー検出**: 分岐スコープ外での未定義変数

## 🎯 今回の目標

**入力**: if文を含むAST
```rust
Program {
    statements: [
        LetDeclaration { name: "x", value: Number(1) },
        IfStatement {
            condition: Identifier("x"),
            then_branch: Block {
                statements: [
                    LetDeclaration { name: "y", value: Number(2) },
                    Expression(Binary { left: Identifier("x"), right: Identifier("y") })
                ]
            },
            else_branch: None
        },
        Expression(Identifier("x")),  // OK
        Expression(Identifier("y"))   // Error: yは見えない
    ]
}
```

**出力**: 正確なif文スコープ解析
```rust
// グローバル: x定義
// if文内: y定義、x&y参照OK
// if文外: x参照OK、y参照Error
```

## 🏗️ AST構造の拡張

### if文ノードの追加

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
    Block { statements: Vec<Stmt> },
    IfStatement {                    // 新規追加
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
}
```

## 🔧 if文のスコープ解析

### 1. if文解析の基本フロー

```rust
fn analyze_if_statement(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Box<Stmt>>) -> Result<(), String> {
    // 1. 条件式を解析（現在のスコープで）
    self.analyze_expression(condition)?;
    
    // 2. then分岐を新しいスコープで解析
    self.analyze_statement(then_branch)?;
    
    // 3. else分岐がある場合、新しいスコープで解析
    if let Some(else_stmt) = else_branch {
        self.analyze_statement(else_stmt)?;
    }
    
    Ok(())
}
```

### 2. 重要な原則

1. **条件式**: 現在のスコープで評価
2. **then分岐**: 独立したスコープ
3. **else分岐**: 独立したスコープ（thenとは別）
4. **分岐後**: 分岐内の変数は見えない

## 🎬 実行例：基本的なif文

### 入力コード

```rust
let x = 1;
if x > 0 {
    let positive = true;
    print(x);           // OK: 外側のx
    print(positive);    // OK: 同じスコープのpositive
}
print(x);               // OK: 外側のx
print(positive);        // Error: positiveは見えない
```

### 解析プロセス

#### Step 1: let x = 1; (レベル0)
```rust
scope_level: 0
symbols: {"x": Symbol{scope_level: 0}}
```

#### Step 2: if x > 0 の条件解析
```rust
resolve("x"): 現在のスコープ（レベル0）で発見 → OK
```

#### Step 3: then分岐開始 (レベル1)
```rust
// Blockの場合、enter_scope()が自動実行される
scope_level: 1
symbols: {}
parent: レベル0のスコープ
```

#### Step 4: let positive = true; (レベル1)
```rust
scope_level: 1
symbols: {"positive": Symbol{scope_level: 1}}
parent: レベル0のスコープ
```

#### Step 5: 分岐内での変数参照
```rust
resolve("x"): レベル1→レベル0で発見 → OK
resolve("positive"): レベル1で発見 → OK
```

#### Step 6: then分岐終了
```rust
// Blockの場合、exit_scope()が自動実行される
scope_level: 0 (元に戻る)
```

#### Step 7: 分岐外での変数参照
```rust
resolve("x"): レベル0で発見 → OK
resolve("positive"): レベル0で見つからない → Error
```

## 🔍 if-else文の場合

### より複雑な例

```rust
let x = 1;
if x > 0 {
    let msg = "positive";
    print(msg);
} else {
    let msg = "not positive";  // 別のスコープなので同名でもOK
    print(msg);
}
print(x);    // OK
print(msg);  // Error: どちらのmsgも見えない
```

### 解析の流れ

1. **条件式**: 現在のスコープで`x > 0`を評価
2. **then分岐**: 新しいスコープでthen_branchを解析
3. **else分岐**: 新しいスコープでelse_branchを解析
4. **分岐後**: 両方の分岐の変数は見えない

## 🔧 実装のポイント

### 1. IfStatement解析の実装

```rust
fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
    match stmt {
        Stmt::LetDeclaration { name, value } => {
            self.analyze_expression(value)?;
            self.symbol_table.define(name.clone())
        }
        Stmt::Expression(expr) => self.analyze_expression(expr),
        Stmt::Block { statements } => {
            self.symbol_table.enter_scope();
            
            for stmt in statements {
                self.analyze_statement(stmt)?;
            }
            
            self.symbol_table.exit_scope();
            Ok(())
        }
        Stmt::IfStatement { condition, then_branch, else_branch } => {
            // 条件式を現在のスコープで解析
            self.analyze_expression(condition)?;
            
            // then分岐を解析（Blockの場合は自動でスコープ管理）
            self.analyze_statement(then_branch)?;
            
            // else分岐を解析（ある場合）
            if let Some(else_stmt) = else_branch {
                self.analyze_statement(else_stmt)?;
            }
            
            Ok(())
        }
    }
}
```

### 2. 重要な注意点

- **条件式は分岐前**: 現在のスコープで評価
- **各分岐は独立**: then/elseは別々のスコープ
- **Block自動処理**: Blockは既にスコープ管理を行う

## 🐛 エラーケースの例

### 1. 分岐内変数の外部参照

```rust
if true {
    let temp = 42;
}
print(temp);  // Error: tempは見えない
```

### 2. 条件式での未定義変数

```rust
if undefined_var > 0 {  // Error: undefined_var not defined
    print("positive");
}
```

### 3. 分岐内での外側変数参照（OK）

```rust
let x = 1;
if true {
    print(x);  // OK: 外側のxが見える
}
```

## ✅ 実装の進め方

1. **AST拡張**: IfStatementをStmtに追加
2. **解析メソッド追加**: analyze_statementにIfStatement処理を追加
3. **テストケース作成**: 基本的なif文スコープのテスト
4. **エラーケース確認**: 分岐外での変数参照エラー

**実行コマンド**: `cargo test lesson_3::lesson_3_6`

## 🎯 テストケース（3つ）

1. **基本if文**: then分岐のみのスコープ管理
2. **if-else文**: then/else両分岐の独立スコープ
3. **分岐外エラー**: 分岐内変数の外部参照エラー

## 🔄 lesson_3_5からの進化

### 追加機能
- ✅ if文ノードの解析
- ✅ 条件分岐スコープ管理
- ✅ then/else独立スコープ

### 継承機能
- ✅ ネストしたスコープ管理
- ✅ シャドウイング処理
- ✅ 階層的変数検索

## 🎉 完了後の効果

lesson_3_6が完了すると：
- 制御フロー構造の基本的な解析
- 条件分岐でのスコープ管理
- **lesson_3_7**でwhile文のスコープに進む準備完了

**if文は制御フロー解析の基盤なので、しっかり理解しましょう！**