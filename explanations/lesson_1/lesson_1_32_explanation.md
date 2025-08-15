# Lesson 1-32: コードを美しく光らせよう！ (LSPセマンティックトークン)

LSPのコールハイアラーキー機能ができるようになりましたね。素晴らしいです！

次に、エディタでコードを美しく表示するための重要な機能「**セマンティックトークン（Semantic Tokens）**」について学びます。この機能は、コードの意味に基づいた正確なシンタックスハイライトを提供します。

## セマンティックトークン (Semantic Tokens) とは？

セマンティックトークン機能は、コード内の各要素（キーワード、関数名、変数名、文字列など）を**意味に基づいて**分類し、エディタに色付け情報を提供する機能です。

### 🎨 **従来のシンタックスハイライト vs セマンティックトークン**

**従来（正規表現ベース）**:
```rust
fn calculate(value: i32) -> i32 {    // fn, i32 は「キーワード色」
    let result = value * 2;          // let は「キーワード色」
    result                           // すべて同じ「識別子色」
}
```

**✨ セマンティックトークン（意味ベース）**:
```rust
fn calculate(value: i32) -> i32 {    // fn=キーワード, calculate=関数名, i32=型名
    let result = value * 2;          // let=キーワード, result=変数, value=パラメータ
    result                           // result=変数（定義済み）
}
```

### 🌟 セマンティックトークンの利点

1. **正確な色分け**: コードの意味に基づいた適切な色付け
2. **コンテキスト認識**: 同じ単語でも用途に応じて異なる色
3. **可読性向上**: 関数、変数、型などが一目で分かる
4. **エラー予防**: 未定義変数や型ミスが視覚的に分かりやすい

## LSPセマンティックトークンの仕組み

### 1. **トークンタイプの定義**

LSPでは標準的なトークンタイプが定義されています：

```rust
const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,     // 0: fn, let, mut, if など
    SemanticTokenType::FUNCTION,    // 1: 関数名
    SemanticTokenType::VARIABLE,    // 2: 変数名  
    SemanticTokenType::STRING,      // 3: 文字列リテラル
    SemanticTokenType::NUMBER,      // 4: 数値リテラル
    SemanticTokenType::TYPE,        // 5: 型名（i32, String など）
    SemanticTokenType::PARAMETER,   // 6: 関数パラメータ
    SemanticTokenType::PROPERTY,    // 7: 構造体フィールド
    // ... その他多数
];
```

### 2. **Delta エンコーディング**

セマンティックトークンは効率的な「Delta エンコーディング」を使用します：

```rust
pub struct SemanticToken {
    pub delta_line: u32,            // 前のトークンからの行の差分
    pub delta_start: u32,           // 前のトークンからの列の差分
    pub length: u32,                // トークンの文字数
    pub token_type: u32,            // トークンタイプのインデックス
    pub token_modifiers_bitset: u32, // 修飾子（static, readonly など）
}
```

**具体例**:
```rust
fn main() {
    let x = 42;
}
```

トークン化結果：
```rust
[
    // "fn" (0行目, 0文字目, 長さ2, タイプ:KEYWORD)
    SemanticToken { delta_line: 0, delta_start: 0, length: 2, token_type: 0, .. },
    
    // "main" (同じ行, +3文字, 長さ4, タイプ:FUNCTION)  
    SemanticToken { delta_line: 0, delta_start: 3, length: 4, token_type: 1, .. },
    
    // "let" (次の行=+1, 先頭から+4文字, 長さ3, タイプ:KEYWORD)
    SemanticToken { delta_line: 1, delta_start: 4, length: 3, token_type: 0, .. },
    
    // "x" (同じ行, +4文字, 長さ1, タイプ:VARIABLE)
    SemanticToken { delta_line: 0, delta_start: 4, length: 1, token_type: 2, .. },
    
    // "42" (同じ行, +4文字, 長さ2, タイプ:NUMBER)
    SemanticToken { delta_line: 0, delta_start: 4, length: 2, token_type: 4, .. },
]
```

## 実装アルゴリズム

### 1. **字句解析（トークン分割）**

```rust
fn tokenize_line(line: &str, line_number: usize) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = line.char_indices().peekable();
    
    while let Some((start, ch)) = chars.next() {
        match ch {
            // 文字列リテラル
            '"' => {
                let end = find_string_end(&mut chars, start);
                tokens.push(Token {
                    text: &line[start..=end],
                    token_type: TokenType::String,
                    position: (line_number, start),
                });
            }
            
            // 数値リテラル
            c if c.is_ascii_digit() => {
                let end = find_number_end(&mut chars, start);
                tokens.push(Token {
                    text: &line[start..=end],
                    token_type: TokenType::Number,
                    position: (line_number, start),
                });
            }
            
            // 識別子（キーワード、関数名、変数名）
            c if is_identifier_start(c) => {
                let end = find_identifier_end(&mut chars, start);
                let text = &line[start..=end];
                let token_type = classify_identifier(text);
                
                tokens.push(Token {
                    text,
                    token_type,
                    position: (line_number, start),
                });
            }
            
            // その他（演算子、括弧など）は今回はスキップ
            _ => continue,
        }
    }
    
    tokens
}
```

### 2. **トークン分類**

