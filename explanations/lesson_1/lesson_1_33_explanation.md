# Lesson 1-33: コードを整理しよう！ (LSPドキュメントフォーマット・基本編)

セマンティックトークン機能ができるようになりましたね。今度は、**インデント調整**でコードを整理する機能を学びます。

## ドキュメントフォーマットとは？

コードのインデントを統一して読みやすくする機能です。

**❌ フォーマット前:**
```rust
fn main() {
let x = 1;
if x > 0 {
println!("positive");
}
}
```

**✅ フォーマット後:**
```rust
fn main() {
    let x = 1;
    if x > 0 {
        println!("positive");
    }
}
```

## 基本ルール

- `{` の後：次の行のインデント +1レベル
- `}` の行：インデント -1レベル  
- 1レベル = 4スペース

## 実装のポイント

### 1. インデントレベルの管理
```rust
let mut indent_level = 0;

// } がある行：レベルを減らす
if line.trim().starts_with('}') {
    indent_level -= 1;
}

// { がある行：次の行のレベルを増やす
if line.contains('{') {
    indent_level += 1;
}
```

### 2. インデント適用
```rust
fn create_indent(level: i32) -> String {
    " ".repeat((level * 4).max(0) as usize)
}

// 空行以外にインデントを適用
let trimmed = line.trim();
if !trimmed.is_empty() {
    format!("{}{}", create_indent(indent_level), trimmed)
}
```

## やってみよう！

`format_document` 関数を実装してください：

1. 各行を処理してブレースを検出
2. インデントレベルを管理  
3. 正しいインデントを適用
4. 結果を結合して返す

`cargo test lesson_1_33` でテストがすべて緑色になったらクリアです！