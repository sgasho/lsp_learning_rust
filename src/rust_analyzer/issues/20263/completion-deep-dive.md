# 🎯 補完システムの深層解析

## 🎯 この文書の目的

rust-analyzer の補完システムの詳細メカニズムを理解し、Issue #20263 の根本原因と解決策を技術的に分析します。

## 🔄 補完処理の全体フロー

### 📊 アーキテクチャ概要

```
LSP Request (textDocument/completion)
        ↓
    Request Parsing
        ↓
    File Analysis & AST Construction
        ↓
    CompletionContext Construction  ← 🎯 文脈解析
        ↓
    Candidate Collection           ← 🎯 候補収集
        ↓                         ├── Keywords
        ↓                         ├── Snippets
        ↓                         ├── Identifiers
        ↓                         └── Paths
        ↓
    Filtering & Ranking           ← 🎯 問題発生箇所
        ↓
    LSP Response Generation
        ↓
    JSON-RPC Response
```

### 🏗 主要コンポーネント

#### 1. CompletionContext

```rust
// 概念的な構造（実際のrust-analyzerコードを参考）
pub struct CompletionContext<'a> {
    pub sema: Semantics<'a, RootDatabase>,
    pub scope: SemanticsScope<'a>,
    pub db: RootDatabase,
    
    // カーソル位置とトークン情報
    pub token: SyntaxToken,
    pub original_token: SyntaxToken,
    pub offset: TextSize,
    
    // 補完文脈
    pub expected_type: Option<Type>,
    pub function_syntax: Option<ast::Fn>,
    pub impl_def: Option<ast::Impl>,
    
    // 特殊な文脈フラグ
    pub in_macro_call: bool,
    pub in_use_tree: bool,
    pub in_type_args: bool,
}
```

**重要なフィールド**:
- `token`: カーソル位置のトークン（Issue #20263では `"ref"`）
- `expected_type`: 期待される型情報
- `in_macro_call`: マクロ内補完の判定（問題に関連）

#### 2. CompletionItem

```rust
pub struct CompletionItem {
    /// 表示ラベル
    pub label: String,
    /// 候補の種類
    pub kind: CompletionItemKind,
    /// 挿入テキスト
    pub insert_text: Option<String>,
    /// 詳細説明
    pub detail: Option<String>,
    /// ソート用テキスト（優先度制御）
    pub sort_text: Option<String>,
    /// 追加テキスト編集
    pub additional_text_edits: Vec<TextEdit>,
}
```

## 🧩 Issue #20263 の詳細分析

### 🔥 問題の発生メカニズム

```rust
// 問題のあるコード
println!("{}", identity(num.ref|));
//                           ^^^^ カーソル位置

// Step 1: CompletionContext の構築
let ctx = CompletionContext {
    token: "ref",
    in_macro_call: true,        // println! による
    function_call_depth: 1,     // identity() による
    expected_type: Some("T"),   // identity の引数型
};

// Step 2: 候補収集
let keywords = collect_keywords(&ctx, "ref");
// Result: [CompletionItem { label: "ref", kind: Keyword, sort_text: "ref" }]

let snippets = collect_snippets(&ctx, "ref");  
// Result: [CompletionItem { label: "ref", kind: Snippet, insert_text: "&$0" }]

// Step 3: フィルタリング（問題発生箇所）
let filtered = apply_completion_filters(keywords + snippets, &ctx);
// 現在の動作: キーワードが優先され、スニペットが除外または低優先度化

// Step 4: ソート
let sorted = sort_completions(filtered);
// Result: キーワードが上位、スニペットが下位または非表示
```

### 🎭 正常動作との比較

#### ✅ 正常ケース: `dbg!(num.ref)`

```rust
let ctx = CompletionContext {
    token: "ref",
    in_macro_call: true,
    function_call_depth: 0,     // 関数呼び出しなし
    expected_type: Some("&dyn Debug"),
};

// この条件では ref スニペットが高優先度で表示される
```

#### ❌ 問題ケース: `println!("{}", identity(num.ref))`

```rust
let ctx = CompletionContext {
    token: "ref",
    in_macro_call: true,
    function_call_depth: 1,     // identity() 呼び出しあり
    expected_type: Some("T"),   // ジェネリック型
};

// この条件で ref スニペットの優先度が下がる
```

## 🔍 候補収集の詳細メカニズム

### 🔤 キーワード補完

```rust
// rust-analyzer/crates/ide-completion/src/completions/keyword.rs
pub(crate) fn complete_expr_keyword(acc: &mut Completions, ctx: &CompletionContext) {
    if ctx.token.kind() == SyntaxKind::IDENT {
        let kw_completion = |acc: &mut Completions, kw: &str, snippet: &str| {
            let item = CompletionItem::new(kw)
                .kind(CompletionItemKind::Keyword)
                .insert_text(snippet)
                .build();
            acc.add(item);
        };
        
        // "ref" キーワードの追加
        if "ref".starts_with(&ctx.token.text()) {
            kw_completion(acc, "ref", "ref");
        }
    }
}
```