```rust
fn classify_identifier(text: &str) -> TokenType {
    match text {
        // Rustキーワード
        "fn" | "let" | "mut" | "if" | "else" | "for" | "while" | "match" 
        | "struct" | "enum" | "impl" | "trait" | "use" | "pub" | "return" => {
            TokenType::Keyword
        }
        
        // Rust組み込み型
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" 
        | "f32" | "f64" | "bool" | "char" | "str" | "String" => {
            TokenType::Type
        }
        
        // コンテキストに基づく判定が必要（簡略化）
        _ => TokenType::Variable,
    }
}
```

### 3. **Delta エンコーディング変換**

```rust
fn convert_to_semantic_tokens(tokens: Vec<Token>) -> Vec<SemanticToken> {
    let mut semantic_tokens = Vec::new();
    let mut prev_line = 0;
    let mut prev_start = 0;
    
    for token in tokens {
        let (line, start) = token.position;
        
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 {
            start - prev_start
        } else {
            start  // 新しい行では絶対位置
        };
        
        semantic_tokens.push(SemanticToken {
            delta_line: delta_line as u32,
            delta_start: delta_start as u32,
            length: token.text.len() as u32,
            token_type: token.token_type as u32,
            token_modifiers_bitset: 0,
        });
        
        prev_line = line;
        prev_start = start;
    }
    
    semantic_tokens
}
```

## 実装のポイント

### 1. **正確な文字列解析**

```rust
// ✅ 正しい文字列リテラル検出
fn find_string_end(chars: &mut PeekIterator, start: usize) -> usize {
    let mut escaped = false;
    let mut end = start;
    
    while let Some((pos, ch)) = chars.next() {
        end = pos;
        
        if escaped {
            escaped = false;
            continue;
        }
        
        match ch {
            '\\' => escaped = true,
            '"' => break,  // 文字列終了
            _ => {}
        }
    }
    
    end
}

// ❌ 間違った例：エスケープシーケンスを考慮しない
// "He said \"Hello\"" → 途中で文字列が終わってしまう
```

### 2. **コンテキスト認識**

```rust
// ✅ 関数定義 vs 関数呼び出しの区別
fn classify_identifier_in_context(text: &str, context: &ParseContext) -> TokenType {
    match context.current_context {
        Context::FunctionDefinition => TokenType::Function,
        Context::FunctionCall => TokenType::Function,
        Context::VariableDeclaration => TokenType::Variable,
        Context::TypeAnnotation => TokenType::Type,
        _ => classify_identifier(text),
    }
}

// 例：
// fn calculate() {}     // calculate = FUNCTION (定義)
// let x = calculate();  // calculate = FUNCTION (呼び出し)
// let calculate = 42;   // calculate = VARIABLE (変数)
```

### 3. **効率的な処理**

```rust
// ✅ 1パスでの処理
fn parse_semantic_tokens(content: &str) -> SemanticTokens {
    let mut tokens = Vec::new();
    let mut prev_line = 0;
    let mut prev_start = 0;
    
    for (line_number, line) in content.lines().enumerate() {
        for token in tokenize_line(line, line_number) {
            // Delta エンコーディングと分類を同時に実行
            let semantic_token = convert_token_with_delta(
                token, &mut prev_line, &mut prev_start
            );
            tokens.push(semantic_token);
        }
    }
    
    SemanticTokens {
        result_id: None,
        data: tokens,
    }
}
```

### 4. **エラーハンドリング**

```rust
// ✅ 不正なコードでもクラッシュしない
fn safe_tokenize(content: &str) -> SemanticTokens {
    match parse_tokens(content) {
        Ok(tokens) => tokens,
        Err(parse_error) => {
            // パースエラーでも部分的な結果を返す
            eprintln!("Parse error: {}, returning partial tokens", parse_error);
            SemanticTokens {
                result_id: None,
                data: Vec::new(),
            }
        }
    }
}
```

## やってみよう！

あなたの今回のミッションは、`provide_semantic_tokens` 関数を完成させることです。

### 🎯 実装ステップ

1. **📝 STEP 1**: コンテンツを行ごとに処理
   - 各行をトークンに分割
   - 文字列、数値、識別子を検出

2. **🔍 STEP 2**: 各トークンの種類を判定
   - `get_token_type` ヘルパー関数を実装
   - キーワード、型名、関数名、変数名を分類

3. **📐 STEP 3**: 位置情報を Delta 形式で計算
   - 前のトークンからの相対位置を計算
   - 行をまたぐ場合の処理

4. **🏗️ STEP 4**: `SemanticToken` オブジェクトを作成
   - `delta_line`, `delta_start`, `length` を設定
   - `token_type` インデックスを設定

5. **📤 STEP 5**: `SemanticTokens` として返す
   - すべてのトークンを配列にまとめる

### 🚨 重要なポイント

- **Delta エンコーディング**: 各トークンの位置は前のトークンからの**相対位置**
- **インデックス管理**: トークンタイプは配列のインデックス（0=KEYWORD, 1=FUNCTION, ...）
- **境界チェック**: 文字列や識別子の境界を正確に検出
- **効率性**: 大きなファイルでも高速に処理

`src/lessons/lesson_1_32.rs` を開いて、挑戦しましょう。

`cargo test lesson_1_32` でテストがすべて緑色になったらクリアです！