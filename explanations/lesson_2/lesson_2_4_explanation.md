# Lesson 2-4: 関数呼び出しの構文解析

lesson_2_3で括弧の処理を学びましたね。今度は、**関数呼び出しの構文解析**について学びます。

## 📚 関数呼び出しとは？

### 🤔 プログラミングでの関数呼び出し

```rust
// 基本的な関数呼び出し
func()           // 引数なし
add(1, 2)        // 引数2つ
print("hello")   // 文字列引数

// 複雑な関数呼び出し
math(1 + 2, 3 * 4)           // 式を引数に
nested(inner(x), y)          // ネストした関数呼び出し
calculate(a, func(b), c + d)  // 混在した引数
```

### 🎯 構文の構造

```rust
// 関数呼び出しの構文
function_name ( argument1 , argument2 , ... )
```

## 🌳 関数呼び出しとAST

### 単純な例

```rust
// add(1, 2)
FunctionCall {
    name: "add",
    arguments: [
        Number(1),
        Number(2)
    ]
}
```

### 複雑な例

```rust
// func(1 + 2, inner(3))
FunctionCall {
    name: "func",
    arguments: [
        Binary {
            left: Number(1),
            operator: Add,
            right: Number(2)
        },
        FunctionCall {
            name: "inner",
            arguments: [Number(3)]
        }
    ]
}
```

## 🔧 実装の課題

### 1. **識別子の区別**

```rust
// これは識別子？関数呼び出し？
x      // 識別子
x()    // 関数呼び出し
```

**解決策**: 識別子の後の`(`で判断

### 2. **引数の解析**

```rust
// 引数の種類
func()                    // 引数なし
func(42)                  // 単一引数
func(1, 2, 3)            // 複数引数
func(1 + 2, x * y)       // 式の引数
func(inner(x), y)        // ネストした関数呼び出し
```

### 3. **エラーハンドリング**

```rust
// エラーケース
func(1 2)        // カンマなし
func(1,)         // 末尾カンマ
func(1, , 2)     // 空の引数
func(1           // 閉じ括弧なし
```

## 🔍 実装の戦略

### 基本的な流れ

1. **識別子を検出**: `Identifier`トークンを見つける
2. **次のトークンをチェック**: `(`があるか確認
3. **関数呼び出しか判定**:
   - `(`がある → 関数呼び出し
   - `(`がない → 通常の識別子
4. **引数を解析**: カンマ区切りの式リストを解析
5. **ASTノードを作成**: `FunctionCall`ノードを構築

### 優先順位の位置

```rust
parse_expression()     // エントリーポイント
    ↓
parse_additive()       // 低優先度 (+, -)
    ↓
parse_multiplicative() // 高優先度 (*, /)
    ↓
parse_primary()        // 最高優先度 (数値, 識別子, 括弧, 関数呼び出し)
```

**重要**: 関数呼び出しは**最高優先度**で処理されます。

## 🔄 parse_primary()の拡張

### 従来の実装

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current_token() {
        Token::Number(n) => { /* ... */ },
        Token::Identifier(s) => {
            let result = Ok(Expr::Identifier(s.clone()));
            self.advance();
            result
        },
        Token::LeftParen => { /* ... */ },
        token => Err(format!("Expected number, identifier, or '(', found {:?}", token)),
    }
}
```

### 拡張された実装

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current_token() {
        Token::Number(n) => { /* 同じ */ },
        Token::Identifier(s) => {
            let name = s.clone();
            self.advance();
            
            // 次のトークンをチェック
            if matches!(self.current_token(), Token::LeftParen) {
                // 関数呼び出し
                self.advance();  // '('を消費
                let arguments = self.parse_arguments()?;
                self.expect(Token::RightParen)?;
                Ok(Expr::FunctionCall { name, arguments })
            } else {
                // 通常の識別子
                Ok(Expr::Identifier(name))
            }
        },
        Token::LeftParen => { /* 同じ */ },
        token => Err(format!("Expected number, identifier, or '(', found {:?}", token)),
    }
}
```

## 🔧 parse_arguments()の実装

### 基本的な構造

```rust
fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
    let mut arguments = Vec::new();
    
    // 空の引数リスト: func()
    if matches!(self.current_token(), Token::RightParen) {
        return Ok(arguments);
    }
    
    // 最初の引数
    arguments.push(self.parse_expression()?);
    
    // 残りの引数
    while matches!(self.current_token(), Token::Comma) {
        self.advance();  // カンマを消費
        arguments.push(self.parse_expression()?);
    }
    
    Ok(arguments)
}
```

### 詳細な実装

```rust
fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
    let mut arguments = Vec::new();
    
    // 空の引数リストをチェック
    if matches!(self.current_token(), Token::RightParen) {
        return Ok(arguments);
    }
    
    // 最初の引数を解析
    arguments.push(self.parse_expression()?);
    
    // 残りの引数を解析
    while matches!(self.current_token(), Token::Comma) {
        self.advance();  // カンマを消費
        
        // 末尾カンマをチェック: func(1, 2,)
        if matches!(self.current_token(), Token::RightParen) {
            return Err("Trailing comma in function arguments".to_string());
        }
        
        // 次の引数を解析
        arguments.push(self.parse_expression()?);
    }
    
    Ok(arguments)
}
```

