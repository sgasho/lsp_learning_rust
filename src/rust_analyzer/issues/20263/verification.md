# 🧪 動作確認・検証方法

## 🎯 この文書の目的

Issue #20263 の修正前後での動作確認方法と、包括的な検証手順を提供します。修正が期待通りに動作し、既存機能に悪影響がないことを確認します。

## 🔍 検証の全体戦略

### 📊 検証の観点

1. **機能性**: 修正により問題が解決されているか
2. **互換性**: 既存の補完機能が壊れていないか
3. **性能**: パフォーマンスに悪影響がないか
4. **ユーザビリティ**: 実際の開発体験が改善されているか

### 🔄 検証フロー

```
修正前ベースライン確認
        ↓
修正実装
        ↓
単体テスト実行
        ↓
統合テスト実行
        ↓
手動テスト実行
        ↓
エディタでの実使用テスト
        ↓
パフォーマンステスト
        ↓
リグレッションテスト
        ↓
最終承認
```

## 📝 修正前のベースライン確認

### 🔍 現在の問題動作の記録

**テスト環境の準備**:
```bash
# rust-analyzer のクローン（修正前）
git clone https://github.com/rust-lang/rust-analyzer.git
cd rust-analyzer
git checkout main

# ビルド
cargo build --release

# テスト用ファイルの作成
cat > test_issue_20263.rs << 'EOF'
use std::convert::identity;

fn main() {
    let num = 42;
    
    // Case 1: 正常動作（ベースライン）
    dbg!(num.ref);  
    
    // Case 2: 問題のあるケース  
    println!("{}", identity(num.ref));
    
    // Case 3: 中間ケース
    println!("{}", num.ref);
}
EOF
```

**問題の再現確認**:
```bash
# LSPサーバーの起動
./target/release/rust-analyzer

# エディタでの補完確認（VSCode例）
code test_issue_20263.rs
# Line 9: identity(num.ref| の位置で補完を実行
# 期待: ref スニペットが表示されない or 低優先度
```

**ベースライン記録例**:
```
修正前の補完候補（println!("{}", identity(num.ref|))）:
1. ref (keyword) - Rust keyword
2. return (keyword) - Return statement  
3. [ref スニペットは非表示または最下位]

修正前の補完候補（dbg!(num.ref|)）:
1. ref (snippet) - Reference snippet (&expr)
2. ref (keyword) - Rust keyword
3. return (keyword) - Return statement
```

## 🧪 単体テスト実行

### 📋 テストスイートの実行

```bash
# 補完関連のテスト全実行
cargo test -p ide-completion

# スニペット関連のテストのみ実行
cargo test -p ide-completion snippet

# キーワード関連のテストのみ実行  
cargo test -p ide-completion keyword

# Issue #20263 固有のテストを実行
cargo test -p ide-completion test_ref_snippet_in_complex_macro
```

### 🎯 具体的なテストケース

#### Test Case 1: 基本的な問題解決確認

```rust
#[test]
fn verify_issue_20263_fix() {
    check_completions(
        r#"
use std::convert::identity;
fn main() {
    let num = 42;
    println!("{}", identity(num.ref$0));
}
"#,
        expect![[r#"
            sn ref Reference snippet (&expr)  [HIGH PRIORITY]
            kw ref Rust keyword               [MEDIUM PRIORITY]
        "#]],
    );
}
```

#### Test Case 2: 優先度順序の確認

```rust
#[test]
fn verify_completion_priority_order() {
    let completions = get_completions(
        r#"println!("{}", identity(value.ref$0));"#
    );
    
    let items: Vec<_> = completions.into_iter()
        .filter(|item| item.label == "ref")
        .collect();
    
    // スニペットがキーワードより上位にあることを確認
    let snippet_pos = items.iter().position(|item| 
        item.kind == CompletionItemKind::Snippet
    ).expect("ref snippet should be present");
    
    let keyword_pos = items.iter().position(|item|
        item.kind == CompletionItemKind::Keyword  
    ).expect("ref keyword should be present");
    
    assert!(snippet_pos <= keyword_pos, 
        "Snippet should appear before or at same position as keyword");
}
```

