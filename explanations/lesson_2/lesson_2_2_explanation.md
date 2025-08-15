# Lesson 2-2: 構文解析（Parsing）の基礎

lesson_2_1で字句解析を学びましたね。今度は、**rust-analyzerの次のステップ**である**構文解析**について学びます。

## 📚 構文解析とは？

**構文解析**（Parsing）は、トークンの列を**意味のある構造**に変換する処理です。

### 🤔 なぜ構文解析が必要なのか？

lesson_2_1で文字列をトークンに変換しました：
```rust
"1 + 2 * 3" → [Number(1), Plus, Number(2), Star, Number(3), Eof]
```

でも、このトークンリストだけでは：
- **演算子の優先順位**がわからない
- **計算の順序**が不明
- **プログラムの構造**が理解できない

構文解析では、これを**構造化**します：

```rust
// 1 + 2 * 3 は 1 + (2 * 3) として解析
Binary {
    left: Number(1),
    operator: Add,
    right: Binary {
        left: Number(2),
        operator: Multiply,
        right: Number(3)
    }
}
```

## 🌳 抽象構文木（AST）とは？

**AST（Abstract Syntax Tree）**は、プログラムの構造を**木構造**で表現したものです。

### 木構造の例

数式 `1 + 2 * 3` のAST：

```
    +
   / \
  1   *
     / \
    2   3
```

この木構造により：
- **演算の順序**が明確
- **プログラムの構造**が理解しやすい
- **コード生成**や**最適化**が可能

## 🎯 今回の目標

**入力**: トークンのリスト
```rust
vec![Token::Number(1), Token::Plus, Token::Number(2), Token::Star, Token::Number(3), Token::Eof]
```

**出力**: 抽象構文木（AST）
```rust
Expr::Binary {
    left: Box::new(Expr::Number(1)),
    operator: BinaryOp::Add,
    right: Box::new(Expr::Binary {
        left: Box::new(Expr::Number(2)),
        operator: BinaryOp::Multiply,
        right: Box::new(Expr::Number(3)),
    }),
}
```

## 🔍 演算子の優先順位

数学では演算子に**優先順位**があります：

1. **乗算・除算** (`*`, `/`) → 高優先度
2. **加算・減算** (`+`, `-`) → 低優先度

### 優先順位の例

```rust
1 + 2 * 3   // 1 + (2 * 3) = 7
4 * 5 + 6   // (4 * 5) + 6 = 26
```

### 結合性（Associativity）

同じ優先順位の演算子は**左結合**：

```rust
1 - 2 - 3   // (1 - 2) - 3 = -4
8 / 2 / 2   // (8 / 2) / 2 = 2
```

## 🔧 再帰降下パーサーの詳細解説

**再帰降下パーサー**は、優先順位を**関数の呼び出し階層**で表現する手法です。

### 🤔 なぜ「再帰降下」なのか？

**再帰**: 関数が自分自身を呼び出す  
**降下**: 高い階層から低い階層へ下りていく

```
parse_expression()     ← 一番上の階層（入り口）
    ↓
parse_additive()       ← 低優先度の演算子
    ↓
parse_multiplicative() ← 高優先度の演算子
    ↓
parse_primary()        ← 一番下の階層（数値・識別子）
```

### 🧠 基本的な考え方

#### 1. 数学の計算順序を思い出そう

```
1 + 2 * 3
```

この式を計算するとき、私たちは：
1. まず `2 * 3` を計算（乗算は優先度が高い）
2. 次に `1 + 6` を計算（加算は優先度が低い）

#### 2. 木構造で表現すると

```
    +           ← 低優先度の演算子が上に来る
   / \
  1   *         ← 高優先度の演算子が下に来る
     / \
    2   3
```

**重要**: 木の**上にあるもの**が**後から実行**される！

#### 3. パーサーの処理順序

```
Step 1: parse_additive()を呼び出し
Step 2: parse_additive()内でparse_multiplicative()を呼び出し
Step 3: parse_multiplicative()内でparse_primary()を呼び出し
Step 4: parse_primary()が1を返す
Step 5: parse_multiplicative()が*を見つけて、右辺を解析
Step 6: parse_primary()が2を返す
Step 7: parse_multiplicative()が*を見つけて、右辺を解析
Step 8: parse_primary()が3を返す
Step 9: parse_multiplicative()が(2 * 3)のASTを作成
Step 10: parse_additive()が+を見つけて、右辺を解析
Step 11: parse_additive()が(1 + (2 * 3))のASTを作成
```

### 📊 視覚的な理解

#### 関数呼び出しの階層構造

```
parse_expression()
│
└─ parse_additive()      ← 「+」「-」を探す
   │
   └─ parse_multiplicative() ← 「*」「/」を探す
      │
      └─ parse_primary()     ← 数値・識別子を読む
```

