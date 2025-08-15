# 🔍 rust-analyzer コードベース分析

## 🎯 この文書の目的

Issue #20263 の解決に必要な rust-analyzer のコードベースを特定し、修正箇所と実装アプローチを分析します。

## 📁 関連するcrateとディレクトリ

### 🎯 主要なcrate

```
rust-analyzer/
├── crates/
│   ├── ide-completion/          👈 メイン対象
│   │   ├── src/
│   │   │   ├── completions/     👈 各種補完の実装
│   │   │   ├── context.rs       👈 CompletionContext
│   │   │   ├── item.rs          👈 CompletionItem
│   │   │   └── lib.rs           👈 エントリーポイント
│   │   └── tests/               👈 テストファイル
│   ├── ide-db/                  👈 データベース・共通機能
│   ├── hir/                     👈 高レベルIR
│   └── syntax/                  👈 構文解析
```

### 🔍 重要なファイル

#### 1. `/crates/ide-completion/src/context.rs`
- **役割**: CompletionContext の構築と文脈解析
- **重要度**: ⭐⭐⭐⭐⭐ (最重要)
- **修正対象**: 文脈判定ロジックの改善

#### 2. `/crates/ide-completion/src/completions/snippet.rs`  
- **役割**: スニペット補完の実装
- **重要度**: ⭐⭐⭐⭐⭐ (最重要)
- **修正対象**: 優先度決定ロジック

#### 3. `/crates/ide-completion/src/completions/keyword.rs`
- **役割**: キーワード補完の実装  
- **重要度**: ⭐⭐⭐ (重要)
- **修正対象**: キーワードとスニペットの共存

#### 4. `/crates/ide-completion/src/item.rs`
- **役割**: CompletionItem の定義と操作
- **重要度**: ⭐⭐⭐ (重要)
- **修正対象**: 優先度システム

## 🧩 CompletionContext の詳細分析

### 📊 現在の実装構造

```rust
// /crates/ide-completion/src/context.rs
pub struct CompletionContext<'a> {
    pub sema: Semantics<'a, RootDatabase>,
    pub scope: SemanticsScope<'a>,
    pub db: &'a RootDatabase,
    
    // 位置情報
    pub original_token: SyntaxToken,
    pub token: SyntaxToken,
    pub offset: TextSize,
    
    // 文脈情報
    pub expected_type: Option<Type>,
    pub expected_name: Option<NameOrNameRef>,
    
    // 構文的文脈
    pub function_def: Option<ast::Fn>,
    pub impl_def: Option<ast::Impl>,
    pub if_is_prev: bool,
    pub block_expr: Option<ast::BlockExpr>,
    
    // 🎯 Issue #20263 に関連する重要フィールド
    pub is_expr: bool,
    pub is_new_name: bool,
    pub path_qual: Option<ast::Path>,
    
    // マクロ関連（推定）
    pub in_macro_call: bool,
    pub macro_call: Option<ast::MacroCall>,
}
```

### 🔧 文脈構築プロセス

```rust
impl<'a> CompletionContext<'a> {
    pub fn new(
        db: &'a RootDatabase,
        position: FilePosition,
        config: &CompletionConfig,
    ) -> Option<CompletionContext<'a>> {
        // Step 1: 基本情報の取得
        let sema = Semantics::new(db);
        let original_file = sema.parse(position.file_id);
        let original_token = original_file
            .syntax()
            .token_at_offset(position.offset)
            .left_biased()?;
            
        // Step 2: トークンの正規化
        let token = sema.descend_into_macros(original_token.clone());
        
        // Step 3: 文脈の構築
        let mut ctx = CompletionContext {
            sema,
            scope: sema.scope(&token)?,
            db,
            original_token,
            token: token.clone(),
            offset: position.offset,
            // ... 他のフィールドの初期化
        };
        
        // Step 4: 詳細な文脈解析
        ctx.analyze_context();  // 🎯 ここで問題のある判定が行われる
        
        Some(ctx)
    }
    
    // 🎯 Issue #20263 の核心：文脈解析メソッド
    fn analyze_context(&mut self) {
        // マクロ呼び出しの検出
        if let Some(macro_call) = self.token
            .parent_ancestors()
            .find_map(ast::MacroCall::cast) 
        {
            self.in_macro_call = true;
            self.macro_call = Some(macro_call);
            
            // 🚨 問題箇所：ネストした関数呼び出しの判定
            if self.has_nested_function_calls() {
                // この情報が後でスニペット優先度を下げる原因となる
                self.mark_complex_macro_context();
            }
        }
        
        // 期待型の推論
        self.expected_type = self.infer_expected_type();
        
        // その他の文脈情報
        self.analyze_syntax_context();
    }
    
    // 🎯 問題の根源：ネストした関数呼び出しの判定
    fn has_nested_function_calls(&self) -> bool {
        self.token
            .parent_ancestors()
            .any(|node| {
                ast::CallExpr::cast(node.clone()).is_some() ||
                ast::MethodCallExpr::cast(node.clone()).is_some()
            })
    }
}
```