#### Test Case 3: リグレッション防止

```rust
#[test]
fn verify_no_regression_in_normal_cases() {
    let test_cases = [
        // 既存の正常ケースが壊れていないことを確認
        ("let x = 42; x.ref$0", "Simple expression"),
        ("let x = 42; dbg!(x.ref$0)", "Debug macro"),
        ("let x = 42; format!(\"{}\", x.ref$0)", "Format macro"),
    ];
    
    for (input, description) in test_cases {
        let completions = get_completions(input);
        assert!(
            has_high_priority_ref_snippet(&completions),
            "Regression in case: {}",
            description
        );
    }
}
```

### 📊 テスト結果の評価

```bash
# テスト結果の詳細表示
cargo test -p ide-completion -- --nocapture

# 失敗したテストの詳細確認
cargo test -p ide-completion test_ref_snippet_in_complex_macro -- --exact

# カバレッジレポートの生成
cargo tarpaulin --out Html -p ide-completion
```

## 🔧 手動テスト実行

### 🎯 手動テストケース一覧

#### Category A: Issue #20263 直接テスト

**Test A1: 基本問題ケース**
```rust
use std::convert::identity;

fn main() {
    let num = 42;
    println!("{}", identity(num.ref/*カーソル位置*/));
}
```
- **操作**: `num.ref` の後でCtrl+Space（または補完トリガー）
- **期待結果**: `ref` スニペット（&expr）が最上位または上位に表示
- **確認項目**: スニペットが表示される、適切な優先度

**Test A2: 複雑なネストケース**
```rust
fn main() {
    let data = vec![1, 2, 3];
    println!("{}", transform(process(data.ref/*カーソル位置*/)));
}
```
- **期待結果**: 複雑なネストでも`ref`スニペットが表示

**Test A3: 異なるマクロでのテスト**
```rust
fn main() {
    let value = "test";
    format!("{}", identity(value.ref/*カーソル位置*/));
    eprintln!("{}", transform(value.ref/*カーソル位置*/));
}
```
- **期待結果**: `format!`, `eprintln!` でも同様に動作

#### Category B: リグレッション確認

**Test B1: 通常の式での補完**
```rust
fn main() {
    let value = 42;
    let reference = value.ref/*カーソル位置*/;
}
```
- **期待結果**: 従来通り`ref`スニペットが高優先度で表示

**Test B2: dbg! マクロでの補完**
```rust
fn main() {
    let value = vec![1, 2, 3];
    dbg!(value.ref/*カーソル位置*/);
}
```
- **期待結果**: 従来通りの動作（変化なし）

**Test B3: メソッドチェーンでの補完**
```rust
fn main() {
    let data = vec![1, 2, 3];
    data.iter().collect::<Vec<_>>().ref/*カーソル位置*/;
}
```
- **期待結果**: メソッドチェーン後でも適切に動作

#### Category C: エッジケース

**Test C1: カスタムマクロ**
```rust
macro_rules! custom_debug {
    ($e:expr) => { println!("Debug: {:?}", $e) };
}

fn main() {
    let value = 42;
    custom_debug!(value.ref/*カーソル位置*/);
}
```
- **期待結果**: カスタムマクロでも適切に動作

**Test C2: 型推論との相互作用**
```rust
fn takes_reference<T>(x: &T) {}

fn main() {
    let value = 42;
    takes_reference(value.ref/*カーソル位置*/);
}
```
- **期待結果**: 期待型が`&T`の場合に`ref`スニペットが最優先

### 📝 手動テスト結果記録シート

