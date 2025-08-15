# 🎯 Issue #20326: Move `use` statements to top-level

## 📖 この Issue について

**関数内にある `use` 文をファイルの先頭に自動移動**する新しいコードアシスト機能を rust-analyzer に追加する提案です。

### 🤔 現在の問題

```rust
// ❌ 現在: 関数内に散らばった use 文
fn calculate_metrics() {
    use std::collections::HashMap;  // 👈 ここにある
    use std::fs::File;             // 👈 ここにも
    
    let mut data = HashMap::new();
    let file = File::open("data.txt").unwrap();
    // ...
}

fn process_data() {
    use std::io::Read;             // 👈 ここにも...
    // ...
}
```

### ✨ 目指す状態

```rust
// ✅ 理想: ファイル先頭に整理された use 文
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn calculate_metrics() {
    let mut data = HashMap::new();
    let file = File::open("data.txt").unwrap();
    // ...
}

fn process_data() {
    // ...
}
```

## 🎮 使い方 (実装後)

### ステップ1: カーソルを置く
```rust
fn main() {
    use std::collections::HashMap;  // 👈 この行にカーソルを置く
    //  ↑ここでも  ↑ここでも  ↑ここでもOK
    let map = HashMap::new();
}
```

### ステップ2: Code Action を実行
- **VS Code**: `Ctrl+.` (Windows/Linux) または `Cmd+.` (Mac)
- **他のエディタ**: LSP の Code Action キーバインド

### ステップ3: "Move use statement to top-level" を選択
```
💡 Available Actions:
   🔧 Move use statement to top-level  👈 これを選択
   📝 Other actions...
```

## 🧩 Key Requirements

### ✅ 必須機能
- [ ] 関数内の`use`文を検出
- [ ] ファイル先頭に移動
- [ ] 既存の`use`文と統合（グループ化）
- [ ] カーソル位置での発動

### ❌ 除外事項
- すでにトップレベルにある`use`文は対象外
- モジュール内の`use`文は慎重に検討

## 🎬 動作イメージ

### シンプルな移動
```
Before:                           After:
┌───────────────────────────┐    ┌───────────────────────────┐
│ fn process_file() {       │    │ use std::fs::File;        │ ← 移動！
│   use std::fs::File;      │ => │                           │
│   let f = File::open(...) │    │ fn process_file() {       │
│ }                         │    │   let f = File::open(...) │
└───────────────────────────┘    │ }                         │
                                 └───────────────────────────┘
```

### 既存の use 文と統合
```
Before:                           After:
┌───────────────────────────┐    ┌───────────────────────────┐
│ use std::collections::*;  │    │ use std::collections::*;  │
│                           │    │ use std::fs::File;        │ ← 追加・整理
│ fn main() {               │ => │                           │
│   use std::fs::File;      │    │ fn main() {               │
│   // code...              │    │   // code...              │
│ }                         │    │ }                         │
└───────────────────────────┘    └───────────────────────────┘
```

## 🔗 Related GitHub Issue
- **Issue URL**: https://github.com/rust-lang/rust-analyzer/issues/20326
- **Status**: Open
- **Labels**: A-assists (Code Assist)
- **Created**: July 28, 2025