# 🛠 実装戦略：段階的アプローチ

## 🎯 この文書の目的

Issue #20263 の解決に向けた具体的な実装戦略を段階的に提示し、リスクを最小化しながら効果的に問題を解決するアプローチを示します。

## 📊 実装の全体戦略

### 🎯 基本方針

1. **最小限の変更**: 既存システムへの影響を最小化
2. **段階的実装**: Phase毎に動作確認しながら進める
3. **テスト駆動**: 修正前にテストケースを整備
4. **後方互換性**: 既存の補完動作を壊さない

### 🔄 実装フロー

```
Phase 1: 問題再現・分析
    ↓
Phase 2: テストケース整備
    ↓  
Phase 3: 優先度ロジック修正
    ↓
Phase 4: 検証・調整
    ↓
Phase 5: 文書化・PR作成
```

## 🚀 Phase 1: 問題再現・分析

### 🎯 目標
- 問題を確実に再現できる環境を構築
- 現在の動作を詳細に分析・文書化
- 修正対象箇所を正確に特定

### 📝 作業内容

#### 1.1 開発環境のセットアップ

```bash
# rust-analyzer のクローン
git clone https://github.com/rust-lang/rust-analyzer.git
cd rust-analyzer

# 依存関係のインストール
cargo build

# テスト実行（現状確認）
cargo test -p ide-completion -- snippet
```

#### 1.2 問題の再現確認

**テストファイル作成**: `test_issue_20263.rs`

```rust
use std::convert::identity;

fn main() {
    let num = 42;
    
    // Case 1: 正常動作（参考用）
    dbg!(num.ref);  // <- ref スニペットが表示される
    
    // Case 2: 問題のあるケース
    println!("{}", identity(num.ref));  // <- ref スニペットが消える
    
    // Case 3: 中間ケース
    println!("{}", num.ref);  // <- 動作確認用
}
```

#### 1.3 現在の動作分析

```bash
# 補完動作をデバッグモードで確認
RUST_LOG=debug cargo run --bin rust-analyzer
```

**分析項目**:
- 各ケースでの補完候補リスト
- CompletionItemの優先度
- フィルタリング処理の動作
- ソート順序の決定プロセス

#### 1.4 コードベースの詳細調査

**対象ファイル**:
- `/crates/ide-completion/src/completions/snippet.rs`
- `/crates/ide-completion/src/context.rs`
- `/crates/ide-completion/src/completions/keyword.rs`

**調査内容**:
- 現在の優先度決定ロジック
- マクロ文脈の判定方法
- キーワードとスニペットの処理順序

### 📊 Phase 1 完了基準

- [ ] 問題が100%再現可能
- [ ] 現在の動作が詳細に文書化されている
- [ ] 修正対象のコード箇所が特定されている
- [ ] 修正前のベースラインテストが作成されている

## 🧪 Phase 2: テストケース整備

### 🎯 目標
- 修正前後の動作を検証するテストケースを作成
- リグレッション防止のためのテストを充実
- CI/CDでの自動テストを確立

### 📝 作業内容

#### 2.1 基本テストケースの作成

**ファイル**: `/crates/ide-completion/src/completions/snippet/tests.rs`

```rust
use expect_test::{expect, Expect};
use super::*;

// Issue #20263 のテストケース
#[test]
fn test_ref_snippet_in_complex_macro() {
    // 問題のあるケース：修正前は失敗、修正後は成功
    check_snippet_priority(
        r#"
use std::convert::identity;
fn main() {
    let num = 42;
    println!("{}", identity(num.ref$0));
}
"#,
        "ref",
        CompletionPriority::High,  // 期待する優先度
        expect![[r#"
            sn ref Reference snippet (&expr)
            kw ref Rust keyword
        "#]],
    );
}

#[test]
fn test_ref_snippet_priority_consistency() {
    let test_cases = [
        // (input, description, expected_priority)
        (
            "fn main() { let x = 42; dbg!(x.ref$0); }",
            "Simple debug macro",
            CompletionPriority::High,
        ),
        (
            "fn main() { let x = 42; println!(\"{}\", x.ref$0); }",
            "Print macro without function call",
            CompletionPriority::High,
        ),
        (
            "fn main() { let x = 42; println!(\"{}\", identity(x.ref$0)); }",
            "Print macro with function call",
            CompletionPriority::High,  // 修正後の期待値
        ),
        (
            "fn main() { let x = 42; format!(\"{}\", transform(x.ref$0)); }",
            "Format macro with function call",
            CompletionPriority::High,
        ),
    ];
    
    for (input, description, expected_priority) in test_cases {
        test_ref_snippet_priority(input, expected_priority, description);
    }
}

// ヘルパー関数
fn test_ref_snippet_priority(
    input: &str,
    expected_priority: CompletionPriority,
    description: &str,
) {
    let completions = get_completions(input);
    let ref_snippet = completions.iter()
        .find(|item| item.kind == CompletionItemKind::Snippet && item.label == "ref")
        .unwrap_or_else(|| panic!("ref snippet not found in: {}", description));
    
    assert_eq!(
        ref_snippet.priority, 
        expected_priority,
        "Priority mismatch in case: {}",
        description
    );
}
```

