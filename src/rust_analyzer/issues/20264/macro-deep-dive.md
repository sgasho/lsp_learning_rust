# 🧬 マクロ処理の深層解析

## 🎯 この文書の目的

rust-analyzer におけるマクロ処理の詳細メカニズムを理解し、Issue #20264 の根本原因と解決策を技術的に分析します。

## 🧩 Rust マクロの基礎

### 🎭 マクロの種類と特徴

#### 1. 宣言的マクロ (Declarative Macros)

```rust
macro_rules! dbg {
    () => {
        eprintln!("[{}:{}]", file!(), line!())
    };
    ($val:expr) => {
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
}
```

**特徴**:
- パターンマッチングベース
- コンパイル時にトークンレベルで展開
- `TokenTree` として処理される

#### 2. 手続き的マクロ (Procedural Macros)

```rust
// Function-like macro
my_macro!(input tokens) -> output tokens

// Derive macro  
#[derive(MyTrait)]
struct Data { ... }

// Attribute macro
#[my_attr]
fn function() { ... }
```

### 🌳 TokenTree の構造

```rust
enum TokenTree {
    Leaf(Leaf),      // 単一トークン (identifier, literal, punct)
    Subtree(Subtree), // グループ化されたトークン (parentheses, braces, brackets)
}

// 例: dbg!(s.field)
// TokenTree representation:
Subtree {
    delimiter: Parenthesis,
    token_trees: [
        Leaf(Ident("s")),
        Leaf(Punct(".")),  
        Leaf(Ident("field"))
    ]
}
```

## 🔄 rust-analyzer でのマクロ展開プロセス

### 📊 アーキテクチャ概要

```
Input Source Code
        ↓
    Tokenization (lexer)
        ↓
    Parsing (syntax tree)
        ↓
    Macro Discovery
        ↓
    Macro Expansion  ← 🎯 核心プロセス
        ↓
    HIR Construction
        ↓
    Type Inference
        ↓
    IDE Features (completion, etc.)
```

### 🏗 主要コンポーネント

#### 1. `hir-expand` crate

```rust
// 主要な構造体
pub struct MacroExpander {
    db: dyn ExpandDatabase,
}

pub struct MacroCallLoc {
    pub def: MacroDefId,
    pub krate: CrateId, 
    pub call_site: AstPtr<ast::MacroCall>,
}

pub struct ExpandResult<T> {
    pub value: T,
    pub err: Option<ExpandError>,
}
```

#### 2. MacroCall の処理フロー

```rust
// 1. MacroCall の発見
ast::MacroCall {
    path: "dbg",
    args: TokenTree(s.field)
}

// 2. MacroDef の解決
MacroDef::find("dbg") -> MacroDefId

// 3. 展開の実行
expand_macro(
    def_id: MacroDefId,
    args: TokenTree,
    call_site: SyntaxNode
) -> ExpandResult<SyntaxNode>
```

### 🎯 Issue #20264 における具体的フロー

```rust
// Input: dbg!(s.field)
//                  ^ cursor position

// Step 1: Tokenization
tokens = [
    Ident("dbg"), Punct("!"), 
    Delimiter::Paren(Open),
    Ident("s"), Punct("."), Ident("field"),
    Delimiter::Paren(Close)
]

// Step 2: Parse as MacroCall
MacroCall {
    path: Path { segments: ["dbg"] },
    args: Some(TokenTree::Subtree(...))
}

// Step 3: Completion Request (問題発生箇所)
// - カーソル位置: s.field の "field" 部分
// - 期待: field と &field の両方を補完
// - 実際: field のみ補完される

// Step 4: Macro Expansion (理想的な処理)
// dbg!($val:expr) パターンにマッチ
// 展開結果:
match s.field {
    tmp => {
        eprintln!("[{}:{}] {} = {:#?}",
            file!(), line!(), "s.field", &tmp);
        //                                ^^^^ ここで &tmp が使われる！
        tmp
    }
}
```

## 🔍 補完エンジンとマクロの相互作用

