# 🔍 Issue #20263: 技術的詳細分析

## 📋 Issue 情報

- **Issue番号**: #20263
- **タイトル**: `ref` snippet shadowed by `ref` keyword in macro
- **作成日**: 2025-07-21
- **作成者**: wmstack
- **ラベル**: A-completion, A-macro, C-bug
- **コメント数**: 0 (取り組みやすい)
- **難易度**: 低〜中級（補完システムの理解が必要）

## 🎯 問題の核心

### 🔥 現象の詳細

```rust
use std::convert::identity;

fn main() {
    let num = 42;
    
    // Step 1: "num.re" と入力
    println!("{}", identity(num.re|));
    // 補完候補: "ref" スニペット が表示される ✅
    
    // Step 2: "num.ref" と完全入力  
    println!("{}", identity(num.ref|));
    // 補完候補: "ref" スニペット が消える ❌
    //           "ref" キーワード のみ表示
}
```

### 🧬 根本原因の推定

1. **文字列マッチング優先度の問題**
   - `ref` 完全一致時にキーワードが優先される
   - スニペットが除外される

2. **補完フィルタリングロジックの不備**
   - キーワードとスニペットの共存ロジックが不完全
   - 文脈に関係なく文字列マッチングが優先

3. **マクロ内特有の条件**
   - `println!` + `identity()` の組み合わせで発生
   - 特定の補完文脈での問題

## 🔬 技術的分析

### 🌊 補完処理のフロー

```rust
// 入力: num.ref|
//           ^^^^ カーソル位置

// Step 1: トークン解析
let token = "ref";
let prefix = "num.";

// Step 2: 補完候補の収集
let mut candidates = vec![];

// Step 2a: キーワード候補
candidates.extend(collect_keyword_completions("ref")); // "ref" キーワード

// Step 2b: スニペット候補  
candidates.extend(collect_snippet_completions("ref")); // "ref" スニペット

// Step 3: フィルタリング（問題発生箇所）
let filtered = filter_candidates(candidates, "ref");
// 現在: キーワードが優先されスニペットが除外される
// 期待: 両方が表示される、またはスニペットが優先

// Step 4: ソート・表示
display_completions(filtered);
```

### 🎭 スニペット vs キーワードの特徴

#### `ref` キーワード
```rust
// パターンマッチングでの使用
match value {
    ref x => println!("{}", x),  // 参照をキャプチャ
}

// let バインディングでの使用
let ref x = value;  // 参照バインディング
```

#### `ref` スニペット
```rust
// 実用的なコード生成
let value = 42;
value.ref  // 展開後: &value

// より実用的
some_function(&value);  // これを簡単に入力したい
```

### 🎯 期待される動作

```rust
println!("{}", identity(num.ref|));
// 期待される補完候補（優先度順）:
// 1. ref スニペット: `&num` (最も実用的)
// 2. ref キーワード: `ref` (言語機能として必要)
```

## 🧩 rust-analyzer での補完アーキテクチャ

### 📊 補完システム概要

```
CompletionRequest
        ↓
    Context Analysis
        ↓
    Candidate Collection ← 複数のソース
        ↓               ├── Keywords
        ↓               ├── Snippets  
        ↓               ├── Identifiers
        ↓               └── Paths
        ↓
    Filtering & Ranking ← 🎯 問題発生箇所
        ↓
    Response Generation
```

### 🔍 主要コンポーネント

#### 1. CompletionContext
```rust
pub struct CompletionContext<'a> {
    pub sema: Semantics<'a, RootDatabase>,
    pub scope: SemanticsScope<'a>,
    pub db: RootDatabase,
    // カーソル位置と周辺の構文情報
    pub token: SyntaxToken,
    pub original_token: SyntaxToken,
    // 補完期待型や文脈情報
    pub expected_type: Option<Type>,
}
```

#### 2. CompletionItem
```rust
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,  // Keyword, Snippet, etc.
    pub detail: Option<String>,
    pub insert_text: Option<String>,
    pub priority: CompletionPriority,  // 🎯 重要
}
```

### 🎯 Issue #20263 における具体的処理

```rust
// Input: println!("{}", identity(num.ref|))

// Step 1: Context 構築
let ctx = CompletionContext {
    token: SyntaxToken("ref"),
    // マクロ内の文脈情報
    // identity() 関数呼び出しの文脈
};

// Step 2: 候補収集
// キーワード収集
let keyword_items = complete_keywords(&ctx);
// Result: [CompletionItem { label: "ref", kind: Keyword, ... }]

// スニペット収集
let snippet_items = complete_snippets(&ctx);  
// Result: [CompletionItem { label: "ref", kind: Snippet, insert_text: "&$0", ... }]

// Step 3: フィルタリング（問題箇所）
let all_items = [keyword_items, snippet_items].concat();
let filtered = filter_by_prefix(all_items, "ref");

// 現在の問題: "ref" 完全一致でキーワードが優先
// 期待する動作: スニペットも残す、または優先度で調整
```

## 🔍 特定条件での問題発生

### 🎭 再現条件の分析