#### 実際の処理の流れ（1 + 2 * 3の場合）

```
1. parse_additive()開始
   │
   ├─ parse_multiplicative()を呼び出し
   │  │
   │  ├─ parse_primary()を呼び出し → "1"を読む
   │  └─ "*"がないので、1を返す
   │
   ├─ "+"を発見！
   │
   ├─ parse_multiplicative()を再び呼び出し
   │  │
   │  ├─ parse_primary()を呼び出し → "2"を読む
   │  ├─ "*"を発見！
   │  ├─ parse_primary()を呼び出し → "3"を読む
   │  └─ (2 * 3)のASTを作成
   │
   └─ (1 + (2 * 3))のASTを作成
```

### 🎯 なぜ低優先度から開始するのか？

#### 理由1: 木の構造

```
1 + 2 * 3 の場合:

期待する木:        間違った木:
    +                 *
   / \               / \
  1   *             +   3
     / \           / \
    2   3         1   2
```

**低優先度の演算子**が**木の根**に来るべきなので、低優先度から開始します。

#### 理由2: 左結合性

```
1 - 2 - 3 の場合:

期待する木:        間違った木:
    -                 -
   / \               / \
  -   3             1   -
 / \                   / \
1   2                 2   3

(1 - 2) - 3 = -4    1 - (2 - 3) = 2
```

左から右へ処理するため、低優先度から開始が正しいです。

### 🔄 実装の詳細

#### 1. parse_primary() - 最も基本的な要素

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current_token() {
        Token::Number(n) => {
            let result = Expr::Number(*n);
            self.advance();  // 次のトークンへ
            Ok(result)
        }
        Token::Identifier(name) => {
            let result = Expr::Identifier(name.clone());
            self.advance();  // 次のトークンへ
            Ok(result)
        }
        _ => Err("Expected number or identifier".to_string()),
    }
}
```

#### 2. parse_multiplicative() - 乗算・除算

```rust
fn parse_multiplicative(&mut self) -> Result<Expr, String> {
    let mut left = self.parse_primary()?;  // 左辺を取得
    
    // *または/が続く限り繰り返し
    while matches!(self.current_token(), Token::Star | Token::Slash) {
        let op = match self.current_token() {
            Token::Star => BinaryOp::Multiply,
            Token::Slash => BinaryOp::Divide,
            _ => unreachable!(),
        };
        self.advance();  // 演算子を消費
        
        let right = self.parse_primary()?;  // 右辺を取得
        
        // 新しいASTノードを作成
        left = Expr::Binary {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        };
    }
    
    Ok(left)
}
```

#### 3. parse_additive() - 加算・減算

```rust
fn parse_additive(&mut self) -> Result<Expr, String> {
    let mut left = self.parse_multiplicative()?;  // より高い優先度を先に
    
    // +または-が続く限り繰り返し
    while matches!(self.current_token(), Token::Plus | Token::Minus) {
        let op = match self.current_token() {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Subtract,
            _ => unreachable!(),
        };
        self.advance();  // 演算子を消費
        
        let right = self.parse_multiplicative()?;  // 右辺を取得
        
        // 新しいASTノードを作成
        left = Expr::Binary {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        };
    }
    
    Ok(left)
}
```

### 🎬 ステップバイステップ実行例

#### 入力: `"1 + 2 * 3"`

```
トークン: [Number(1), Plus, Number(2), Star, Number(3), Eof]
現在位置: 0
```

**Step 1**: `parse_additive()`開始
```
現在位置: 0 (Number(1))
parse_multiplicative()を呼び出し
```

**Step 2**: `parse_multiplicative()`開始
```
現在位置: 0 (Number(1))
parse_primary()を呼び出し
```

**Step 3**: `parse_primary()`実行
```
現在位置: 0 (Number(1))
Number(1)を読んで、advance()
現在位置: 1 (Plus)
Expr::Number(1)を返す
```

**Step 4**: `parse_multiplicative()`に戻る
```
現在位置: 1 (Plus)
left = Expr::Number(1)
current_token() = Plus (StarでもSlashでもない)
whileループを抜ける
Expr::Number(1)を返す
```

**Step 5**: `parse_additive()`に戻る
```
現在位置: 1 (Plus)
left = Expr::Number(1)
current_token() = Plus (Plusなので継続)
```

**Step 6**: `parse_additive()`のwhileループ
```
現在位置: 1 (Plus)
op = BinaryOp::Add
advance() → 現在位置: 2 (Number(2))
parse_multiplicative()を呼び出し
```

**Step 7**: 2回目の`parse_multiplicative()`
```
現在位置: 2 (Number(2))
parse_primary()を呼び出し → Expr::Number(2)
現在位置: 3 (Star)
current_token() = Star (Starなので継続)
```

**Step 8**: `parse_multiplicative()`のwhileループ
```
現在位置: 3 (Star)
op = BinaryOp::Multiply
advance() → 現在位置: 4 (Number(3))
parse_primary()を呼び出し → Expr::Number(3)
現在位置: 5 (Eof)
```

**Step 9**: ASTノード作成
```
left = Expr::Binary {
    left: Box::new(Expr::Number(2)),
    operator: BinaryOp::Multiply,
    right: Box::new(Expr::Number(3)),
}
```

**Step 10**: `parse_additive()`のASTノード作成
```
left = Expr::Binary {
    left: Box::new(Expr::Number(1)),
    operator: BinaryOp::Add,
    right: Box::new(Expr::Binary {
        left: Box::new(Expr::Number(2)),
        operator: BinaryOp::Multiply,
        right: Box::new(Expr::Number(3)),
    }),
}
```

### 🧩 関数の役割分担

```
parse_expression()
├─ エントリーポイント
└─ parse_additive()を呼び出すだけ