### 💭 CompletionContext でのマクロ処理

```rust
pub fn new(
    db: &RootDatabase,
    position: FilePosition,
    config: &CompletionConfig,
) -> Option<CompletionContext<'_>> {
    let sema = Semantics::new(db);
    let original_file = sema.parse(position.file_id);
    
    // 🎯 重要: original_file vs expanded_file
    let token = original_file
        .syntax()
        .token_at_offset(position.offset)
        .left_biased()?;
        
    // 問題: マクロ内のトークンの場合、
    // 展開後の文脈が考慮されない
}
```

### 🎭 マクロ内補完の現在の限界

#### ❌ 現在の動作

```rust
// dbg!(s.|) での補完要求

// 1. Token の特定
token = Ident("field") // original file 内のトークン

// 2. Context の構築  
context = CompletionContext {
    scope: 元のスコープ (dbg! マクロ呼び出し位置),
    expected_type: None, // ここが問題！
}

// 3. 補完候補の生成
completions = [
    "field: Vec<u8>", // フィールド自体
    // "&field" が欠落！
]
```

#### ✅ 理想的な動作

```rust
// dbg!(s.|) での補完要求

// 1. Token の特定
token = Ident("field") // original file 内のトークン

// 2. Macro Expansion の考慮
expanded_context = analyze_macro_expansion(token, macro_call);

// 3. Enhanced Context の構築
context = CompletionContext {
    scope: 元のスコープ,
    expected_type: Some(inferred_from_expansion), // 改善ポイント！
    macro_info: Some(MacroExpansionInfo { ... }),
}

// 4. 拡張された補完候補の生成
completions = [
    "field: Vec<u8>",     // フィールド自体
    "&field: &Vec<u8>",   // 参照 (新規追加)
]
```

## 🧬 マクロ展開での型推論

### 🔬 dbg! マクロの詳細分析

```rust
macro_rules! dbg {
    ($val:expr) => {
        match $val {  // <- $val の型は何でも良い
            tmp => {
                eprintln!("...", &tmp);  // <- &tmp で参照を取る
                tmp  // <- 元の値を返す
            }
        }
    };
}
```

**型推論のフロー**:

1. `s.field` の型推論: `Vec<u8>`
2. `match s.field` での `tmp` の型: `Vec<u8>`  
3. `&tmp` の型: `&Vec<u8>`
4. `eprintln!` での要求型: `&dyn Debug`
5. 型制約: `Vec<u8>: Debug` ✓

### 🎯 期待型推論の改善ポイント

```rust
// 現在の期待型推論（不完全）
fn complete_dot_receiver(
    acc: &mut Completions,
    ctx: &CompletionContext,
    receiver: &ast::Expr,
) {
    let receiver_ty = ctx.sema.type_of_expr(receiver)?;
    
    // 問題: マクロ内での追加的な型制約を考慮していない
    let expected_types = vec![receiver_ty];
    
    // 補完候補生成...
}

// 改善後の期待型推論（提案）
fn complete_dot_receiver_enhanced(
    acc: &mut Completions,
    ctx: &CompletionContext,
    receiver: &ast::Expr,
) {
    let receiver_ty = ctx.sema.type_of_expr(receiver)?;
    
    let mut expected_types = vec![receiver_ty.clone()];
    
    // 🆕 マクロ展開での追加的期待型を分析
    if let Some(macro_info) = &ctx.macro_expansion_info {
        if let Some(additional_types) = analyze_macro_expected_types(
            macro_info, 
            receiver_ty
        ) {
            expected_types.extend(additional_types);
        }
    }
    
    // 拡張された期待型で補完候補生成...
}
```

## 🗺 Span Mapping の詳細

### 📍 Span とは

```rust
pub struct Span {
    pub start: TextSize,
    pub end: TextSize,
    pub file_id: FileId,
}

// Example: dbg!(s.field)
//               ^^^^^ このSpan
Span {
    start: TextSize(7),   // 's' の位置
    end: TextSize(13),    // 'd' の次の位置
    file_id: FileId(1),
}
```