#### 2.2 リグレッションテストの作成

```rust
#[test]
fn test_no_regression_in_normal_cases() {
    // 既存の正常ケースが壊れていないことを確認
    let normal_cases = [
        "fn main() { let x = 42; x.ref$0; }",
        "fn main() { let x = 42; dbg!(x.ref$0); }",
        "fn test() { let data = vec![]; data.ref$0; }",
    ];
    
    for case in normal_cases {
        let completions = get_completions(case);
        assert!(has_ref_snippet_with_high_priority(&completions),
            "Regression detected in case: {}", case);
    }
}

#[test]
fn test_keyword_snippet_coexistence() {
    // キーワードとスニペットが共存することを確認
    let input = "fn main() { let x = 42; println!(\"{}\", identity(x.ref$0)); }";
    let completions = get_completions(input);
    
    let has_keyword = completions.iter()
        .any(|item| item.kind == CompletionItemKind::Keyword && item.label == "ref");
    let has_snippet = completions.iter()
        .any(|item| item.kind == CompletionItemKind::Snippet && item.label == "ref");
    
    assert!(has_keyword, "ref keyword should be present");
    assert!(has_snippet, "ref snippet should be present");
    
    // スニペットの方が高優先度であることを確認
    let snippet_priority = get_item_priority(&completions, CompletionItemKind::Snippet, "ref");
    let keyword_priority = get_item_priority(&completions, CompletionItemKind::Keyword, "ref");
    
    assert!(snippet_priority >= keyword_priority, 
        "Snippet should have equal or higher priority than keyword");
}
```

#### 2.3 パフォーマンステストの作成

```rust
#[test]
fn test_completion_performance() {
    use std::time::Instant;
    
    let complex_input = r#"
fn main() {
    let data = vec![1, 2, 3];
    println!("{}", identity(transform(process(data.ref$0))));
}
"#;
    
    let start = Instant::now();
    
    // 大量の補完リクエストを実行
    for _ in 0..100 {
        let _completions = get_completions(complex_input);
    }
    
    let duration = start.elapsed();
    
    // 適切なパフォーマンス基準を設定
    assert!(duration.as_millis() < 1000, 
        "Completion performance regression detected: {}ms", duration.as_millis());
}
```

### 📊 Phase 2 完了基準

- [ ] Issue #20263 の具体的テストケースが作成されている
- [ ] リグレッション防止テストが整備されている  
- [ ] パフォーマンステストが追加されている
- [ ] 修正前のテスト実行で適切に失敗する

## 🔧 Phase 3: 優先度ロジック修正

### 🎯 目標
- 最小限の変更で最大の効果を得る
- 既存の動作を壊さない
- 実用性を重視した優先度調整

### 📝 作業内容

#### 3.1 優先度決定ロジックの修正

**ファイル**: `/crates/ide-completion/src/completions/snippet.rs`

```rust
// 現在のコード（修正前）
fn determine_ref_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    if ctx.in_macro_call && is_complex_macro_context(ctx) {
        CompletionPriority::Low  // 🚨 問題の原因
    } else {
        CompletionPriority::High
    }
}

// 修正後のコード
fn determine_ref_snippet_priority(ctx: &CompletionContext) -> CompletionPriority {
    // Approach 1: シンプルな修正（推奨）
    // ref スニペットは実用性が極めて高いため、常に高優先度
    CompletionPriority::High
    
    // Approach 2: より精密な制御（必要に応じて）
    /*
    match analyze_snippet_utility(ctx, "ref") {
        SnippetUtility::High => CompletionPriority::High,
        SnippetUtility::Medium => CompletionPriority::Medium,
        SnippetUtility::Low => CompletionPriority::Low,
    }
    */
}

// 不要になったヘルパー関数を削除または改修
// fn is_complex_macro_context(ctx: &CompletionContext) -> bool {
//     // この関数は削除するか、他の用途に活用
// }
```

