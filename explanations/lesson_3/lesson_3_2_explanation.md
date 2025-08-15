# Lesson 3-2: 重複定義のチェック

lesson_3_1で基本的な変数管理ができるようになりましたね。今度は、**同一スコープでの重複定義を検出する**機能を追加します。

## 📚 重複定義とは？

### 🤔 なぜ重複定義はエラーなのか？

**重複定義**は、同じスコープで同じ名前の変数を2回以上定義することです：

```rust
let x = 5;
let x = 10;  // Error: Variable 'x' already defined in this scope
```

### rust-analyzerでの重要性

rust-analyzerは以下のために重複定義チェックが必要です：
- **エラー検出**: コンパイル前にエラーを発見
- **コード品質**: 意図しない変数の上書きを防止
- **開発体験**: リアルタイムなエラー表示

## 🎯 今回の目標

**入力**: 重複定義を含むAST
```rust
Program {
    statements: [
        LetDeclaration { name: "x", value: Number(5) },
        LetDeclaration { name: "x", value: Number(10) }  // 重複！
    ]
}
```

**出力**: エラー検出
```rust
Err(["Variable 'x' already defined in this scope"])
```

## 🔧 実装の変更点

### lesson_3_1からの違い

lesson_3_1では重複チェックをスキップしていました：

```rust
// lesson_3_1 (重複チェックなし)
pub fn define(&mut self, name: String) -> Result<(), String> {
    self.symbols.insert(name.clone(), Symbol { name });
    Ok(())  // 常に成功
}
```

今回は重複チェックを追加します：

```rust
// lesson_3_2 (重複チェックあり)
pub fn define(&mut self, name: String) -> Result<(), String> {
    // 1. 既に定義されているかチェック
    if self.symbols.contains_key(&name) {
        return Err(format!("Variable '{}' already defined in this scope", name));
    }
    
    // 2. 定義されていない場合のみ追加
    self.symbols.insert(name.clone(), Symbol { name });
    Ok(())
}
```

## 🎬 実行例：重複定義の検出

### 入力コード

```rust
let x = 5;
let y = x;
let x = 10;  // 重複定義！
```

### 解析プロセス

#### Step 1: let x = 5;
```rust
// 1. 右辺の式を解析: Number(5) → エラーなし
// 2. 変数xを定義: define("x") → 初回なのでOK
// 結果: symbols = {"x": Symbol}
```

#### Step 2: let y = x;
```rust
// 1. 右辺の式を解析: Identifier("x") → resolve("x") → OK
// 2. 変数yを定義: define("y") → 初回なのでOK
// 結果: symbols = {"x": Symbol, "y": Symbol}
```

#### Step 3: let x = 10;
```rust
// 1. 右辺の式を解析: Number(10) → エラーなし
// 2. 変数xを定義: define("x") → 既に存在！
// 結果: Error: "Variable 'x' already defined in this scope"
```

## 🐛 検出すべきエラーパターン

### 1. 基本的な重複定義

```rust
let x = 1;
let x = 2;  // Error
```

### 2. 異なる型での重複定義

```rust
let name = "hello";
let name = 42;  // Error
```

### 3. 複数の重複定義

```rust
let a = 1;
let b = 2;
let a = 3;  // Error: aが重複
let b = 4;  // Error: bも重複
```

## 💡 実装のヒント

### 1. define()メソッドの修正

```rust
pub fn define(&mut self, name: String) -> Result<(), String> {
    // ヒント：
    // 1. contains_key()で既に存在するかチェック
    // 2. 存在する場合はエラーメッセージを返す
    // 3. 存在しない場合のみinsert()で追加
}
```

### 2. エラーメッセージの統一

```rust
// 推奨フォーマット
"Variable '{}' already defined in this scope"
```

### 3. 他のメソッドは変更不要

- `resolve()`: 変更なし
- `analyze_program()`: 変更なし
- `analyze_statement()`: 変更なし  
- `analyze_expression()`: 変更なし

**たった1つのメソッド修正だけ**で機能を追加できます！

## ✅ 実装の進め方

1. **define()メソッドを修正**: 重複チェックを追加
2. **テストで確認**: 3つのテストケース

**実行コマンド**: `cargo test lesson_3::lesson_3_2`

## 🎯 テストケース（3つのみ）

1. **重複定義エラー**: 同じ変数を2回定義
2. **正常な異なる変数**: 異なる名前での定義
3. **複数重複定義**: 複数の変数で重複エラー

## 🔄 lesson_3_1との違い

### 変更点
- ✅ `define()`メソッドに重複チェック追加
- ✅ 新しいエラーケースの検出

### 変更なし
- ✅ 基本的な変数定義・参照は同じ
- ✅ 未定義変数エラーも同じ
- ✅ プログラム解析の流れも同じ

## 🎉 完了後の効果

lesson_3_2が完了すると：
- より堅牢な変数管理システム
- 実用的なエラー検出機能
- **lesson_3_3**でブロックスコープに進む準備完了

**シンプルな改良**なので、リラックスして取り組んでください！