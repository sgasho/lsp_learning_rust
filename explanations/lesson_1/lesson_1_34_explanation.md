# Lesson 1-34: コードを折りたたもう！ (LSPフォールディングレンジ)

Document Formatting機能ができるようになりましたね。今度は、**コードブロックを折りたたみ**できるようにする機能を学びます。

## フォールディングレンジ (Folding Range) とは？

大きなコードを見やすくするため、ブロックを折りたたんで隠す機能です。

**例：折りたたみ前後**
```rust
// 📖 展開時
fn main() {
    if true {
        println!("Hello");
        let x = 1;
    }
}

// 📁 折りたたみ時  
fn main() { ... }
```

## 検出するブロックの種類

### 1. 関数ブロック
```rust
fn main() {        // ← 開始
    let x = 1;
    println!("Hi");
}                  // ← 終了
```

### 2. 構造体定義
```rust
struct User {      // ← 開始
    name: String,
    age: u32,
}                  // ← 終了
```

### 3. 制御文ブロック
```rust
if condition {     // ← 開始
    do_something();
}                  // ← 終了
```

## 実装のポイント

### 1. FoldingRange構造体の作成
```rust
FoldingRange {
    start_line: start_line as u32,     // 開始行（0ベース）
    end_line: end_line as u32,         // 終了行（0ベース）
    start_character: None,             // 開始文字位置（オプション）
    end_character: None,               // 終了文字位置（オプション）
    kind: Some(FoldingRangeKind::Region), // 折りたたみの種類
}
```

### 2. ネストしたブレースの対応検出
```rust
fn find_matching_brace(lines: &[&str], start_line: usize) -> Option<usize> {
    let mut brace_count = 0;
    
    for (i, line) in lines.iter().enumerate().skip(start_line) {
        for ch in line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        return Some(i);  // 対応するブレース発見
                    }
                }
                _ => {}
            }
        }
    }
    None
}
```

### 3. 重要な条件
- 複数行にわたるブロックのみ折りたたみ対象
- 1行で完結するブロック（`fn test() { return; }`）は除外

## やってみよう！

`provide_folding_ranges`関数を実装してください：

1. 各行をスキャンしてブロック開始を検出
2. 対応する終了ブレースを見つける
3. 複数行にわたるブロックのみ対象
4. フォールディングレンジを作成して返す

`cargo test lesson_1_34`でテストがすべて緑色になったらクリアです！