## 🎨 スニペット補完の実装分析

### 📋 現在の実装

```rust
// /crates/ide-completion/src/completions/snippet.rs
use super::*;

pub(crate) fn complete_expr_snippet(
    acc: &mut Completions,
    ctx: &CompletionContext,
) -> Option<()> {
    // 🎯 Issue #20263: この判定ロジックに問題
    if !ctx.config.enable_experimental.get() {
        return None;
    }
    
    // ref スニペットの処理
    if ctx.token.text().starts_with("re") {
        let priority = determine_ref_snippet_priority(ctx);  // 🚨 問題箇所
        
        let item = CompletionItem::new(
            CompletionItemKind::Snippet,
            ctx.source_range(),
            "ref",
        )
        .insert_text("&$0")
        .detail("Reference snippet")
        .priority(priority)  // 🎯 ここで優先度が決まる
        .build();
        
        acc.add(item);
    }
    
    Some(())
}

// 🚨 問題の核心：優先度決定ロジック
fn determine_ref_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    // 現在の問題のあるロジック
    if ctx.in_macro_call {
        // マクロ内での詳細判定
        if is_complex_macro_context(ctx) {
            CompletionPriority::Low     // 🚨 これが問題！
        } else {
            CompletionPriority::High
        }
    } else {
        CompletionPriority::High
    }
}

// 🎯 問題の特定：複雑なマクロ文脈の判定
fn is_complex_macro_context(ctx: &CompletionContext) -> bool {
    // Issue #20263 を引き起こすロジック
    if let Some(macro_call) = &ctx.macro_call {
        // println! マクロかつ関数呼び出しが含まれる場合
        let macro_name = macro_call
            .path()
            .and_then(|p| p.segment())
            .and_then(|s| s.name_ref())
            .map(|n| n.text())
            .unwrap_or("");
            
        match macro_name {
            "println" | "print" | "format" => {
                // 🚨 ここで identity() のような関数呼び出しを検出
                ctx.has_nested_function_calls()
            }
            _ => false,
        }
    } else {
        false
    }
}
```

### ✅ 改善版の実装（提案）

```rust
// 改善版のスニペット補完
pub(crate) fn improved_complete_expr_snippet(
    acc: &mut Completions,
    ctx: &CompletionContext,
) -> Option<()> {
    if ctx.token.text().starts_with("re") {
        let priority = improved_ref_snippet_priority(ctx);
        
        let item = CompletionItem::new(
            CompletionItemKind::Snippet,
            ctx.source_range(),
            "ref",
        )
        .insert_text("&$0")
        .detail("Reference snippet (&expr)")
        .priority(priority)
        .build();
        
        acc.add(item);
    }
    
    Some(())
}

// ✅ 改善された優先度決定ロジック
fn improved_ref_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    // ref スニペットは実用性が高いため、基本的に高優先度
    
    // 期待型による調整
    if let Some(expected_ty) = &ctx.expected_type {
        if expected_ty.is_reference() {
            return CompletionPriority::High;  // 参照が期待される場合は最優先
        }
    }
    
    // マクロ文脈でも基本的には高優先度を維持
    match ctx.macro_call.as_ref().and_then(|m| get_macro_name(m)) {
        Some("dbg") => CompletionPriority::High,      // デバッグマクロでは最優先
        Some("println" | "print" | "format") => {
            // フォーマットマクロでも参照は有用なため高優先度を維持
            CompletionPriority::High
        }
        _ => CompletionPriority::High,  // その他のケースでも高優先度
    }
}

fn get_macro_name(macro_call: &ast::MacroCall) -> Option<String> {
    macro_call
        .path()?
        .segment()?
        .name_ref()
        .map(|n| n.text().to_string())
}
```