```
Test Case: [Test ID]
Input Code: [テストコード]
Operation: [実行した操作]
Expected: [期待される結果]
Actual: [実際の結果]
Status: [PASS/FAIL/PARTIAL]
Notes: [追加の注意点]

Example:
Test Case: A1
Input Code: println!("{}", identity(num.ref));
Operation: Trigger completion at cursor position
Expected: ref snippet appears at top with high priority
Actual: ref snippet appears at position 1, keyword at position 2
Status: PASS
Notes: Working as expected after fix
```

## 🖥 エディタでの実使用テスト

### 📝 対象エディタ

1. **Visual Studio Code** (rust-analyzer extension)
2. **Neovim** (with LSP client)
3. **Emacs** (with lsp-mode)
4. **IntelliJ IDEA** (with rust plugin)

### 🔧 エディタテスト手順

#### VSCode でのテスト

```bash
# 修正後のrust-analyzerをビルド
cargo build --release

# パスの確認
which rust-analyzer
# または設定でバイナリパスを指定

# VSCodeでテストファイルを開く
code test_issue_20263.rs
```

**VSCode 固有の確認項目**:
- 補完候補のポップアップ表示
- 候補の順序（最重要）
- スニペット展開の動作
- 詳細情報（hover）の表示

#### Neovim でのテスト

```lua
-- init.lua での rust-analyzer 設定例
require('lspconfig').rust_analyzer.setup({
    cmd = {'/path/to/modified/rust-analyzer'},
    settings = {
        ['rust-analyzer'] = {
            completion = {
                enable = true,
                autoImport = {
                    enable = true,
                },
            },
        },
    },
})
```

**Neovim 固有の確認項目**:
- LSP補完の動作
- 候補の表示順序
- 補完メニューの内容

### 📊 エディタテスト結果マトリックス

```
| Editor    | Issue Case | Normal Case | Performance | Notes        |
|-----------|------------|-------------|-------------|--------------|
| VSCode    | ✅ PASS    | ✅ PASS     | ✅ Good     | No issues    |
| Neovim    | ✅ PASS    | ✅ PASS     | ✅ Good     | LSP working  |
| Emacs     | ❓ TODO    | ❓ TODO     | ❓ TODO     | Need testing |
| IntelliJ  | ❓ TODO    | ❓ TODO     | ❓ TODO     | Need testing |
```

## ⚡ パフォーマンステスト

### 🏃 ベンチマークテスト

#### 補完レスポンス時間の測定

```rust
#[bench]
fn bench_completion_performance(b: &mut Bencher) {
    let input = r#"
fn main() {
    let data = vec![1, 2, 3];
    println!("{}", identity(transform(data.ref$0)));
}
"#;
    
    b.iter(|| {
        black_box(get_completions(input))
    });
}
```

#### メモリ使用量の測定

```bash
# Valgrind を使ったメモリプロファイリング
valgrind --tool=massif --massif-out-file=massif.out \
    cargo test -p ide-completion bench_completion_performance

# メモリ使用量の確認
ms_print massif.out
```

#### 大量リクエストでの負荷テスト

```rust
#[test]
fn stress_test_completion() {
    let complex_cases = [
        "println!(\"{}\", identity(a.ref))",
        "format!(\"{}\", transform(b.ref))", 
        "dbg!(process(c.ref))",
        "eprintln!(\"{}\", convert(d.ref))",
    ];
    
    let start = std::time::Instant::now();
    
    // 1000回の補完リクエストを実行
    for _ in 0..1000 {
        for case in &complex_cases {
            let _completions = get_completions(case);
        }
    }
    
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 5000, 
        "Performance regression: {}ms for 4000 completions", 
        duration.as_millis());
}
```

### 📊 パフォーマンス基準

| 指標 | 修正前 | 修正後 | 判定基準 |
|------|--------|--------|----------|
| 単一補完レスポンス時間 | ~50ms | ≤60ms | ±20%以内 |
| メモリ使用量 | ~100MB | ≤120MB | ±20%以内 |
| 1000回補完の合計時間 | ~2s | ≤2.5s | ±25%以内 |