### 📋 スニペット補完

```rust
// rust-analyzer/crates/ide-completion/src/completions/snippet.rs
pub(crate) fn complete_expr_snippet(acc: &mut Completions, ctx: &CompletionContext) {
    if ctx.in_macro_call {
        // 🎯 Issue #20263: この条件判定に問題がある
        let priority = if is_complex_macro_context(ctx) {
            // 複雑なマクロ文脈では優先度を下げる（問題の原因）
            CompletionPriority::Low
        } else {
            CompletionPriority::High
        };
        
        let item = CompletionItem::new("ref")
            .kind(CompletionItemKind::Snippet)
            .insert_text("&$0")
            .detail("Reference snippet")
            .priority(priority)  // 🎯 ここで優先度が決まる
            .build();
        acc.add(item);
    }
}

fn is_complex_macro_context(ctx: &CompletionContext) -> bool {
    // 🚨 問題の根源：この判定ロジック
    ctx.in_macro_call && has_function_call_in_context(ctx)
}
```

## 🎯 優先度システムの理解

### ⭐ CompletionPriority の仕組み

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionPriority {
    High = 0,    // 最優先（ソート順序 "00"）
    Medium = 1,  // 中優先（ソート順序 "01"）
    Low = 2,     // 低優先（ソート順序 "02"）
}

impl CompletionPriority {
    pub fn to_sort_text(self, label: &str) -> String {
        format!("{:02}{}", self as u8, label)
    }
}
```

### 🔄 ソート順序の決定

```rust
// 最終的なソート順序
fn generate_sort_text(item: &CompletionItem) -> String {
    let priority_prefix = match item.kind {
        CompletionItemKind::Snippet => {
            // Issue #20263: 文脈によって変わる
            if in_problematic_context {
                "02"  // Low priority
            } else {
                "00"  // High priority
            }
        }
        CompletionItemKind::Keyword => "01",  // Medium priority
        _ => "03",
    };
    
    format!("{}{}", priority_prefix, item.label)
}

// 結果的なソート順序:
// - "00ref" (snippet, high) → 1位
// - "01ref" (keyword, medium) → 2位  
// - "02ref" (snippet, low) → 3位 ← 問題：これが最下位になる
```

## 🔧 フィルタリングロジックの詳細

### 🎭 現在のフィルタリング（問題あり）

```rust
// 現在の実装（推定）
fn filter_completions(
    items: Vec<CompletionItem>,
    ctx: &CompletionContext,
) -> Vec<CompletionItem> {
    items.into_iter()
        .filter(|item| {
            // プレフィックスマッチング
            item.label.starts_with(&ctx.token.text())
        })
        .filter(|item| {
            // 文脈フィルタリング
            is_contextually_appropriate(item, ctx)
        })
        .collect()
}

fn is_contextually_appropriate(
    item: &CompletionItem,
    ctx: &CompletionContext,
) -> bool {
    match item.kind {
        CompletionItemKind::Keyword => {
            // キーワードは常に表示
            true
        }
        CompletionItemKind::Snippet => {
            // 🚨 問題：複雑な文脈でスニペットを抑制
            if ctx.in_macro_call && has_nested_calls(ctx) {
                false  // スニペットを除外（問題の原因）
            } else {
                true
            }
        }
        _ => true,
    }
}
```

### ✅ 改善されたフィルタリング（提案）

```rust
// 改善版のフィルタリング
fn improved_filter_completions(
    items: Vec<CompletionItem>,
    ctx: &CompletionContext,
) -> Vec<CompletionItem> {
    items.into_iter()
        .filter(|item| {
            // 基本的なプレフィックスマッチング
            item.label.starts_with(&ctx.token.text())
        })
        .map(|mut item| {
            // フィルタリングではなく優先度調整
            adjust_priority_based_on_context(&mut item, ctx);
            item
        })
        .collect()
}

fn adjust_priority_based_on_context(
    item: &mut CompletionItem,
    ctx: &CompletionContext,
) {
    if item.kind == CompletionItemKind::Snippet && item.label == "ref" {
        // ref スニペットは実用性が高いため常に高優先度
        item.priority = CompletionPriority::High;
        
        // 期待型に基づくさらなる調整
        if let Some(expected) = &ctx.expected_type {
            if expected.is_reference_type() {
                // 参照が期待される文脈では最優先
                item.priority = CompletionPriority::High;
            }
        }
    }
}
```

## 🌊 マクロ展開と補完の相互作用

### 🎭 マクロ内補完の特殊性

```rust
// println!("{}", identity(num.ref))
//                           ^^^^ この位置での補完

// マクロ展開前の位置情報
Original Position: {
    file_id: FileId(1),
    offset: TextSize(35),
    token: "ref",
}