parse_additive()
├─ +, - の演算子を処理
├─ 左結合性を実装
└─ より高い優先度の関数を呼び出し

parse_multiplicative()
├─ *, / の演算子を処理
├─ 左結合性を実装
└─ より高い優先度の関数を呼び出し

parse_primary()
├─ 数値、識別子を処理
├─ 最も基本的な要素
└─ 終端記号の処理
```

### 💡 実装のコツ

#### 1. whileループのパターン

```rust
let mut left = self.parse_higher_precedence()?;

while matches!(self.current_token(), 適切な演算子) {
    let op = /* 演算子を判定 */;
    self.advance();  // 演算子を消費
    let right = self.parse_higher_precedence()?;
    
    left = Expr::Binary {
        left: Box::new(left),
        operator: op,
        right: Box::new(right),
    };
}

Ok(left)
```

#### 2. エラーハンドリング

```rust
// 期待しないトークンの場合
_ => Err(format!("Unexpected token: {:?}", self.current_token())),
```

#### 3. 終端チェック

```rust
// トークンが尽きた場合
if matches!(self.current_token(), Token::Eof) {
    return Err("Unexpected end of input".to_string());
}
```

### 🎯 練習問題: 手で追跡してみよう！

#### 簡単な例: `"2 + 3"`

```
トークン: [Number(2), Plus, Number(3), Eof]
```

**あなたの課題**: 上記の詳細説明を参考に、`"2 + 3"`がどのように解析されるかを手で追跡してみてください。

<details>
<summary>答え（クリックして展開）</summary>

```
Step 1: parse_additive()開始
        現在位置: 0 (Number(2))
        
Step 2: parse_multiplicative()呼び出し
        現在位置: 0 (Number(2))
        
Step 3: parse_primary()呼び出し
        現在位置: 0 (Number(2))
        Number(2)を読んで、advance()
        現在位置: 1 (Plus)
        Expr::Number(2)を返す
        
Step 4: parse_multiplicative()に戻る
        現在位置: 1 (Plus)
        left = Expr::Number(2)
        current_token() = Plus (乗算・除算ではない)
        whileループを抜ける
        Expr::Number(2)を返す
        
Step 5: parse_additive()に戻る
        現在位置: 1 (Plus)
        left = Expr::Number(2)
        current_token() = Plus (加算なので継続)
        
Step 6: parse_additive()のwhileループ
        現在位置: 1 (Plus)
        op = BinaryOp::Add
        advance() → 現在位置: 2 (Number(3))
        parse_multiplicative()を呼び出し
        
Step 7: 2回目のparse_multiplicative()
        現在位置: 2 (Number(3))
        parse_primary()を呼び出し → Expr::Number(3)
        現在位置: 3 (Eof)
        current_token() = Eof (乗算・除算ではない)
        whileループを抜ける
        Expr::Number(3)を返す
        
Step 8: parse_additive()のASTノード作成
        left = Expr::Binary {
            left: Box::new(Expr::Number(2)),
            operator: BinaryOp::Add,
            right: Box::new(Expr::Number(3)),
        }
        
Step 9: 完了
        現在位置: 3 (Eof)
        current_token() = Eof (加算・減算ではない)
        whileループを抜ける
        最終的なASTを返す
```

結果のAST:
```
Binary {
    left: Box::new(Expr::Number(2)),
    operator: BinaryOp::Add,
    right: Box::new(Expr::Number(3)),
}
```

対応する木構造:
```
    +
   / \
  2   3
