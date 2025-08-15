# Lesson 2-5: 変数宣言とlet文の解析

lesson_2_4で関数呼び出しを学びましたね。今度は、**変数宣言とlet文の構文解析**について学びます。

## 📚 文（Statement）と式（Expression）の違い

### 🤔 プログラムの基本構造

プログラムは**文（Statement）**の集合です：

```rust
let x = 42;        // 文
let y = x + 1;     // 文
print(y);          // 文
```

### 文と式の違い

#### **式（Expression）** - 値を返す
```rust
42              // 数値式
x + 1           // 二項演算式
func(x, y)      // 関数呼び出し式
(x + y) * z     // 括弧付き式
```

#### **文（Statement）** - 動作を実行する
```rust
let x = 42;     // 変数宣言文
print(x);       // 式文（式 + セミコロン）
```

## 🌳 ASTの拡張

### 新しいASTノード

```rust
// 文（Statement）
enum Stmt {
    LetDeclaration {
        name: String,
        value: Expr,
    },
    Expression(Expr),  // 式文
}

// プログラム全体
struct Program {
    statements: Vec<Stmt>,
}
```

### ASTの階層構造

```
Program
├─ statements: Vec<Stmt>
   ├─ LetDeclaration { name: "x", value: Expr }
   ├─ Expression(Expr)
   └─ LetDeclaration { name: "y", value: Expr }
```

## 🔧 let文の構文

### 基本的な構文

```rust
let variable_name = expression;
```

### 構文要素

1. **`let`キーワード** - 変数宣言の開始
2. **変数名** - `Identifier`トークン
3. **`=`記号** - 代入演算子
4. **初期値** - 任意の式
5. **`;`記号** - 文の終了

### 例

```rust
let x = 42;                    // 数値
let y = x + 1;                 // 式
let result = func(x, y);       // 関数呼び出し
let complex = (a + b) * c;     // 複雑な式
```

## 🔍 実装の戦略

### パーサーの拡張

今まで**式**のみを解析していましたが、今度は**文**と**プログラム全体**を解析します。

```rust
// 従来：式のみ
parse_expression(tokens) -> Expr

// 新規：プログラム全体
parse_program(tokens) -> Program
```

### 解析の階層

```rust
parse_program()         // プログラム全体
    ↓
parse_statement()       // 個別の文
    ↓                   
parse_let_statement()   // let文（必要に応じて）
parse_expression()      // 式文の場合
```

## 🔄 実装の詳細

### 1. parse_let_statement()

```rust
fn parse_let_statement(&mut self) -> Result<Stmt, String> {
    // 1. 'let'キーワードを消費
    self.expect(Token::Let)?;
    
    // 2. 変数名を取得
    let name = match self.current_token() {
        Token::Identifier(name) => name.clone(),
        _ => return Err("Expected identifier after 'let'".to_string()),
    };
    self.advance();
    
    // 3. '='を確認
    self.expect(Token::Assign)?;
    
    // 4. 初期値の式を解析
    let value = self.parse_expression()?;
    
    // 5. ';'を確認
    self.expect(Token::Semicolon)?;
    
    // 6. ASTノードを作成
    Ok(Stmt::LetDeclaration { name, value })
}
```

### 2. parse_statement()

```rust
fn parse_statement(&mut self) -> Result<Stmt, String> {
    match self.current_token() {
        Token::Let => {
            // let文
            self.parse_let_statement()
        },
        _ => {
            // 式文
            let expr = self.parse_expression()?;
            self.expect(Token::Semicolon)?;
            Ok(Stmt::Expression(expr))
        }
    }
}
```

### 3. parse_program()

```rust
fn parse_program(&mut self) -> Result<Program, String> {
    let mut statements = Vec::new();
    
    // EOF以外の限り文を解析
    while !matches!(self.current_token(), Token::Eof) {
        statements.push(self.parse_statement()?);
    }
    
    Ok(Program { statements })
}
```

## 🎬 実行例：複雑なプログラム

### 入力

```rust
let x = 10;
let y = x + 5;
print(y);
```

### トークン列

```rust
[Let, Identifier("x"), Assign, Number(10), Semicolon,
 Let, Identifier("y"), Assign, Identifier("x"), Plus, Number(5), Semicolon,
 Identifier("print"), LeftParen, Identifier("y"), RightParen, Semicolon,
 Eof]
```

