# Lesson 2-6: 条件分岐（if-else）の構文解析

lesson_2_5でlet文を学びましたね。今度は、**条件分岐（if-else）の構文解析**について学びます。

## 📚 条件分岐とは？

### 🤔 プログラムの制御構造

**条件分岐**は、プログラムの実行フローを制御する基本的な構造です：

```rust
if condition {
    // 条件が真の場合の処理
} else {
    // 条件が偽の場合の処理
}
```

### 基本的な例

```rust
if x > 0 {
    let result = 1;
} else {
    let result = 0;
}
```

## 🌳 ASTの拡張

### 新しいASTノード

```rust
// 文（Statement）の拡張
enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,  // elseは省略可能
    },
}

// ブロック（文のリスト）
struct Block {
    statements: Vec<Stmt>,
}

// 比較演算子の追加
enum BinaryOp {
    Add, Subtract, Multiply, Divide,
    Greater,   // >
    Less,      // <
    Equal,     // ==
    NotEqual,  // !=
}
```

### ASTの構造例

```rust
// if x > 0 { let y = 1; } else { let y = 0; }
Stmt::If {
    condition: Binary {
        left: Identifier("x"),
        operator: Greater,
        right: Number(0)
    },
    then_block: Block {
        statements: [
            LetDeclaration {
                name: "y",
                value: Number(1)
            }
        ]
    },
    else_block: Some(Block {
        statements: [
            LetDeclaration {
                name: "y", 
                value: Number(0)
            }
        ]
    })
}
```

## 🔧 if文の構文

### 基本的な構文

```rust
if condition_expression {
    statements...
}

// または

if condition_expression {
    statements...
} else {
    statements...
}
```

### 構文要素

1. **`if`キーワード** - 条件分岐の開始
2. **条件式** - 真偽を判定する式
3. **`{`** - thenブロックの開始
4. **文のリスト** - 条件が真の場合の処理
5. **`}`** - thenブロックの終了
6. **`else`キーワード** - 省略可能
7. **`{`** - elseブロックの開始
8. **文のリスト** - 条件が偽の場合の処理
9. **`}`** - elseブロックの終了

## 🔍 新しいトークンの追加

### 拡張されたトークン

```rust
enum Token {
    // 既存のトークン...
    If,          // if
    Else,        // else
    LeftBrace,   // {
    RightBrace,  // }
    Greater,     // >
    Less,        // <
    Equal,       // ==
    NotEqual,    // !=
}
```

### トークンの役割

- **`If`/`Else`**: 条件分岐のキーワード
- **`LeftBrace`/`RightBrace`**: ブロックの境界
- **比較演算子**: 条件式で使用

## 🔄 実装の戦略

### 演算子の優先順位

比較演算子を式の優先順位に追加します：

```rust
parse_expression()      // 最上位
    ↓
parse_comparison()      // 比較演算（新規追加）
    ↓
parse_additive()        // 加算・減算
    ↓
parse_multiplicative()  // 乗算・除算
    ↓
parse_primary()         // 最高優先度
```

### 解析の階層

```rust
parse_program()         // プログラム全体
    ↓
parse_statement()       // 個別の文
    ↓                   
parse_if_statement()    // if文（必要に応じて）
parse_let_statement()   // let文
parse_expression()      // 式文の場合
```

## 🔄 実装の詳細

### 1. parse_comparison()

```rust
fn parse_comparison(&mut self) -> Result<Expr, String> {
    let mut left = self.parse_additive()?;
    
    while matches!(self.current_token(), 
                   Token::Greater | Token::Less | Token::Equal | Token::NotEqual) {
        let op = match self.current_token() {
            Token::Greater => BinaryOp::Greater,
            Token::Less => BinaryOp::Less,
            Token::Equal => BinaryOp::Equal,
            Token::NotEqual => BinaryOp::NotEqual,
            _ => unreachable!(),
        };
        self.advance();
        let right = self.parse_additive()?;
        
        left = Expr::Binary {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        };
    }
    
    Ok(left)
}
```

### 2. parse_block()

```rust
fn parse_block(&mut self) -> Result<Block, String> {
    self.expect(Token::LeftBrace)?;
    
    let mut statements = Vec::new();
    
    // '}'以外の限り文を解析
    while !matches!(self.current_token(), Token::RightBrace) {
        if matches!(self.current_token(), Token::Eof) {
            return Err("Missing closing brace".to_string());
        }
        statements.push(self.parse_statement()?);
    }
    
    self.expect(Token::RightBrace)?;
    Ok(Block { statements })
}
```

### 3. parse_if_statement()

```rust
fn parse_if_statement(&mut self) -> Result<Stmt, String> {
    self.expect(Token::If)?;
    
    // 条件式を解析
    let condition = self.parse_expression()?;
    
    // thenブロックを解析
    let then_block = self.parse_block()?;
    
    // elseブロック（省略可能）
    let else_block = if matches!(self.current_token(), Token::Else) {
        self.advance();  // 'else'を消費
        Some(self.parse_block()?)
    } else {
        None
    };
    
    Ok(Stmt::If {
        condition,
        then_block,
        else_block,
    })
}
```

