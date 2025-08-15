# 🧪 動作確認の方法

## 🎬 デモンストレーション

実装した機能がちゃんと動くかを**視覚的に**確認する方法を説明します。

### 🎯 期待される動作フロー

```
👆 カーソルを use 文に置く
     ↓
💡 Code Action が表示される
     ↓  
✨ ワンクリックで移動完了！
```

## 🚀 Step-by-Step テスト

### 1. 準備：テストケースを作成

```rust
// test_file.rs
fn example_function() {
    use std::collections::HashMap;
    use std::vec::Vec;
    
    let map = HashMap::new();
    let vec = Vec::new();
}

mod inner_module {
    fn another_function() {
        use std::fs::File;
        // some code
    }
}
```

### 2. 基本動作の確認

#### ✅ Step 1: カーソル位置テスト
```
1. `use std::collections::HashMap;` の行にカーソルを置く
2. Code Action (Ctrl+. / Cmd+.) を呼び出す
3. "Move use statement to top-level" のようなアクションが表示されるか確認
```

#### ✅ Step 2: 移動動作テスト
```
Expected Result:
┌─────────────────────────────────┐
│ use std::collections::HashMap;  │ ← 移動された
│ use std::vec::Vec;              │ ← 移動された
│                                 │
│ fn example_function() {         │
│     let map = HashMap::new();   │
│     let vec = Vec::new();       │
│ }                               │
└─────────────────────────────────┘
```

## 🎯 詳細テストケース

### Case 1: 単一の use 文
```rust
// Before
fn test() {
    use std::fs::File;  // 👈 カーソルをここに
    let f = File::open("test.txt");
}

// After
use std::fs::File;

fn test() {
    let f = File::open("test.txt");
}
```

### Case 2: 既存の use 文がある場合
```rust
// Before
use std::collections::Vec;  // 既存

fn test() {
    use std::fs::File;  // 👈 カーソルをここに
}

// After
use std::collections::Vec;
use std::fs::File;  // グループ化される

fn test() {
    // ...
}
```

### Case 3: ネストした関数
```rust
// Before
fn outer() {
    fn inner() {
        use std::thread;  // 👈 カーソルをここに
        thread::spawn(|| {});
    }
}

// After
use std::thread;

fn outer() {
    fn inner() {
        thread::spawn(|| {});
    }
}
```

## 🔍 確認すべきポイント

### ✅ 正常動作チェックリスト
- [ ] カーソルが`use`文にあるときだけアシストが表示される
- [ ] `use`文がファイル先頭に移動される
- [ ] 既存の`use`文と適切にグループ化される
- [ ] 元の`use`文が削除される
- [ ] インデントが正しく保たれる

### ❌ エラーケースチェックリスト
- [ ] トップレベルの`use`文では動作しない
- [ ] 複数行にわたる`use`文でも動作する
- [ ] コメント付きの`use`文でも動作する

## 🛠 開発時のテスト方法

### rust-analyzer の開発環境で：
```bash
# テストの実行
cargo test assists::move_use_to_top

# 特定のテストケースを実行
cargo test assists::move_use_to_top::test_basic_move
```

### VS Code での確認：
1. rust-analyzer を開発モードでビルド
2. VS Code の拡張機能として読み込み
3. 上記のテストケースでマニュアル確認