#### 3.2 Alternative Approach: より柔軟な制御

より将来性のある実装として、設定可能な優先度システムも検討：

```rust
// 設定ベースの優先度制御
fn determine_snippet_priority(
    snippet_name: &str,
    ctx: &CompletionContext,
    config: &CompletionConfig,
) -> CompletionPriority {
    // ユーザー設定からの優先度取得
    if let Some(user_priority) = config.snippet_priorities.get(snippet_name) {
        return *user_priority;
    }
    
    // デフォルトの実用性ベース優先度
    match snippet_name {
        "ref" => CompletionPriority::High,  // 最も実用的
        "mut" => CompletionPriority::Medium,
        _ => CompletionPriority::Medium,
    }
}
```

#### 3.3 文脈判定の改善（オプショナル）

現在の文脈判定ロジックをより精密にする場合：

```rust
// より精密な文脈分析
fn analyze_completion_context(ctx: &CompletionContext) -> CompletionContextInfo {
    CompletionContextInfo {
        macro_kind: classify_macro_kind(&ctx),
        utility_score: calculate_snippet_utility(&ctx),
        user_preference: get_user_preference(&ctx),
    }
}

fn classify_macro_kind(ctx: &CompletionContext) -> MacroKind {
    match ctx.macro_call.as_ref().and_then(get_macro_name) {
        Some(name) if name.contains("debug") || name == "dbg" => MacroKind::Debug,
        Some(name) if name.contains("print") => MacroKind::Output,
        Some(name) if name.contains("format") => MacroKind::Format,
        _ => MacroKind::Other,
    }
}
```

#### 3.4 修正の段階的適用

```rust
// Phase 3.1: 最小限の修正
fn phase_1_fix() {
    // determine_ref_snippet_priority を常にHighに変更
}

// Phase 3.2: テスト・検証
fn phase_2_validation() {
    // テストスイートの実行
    // 手動テストでの動作確認
}

// Phase 3.3: 必要に応じた調整
fn phase_3_adjustment() {
    // パフォーマンスやユーザビリティの微調整
}
```

### 📊 Phase 3 完了基準

- [ ] 優先度ロジックが修正されている
- [ ] テストケースが全て通過する
- [ ] 既存のテストにリグレッションがない
- [ ] パフォーマンスに悪影響がない

## ✅ Phase 4: 検証・調整

### 🎯 目標
- 修正が期待通りに動作することを確認
- エッジケースでの動作を検証
- 必要に応じて微調整を実施

### 📝 作業内容

#### 4.1 機能テスト

```bash
# 基本テストスイートの実行
cargo test -p ide-completion

# 特定のスニペットテストの実行
cargo test -p ide-completion snippet::tests

# Issue #20263 固有のテストの実行
cargo test -p ide-completion test_ref_snippet_in_complex_macro
```

#### 4.2 手動テスト

**テストケース一覧**:

```rust
// Test Case 1: 基本的な問題ケース
fn test_case_1() {
    let num = 42;
    println!("{}", identity(num.ref$0));
    // 期待: ref スニペットが最上位に表示
}

// Test Case 2: 複雑なネストケース
fn test_case_2() {
    let data = vec![1, 2, 3];
    println!("{}", transform(process(data.ref$0)));
    // 期待: ref スニペットが表示される
}

// Test Case 3: カスタムマクロ
macro_rules! custom_debug {
    ($e:expr) => { println!("Debug: {:?}", $e) };
}
fn test_case_3() {
    let value = "test";
    custom_debug!(value.ref$0);
    // 期待: ref スニペットが表示される
}

// Test Case 4: 型推論との相互作用
fn test_case_4() {
    let value: i32 = 42;
    takes_reference(&value.ref$0);  // &i32 が期待される
    // 期待: ref スニペットが最優先
}
```

#### 4.3 エディタでの実際の動作確認

