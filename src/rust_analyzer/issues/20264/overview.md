# 🔍 Issue #20264: 技術的詳細分析

## 📋 Issue 情報

- **Issue番号**: #20264
- **タイトル**: `&field` missed inside macro
- **作成日**: 2025-07-21
- **ラベル**: A-completion, A-macro, C-bug
- **コメント数**: 0 (取り組みやすい)
- **難易度**: 中級（マクロ処理の理解が必要）

## 🎯 問題の核心

### 🔥 現象の詳細

```rust
struct NamedField {
    out: Vec<u8>,
}

fn main() {
    let s = NamedField { out: Vec::new() };
    
    // ✅ 通常コード：期待通りに動作
    str::from_utf8(s.|); // カーソル位置
    // 補完候補: `out: Vec<u8>`, `&out: &Vec<u8>`
    
    // ❌ マクロ内：&out が欠落
    dbg!(s.|); // カーソル位置
    // 補完候補: `out: Vec<u8>` のみ
    //           `&out: &Vec<u8>` が表示されない
}
```

### 🧬 根本原因の推定

1. **マクロ展開時の型推論不完全**
   - `dbg!` マクロは内部で `&expr` を使用
   - しかし補完エンジンがマクロ内での期待型を正しく推論できない

2. **補完文脈の継承問題**
   - マクロ展開前のオリジナル位置での補完文脈
   - 展開後の実際の型要求の不一致

3. **Span マッピングの限界**
   - オリジナルコードの位置と展開後コードの対応関係
   - 型情報の継承メカニズムの不備

## 🔬 技術的分析

### 🌊 dbg! マクロの展開プロセス

```rust
// 入力
dbg!(s.field)

// 展開ステップ1：パターンマッチング
macro_rules! dbg {
    ($val:expr) => {
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
}

// 展開ステップ2：実際の生成コード
match s.field {  // <- ここで s.field の型が必要
    tmp => {
        eprintln!("[{}:{}] {} = {:#?}",
            file!(), line!(), "s.field", &tmp);  // <- &tmp で参照を取る
        tmp
    }
}
```

### 🎯 期待される動作

```rust
// dbg!(s.|) での補完時
// 
// 期待1: field 自体の補完
s.field  // -> Vec<u8>
//
// 期待2: 参照での補完（これが欠落している）
s.&field // -> &Vec<u8> (conceptual)
// または
&s.field // -> &Vec<u8> (actual syntax)
```

### 🔍 str::from_utf8 での正常動作

```rust
str::from_utf8(s.|);
//            ^~~~~~~ この位置では &[u8] が期待される
//
// CompletionContext で推論される期待型:
// - FunctionCall の引数位置
// - str::from_utf8 の第1引数は &[u8]
// - s.field は Vec<u8> なので Deref で &[u8] に変換可能
// 
// 結果：&out が補完候補に含まれる
```

## 🧩 マクロ補完の技術的課題

### 1. 🎭 マクロ展開のタイミング問題

```
User Types: dbg!(s.|)
               ↓
Lexer: tokens = [dbg, !, (, s, ., |, )]
               ↓
Parser: MacroCall { path: dbg, args: TokenTree }
               ↓
??? WHEN TO EXPAND ???
               ↓
Completion Request: 位置情報 + 文脈情報
```

**問題**: 補完要求時にマクロを展開するか、しないかの判断

### 2. 🗺 位置マッピングの複雑さ

```rust
// Original (user code)
dbg!(s.field)
//      ^ cursor position: offset 7

// Expanded (generated code)  
match s.field {
    tmp => {
        eprintln!("...", &tmp);
        tmp
    }
}
//  ^ corresponding position???
```

**課題**: オリジナル位置から展開後位置への正確なマッピング

### 3. 🔄 文脈継承メカニズム

```rust
// 補完文脈の継承チェーン
Original Context (dbg!(s.|))
    ↓ macro expansion
Generated Context (match s.| { ... })
    ↓ type inference  
Expected Type (&T where T: Debug)
    ↓ completion generation
Available completions (missing &field)
```

## 🛠 rust-analyzer での処理フロー

### 📊 現在の処理フロー

```mermaid
graph TD
    A[User Input: dbg!s.|] --> B[Tokenization]
    B --> C[Parse MacroCall]
    C --> D[Completion Request]
    D --> E[Context Analysis]
    E --> F[Type Inference]
    F --> G[Generate Completions]
    G --> H[Return: only 'field']
    
    style H fill:#ffcccc
```

### ✅ 理想的な処理フロー

```mermaid
graph TD
    A[User Input: dbg!s.|] --> B[Tokenization] 
    B --> C[Parse MacroCall]
    C --> D[Macro Expansion Analysis]
    D --> E[Context Mapping]
    E --> F[Expected Type Inference]
    F --> G[Enhanced Completion Generation]
    G --> H[Return: 'field' + '&field']
    
    style D fill:#ccffcc
    style E fill:#ccffcc  
    style F fill:#ccffcc
    style H fill:#ccffcc
```

## 🎯 解決すべき具体的な技術問題

### 1. CompletionContext の拡張

```rust
// 現在のCompletionContext
pub struct CompletionContext {
    pub sema: Semantics<RootDatabase>,
    pub scope: SemanticsScope,
    pub db: RootDatabase,
    // ...existing fields...
}

// 必要な拡張
pub struct CompletionContext {
    // ...existing...
    pub macro_expansion_info: Option<MacroExpansionInfo>,  // 新規
    pub original_expected_type: Option<Type>,              // 新規  
}
```

### 2. MacroExpansionInfo の設計

```rust
pub struct MacroExpansionInfo {
    pub macro_def: MacroDef,
    pub call_site: SyntaxNode,
    pub expansion_site: SyntaxNode,
    pub span_mapping: SpanMap,
}
```

### 3. 期待型推論の改善

```rust
// 現在：マクロ内での期待型推論が不完全
fn infer_expected_type_in_macro(
    ctx: &CompletionContext,
    position: TextSize,
) -> Option<Type> {
    // TODO: マクロ展開を考慮した期待型推論
}
```

## 🚨 実装時の注意点

### ⚠️ パフォーマンスへの影響

- マクロ展開は計算コストが高い
- 補完のたびに展開するのは非効率
- キャッシュ戦略が重要

### 🔧 既存コードへの影響

- 補完エンジンの中核部分への変更
- 他の補完機能への副作用を防ぐ
- 後方互換性の維持

### 🧪 テストケースの複雑さ

- 多様なマクロパターン
- ネストしたマクロ
- エラーケースの処理

## 🎯 成功の判定基準

### ✅ 基本要件

```rust
struct Test { field: String }
let t = Test { field: "hello".to_string() };

// これらすべてで &field が補完候補に表示される
dbg!(t.|);
println!("{}", t.|);
format!("{}", t.|);
```

### 🚀 追加目標

- ネストしたマクロでの動作
- カスタムマクロでの動作
- パフォーマンスの劣化なし
- 既存テストの全通過

## 🔗 関連Issue・PR

- **Macro expansion**: hir-expand crate
- **Completion engine**: ide-completion crate
- **Similar issues**: マクロ関連の既存バグ報告
- **Reference implementations**: 類似の修正事例

---

この技術的分析を基に、次のステップで詳細な実装戦略を検討します。マクロという複雑な領域ですが、段階的に取り組むことで確実に解決可能な問題です。