### 4. parse_statement()の拡張

```rust
fn parse_statement(&mut self) -> Result<Stmt, String> {
    match self.current_token() {
        Token::Let => self.parse_let_statement(),
        Token::If => self.parse_if_statement(),  // 新規追加
        _ => {
            // 式文
            let expr = self.parse_expression()?;
            self.expect(Token::Semicolon)?;
            Ok(Stmt::Expression(expr))
        }
    }
}
```

## 🎬 実行例：複雑なif文

### 入力

```rust
if x > 0 {
    let y = 1;
    print(y);
} else {
    let y = 0;
}
```

### トークン列

```rust
[If, Identifier("x"), Greater, Number(0), LeftBrace,
 Let, Identifier("y"), Assign, Number(1), Semicolon,
 Identifier("print"), LeftParen, Identifier("y"), RightParen, Semicolon,
 RightBrace, Else, LeftBrace,
 Let, Identifier("y"), Assign, Number(0), Semicolon,
 RightBrace, Eof]
```

### 解析過程

#### Step 1: if文の開始
```rust
parse_statement()
├─ Token::If発見
├─ parse_if_statement()
│  ├─ expect(If) ✓
│  ├─ parse_expression()
│  │  └─ Binary(x > 0)
│  └─ 条件式完了
```

#### Step 2: thenブロックの解析
```rust
├─ parse_block()
│  ├─ expect(LeftBrace) ✓
│  ├─ parse_statement() → LetDeclaration("y", 1)
│  ├─ parse_statement() → Expression(FunctionCall("print", [y]))
│  ├─ expect(RightBrace) ✓
│  └─ Block完了
```

#### Step 3: elseブロックの解析
```rust
├─ Token::Else発見
├─ advance() → Elseを消費
├─ parse_block()
│  ├─ expect(LeftBrace) ✓
│  ├─ parse_statement() → LetDeclaration("y", 0)
│  ├─ expect(RightBrace) ✓
│  └─ Block完了
└─ If文完了
```

## 🔍 演算子の優先順位

### 比較演算の位置

```rust
// 1 + 2 > 3 * 4
// (1 + 2) > (3 * 4) として解析される

Binary {
    left: Binary(1 + 2),      // 加算が先に計算
    operator: Greater,
    right: Binary(3 * 4),     // 乗算が先に計算
}
```

### 優先順位の階層

1. **乗算・除算** (`*`, `/`) - 最高優先度
2. **加算・減算** (`+`, `-`) - 高優先度
3. **比較演算** (`>`, `<`, `==`, `!=`) - 低優先度

## 🐛 よくあるエラー

### 1. ブロックの括弧忘れ

```rust
if x > 0
    let y = 1;  // '{'がない
```

**対策**: `parse_block()`で`expect(LeftBrace)`

### 2. 条件式の省略

```rust
if {           // 条件式がない
    let y = 1;
}
```

**対策**: `parse_expression()`で条件式を必須に

### 3. ネストしたブロックの不一致

```rust
if x > 0 {
    if y > 0 {
        let z = 1;
    // '}'の不足
}
```

**対策**: `parse_block()`でEOFチェック

## 🎯 実装のポイント

### 1. ブロックの概念

```rust
// ブロック = '{' + 文のリスト + '}'
struct Block {
    statements: Vec<Stmt>,
}
```

### 2. Option型の活用

```rust
// elseは省略可能
else_block: Option<Block>
```

### 3. 再帰的な構造

```rust
// if文の中にif文をネスト可能
if condition1 {
    if condition2 {
        // ...
    }
}
```

## 💡 実装のヒント

### 1. parse_comparison()

- `parse_additive()`を呼んで左辺を取得
- 比較演算子のwhileループ
- 左結合性の実装

### 2. parse_block()

- `expect(LeftBrace)`で開始
- whileループで文のリストを解析
- `expect(RightBrace)`で終了

### 3. parse_if_statement()

- 条件式、thenブロック、elseブロックの順
- elseブロックは`Option<Block>`

## ✅ 実装の進め方

1. **parse_comparison()を実装**: 比較演算子の解析
2. **parse_block()を実装**: ブロックの解析
3. **parse_if_statement()を実装**: if文の解析
4. **parse_statement()を拡張**: if文を追加
5. **テストで確認**: 段階的にテストを実行

**実行コマンド**: `cargo test lesson_2::lesson_2_6`

## 🎉 完了後

このレッスンが完了すると：
- 条件分岐の構文解析ができる
- ブロック構造を理解できる
- 比較演算子を扱える
- より複雑な制御構造を解析できる

**条件分岐は全ての言語の基本制御構造**です。しっかり理解して進みましょう！