```bash
# rust-analyzer のビルド
cargo build --release

# VSCode での確認
code test_file.rs
# 各テストケースで補完動作を確認

# Neovim での確認（rust-analyzer LSPクライアント）
nvim test_file.rs
# 補完候補の順序と内容を確認
```

#### 4.4 パフォーマンス検証

```bash
# ベンチマークテストの実行
cargo bench -p ide-completion

# メモリ使用量の確認
valgrind --tool=massif cargo test -p ide-completion

# レスポンス時間の測定
hyperfine 'cargo test -p ide-completion snippet::tests'
```

### 📊 Phase 4 完了基準

- [ ] 全テストケースが期待通りに動作
- [ ] 実際のエディタで問題が解決されている
- [ ] パフォーマンスリグレッションがない
- [ ] エッジケースでも適切に動作

## 📚 Phase 5: 文書化・PR作成

### 🎯 目標
- 変更内容を適切に文書化
- コミュニティレビューのためのPR作成
- メンテナブルな状態での貢献

### 📝 作業内容

#### 5.1 変更内容の文書化

**コミットメッセージの例**:
```
fix(completion): prioritize ref snippet in complex macro contexts

Fixes issue where `ref` snippet was incorrectly deprioritized
in complex macro contexts like `println!("{}", identity(expr.ref))`.

The `ref` snippet is highly practical and should maintain high
priority regardless of macro complexity.

- Always set CompletionPriority::High for ref snippet
- Remove complex macro context penalty
- Add comprehensive test cases for various macro scenarios
- Ensure backward compatibility with existing behavior

Fixes #20263
```

#### 5.2 PR説明の作成

```markdown
## Summary

Fixes #20263 where the `ref` snippet was incorrectly hidden or deprioritized in complex macro contexts.

## Problem

In expressions like `println!("{}", identity(value.ref))`, the `ref` snippet (which expands to `&value`) was not showing up in completion candidates due to overly aggressive filtering in complex macro contexts.

## Solution

- Simplified the priority determination logic for the `ref` snippet
- Always assign `CompletionPriority::High` to `ref` snippet due to its high utility
- Removed the penalty for "complex macro contexts"
- Added comprehensive test cases

## Testing

- [x] All existing tests pass
- [x] New tests added for the specific issue
- [x] Manual testing in VSCode and other editors
- [x] Performance regression tests pass

## Breaking Changes

None. This change only improves the behavior without breaking existing functionality.
```

#### 5.3 テストカバレッジの確認

```bash
# カバレッジレポートの生成
cargo tarpaulin --out Html -p ide-completion

# 追加されたコード行のカバレッジ確認
```

#### 5.4 最終チェックリスト

- [ ] コードスタイルがプロジェクトの規約に準拠
- [ ] すべてのテストが通過
- [ ] ドキュメントが更新されている
- [ ] Breaking changeがない
- [ ] パフォーマンスリグレッションがない

### 📊 Phase 5 完了基準

- [ ] PRが作成され、適切に説明されている
- [ ] CI/CDが全て通過している
- [ ] コードレビューの準備が完了している
- [ ] メンテナンス性が確保されている

## 🚨 リスク管理と対策

### ⚠️ 主要なリスク

1. **既存機能の破綻**
   - 対策: 包括的なリグレッションテスト
   - 軽減: 段階的な実装と継続的テスト

2. **パフォーマンス劣化**
   - 対策: ベンチマークテストの実施
   - 軽減: 最小限の変更を心がける

3. **複雑性の増加**
   - 対策: シンプルな解決策を優先
   - 軽減: 十分な文書化とコメント

4. **コミュニティからの反対**
   - 対策: 透明性のある説明とデモ
   - 軽減: 段階的な実装で合意形成

### 🔄 ロールバック計画

各Phaseでの問題発生時のロールバック手順：

```bash
# Phase 3でのロールバック
git revert <commit-hash>
cargo test -p ide-completion  # 元の状態での動作確認

# Phase 4での部分的ロールバック
git checkout HEAD~1 specific/file.rs
# 問題のある変更のみを取り消し
```

### 📊 成功指標

- Issue #20263 の報告者による動作確認
- 関連するIssueやバグ報告の減少
- コミュニティからのポジティブなフィードバック
- 他の補完機能の動作に悪影響がない

---

この実装戦略に従って段階的に進めることで、リスクを最小化しながら効果的にIssue #20263を解決できます。各Phaseでの完了基準を満たしながら、慎重かつ確実に実装を進めましょう。