```

</details>

#### 挑戦問題: `"1 * 2 + 3"`

この式がどのように解析されるか、自分で追跡してみてください。特に：
- どの演算子が先に処理されるか
- 最終的な木構造はどうなるか

正解は実装してテストを通すことで確認できます！

### 🔍 デバッグのコツ

実装中に困ったら、以下を確認してください：

1. **現在位置の確認**: `println!("現在位置: {}, トークン: {:?}", self.current, self.current_token());`
2. **関数の呼び出し順序**: 各関数の開始時に`println!("parse_xxx開始");`
3. **ASTの構造**: 結果のASTを`println!("{:#?}", ast);`で確認

### 💪 実装の心構え

- **一度に全部作ろうとしない**: まず`parse_primary()`から始める
- **テストで確認**: 各関数が完成したらテストで動作確認
- **エラーメッセージを大切に**: コンパイルエラーは親切なヒント
- **分からなくなったら図を描く**: 木構造を紙に描いてみる

この解説で再帰降下パーサーの概念がより理解しやすくなりましたでしょうか？

## 🎬 実行例

入力: `1 + 2 * 3`

```
Step 1: parse_expression() → parse_additive()
Step 2: parse_additive() → parse_multiplicative()
Step 3: parse_multiplicative() → parse_primary()
Step 4: parse_primary() → Number(1)を返す
Step 5: parse_multiplicative() → Number(1)を返す
Step 6: parse_additive() → Plus演算子を発見
Step 7: parse_additive() → parse_multiplicative()で右辺を解析
Step 8: parse_multiplicative() → parse_primary()
Step 9: parse_primary() → Number(2)を返す
Step 10: parse_multiplicative() → Star演算子を発見
Step 11: parse_multiplicative() → parse_primary()で右辺を解析
Step 12: parse_primary() → Number(3)を返す
Step 13: parse_multiplicative() → Binary{2 * 3}を返す
Step 14: parse_additive() → Binary{1 + (2 * 3)}を返す
```

## 🔧 実装のポイント

### 1. パーサーの状態管理

```rust
struct Parser {
    tokens: Vec<Token>,
    current: usize,  // 現在読んでいるトークンの位置
}
```

### 2. 現在のトークン確認

```rust
fn current_token(&self) -> &Token {
    self.tokens.get(self.current).unwrap_or(&Token::Eof)
}
```

### 3. 次のトークンへ進む

```rust
fn advance(&mut self) {
    if self.current < self.tokens.len() {
        self.current += 1;
    }
}
```

### 4. 左結合性の実装

```rust
fn parse_additive(&mut self) -> Result<Expr, String> {
    let mut left = self.parse_multiplicative()?;
    
    while let Token::Plus | Token::Minus = self.current_token() {
        let op = match self.current_token() {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Subtract,
            _ => unreachable!(),
        };
        self.advance();
        let right = self.parse_multiplicative()?;
        left = Expr::Binary {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        };
    }
    
    Ok(left)
}
```

## 🔄 Box<T>の使用

RustでのASTでは`Box<T>`を使用します：

```rust
Binary {
    left: Box<Expr>,   // ヒープに格納
    operator: BinaryOp,
    right: Box<Expr>,  // ヒープに格納
}
```

### なぜBox<T>が必要？

1. **再帰的な構造**: ASTは自己参照的な構造
2. **サイズの決定**: コンパイル時にサイズが決まらない
3. **メモリ効率**: 大きなASTをスタックに置かない

## 🐛 よくある落とし穴

### 1. 無限再帰

優先順位を間違えると無限再帰に陥ります：

```rust
// 間違い: 高優先度が低優先度を呼ぶ
fn parse_primary(&mut self) {
    self.parse_additive()  // 無限再帰！
}
```

### 2. トークンの消費忘れ

`advance()`を呼ばないと無限ループになります：

```rust
while let Token::Plus = self.current_token() {
    // self.advance()を忘れると無限ループ
}
```

### 3. エラーハンドリング

予期しないトークンの処理：

```rust
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current_token() {
        Token::Number(n) => Ok(Expr::Number(*n)),
        Token::Identifier(name) => Ok(Expr::Identifier(name.clone())),
        _ => Err("Expected number or identifier".to_string()),
    }
}
```

## ✅ 実装の進め方

1. **parse_primary()を実装**: 数値と識別子の解析
2. **parse_multiplicative()を実装**: 乗算・除算の解析
3. **parse_additive()を実装**: 加算・減算の解析
4. **parse_expression()を実装**: エントリーポイント
5. **テストで確認**: 各段階でテストを実行

**実行コマンド**: `cargo test lesson_2::lesson_2_2`

## 🎉 完了後

このレッスンが完了すると：
- 構文解析の基本概念が理解できる
- 演算子の優先順位を扱える
- ASTの構築ができる
- 再帰降下パーサーが実装できる

**構文解析はコンパイラの心臓部**です。概念をしっかり理解して進みましょう！

## 🚀 次のステップ

lesson_2_3では、より複雑な構文（括弧、関数呼び出しなど）を扱い、rust-analyzerの実際の構造に近づいていきます。