### 解析過程

#### Step 1: 1番目の文
```rust
parse_statement()
├─ Token::Let発見
├─ parse_let_statement()
│  ├─ expect(Let) ✓
│  ├─ 変数名 "x" 取得
│  ├─ expect(Assign) ✓ 
│  ├─ parse_expression() → Number(10)
│  ├─ expect(Semicolon) ✓
│  └─ LetDeclaration { name: "x", value: Number(10) }
└─ 完了
```

#### Step 2: 2番目の文
```rust
parse_statement()
├─ Token::Let発見
├─ parse_let_statement()
│  ├─ expect(Let) ✓
│  ├─ 変数名 "y" 取得
│  ├─ expect(Assign) ✓
│  ├─ parse_expression() → Binary(x + 5)
│  ├─ expect(Semicolon) ✓
│  └─ LetDeclaration { name: "y", value: Binary(...) }
└─ 完了
```

#### Step 3: 3番目の文
```rust
parse_statement()
├─ Token::Letでない
├─ 式文として処理
├─ parse_expression() → FunctionCall("print", [Identifier("y")])
├─ expect(Semicolon) ✓
└─ Expression(FunctionCall(...))
```

### 最終的なAST

```rust
Program {
    statements: [
        LetDeclaration {
            name: "x",
            value: Number(10)
        },
        LetDeclaration {
            name: "y", 
            value: Binary {
                left: Identifier("x"),
                operator: Add,
                right: Number(5)
            }
        },
        Expression(FunctionCall {
            name: "print",
            arguments: [Identifier("y")]
        })
    ]
}
```

## 🔍 新しいトークンの追加

### 拡張されたトークン

```rust
enum Token {
    Number(i64),
    Identifier(String),
    Plus, Minus, Star, Slash,
    LeftParen, RightParen, Comma,
    Let,        // let
    Assign,     // =
    Semicolon,  // ;
    Eof,
}
```

### トークンの役割

- **`Let`**: 変数宣言の開始を示す
- **`Assign`**: 代入演算子
- **`Semicolon`**: 文の終了を示す

## 🐛 よくあるエラー

### 1. セミコロンの忘れ

```rust
let x = 42  // セミコロンなし
```

**対策**: `expect(Token::Semicolon)`でエラー検出

### 2. 代入演算子の忘れ

```rust
let x 42;  // =がない
```

**対策**: `expect(Token::Assign)`でエラー検出

### 3. 変数名の省略

```rust
let = 42;  // 変数名がない
```

**対策**: `Identifier`のパターンマッチでエラー検出

## 🎯 実装のポイント

### 1. 文と式の区別

```rust
// 文レベルでの解析
fn parse_statement(&mut self) -> Result<Stmt, String>

// 式レベルでの解析
fn parse_expression(&mut self) -> Result<Expr, String>
```

### 2. エラーハンドリング

```rust
// 具体的なエラーメッセージ
match self.current_token() {
    Token::Identifier(name) => name.clone(),
    token => return Err(format!("Expected identifier after 'let', found {:?}", token)),
}
```

### 3. 既存コードの再利用

```rust
// 式の解析は既存の実装を再利用
let value = self.parse_expression()?;
```

## 💡 実装のヒント

### 1. parse_let_statement()

- `expect()`でキーワードと記号を確認
- パターンマッチで変数名を取得
- `parse_expression()`で初期値を解析

### 2. parse_statement()

- `current_token()`で文の種類を判定
- `Token::Let`なら`parse_let_statement()`
- それ以外なら式文として処理

### 3. parse_program()

- `Vec<Stmt>`で文のリストを管理
- `while`ループで全ての文を解析

## ✅ 実装の進め方

1. **parse_let_statement()を実装**: let文の解析
2. **parse_statement()を実装**: 文の種類を判定
3. **parse_program()を実装**: プログラム全体の解析
4. **既存の式解析をコピー**: lesson_2_4から実装をコピー
5. **テストで確認**: 段階的にテストを実行

**実行コマンド**: `cargo test lesson_2::lesson_2_5`

## 🎉 完了後

このレッスンが完了すると：
- 文と式の違いが理解できる
- let文の構文解析ができる
- プログラム全体の構造を扱える
- より実用的なパーサーに近づく

**変数宣言は多くの言語の基本**です。しっかり理解して進みましょう！