## 🔄 リグレッションテスト

### 📋 既存機能の確認

```bash
# 全体的なテストスイートの実行
cargo test

# IDE機能全体のテスト
cargo test -p ide

# 補完以外のIDE機能のテスト
cargo test -p ide-diagnostics
cargo test -p ide-hover
cargo test -p ide-rename
```

### 🎯 重点確認項目

#### 1. 他の補完機能への影響

```rust
#[test]
fn test_other_completions_unaffected() {
    // 他のスニペットへの影響確認
    check_completions(
        "fn main() { for$0 }",
        expect![[r#"
            sn for Loop with iterator
            kw for For loop keyword
        "#]]
    );
    
    // キーワード補完への影響確認
    check_completions(
        "fn main() { ret$0 }",
        expect![[r#"
            kw return Return statement
        "#]]
    );
}
```

#### 2. マクロ以外の文脈での動作

```rust
#[test]  
fn test_non_macro_contexts() {
    let cases = [
        "fn test() { value.ref$0 }",           // 関数内
        "impl Struct { fn m() { self.ref$0 } }", // impl内
        "let x = { value.ref$0 };",            // ブロック式内
        "match value { _ => other.ref$0 }",    // match内
    ];
    
    for case in cases {
        let completions = get_completions(case);
        assert!(has_high_priority_ref_snippet(&completions));
    }
}
```

#### 3. エラー処理の確認

```rust
#[test]
fn test_error_cases() {
    // 不正な構文での補完
    let error_cases = [
        "fn main() { println!(, value.ref$0 }",  // 構文エラー
        "fn main() { value.ref$0.invalid }",     // 不正なフィールドアクセス
    ];
    
    for case in error_cases {
        // パニックしないことを確認
        let result = std::panic::catch_unwind(|| {
            get_completions(case)
        });
        assert!(result.is_ok(), "Should not panic on invalid syntax");
    }
}
```

## 📝 検証結果レポート

### 📊 検証結果サマリー

```markdown
# Issue #20263 修正検証レポート

## 検証日時
- 実施日: YYYY-MM-DD
- 検証者: [Your Name]
- rust-analyzer version: [commit hash]

## 修正内容概要
- `ref` スニペットの優先度決定ロジックを修正
- 複雑なマクロ文脈での優先度低下を解消

## 検証結果

### ✅ 機能テスト結果
- [x] Issue #20263 のケースで修正確認
- [x] 各種マクロ文脈での動作確認
- [x] エッジケースでの動作確認

### ✅ 互換性テスト結果  
- [x] 既存の補完機能に影響なし
- [x] 他のスニペットへの影響なし
- [x] キーワード補完への影響なし

### ✅ パフォーマンステスト結果
- [x] レスポンス時間: 基準内
- [x] メモリ使用量: 基準内  
- [x] スループット: 基準内

### ✅ エディタテスト結果
- [x] VSCode: 正常動作
- [x] Neovim: 正常動作
- [ ] その他のエディタ: 要確認

## 発見された問題
なし

## 総合評価
修正は期待通りに動作し、既存機能への悪影響もない。
リリース準備完了。
```

### 🎯 最終承認基準

- [ ] 全単体テストが通過
- [ ] 手動テストで問題解決を確認
- [ ] リグレッションテストで既存機能の正常動作を確認
- [ ] パフォーマンステストで性能劣化がないことを確認
- [ ] 最低2つのエディタで実使用テストを実施
- [ ] コードレビューが完了
- [ ] ドキュメントが更新されている

---

この検証手順に従って包括的にテストを実施することで、Issue #20263 の修正が確実に動作し、既存機能に悪影響がないことを保証できます。各段階での結果を詳細に記録し、問題が発見された場合は迅速に対応しましょう。