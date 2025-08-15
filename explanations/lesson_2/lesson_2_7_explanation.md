# Lesson 2-7: ループ文（while）の構文解析

lesson_2_6で条件分岐（if-else）を学びましたね。今度は、**ループ文（while）の構文解析**について学びます。

## 📚 ループ文とは？

### 🔄 繰り返し処理の基本

**ループ文**は、条件が満たされている間、同じ処理を繰り返し実行する制御構造です：

```rust
while condition {
    // 条件が真の間、この処理を繰り返す
}
```

### 基本的な例

```rust
let mut i = 0;
while i < 5 {
    print(i);
    i = i + 1;
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
        else_block: Option<Block>,
    },
    While {
        condition: Expr,    // ループ条件
        body: Block,        // ループ本体
    },
}
```

### ASTの構造例

```rust
// while i < 5 { print(i); i = i + 1; }
Stmt::While {
    condition: Binary {
        left: Identifier("i"),
        operator: Less,
        right: Number(5)
    },
    body: Block {
        statements: [
            Expression(FunctionCall {
                name: "print",
                arguments: [Identifier("i")]
            }),
            Assignment {
                name: "i",
                value: Binary {
                    left: Identifier("i"),
                    operator: Add,
                    right: Number(1)
                }
            }
        ]
    }
}
```

## 🔧 while文の構文

### 基本的な構文

```rust
while condition_expression {
    statements...
}
```

### 構文要素

1. **`while`キーワード** - ループの開始
2. **条件式** - 真偽を判定する式
3. **`{`** - ボディブロックの開始
4. **文のリスト** - 繰り返し実行される処理
5. **`}`** - ボディブロックの終了

## 🔍 新しいトークンの追加

### 拡張されたトークン

```rust
enum Token {
    // 既存のトークン...
    While,       // while
    // LeftBrace/RightBraceは既に存在
}
```

### whileとifの類似点

- 両方とも条件式を持つ
- 両方ともブロック構造を使用
- 既存の`parse_block()`を再利用可能

## 🔄 実装の戦略

### 解析の階層（拡張）

```rust
parse_program()         // プログラム全体
    ↓
parse_statement()       // 個別の文
    ↓                   
parse_while_statement() // while文（新規追加）
parse_if_statement()    // if文
parse_let_statement()   // let文
parse_expression()      // 式文の場合
```

### ブロック構造の再利用

```rust
// if文とwhile文の両方で使用
fn parse_block(&mut self) -> Result<Block, String> {
    // 既存の実装をそのまま使用
}
```

## 🔄 実装の詳細

### 1. parse_while_statement()

```rust
fn parse_while_statement(&mut self) -> Result<Stmt, String> {
    // 1. 'while'キーワードを消費
    self.expect(Token::While)?;
    
    // 2. 条件式を解析
    let condition = self.parse_expression()?;
    
    // 3. ボディブロックを解析
    let body = self.parse_block()?;
    
    // 4. While ASTノードを作成
    Ok(Stmt::While {
        condition,
        body,
    })
}
```

### 2. parse_statement()の拡張

```rust
fn parse_statement(&mut self) -> Result<Stmt, String> {
    match self.current_token() {
        Token::Let => self.parse_let_statement(),
        Token::If => self.parse_if_statement(),
        Token::While => self.parse_while_statement(),  // 新規追加
        _ => {
            // 式文
            let expr = self.parse_expression()?;
            self.expect(Token::Semicolon)?;
            Ok(Stmt::Expression(expr))
        }
    }
}
```

## 🎬 実行例：シンプルなwhile文

### 入力

```rust
while x > 0 {
    print(x);
    x = x - 1;
}
```

### トークン列

```rust
[While, Identifier("x"), Greater, Number(0), LeftBrace,
 Identifier("print"), LeftParen, Identifier("x"), RightParen, Semicolon,
 Identifier("x"), Assign, Identifier("x"), Minus, Number(1), Semicolon,
 RightBrace, Eof]
```

### 解析過程

#### Step 1: while文の開始
```rust
parse_statement()
├─ Token::While発見
├─ parse_while_statement()
│  ├─ expect(While) ✓
│  ├─ parse_expression()
│  │  └─ Binary(x > 0)
│  └─ 条件式完了
```

#### Step 2: ボディブロックの解析
```rust
├─ parse_block()
│  ├─ expect(LeftBrace) ✓
│  ├─ parse_statement() → Expression(FunctionCall("print", [x]))
│  ├─ parse_statement() → Assignment("x", Binary(x - 1))
│  ├─ expect(RightBrace) ✓
│  └─ Block完了
└─ While文完了
```

## 🔄 ループの特徴

### 1. 反復実行

```rust
// 条件が真の間、ボディを繰り返し実行
while condition {
    // この部分が何度も実行される
}
```

### 2. 条件による制御

```rust
// 条件が偽になったらループ終了
while false {
    // この部分は実行されない
}
```

### 3. 無限ループの可能性

```rust
while true {
    // この部分は永遠に実行される（注意！）
}
```

## 🐛 よくあるエラー

### 1. 条件式の省略

```rust
while {           // 条件式がない
    print("hello");
}
```

**対策**: `parse_expression()`で条件式を必須に

### 2. ブロックの括弧忘れ

```rust
while x > 0
    print(x);     // '{'がない
```

**対策**: `parse_block()`で`expect(LeftBrace)`

### 3. セミコロンの混乱

```rust
while x > 0; {    // セミコロンは不要
    print(x);
}
```

**対策**: while文にはセミコロンは不要

## 🎯 実装のポイント

### 1. if文との類似性

```rust
// 両方とも同じ構造
if condition { statements }
while condition { statements }
```

### 2. 既存コードの再利用

```rust
// parse_block()は既に実装済み
// parse_expression()も既に実装済み
```

### 3. シンプルな構文

```rust
// while文はif文よりもシンプル
// else部分がない
```

## 💡 実装のヒント

### 1. parse_while_statement()

- `expect(Token::While)`でwhileキーワードを消費
- `parse_expression()`で条件式を取得
- `parse_block()`でボディを取得

### 2. エラーハンドリング

- 条件式が無効な場合のエラー
- ブロックの括弧不一致のエラー

### 3. テストケースの考慮

- シンプルなwhile文
- 複雑な条件式を持つwhile文
- 空のボディを持つwhile文
- ネストしたwhile文

## ✅ 実装の進め方

1. **parse_while_statement()を実装**: while文の解析
2. **parse_statement()を拡張**: while文を追加
3. **テストで確認**: 段階的にテストを実行

**実行コマンド**: `cargo test lesson_2::lesson_2_7`

## 🔄 ループとプログラムの関係

### 1. 制御構造の完成

```rust
// 順次実行
let x = 1;

// 条件分岐
if x > 0 { ... }

// 繰り返し
while x < 10 { ... }
```

### 2. rust-analyzerでの重要性

- コード解析でループを理解する必要
- ループ内の変数スコープの追跡
- パフォーマンス分析での重要な要素

## 🎉 完了後

このレッスンが完了すると：
- ループ文の構文解析ができる
- 基本的な制御構造が全て揃う
- より複雑なプログラム構造を解析できる
- rust-analyzerの基礎知識が充実する

**ループ文は繰り返し処理の基本**です。条件分岐と組み合わせることで、より複雑なプログラムロジックを表現できるようになります！