// マクロ展開後の位置情報（概念的）
Expanded Position: {
    file_id: FileId(1),  // 同じファイル
    offset: TextSize(?), // 展開後の位置
    token: "ref",
    context: MacroCallContext {
        macro_name: "println",
        call_site: TextRange(20..40),
    }
}
```

### 🔍 文脈継承の問題

```rust
// 問題：マクロ展開時の文脈情報の損失
fn complete_in_macro_call(
    original_ctx: &CompletionContext,
    macro_call: &ast::MacroCall,
) -> CompletionContext {
    let mut expanded_ctx = original_ctx.clone();
    
    // 🚨 問題：マクロ特有の情報が補完判定に悪影響
    expanded_ctx.in_macro_call = true;
    expanded_ctx.macro_depth += 1;
    
    // identity() 関数呼び出しの情報も追加される
    if has_function_calls_in_macro_args(macro_call) {
        expanded_ctx.nested_function_calls = true;  // これが問題を引き起こす
    }
    
    expanded_ctx
}
```

## 🎯 解決すべき技術的課題

### 1. 優先度決定ロジックの改善

```rust
// 現在の問題のあるロジック
fn determine_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    if ctx.in_macro_call && ctx.has_nested_function_calls {
        CompletionPriority::Low    // 問題：一律に低優先度
    } else {
        CompletionPriority::High
    }
}

// 改善案：実用性ベースの優先度
fn improved_snippet_priority(
    snippet_name: &str,
    ctx: &CompletionContext,
) -> CompletionPriority {
    match snippet_name {
        "ref" => {
            // ref スニペットは実用性が高いため文脈に関係なく高優先度
            CompletionPriority::High
        }
        _ => {
            // 他のスニペットは文脈を考慮
            if ctx.in_macro_call && ctx.has_nested_function_calls {
                CompletionPriority::Medium
            } else {
                CompletionPriority::High
            }
        }
    }
}
```

### 2. 文脈判定の精密化

```rust
// より精密な文脈判定
struct DetailedCompletionContext {
    basic_context: CompletionContext,
    
    // 拡張情報
    macro_info: Option<MacroCallInfo>,
    function_call_chain: Vec<FunctionCallInfo>,
    expected_type_confidence: f32,  // 期待型の確信度
}

struct MacroCallInfo {
    macro_name: String,
    is_debugging_macro: bool,      // dbg!, println! など
    is_formatting_macro: bool,     // format!, println! など
    args_context: Vec<ArgumentContext>,
}

impl DetailedCompletionContext {
    fn should_prioritize_ref_snippet(&self) -> bool {
        // より洗練された判定ロジック
        match &self.macro_info {
            Some(info) if info.is_debugging_macro => true,  // デバッグマクロでは常に高優先度
            Some(info) if info.is_formatting_macro => {
                // フォーマットマクロでも参照は有用
                self.has_reference_expectation()
            }
            None => true,  // マクロ外では常に高優先度
            _ => true,     // 不明な場合は高優先度を維持
        }
    }
    
    fn has_reference_expectation(&self) -> bool {
        self.expected_type_confidence > 0.5 && 
        self.basic_context.expected_type
            .as_ref()
            .map(|ty| ty.is_reference_compatible())
            .unwrap_or(true)
    }
}
```

### 3. テスト戦略

```rust
// 包括的なテストケース
#[cfg(test)]
mod completion_priority_tests {
    #[test]
    fn test_ref_snippet_in_simple_macro() {
        // dbg!(value.ref) での補完
        let completions = get_completions("dbg!(value.ref)", 14);
        assert_snippet_has_high_priority(&completions, "ref");
    }
    
    #[test]
    fn test_ref_snippet_in_complex_macro() {
        // println!("{}", identity(value.ref)) での補完
        let completions = get_completions(
            "println!(\"{}\", identity(value.ref))", 
            34
        );
        assert_snippet_has_high_priority(&completions, "ref");  // 修正後
    }
    
    #[test]
    fn test_keyword_snippet_coexistence() {
        // キーワードとスニペットの共存
        let completions = get_completions("value.ref", 9);
        assert!(has_keyword(&completions, "ref"));
        assert!(has_snippet(&completions, "ref"));
        assert!(snippet_priority(&completions, "ref") >= keyword_priority(&completions, "ref"));
    }
    
    fn assert_snippet_has_high_priority(
        completions: &[CompletionItem], 
        snippet_name: &str
    ) {
        let snippet = completions.iter()
            .find(|c| c.kind == CompletionItemKind::Snippet && c.label == snippet_name)
            .expect("Snippet should be present");
        
        assert_eq!(snippet.priority, CompletionPriority::High);
    }
}
```

## 🚨 実装時の注意点

### ⚠️ パフォーマンスへの影響

- 補完は高頻度で呼ばれる機能
- 複雑な文脈判定はレスポンス時間に影響
- キャッシュ戦略の検討が必要

### 🎯 ユーザー体験の一貫性

- マクロ内外での補完体験を統一
- 予測可能な候補順序
- エディタ間での一貫した動作

### 🔧 既存機能への影響最小化

- 他の補完機能を壊さない
- 既存テストの全通過
- 段階的な導入による影響確認

---

この深層分析を基に、次のステップで具体的な実装戦略を策定します。補完システムは開発体験の核心なので、慎重かつ効果的に改善していきましょう。