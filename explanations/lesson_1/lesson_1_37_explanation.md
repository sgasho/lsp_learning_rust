# Lesson 1-37: 関連箇所を同時に編集しよう！ (LSPリンクドエディティング)

Code Lens機能ができるようになりましたね。今度は、**関連する識別子を同時に編集**する機能を学びます。

## リンクドエディティング (Linked Editing) とは？

### 📖 概念の詳細説明

リンクドエディティングは、**同じ識別子の複数箇所を同時に編集**するLSP機能です。

**従来の編集体験**:
- 変数名を変更したい → 手動で一つずつ置換
- 見落としでバグが発生
- 大きなファイルでは非効率

**✨ リンクドエディティング（便利な体験）**:
- 一箇所を編集すると関連箇所も自動で変更
- **スコープを理解**した安全な一括編集
- **リアルタイム同期**で即座に反映

### 🎯 実際の使用例

```rust
let variable_name = 42;              // ← ここを編集すると...
let result = variable_name + 1;      // ← ここも同時に変更
println!("{}", variable_name);       // ← ここも同時に変更
```

**編集の流れ**:
1. `variable_name`の任意の箇所にカーソルを置く
2. 編集開始（例：`new_name`に変更）
3. **自動的に**3箇所すべてが`new_name`に変更される

## LinkedEditingRanges構造体の詳細

```rust
pub struct LinkedEditingRanges {
    pub ranges: Vec<Range>,          // 連動編集する範囲のリスト
    pub word_pattern: Option<String>, // 識別子パターン（オプション）
}
```

### 🔍 フィールドの意味

- **`ranges`**: 同時編集する全ての範囲
- **`word_pattern`**: 識別子の文字パターン（通常は使用しない）

### 🏗️ 検出パターン

**1. 変数の宣言と使用**
```rust
let count = 0;        // ← 宣言
count += 1;           // ← 使用1
println!("{}", count); // ← 使用2
```

**2. 関数の定義と呼び出し**
```rust
fn calculate(x: i32) -> i32 { ... }  // ← 定義
let result = calculate(10);          // ← 呼び出し
```

**3. 構造体の定義と使用**
```rust
struct User { ... }              // ← 定義
fn create() -> User { ... }      // ← 使用1  
let user: User = create();       // ← 使用2
```

## 実装のポイント

### 1. 識別子の抽出
- カーソル位置から識別子を正確に特定
- 識別子の境界（開始・終了）を計算

### 2. 同名識別子の検索
- ファイル全体から同じ名前を検索
- 単語境界を考慮した正確なマッチング

### 3. Range作成
```rust
LinkedEditingRanges {
    ranges: vec![
        Range::new(Position::new(0, 4), Position::new(0, 12)), // 宣言箇所
        Range::new(Position::new(1, 13), Position::new(1, 21)), // 使用箇所1
        Range::new(Position::new(2, 13), Position::new(2, 21)), // 使用箇所2
    ],
    word_pattern: None,
}
```

## やってみよう！

`provide_linked_editing_ranges`関数を実装してください：

1. 指定位置の識別子を取得（`get_identifier_at_position`）
2. 同じ識別子の全出現箇所を検索（`find_all_occurrences`）
3. LinkedEditingRangesを作成して返す
4. 識別子がない場合はNoneを返す

`cargo test lesson_1_37`でテストがすべて緑色になったらクリアです！