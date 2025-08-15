# Lesson 3-3: 単一レベルのブロックスコープ

lesson_3_2で重複定義のチェックができるようになりましたね。今度は、**ブロックスコープ（`{}`）**の概念を導入します。

## 📚 ブロックスコープとは？

### 🤔 スコープの階層化

**ブロックスコープ**は、`{}`で囲まれた範囲内でのみ有効な変数スコープです：

```rust
let x = 1;        // グローバルスコープ
{                 // ブロックスコープ開始
    let y = 2;    // ブロック内でのみ有効
    print(x);     // OK: 外側のxが見える
    print(y);     // OK: 同じブロック内のy
}                 // ブロックスコープ終了
print(x);         // OK: グローバルスコープのx
print(y);         // Error: yは見えない
```

### rust-analyzerでの重要性

rust-analyzerは以下のためにブロックスコープが必要です：
- **変数の生存期間管理**: いつ変数が使用可能か
- **メモリ効率**: 不要になった変数の管理
- **エラー検出**: スコープ外アクセスの検出

## 🎯 今回の目標

**入力**: ブロックを含むAST
```rust
Program {
    statements: [
        LetDeclaration { name: "x", value: Number(1) },
        Block {
            statements: [
                LetDeclaration { name: "y", value: Number(2) }
            ]
        }
    ]
}
```

**出力**: スコープ階層管理
```rust
// グローバルスコープ: {"x": Symbol}
// ブロック終了後: yは削除され、xのみ残る
```

## 🏗️ データ構造の拡張

### 新しいスコープ構造

```rust
#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,  // 親スコープへの参照
}

#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Scope,        // 現在のスコープ
}
```

### 新しいAST要素

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration { name: String, value: Expr },
    Expression(Expr),
    Block { statements: Vec<Stmt> },  // 新規追加
}
```

## 🔧 新しいメソッド

### 1. スコープ管理

```rust
impl SymbolTable {
    // 新しいスコープに入る
    pub fn enter_scope(&mut self) {
        // 現在のスコープを親として保存
        // 新しい空のスコープを作成
    }
    
    // スコープから出る
    pub fn exit_scope(&mut self) {
        // 親スコープに戻る
        // 現在のスコープの変数は削除される
    }
}
```

### 2. 階層的な変数検索

```rust
pub fn resolve(&self, name: &str) -> Option<&Symbol> {
    // 現在のスコープから開始
    // 見つからなければ親スコープを検索
    // 最上位まで検索して見つからなければNone
}
```

## 🎬 実行例：ブロックスコープの動作

### 入力コード

```rust
let x = 1;
{
    let y = 2;
    print(x);  // OK: xは外側で定義
    print(y);  // OK: yは同じブロック内
}
print(x);      // OK: xはまだ有効
print(y);      // Error: yはスコープ外
```

### 解析プロセス

#### Step 1: let x = 1; (グローバルスコープ)
```rust
current_scope: {
    symbols: {"x": Symbol},
    parent: None
}
```

#### Step 2: ブロック開始 `{`
```rust
enter_scope()実行:
current_scope: {
    symbols: {},                    // 空の新しいスコープ
    parent: Some(Box { グローバルスコープ })
}
```

#### Step 3: let y = 2; (ブロック内)
```rust
current_scope: {
    symbols: {"y": Symbol},         // yを追加
    parent: Some(Box { グローバルスコープ })
}
```

#### Step 4: print(x); (変数解決)
```rust
resolve("x"):
1. 現在のスコープ: {"y"} → xなし
2. 親スコープ: {"x"} → x発見！✅
```

#### Step 5: print(y); (変数解決)
```rust
resolve("y"):
1. 現在のスコープ: {"y"} → y発見！✅
```

#### Step 6: ブロック終了 `}`
```rust
exit_scope()実行:
current_scope: {
    symbols: {"x": Symbol},         // グローバルスコープに戻る
    parent: None
}
// yは削除された
```

#### Step 7: print(y); (エラー)
```rust
resolve("y"):
1. 現在のスコープ: {"x"} → yなし
2. 親スコープ: None → y見つからない❌
Error: "Variable 'y' not defined"
```

## 🔍 実装の詳細

### 1. enter_scope()の実装

```rust
pub fn enter_scope(&mut self) {
    // ヒント：
    // 1. 現在のscopeをclone()して保存
    // 2. 新しいScopeを作成（空のsymbols、parentは現在のscope）
    // 3. current_scopeを新しいscopeに更新
}
```

### 2. exit_scope()の実装

```rust
pub fn exit_scope(&mut self) {
    // ヒント：
    // 1. current_scope.parentがある場合のみ処理
    // 2. parentを取り出してcurrent_scopeに設定
    // 3. 元のスコープの変数は自動的に削除される
}
```

### 3. resolve()の階層検索

```rust
pub fn resolve(&self, name: &str) -> Option<&Symbol> {
    // ヒント：
    // 1. current_scopeから開始
    // 2. whileループでparentを辿る
    // 3. 各レベルでsymbols.get(name)をチェック
}
```

### 4. ブロック文の解析

```rust
fn analyze_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
    match stmt {
        // 既存の処理...
        Stmt::Block { statements } => {
            self.symbol_table.enter_scope();
            
            for stmt in statements {
                self.analyze_statement(stmt)?;
            }
            
            self.symbol_table.exit_scope();
            Ok(())
        }
    }
}
```

## 🐛 エラーケースの例

### 1. ブロック外アクセス

```rust
{
    let temp = 42;
}
print(temp);  // Error: Variable 'temp' not defined
```

### 2. ネストなしの正常ケース

```rust
let a = 1;
{
    let b = 2;
    let c = a + b;  // OK: aは外側で定義
}
```

## 💡 実装のヒント

### 1. Scope構造の設計

```rust
#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: Scope) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
}
```

### 2. エラーハンドリングの注意点

- ブロック内でエラーが発生してもexit_scope()を実行
- エラー発生時のスコープ状態を適切に管理

## ✅ 実装の進め方

1. **Scope構造体を実装**: 階層的なスコープ管理
2. **SymbolTable を拡張**: enter_scope/exit_scope追加
3. **resolve()を修正**: 階層検索に対応
4. **AST拡張**: Block文の追加
5. **analyze_statement()拡張**: Block処理追加
6. **テストで確認**: 4つのテストケース

**実行コマンド**: `cargo test lesson_3::lesson_3_3`

## 🎯 テストケース（4つ）

1. **基本ブロックスコープ**: ブロック内変数の定義と使用
2. **ブロック外アクセスエラー**: スコープ外での変数参照
3. **外側変数へのアクセス**: ブロック内から外側変数を参照
4. **空ブロック**: 何も定義しないブロック

## 🔄 lesson_3_2からの進化

### 追加機能
- ✅ ブロックスコープの階層管理
- ✅ スコープの入退場機能
- ✅ 階層的な変数検索

### 継承機能
- ✅ 重複定義チェック（同一スコープ内）
- ✅ 未定義変数エラー
- ✅ 基本的な変数管理

## 🎉 完了後の効果

lesson_3_3が完了すると：
- 階層的なスコープシステムの理解
- より現実的な変数管理
- **lesson_3_4**でネストしたスコープに進む準備完了

**今回は新しい概念が多いですが、一歩ずつ進めていきましょう！**