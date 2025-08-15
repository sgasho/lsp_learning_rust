# Lesson 2-3: 括弧の処理とグループ化

lesson_2_2で基本的な構文解析を学びましたね。今度は、**括弧の処理とグループ化**について学びます。

## 📚 なぜ括弧が重要なのか？

### 🤔 優先順位の上書き

数学では、括弧は**演算の優先順位を変更**します：

```rust
// 通常の優先順位
1 + 2 * 3   // 1 + (2 * 3) = 7

// 括弧で優先順位を変更
(1 + 2) * 3 // (1 + 2) * 3 = 9
```

### 🎯 プログラミングでの括弧

```rust
// 条件式でのグループ化
if (x > 0) && (y < 10) { ... }

// 関数呼び出し
function(arg1, arg2)

// 型キャスト
(value as i32)
```

## 🌳 括弧と抽象構文木（AST）

### 括弧なしの場合

```rust
// 1 + 2 * 3
    +
   / \
  1   *
     / \
    2   3
```

### 括弧ありの場合

```rust
// (1 + 2) * 3
    *
   / \
  ()  3
  |
  +
 / \
1   2
```

**重要**: 括弧は**ASTの構造を変更**します。

## 🔧 括弧の処理方法

### 1. トークンの拡張

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,   // (
    RightParen,  // )
    Eof,
}
```

### 2. ASTノードの拡張

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Grouped(Box<Expr>),  // 括弧でグループ化された式
}
```

## 🎯 実装の戦略

### 基本的な考え方

1. **`(`を見つけたら**: 括弧内の式を解析
2. **式を解析**: 通常の式解析を再帰的に実行
3. **`)`を確認**: 対応する閉じ括弧を検証
4. **Groupedノード作成**: 括弧でラップされたASTを作成

### 優先順位の階層

```rust
parse_expression()     // エントリーポイント
    ↓
parse_additive()       // 低優先度 (+, -)
    ↓
parse_multiplicative() // 高優先度 (*, /)
    ↓
parse_primary()        // 最高優先度 (数値, 識別子, 括弧)
```

**重要**: 括弧は**最高優先度**で処理されます。

## 🔄 parse_primary()の拡張

### 従来の実装

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current_token() {
        Token::Number(n) => {
            let result = Ok(Expr::Number(*n));
            self.advance();
            result
        },
        Token::Identifier(s) => {
            let result = Ok(Expr::Identifier(s.clone()));
            self.advance();
            result
        },
        token => Err(format!("Expected number or identifier, found {:?}", token)),
    }
}
```

### 拡張された実装

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current_token() {
        Token::Number(n) => {
            let result = Ok(Expr::Number(*n));
            self.advance();
            result
        },
        Token::Identifier(s) => {
            let result = Ok(Expr::Identifier(s.clone()));
            self.advance();
            result
        },
        Token::LeftParen => {
            self.advance();  // '('を消費
            let expr = self.parse_expression()?;  // 括弧内の式を解析
            self.expect(Token::RightParen)?;  // ')'を確認
            Ok(Expr::Grouped(Box::new(expr)))
        },
        token => Err(format!("Expected number, identifier, or '(', found {:?}", token)),
    }
}
```

## 🔍 括弧処理のステップ

### 入力: `(1 + 2) * 3`

```
トークン: [LeftParen, Number(1), Plus, Number(2), RightParen, Star, Number(3), Eof]
```

#### Step 1: parse_expression()開始
```
parse_additive()を呼び出し
```

#### Step 2: parse_additive()開始
```
parse_multiplicative()を呼び出し
```

#### Step 3: parse_multiplicative()開始
```
parse_primary()を呼び出し
```

#### Step 4: parse_primary()で括弧発見
```
現在位置: 0 (LeftParen)
LeftParenを発見
advance() → 現在位置: 1 (Number(1))
parse_expression()を再帰的に呼び出し
```

#### Step 5: 括弧内の式解析
```
parse_expression()
├─ parse_additive()
│  ├─ parse_multiplicative()
│  │  └─ parse_primary() → Number(1)
│  ├─ Plus発見
│  ├─ parse_multiplicative()
│  │  └─ parse_primary() → Number(2)
│  └─ Binary(1 + 2)作成
└─ 結果: Binary { left: Number(1), operator: Add, right: Number(2) }
```