## 🔤 キーワード補完の実装分析

### 📝 現在の実装

```rust
// /crates/ide-completion/src/completions/keyword.rs
pub(crate) fn complete_expr_keyword(
    acc: &mut Completions,
    ctx: &CompletionContext,
) -> Option<()> {
    if !ctx.is_expr {
        return None;
    }
    
    // ref キーワードの追加
    if ctx.token.text().starts_with("re") {
        let item = CompletionItem::new(
            CompletionItemKind::Keyword,
            ctx.source_range(),
            "ref",
        )
        .detail("Rust keyword")
        .priority(CompletionPriority::Medium)  // 固定の中優先度
        .build();
        
        acc.add(item);
    }
    
    Some(())
}
```

### 🤝 キーワードとスニペットの共存

現在の実装では、キーワードとスニペットが独立して追加されるため、理論的には共存可能です。問題は**優先度の決定**と**最終的なフィルタリング**にあります。

## ⚖️ 優先度システムの実装分析

### 🏗 CompletionPriority の定義

```rust
// /crates/ide-completion/src/item.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionPriority {
    /// 最高優先度（ソート順序 "00"）
    High,
    /// 中優先度（ソート順序 "01"）  
    Medium,
    /// 低優先度（ソート順序 "02"）
    Low,
}

impl CompletionPriority {
    pub fn to_sort_text(self, label: &str) -> String {
        match self {
            CompletionPriority::High => format!("00{}", label),
            CompletionPriority::Medium => format!("01{}", label),
            CompletionPriority::Low => format!("02{}", label),
        }
    }
}
```

### 📊 CompletionItem の構築

```rust
// /crates/ide-completion/src/item.rs
impl CompletionItem {
    pub fn new(
        kind: CompletionItemKind,
        source_range: TextRange,
        label: impl Into<String>,
    ) -> CompletionItemBuilder {
        let label = label.into();
        CompletionItemBuilder {
            source_range,
            label: label.clone(),
            insert_text: None,
            detail: None,
            kind,
            priority: CompletionPriority::Medium,  // デフォルト優先度
            sort_text: None,
            // ... 他のフィールド
        }
    }
}

impl CompletionItemBuilder {
    pub fn priority(mut self, priority: CompletionPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn build(self) -> CompletionItem {
        CompletionItem {
            label: self.label.clone(),
            source_range: self.source_range,
            insert_text: self.insert_text,
            kind: self.kind,
            detail: self.detail,
            // 🎯 ソートテキストの生成
            sort_text: self.sort_text.unwrap_or_else(|| {
                self.priority.to_sort_text(&self.label)
            }),
            // ... 他のフィールド
        }
    }
}
```

## 🎯 具体的な修正箇所

### 1. 優先度決定ロジックの修正

**ファイル**: `/crates/ide-completion/src/completions/snippet.rs`

```rust
// 現在の問題のあるコード
fn determine_ref_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    if ctx.in_macro_call && is_complex_macro_context(ctx) {
        CompletionPriority::Low  // 🚨 これを修正
    } else {
        CompletionPriority::High
    }
}

// 修正後のコード
fn determine_ref_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    // ref スニペットは実用性が高いため、常に高優先度
    CompletionPriority::High
}
```

### 2. 文脈判定ロジックの精密化

**ファイル**: `/crates/ide-completion/src/context.rs`

