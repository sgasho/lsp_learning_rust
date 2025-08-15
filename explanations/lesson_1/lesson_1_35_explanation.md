# Lesson 1-35: 賢く選択しよう！ (LSPセレクションレンジ)

Folding Range機能ができるようになりましたね。今度は、**コードの選択を段階的に拡張**する機能を学びます。

## セレクションレンジ (Selection Range) とは？

### 📖 概念の詳細説明

セレクションレンジは、**エディタでの「選択範囲拡張」操作を賢くする**LSP機能です。

**従来の選択（基本）**:
- ダブルクリック → 単語選択
- 手動でドラッグ → 任意の範囲選択

**✨ セレクションレンジ（賢い選択）**:
- 「選択拡張」ショートカット（通常`Ctrl+W`）を連続実行
- **コードの構造を理解**して段階的に範囲を拡張
- **意味のある単位**で選択が広がる

### 🎯 実際の使用例

```rust
fn calculate_result() {
    let final_value = some_function(parameter);
    //      ↑ カーソル位置
}
```

**段階的な選択拡張**:
1. **1回目**: `final_value` （単語選択）
2. **2回目**: `let final_value = some_function(parameter);` （文選択）
3. **3回目**: `{\n    let final_value = ...\n}` （ブロック選択）
4. **4回目**: `fn calculate_result() { ... }` （関数全体選択）

各段階で**構文的に意味のある範囲**が選択される！

## SelectionRange構造体の詳細

```rust
pub struct SelectionRange {
    pub range: Range,                        // 現在の選択範囲
    pub parent: Option<Box<SelectionRange>>, // 次の拡張レベル
}
```

### 🔍 フィールドの意味

- **`range`**: 現在の選択範囲（Position の start/end）
- **`parent`**: より大きな選択範囲への参照（階層の上位レベル）

### 📦 Boxとは何か？

`Box<T>`は、**ヒープにデータを格納するスマートポインタ**です。

**問題**: 再帰的な構造体
```rust
// ❌ これはコンパイルエラー
struct SelectionRange {
    range: Range,
    parent: Option<SelectionRange>, // 無限サイズになってしまう！
}
```

**解決**: Boxによる間接参照
```rust
// ✅ Boxで間接参照することでサイズが確定
struct SelectionRange {
    range: Range,
    parent: Option<Box<SelectionRange>>, // Box = ポインタサイズ（8bytes）
}
```

### 🏗️ Boxの仕組み

```rust
// スタック（固定サイズ）            ヒープ（可変サイズ）
SelectionRange {                    ┌→ SelectionRange {
    range: Range,                   │      range: Range,
    parent: Some(Box::new(━━━━━━━━━━━┘      parent: Some(Box::new(→次の階層))
}                                          }
```

**Boxの特徴**:
- **ヒープ割り当て**: データをヒープに格納
- **所有権**: Boxが自動的にメモリを管理
- **間接参照**: ポインタ経由でデータにアクセス
- **固定サイズ**: Box自体はポインタサイズ（8bytes）

### 🔗 階層構造の例

```rust
let word_selection = SelectionRange {
    range: Range::new(Position::new(0, 4), Position::new(0, 8)), // "word"
    parent: Some(Box::new(SelectionRange {
        range: Range::new(Position::new(0, 0), Position::new(0, 15)), // "let word = 1;"
        parent: Some(Box::new(SelectionRange {
            range: Range::new(Position::new(0, 0), Position::new(2, 1)), // ブロック全体
            parent: None, // 最上位
        }))
    }))
};
```

階層構造で表現：
- 最小の選択範囲から開始
- `parent`を辿ると次の拡張レベル
- `None`まで到達すると最大の選択範囲

## 実装のポイント

### 1. 段階的選択の計算
- **単語検出**: 識別子文字の境界を見つける
- **文拡張**: セミコロンまでの範囲に拡張
- **ブロック拡張**: 包含する`{}`の範囲に拡張

### 2. 階層構造の作成
```rust
SelectionRange {
    range: word_range,
    parent: Some(Box::new(SelectionRange {
        range: statement_range,
        parent: Some(Box::new(SelectionRange {
            range: block_range,
            parent: None,
        }))
    }))
}
```

## やってみよう！

`provide_selection_ranges`関数を実装してください：

1. 各位置について単語を検出
2. 段階的に選択範囲を拡張
3. 階層的なSelectionRangeを作成
4. すべての選択範囲を返す

`cargo test lesson_1_35`でテストがすべて緑色になったらクリアです！