# Lesson 3-8: 関数スコープ

lesson_3_7でwhile文のスコープができるようになりましたね。今度は、**関数でのスコープ管理**を学びます。

## 📚 関数スコープとは？

### 🤔 関数の独立したスコープ

**関数**は独立したスコープを持ち、パラメータと内部変数を管理します：

```rust
let global_var = 10;

fn calculate(x: i32, y: i32) -> i32 {
    let temp = x + y;        // 関数スコープ内
    print(temp);             // OK: 同じスコープのtemp
    print(global_var);       // OK: グローバル変数
    return temp;
}

print(global_var);           // OK: グローバル変数
print(temp);                 // Error: tempは関数の外で見えない
print(x);                    // Error: パラメータも外で見えない
```

### rust-analyzerでの重要性

rust-analyzerは以下のために関数スコープが必要です：
- **関数内変数管理**: 関数ローカル変数のスコープ
- **パラメータ管理**: 関数パラメータの有効範囲
- **関数解決**: 関数呼び出しの解析
- **戻り値解析**: 戻り値型の推論支援

## 🎯 今回の目標

**入力**: 関数定義を含むAST
```rust
Program {
    statements: [
        LetDeclaration { name: "global", value: Number(42) },
        FunctionDeclaration {
            name: "add",
            parameters: [Parameter { name: "a" }, Parameter { name: "b" }],
            body: Block {
                statements: [
                    LetDeclaration { name: "result", value: Binary { left: Identifier("a"), op: Add, right: Identifier("b") } },
                    Expression(Identifier("global")),  // OK: グローバル変数
                    Expression(Identifier("result"))   // OK: ローカル変数
                ]
            }
        },
        Expression(Identifier("global")),  // OK
        Expression(Identifier("result"))   // Error: resultは見えない
    ]
}
```

**出力**: 正確な関数スコープ解析
```rust
// グローバル: global定義、add関数定義
// 関数内: a,b,result定義、全て参照OK
// 関数外: global参照OK、result参照Error
```

## 🏗️ AST構造の拡張