### ↔️ Original ⟷ Expanded Mapping

```rust
// Original code spans
dbg!(s.field)
//   ↑     ↑
//   |     end: 13
//   start: 7

// Expanded code spans  
match s.field {
    tmp => {
        eprintln!("...", &tmp);
        tmp
    }
}
//    ↑     ↑
//    |     mapped_end: ?
//    mapped_start: ?
```

### 🔧 SpanMap の実装

```rust
pub struct SpanMap {
    /// Original -> Expanded のマッピング
    original_to_expanded: FxHashMap<TextRange, TextRange>,
    /// Expanded -> Original のマッピング  
    expanded_to_original: FxHashMap<TextRange, TextRange>,
}

impl SpanMap {
    pub fn map_original_to_expanded(&self, span: TextRange) -> Option<TextRange> {
        // Original コードの位置から展開後の位置を取得
    }
    
    pub fn map_expanded_to_original(&self, span: TextRange) -> Option<TextRange> {
        // 展開後の位置から Original コードの位置を取得
    }
}
```

## 🎯 問題解決のアプローチ

### 🏗 アーキテクチャレベルの変更

#### 1. CompletionContext の拡張

```rust
pub struct CompletionContext<'a> {
    // 既存フィールド...
    pub sema: Semantics<'a, RootDatabase>,
    pub scope: SemanticsScope<'a>,
    
    // 🆕 マクロ関連の新規フィールド
    pub macro_expansion: Option<MacroExpansionContext>,
}

pub struct MacroExpansionContext {
    pub call_site: SyntaxNode,
    pub expansion: SyntaxNode,
    pub span_map: SpanMap,
    pub macro_def: MacroDef,
}
```

#### 2. 期待型推論の強化

```rust
pub fn infer_expected_type(
    ctx: &CompletionContext,
    expr: &ast::Expr,
) -> Vec<Type> {
    let mut types = vec![];
    
    // 基本的な期待型
    if let Some(ty) = ctx.expected_type {
        types.push(ty);
    }
    
    // 🆕 マクロ展開での期待型
    if let Some(macro_ctx) = &ctx.macro_expansion {
        types.extend(infer_macro_expected_types(macro_ctx, expr));
    }
    
    types
}
```

### 🧪 実装ステップ

#### Phase 1: Macro Detection
```rust
// マクロ内での補完かどうかを判定
fn is_completion_in_macro(ctx: &CompletionContext) -> bool {
    // implementation
}
```

#### Phase 2: Expansion Analysis  
```rust
// マクロ展開を分析して期待型を推論
fn analyze_macro_expansion_for_completion(
    macro_call: &ast::MacroCall,
    position: TextSize,
) -> Option<MacroExpansionContext> {
    // implementation  
}
```

#### Phase 3: Enhanced Completion
```rust
// 拡張された補完候補を生成
fn generate_enhanced_completions(
    ctx: &CompletionContext,
    receiver_ty: Type,
    expected_types: Vec<Type>,
) -> Vec<CompletionItem> {
    // implementation
}
```

## 🚨 実装時の注意点

### ⚡ パフォーマンス考慮事項

- **マクロ展開のコスト**: 展開は計算量が多い
- **キャッシュ戦略**: 同じマクロの重複展開を避ける
- **遅延評価**: 必要な時のみ展開を実行

### 🔧 既存システムへの影響

- **後方互換性**: 既存の補完動作を壊さない
- **テストケース**: 包括的なリグレッションテスト
- **エラーハンドリング**: マクロ展開失敗時の適切な処理

### 🎯 段階的実装戦略

1. **最小限のプロトタイプ**: `dbg!` マクロのみ対応
2. **検証とテスト**: 動作確認と既存テストの通過
3. **段階的拡張**: 他のマクロへの対応拡大  
4. **パフォーマンス最適化**: プロファイリングと改善

---

この深層分析を基に、次は具体的な実装戦略を策定します。マクロ処理は複雑ですが、段階的に取り組むことで確実に改善できる領域です。