```rust
// ✅ 正常動作: dbg! マクロ
fn works_fine() {
    let num = 42;
    dbg!(num.ref);  // ref スニペットが表示される
}

// ❌ 問題発生: println! + identity の組み合わせ
fn problematic() {
    let num = 42;
    println!("{}", identity(num.ref));  // ref スニペットが消える
}

// 🤔 identity 関数を除去すると正常
fn also_works() {
    let num = 42;
    println!("{}", num.ref);  // ref スニペットが表示される
}
```

### 🔬 文脈差異の分析

```rust
// Case 1: dbg!(num.ref)
// CompletionContext:
// - マクロ: dbg!
// - 期待型: Debug を実装する型への参照
// - 結果: スニペット表示 ✅

// Case 2: println!("{}", identity(num.ref))  
// CompletionContext:
// - マクロ: println!
// - 関数呼び出し: identity()
// - 期待型: identity の引数型
// - 結果: スニペット非表示 ❌

// Case 3: println!("{}", num.ref)
// CompletionContext: 
// - マクロ: println!
// - 期待型: Display を実装する型
// - 結果: スニペット表示 ✅
```

**仮説**: `identity()` 関数呼び出しの存在が補完文脈に影響し、スニペットのフィルタリングロジックに問題を引き起こす

## 🛠 解決すべき技術的課題

### 1. 補完候補の優先度システム

```rust
// 現在の優先度ロジック（推定）
fn prioritize_completions(items: Vec<CompletionItem>) -> Vec<CompletionItem> {
    items.sort_by(|a, b| {
        match (a.kind, b.kind) {
            (Keyword, Snippet) => Ordering::Less,  // キーワード優先
            (Snippet, Keyword) => Ordering::Greater,
            _ => a.label.cmp(&b.label),
        }
    });
    items
}

// 改善後の優先度ロジック（提案）
fn improved_prioritize_completions(
    items: Vec<CompletionItem>, 
    ctx: &CompletionContext
) -> Vec<CompletionItem> {
    items.sort_by(|a, b| {
        // 文脈に応じた優先度調整
        let score_a = calculate_contextual_score(a, ctx);
        let score_b = calculate_contextual_score(b, ctx);
        score_b.cmp(&score_a)  // 高スコア優先
    });
    items
}
```

### 2. フィルタリングロジックの改善

```rust
// 現在のフィルタリング（推定）
fn filter_by_prefix(items: Vec<CompletionItem>, prefix: &str) -> Vec<CompletionItem> {
    items.into_iter()
        .filter(|item| {
            if item.label == prefix && item.kind == Keyword {
                return true;  // 完全一致キーワードを優先
            }
            item.label.starts_with(prefix)
        })
        .collect()
}

// 改善後のフィルタリング（提案）
fn improved_filter_by_prefix(
    items: Vec<CompletionItem>, 
    prefix: &str,
    ctx: &CompletionContext
) -> Vec<CompletionItem> {
    items.into_iter()
        .filter(|item| item.label.starts_with(prefix))
        .filter(|item| is_contextually_relevant(item, ctx))
        .collect()
    // 完全一致での除外は行わない
}
```

### 3. 文脈依存の候補選択

```rust
// スニペットの文脈適用性判定
fn is_snippet_applicable(snippet: &CompletionItem, ctx: &CompletionContext) -> bool {
    match snippet.label.as_str() {
        "ref" => {
            // &T が期待される文脈かどうか判定
            ctx.expected_type
                .map(|ty| ty.is_reference() || can_coerce_to_reference(ty))
                .unwrap_or(true)  // 不明な場合は表示
        }
        _ => true,
    }
}
```

## 🎯 実装戦略

### Phase 1: 問題の再現と分析
1. 問題を確実に再現する最小限のテストケース作成
2. デバッグログで現在の補完フローを追跡
3. キーワードとスニペット候補の生成・フィルタリング過程を分析

### Phase 2: 優先度システムの調整
1. 文脈に応じた優先度スコアリング実装
2. スニペットの実用性を考慮した重み付け
3. 既存テストに影響しない範囲での調整

### Phase 3: フィルタリングロジックの改善
1. 完全一致時の除外ロジック見直し
2. キーワードとスニペットの共存メカニズム
3. マクロ内での特殊処理の検討

### Phase 4: テストと検証
1. 修正対象の問題ケースでの動作確認
2. 既存の補完機能への影響確認
3. パフォーマンスリグレッションのチェック

## 🚨 実装時の注意点

### ⚠️ 既存機能への影響
- 他の補完機能が壊れないよう慎重に修正
- 大幅な変更は避け、最小限の調整に留める
- 既存のテストスイートを必ず通す

### 🎯 ユーザー体験の考慮
- 最も実用的な候補を優先
- 混乱を避けるため候補数は適切に制限
- 一貫した補完体験の提供

### 🔧 テスト戦略
- 問題の再現ケース
- 正常動作するケース（リグレッション防止）
- エッジケース（複雑なマクロ、ネストした呼び出し）

---

この技術分析を基に、次のステップで具体的な実装戦略を検討します。補完システムは開発体験の核心部分なので、慎重かつ効果的に改善していきましょう。