#### Step 6: 閉じ括弧の確認
```
expect(RightParen) → 現在位置: 4 (RightParen)
advance() → 現在位置: 5 (Star)
Grouped(Binary(1 + 2))を作成
```

#### Step 7: 乗算の処理
```
parse_multiplicative()に戻る
Starを発見
右辺をparse_primary()で解析 → Number(3)
Binary(Grouped(1 + 2) * 3)を作成
```

## 🎬 実行例：複雑な式

### 入力: `((1 + 2) * 3) + 4`

```
Step 1: 最外の括弧を処理
Step 2: 内側の括弧を処理
Step 3: 1 + 2 を解析
Step 4: (1 + 2) * 3 を解析
Step 5: ((1 + 2) * 3) をGrouped化
Step 6: + 4 を解析
Step 7: 最終的なBinary作成
```

**結果のAST**:
```rust
Binary {
    left: Grouped(
        Binary {
            left: Grouped(
                Binary {
                    left: Number(1),
                    operator: Add,
                    right: Number(2),
                }
            ),
            operator: Multiply,
            right: Number(3),
        }
    ),
    operator: Add,
    right: Number(4),
}
```

## 🔧 expect()関数の実装

### 基本的な実装

```rust
fn expect(&mut self, expected: Token) -> Result<(), String> {
    if self.current_token() == &expected {
        self.advance();
        Ok(())
    } else {
        Err(format!("Expected {:?}, found {:?}", expected, self.current_token()))
    }
}
```

### 使用例

```rust
// '('を期待
self.expect(Token::LeftParen)?;

// ')'を期待
self.expect(Token::RightParen)?;
```

## 🐛 よくあるエラー

### 1. 括弧の不一致

```rust
// (1 + 2  // ')'がない
// 1 + 2)  // '('がない
```

**対策**: `expect()`でエラーハンドリング

### 2. 空の括弧

```rust
// ()  // 括弧内に式がない
```

**対策**: `parse_expression()`でのエラーチェック

### 3. 無限再帰

```rust
// 間違った実装
fn parse_primary(&mut self) -> Result<Expr, String> {
    self.parse_expression()  // 無限再帰！
}
```

**対策**: 正しい階層を維持

## 🎯 実装のポイント

### 1. 再帰的な解析

```rust
Token::LeftParen => {
    self.advance();  // '('を消費
    let expr = self.parse_expression()?;  // 再帰的に解析
    self.expect(Token::RightParen)?;  // ')'を確認
    Ok(Expr::Grouped(Box::new(expr)))
}
```

### 2. エラーハンドリング

```rust
// 括弧の不一致をチェック
self.expect(Token::RightParen)?;
```

### 3. 優先順位の維持

```rust
// 括弧内では最初から優先順位を評価
let expr = self.parse_expression()?;
```

## 🔍 デバッグのコツ

### 1. ASTの可視化

```rust
println!("AST: {:#?}", ast);
```

### 2. トークンの追跡

```rust
println!("Current token: {:?} at position {}", self.current_token(), self.current);
```

### 3. 段階的テスト

```rust
// 簡単な例から始める
(42)
(1 + 2)
(1 + 2) * 3
```

## 💡 実装のヒント

### 1. parse_primary()の拡張

- `Token::LeftParen`の場合を追加
- `self.advance()`で開き括弧を消費
- `self.parse_expression()`で括弧内を解析
- `self.expect(Token::RightParen)`で閉じ括弧を確認

### 2. 他の関数はそのまま

- `parse_multiplicative()`と`parse_additive()`は変更不要
- 既存の実装をそのまま使用

### 3. エラーメッセージの改善

- より詳細なエラーメッセージを提供
- デバッグしやすい情報を含める

## ✅ 実装の進め方

1. **expect()関数を理解**: 特定のトークンを期待する仕組み
2. **parse_primary()を拡張**: 括弧の処理を追加
3. **他の関数をコピー**: lesson_2_2から実装をコピー
4. **テストで確認**: 段階的にテストを実行

**実行コマンド**: `cargo test lesson_2::lesson_2_3`

## 🎉 完了後

このレッスンが完了すると：
- 括弧の処理方法が理解できる
- より複雑な式の構文解析ができる
- 再帰的な解析の概念が身につく
- エラーハンドリングの重要性が理解できる

**括弧の処理は構文解析の基本**です。しっかり理解して進みましょう！