## 🎬 実行例：複雑な関数呼び出し

### 入力: `add(1 + 2, 3 * 4)`

```
トークン: [Identifier("add"), LeftParen, Number(1), Plus, Number(2), 
          Comma, Number(3), Star, Number(4), RightParen, Eof]
```

#### Step 1: parse_primary()で識別子発見
```
現在位置: 0 (Identifier("add"))
name = "add"
advance() → 現在位置: 1 (LeftParen)
LeftParenを発見 → 関数呼び出し
```

#### Step 2: parse_arguments()開始
```
advance() → 現在位置: 2 (Number(1))
RightParenではない → 引数あり
```

#### Step 3: 最初の引数を解析
```
parse_expression()を呼び出し
├─ parse_additive()
│  ├─ parse_multiplicative()
│  │  └─ parse_primary() → Number(1)
│  ├─ Plus発見
│  ├─ parse_multiplicative()
│  │  └─ parse_primary() → Number(2)
│  └─ Binary(1 + 2)作成
└─ 結果: Binary { left: Number(1), operator: Add, right: Number(2) }
現在位置: 5 (Comma)
```

#### Step 4: カンマで次の引数へ
```
Comma発見
advance() → 現在位置: 6 (Number(3))
```

#### Step 5: 2番目の引数を解析
```
parse_expression()を呼び出し
├─ parse_additive()
│  ├─ parse_multiplicative()
│  │  ├─ parse_primary() → Number(3)
│  │  ├─ Star発見
│  │  ├─ parse_primary() → Number(4)
│  │  └─ Binary(3 * 4)作成
│  └─ Plus/Minusなし
└─ 結果: Binary { left: Number(3), operator: Multiply, right: Number(4) }
現在位置: 9 (RightParen)
```

#### Step 6: 引数解析完了
```
Commaでない → ループ終了
引数リスト: [Binary(1 + 2), Binary(3 * 4)]
```

#### Step 7: FunctionCallノード作成
```
expect(RightParen) → 成功
FunctionCall {
    name: "add",
    arguments: [Binary(1 + 2), Binary(3 * 4)]
}
```

## 🔍 識別子と関数呼び出しの区別

### 判定ロジック

```rust
Token::Identifier(s) => {
    let name = s.clone();
    self.advance();
    
    match self.current_token() {
        Token::LeftParen => {
            // 関数呼び出し: name()
            self.advance();
            let arguments = self.parse_arguments()?;
            self.expect(Token::RightParen)?;
            Ok(Expr::FunctionCall { name, arguments })
        },
        _ => {
            // 通常の識別子: name
            Ok(Expr::Identifier(name))
        }
    }
}
```

### 実例

```rust
// x + y  →  識別子 + 識別子
x: Identifier("x")
y: Identifier("y")

// func(x) + y  →  関数呼び出し + 識別子
func(x): FunctionCall { name: "func", arguments: [Identifier("x")] }
y: Identifier("y")
```

## 🐛 よくあるエラー

### 1. 末尾カンマ

```rust
// func(1, 2,)
// 最後のカンマの後に式がない
```

**対策**: カンマの後に`)`をチェック

### 2. カンマの欠如

```rust
// func(1 2)
// 引数間にカンマがない
```

**対策**: 引数解析後に適切なトークンをチェック

### 3. 空の引数

```rust
// func(1, , 2)
// カンマの間に式がない
```

**対策**: `parse_expression()`で自動的にエラー

## 🎯 実装のポイント

### 1. 先読み（Lookahead）

```rust
// 識別子の後の1トークンを先読み
let name = s.clone();
self.advance();
if matches!(self.current_token(), Token::LeftParen) {
    // 関数呼び出し
} else {
    // 識別子
}
```

### 2. 再帰的解析

```rust
// 引数内で再び式全体を解析
arguments.push(self.parse_expression()?);
```

### 3. エラーハンドリング

```rust
// 具体的なエラーメッセージ
Err("Trailing comma in function arguments".to_string())
```

## 💡 実装のヒント

### 1. parse_arguments()の実装

- 空の引数リスト`()`を最初にチェック
- 最初の引数を特別に処理
- whileループでカンマ区切りの引数を処理
- 末尾カンマのエラーチェック

### 2. parse_primary()の拡張

- 識別子の名前を保存してから`advance()`
- 次のトークンで関数呼び出しか判定
- 適切なASTノードを作成

### 3. エラーメッセージの改善

- より具体的なエラーメッセージを提供
- デバッグしやすい情報を含める

## ✅ 実装の進め方

1. **parse_arguments()を実装**: 引数リストの解析
2. **parse_primary()を拡張**: 関数呼び出しの処理を追加
3. **他の関数をコピー**: lesson_2_3から実装をコピー
4. **テストで確認**: 段階的にテストを実行

**実行コマンド**: `cargo test lesson_2::lesson_2_4`

## 🎉 完了後

このレッスンが完了すると：
- 関数呼び出しの構文解析ができる
- 複雑な引数リストを処理できる
- 識別子と関数呼び出しを正しく区別できる
- ネストした関数呼び出しを解析できる

**関数呼び出しは多くの言語の核心機能**です。しっかり理解して進みましょう！