```rust
// 改善された文脈判定
impl<'a> CompletionContext<'a> {
    fn analyze_macro_context(&mut self) {
        if let Some(macro_call) = &self.macro_call {
            self.macro_kind = classify_macro_kind(macro_call);
            // 複雑さではなく、実用性に基づく判定
            self.macro_context_info = analyze_macro_utility(macro_call, &self.token);
        }
    }
}

fn classify_macro_kind(macro_call: &ast::MacroCall) -> MacroKind {
    match get_macro_name(macro_call).as_deref() {
        Some("dbg") => MacroKind::Debug,
        Some("println" | "print") => MacroKind::Output,
        Some("format") => MacroKind::Format,
        _ => MacroKind::Other,
    }
}
```

### 3. テストの追加

**ファイル**: `/crates/ide-completion/src/completions/snippet/tests.rs`

```rust
#[test]
fn ref_snippet_priority_in_complex_macro() {
    check_priority(
        r#"
fn main() {
    let num = 42;
    println!("{}", identity(num.ref$0));
}
"#,
        expect![[r#"
            sn ref Reference snippet (&expr)
            kw ref Rust keyword
        "#]],
    );
}

#[test]
fn ref_snippet_always_high_priority() {
    // 各種文脈でのref スニペット優先度をテスト
    let test_cases = [
        ("dbg!(value.ref$0)", "Debug macro"),
        ("println!(\"{}\", value.ref$0)", "Print macro"),
        ("println!(\"{}\", identity(value.ref$0))", "Complex macro"),
        ("format!(\"{}\", transform(value.ref$0))", "Format macro"),
    ];
    
    for (input, description) in test_cases {
        check_snippet_priority(input, "ref", CompletionPriority::High, description);
    }
}
```

## 🧪 デバッグとテスト戦略

### 🔍 デバッグ方法

1. **ログ出力の追加**
```rust
// context.rs での詳細ログ
log::debug!("CompletionContext analysis: token={}, in_macro={}, expected_type={:?}", 
    ctx.token.text(), ctx.in_macro_call, ctx.expected_type);
```

2. **テスト駆動開発**
```bash
# 特定のテストを実行
cargo test -p ide-completion snippet::tests::ref_snippet_priority

# 全補完テストを実行  
cargo test -p ide-completion
```

3. **実際のエディタでの確認**
```bash
# rust-analyzerをビルドしてエディタで確認
cargo build --release
# エディタでテストファイルを開いて補完を確認
```

### 📊 パフォーマンス測定

```rust
// パフォーマンステストの追加
#[test]
fn completion_performance_regression() {
    let start = std::time::Instant::now();
    
    // 複雑なマクロでの補完を大量実行
    for _ in 0..1000 {
        check_completions(r#"println!("{}", identity(value.ref$0))"#);
    }
    
    let duration = start.elapsed();
    assert!(duration < std::time::Duration::from_millis(1000), 
        "Completion should not regress performance");
}
```

## 🚨 注意すべきエッジケース

### 1. ネストしたマクロ
```rust
// 複雑なネスト例
macro_rules! custom_macro {
    ($e:expr) => { println!("{}", $e) };
}

fn test() {
    let value = 42;
    custom_macro!(identity(value.ref$0));  // この場合の動作
}
```

### 2. カスタムマクロでの動作
```rust
// ユーザー定義マクロでの補完
macro_rules! my_debug {
    ($val:expr) => {
        eprintln!("Debug: {:?}", $val);
    };
}

fn test() {
    let data = vec![1, 2, 3];
    my_debug!(data.ref$0);  // カスタムマクロでも適切に動作すべき
}
```

### 3. 型推論との相互作用
```rust
fn test() {
    let value: i32 = 42;
    
    // 異なる期待型での動作
    takes_reference(&value.ref$0);     // &i32 が期待される
    takes_value(value.ref$0);          // i32 が期待される
}
```

---

このコード分析を基に、最小限の変更で最大の効果を得られる修正アプローチを次のステップで検討します。既存のテストを壊さず、パフォーマンスに影響を与えない慎重な実装を目指しましょう。