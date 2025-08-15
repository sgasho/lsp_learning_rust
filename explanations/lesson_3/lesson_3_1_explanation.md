# Lesson 3-1: 基本的な変数定義と参照

lesson_2シリーズで構文解析エンジンが完成しましたね。今度は、**セマンティック解析（意味解析）**の第一歩として、**基本的な変数管理**について学びます。

## 📚 セマンティック解析とは？

### 🤔 構文解析の次のステップ

**構文解析**は「文法的に正しいか」をチェックしますが、**セマンティック解析**は「意味的に正しいか」をチェックします：

```rust
// 構文解析 → OK (文法的に正しい)
// セマンティック解析 → NG (xが定義されていない)
let y = x + 1;
```

### rust-analyzerでの重要性

rust-analyzerは以下のためにシンボル情報が必要です：
- **エラー検出**: 未定義変数の検出
- **コード補完**: 定義済み変数の候補表示
- **参照検索**: 変数の使用箇所を特定

## 🎯 今回の目標（シンプル版）

**入力**: AST（抽象構文木）
```rust
Program {
    statements: [
        LetDeclaration { name: "x", value: Number(5) },
        LetDeclaration { name: "y", value: Identifier("x") }
    ]
}
```

**出力**: 変数管理結果
```rust
// x: 定義済み
// y: 定義済み  
// Identifier("x")の参照: 正常
```

## 🗂️ シンボルテーブルの基本設計

### シンボルの表現

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,           // 変数名
}
```

今回は**名前だけ**を管理します。スコープレベルや位置情報は後のlessonで追加します。

### シンボルテーブル

```rust
#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>, // グローバル変数のみ
}
```

**今回はグローバルスコープのみ**を扱います。ネストしたスコープは次のlessonで学習します。

## 🔧 基本的な変数管理

### 必要な3つの操作

#### 1. 変数を定義する

```rust
fn define(&mut self, name: String) -> Result<(), String> {
    // 変数をシンボルテーブルに追加
}
```

#### 2. 変数を検索する

```rust
fn resolve(&self, name: &str) -> Option<&Symbol> {
    // 変数がシンボルテーブルにあるかチェック
}
```

#### 3. プログラムを解析する

```rust
fn analyze_program(&mut self, program: &Program) -> Result<(), Vec<String>> {
    // 各文を順番に解析
}
```

## 🎬 実行例：シンプルなプログラム

### 入力コード

```rust
let x = 5;
let y = x;
print(y);
```

### 解析プロセス

#### Step 1: let x = 5;
```rust
// 1. 右辺の式を解析: Number(5) → エラーなし
// 2. 変数xを定義: define("x")
// 結果: symbols = {"x": Symbol}
```

#### Step 2: let y = x;
```rust
// 1. 右辺の式を解析: Identifier("x") → resolve("x") → OK
// 2. 変数yを定義: define("y")  
// 結果: symbols = {"x": Symbol, "y": Symbol}
```

#### Step 3: print(y);
```rust
// 1. 引数を解析: Identifier("y") → resolve("y") → OK
// 結果: エラーなし
```

## 🐛 エラー検出の例

### 未定義変数の使用

```rust
let y = x;  // Error: Variable 'x' not defined
```

**解析プロセス**：
```rust
// 1. 右辺を解析: Identifier("x")
// 2. resolve("x") → None
// 3. Error: "Variable 'x' not defined"
```

## 💡 実装のヒント

### 1. SymbolTable構造体

```rust
impl SymbolTable {
    pub fn new() -> Self {
        // 空のHashMapで初期化
    }
    
    pub fn define(&mut self, name: String) -> Result<(), String> {
        // 1. Symbolを作成
        // 2. HashMapに追加
    }
    
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        // HashMapから検索
    }
}
```

### 2. 解析の順序

- **let文**: 右辺→左辺の順で解析
- **式文**: 式の中の変数をチェック

### 3. エラーハンドリング

- 未定義変数が見つかったらエラーメッセージを生成
- エラーがあっても解析は継続

## ✅ 実装の進め方

1. **Symbol構造体を実装**: 基本データ構造
2. **SymbolTableを実装**: 変数管理の核心
3. **ScopeAnalyzerを実装**: プログラム解析
4. **テストで確認**: 4つのテストケース

**実行コマンド**: `cargo test lesson_3::lesson_3_1`

## 🎯 テストケース（4つのみ）

1. **基本的な変数定義と参照**
2. **未定義変数エラー**  
3. **関数引数での変数参照**
4. **式での変数参照**

## 🔄 次のステップ

lesson_3_1が完了すると：
- 変数の基本管理を理解
- セマンティック解析の基礎を習得
- **lesson_3_2**で重複定義のチェックを学習

**今回は基本だけ**なので、リラックスして取り組んでください！