### 関数定義ノードの追加

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
    WhileStatement {
        condition: Expr,
        body: Box<Stmt>,
    },
    FunctionDeclaration {             // 新規追加
        name: String,
        parameters: Vec<Parameter>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {               // 新規追加
    pub name: String,
}
```

### 関数呼び出し（既存のFunctionCall）

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Identifier(String),
    Binary { /* ... */ },
    FunctionCall {               // 既存（拡張）
        name: String,
        arguments: Vec<Expr>,
    },
}
```

## 🔧 関数スコープの解析

### 1. 関数定義の解析フロー

```rust
fn analyze_function_declaration(&mut self, name: &str, parameters: &[Parameter], body: &Stmt) -> Result<(), String> {
    // 1. 関数名をグローバルスコープに定義
    self.symbol_table.define(name.to_string())?;
    
    // 2. 関数スコープを開始
    self.symbol_table.enter_scope();
    
    // 3. パラメータを関数スコープに定義
    for param in parameters {
        self.symbol_table.define(param.name.clone())?;
    }
    
    // 4. 関数本体を解析
    self.analyze_statement(body)?;
    
    // 5. 関数スコープを終了
    self.symbol_table.exit_scope();
    
    Ok(())
}
```

### 2. 関数の特徴

1. **関数名**: グローバルスコープに定義
2. **パラメータ**: 関数スコープに定義
3. **関数本体**: 関数スコープで実行
4. **関数外**: 関数内の変数は見えない

## 🎬 実行例：基本的な関数

### 入力コード

```rust
let global_value = 100;

fn multiply(x: i32, y: i32) -> i32 {
    let temp = x * y;
    print(global_value);  // OK: グローバル変数
    print(temp);          // OK: ローカル変数
    print(x);             // OK: パラメータ
    return temp;
}

print(global_value);      // OK: グローバル変数
let result = multiply(5, 6);  // OK: 関数呼び出し
print(temp);              // Error: tempは見えない
print(x);                 // Error: xは見えない
```

### 解析プロセス

#### Step 1: let global_value = 100; (レベル0)
```rust
scope_level: 0
symbols: {"global_value": Symbol{scope_level: 0}}
```

#### Step 2: 関数定義開始
```rust
// 1. 関数名をグローバルに追加
symbols: {"global_value": Symbol{scope_level: 0}, "multiply": Symbol{scope_level: 0}}

// 2. 関数スコープ開始
scope_level: 1
symbols: {}
parent: レベル0のスコープ
```

#### Step 3: パラメータ定義 (レベル1)
```rust
scope_level: 1
symbols: {"x": Symbol{scope_level: 1}, "y": Symbol{scope_level: 1}}
parent: レベル0のスコープ
```

#### Step 4: 関数本体 - let temp = x * y; (レベル1)
```rust
scope_level: 1
symbols: {"x": Symbol{scope_level: 1}, "y": Symbol{scope_level: 1}, "temp": Symbol{scope_level: 1}}
parent: レベル0のスコープ
```

#### Step 5: 関数内での変数参照
```rust
resolve("global_value"): レベル1→レベル0で発見 → OK
resolve("temp"): レベル1で発見 → OK
resolve("x"): レベル1で発見 → OK
```

#### Step 6: 関数定義終了
```rust
// 関数スコープ終了
scope_level: 0 (元に戻る)
symbols: {"global_value": Symbol{scope_level: 0}, "multiply": Symbol{scope_level: 0}}
```

#### Step 7: 関数外での変数参照
```rust
resolve("global_value"): レベル0で発見 → OK
resolve("multiply"): レベル0で発見 → OK (関数呼び出し)
resolve("temp"): レベル0で見つからない → Error
resolve("x"): レベル0で見つからない → Error
```

## 🔍 関数呼び出しの解析

### 1. 関数呼び出しの確認

```rust
fn analyze_function_call(&mut self, name: &str, arguments: &[Expr]) -> Result<(), String> {
    // 1. 関数名が定義されているか確認
    if self.symbol_table.resolve(name).is_none() {
        return Err(format!("Function '{}' not defined", name));
    }
    
    // 2. 引数式を解析
    for arg in arguments {
        self.analyze_expression(arg)?;
    }
    
    Ok(())
}
```

### 2. 関数呼び出しの例

```rust
// 関数定義
fn add(a: i32, b: i32) -> i32 { return a + b; }

// 関数呼び出し
let x = 5;
let y = 10;
let sum = add(x, y);  // OK: add関数が定義済み、x,yも定義済み
let bad = unknown_func(x);  // Error: unknown_func not defined
```

## 💡 実装のポイント

### 1. 関数定義解析の実装

```rust
Stmt::FunctionDeclaration { name, parameters, body } => {
    // TODO: 実装してください
    // ヒント：
    // 1. self.symbol_table.define(name.clone())? で関数名をグローバルに定義
    // 2. self.symbol_table.enter_scope() で関数スコープ開始
    // 3. parameters.iter().try_for_each(|p| self.symbol_table.define(p.name.clone())) でパラメータ定義
    // 4. self.analyze_statement(body)? で関数本体解析
    // 5. self.symbol_table.exit_scope() で関数スコープ終了
    
    todo!("関数定義解析を実装してください")
}
```

### 2. 関数呼び出し解析の実装

```rust
Expr::FunctionCall { name, arguments } => {
    // TODO: 実装してください
    // ヒント：
    // 1. self.symbol_table.resolve(name).ok_or_else(|| format!("Function '{}' not defined", name))?; で関数存在確認
    // 2. arguments.iter().try_for_each(|arg| self.analyze_expression(arg)) で引数解析
    
    todo!("関数呼び出し解析を実装してください")
}
```

## 🐛 エラーケースの例

### 1. 関数内変数の外部参照

```rust
fn test() {
    let local_var = 42;
}
print(local_var);  // Error: local_varは見えない
```

### 2. 未定義関数の呼び出し

```rust
let result = unknown_function(1, 2);  // Error: unknown_function not defined
```

### 3. 関数内での外側変数参照（OK）

```rust
let global = 100;
fn test() {
    print(global);  // OK: グローバル変数が見える
}
```

## ✅ 実装の進め方

1. **AST拡張**: FunctionDeclaration、Parameterを追加
2. **解析メソッド拡張**: analyze_statementにFunctionDeclaration処理を追加
3. **関数呼び出し修正**: analyze_expressionのFunctionCall処理を修正
4. **テストケース作成**: 基本的な関数スコープのテスト

**実行コマンド**: `cargo test lesson_3::lesson_3_8`

## 🎯 テストケース（3つ）

1. **基本関数定義**: 関数スコープとパラメータ管理
2. **関数呼び出し**: 関数存在確認と引数解析
3. **スコープエラー**: 関数内変数の外部参照エラー

## 🔄 lesson_3_7からの進化

### 追加機能
- ✅ 関数定義ノードの解析
- ✅ パラメータスコープ管理
- ✅ 関数呼び出し確認

### 継承機能
- ✅ while文スコープ管理
- ✅ if文スコープ管理
- ✅ ネストしたスコープ管理

## 🎉 完了後の効果

lesson_3_8が完了すると：
- 関数構造の基本的な解析
- パラメータとローカル変数の管理
- **lesson_3_9**で型システムの基礎に進む準備完了

**関数はプログラムの基本単位なので、しっかり実装しましょう！**