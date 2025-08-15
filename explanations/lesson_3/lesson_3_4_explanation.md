# Lesson 3-4: ネストしたスコープ

lesson_3_3で単一レベルのブロックスコープができるようになりましたね。今度は、**複数レベルにネストしたスコープ**を扱います。

## 📚 ネストしたスコープとは？

### 🤔 スコープの多層構造

**ネストしたスコープ**は、ブロックの中にさらにブロックがある構造です：

```rust
let a = 1;              // レベル0（グローバル）
{                       // レベル1開始
    let b = 2;
    {                   // レベル2開始
        let c = 3;
        print(a);       // OK: レベル0のaが見える
        print(b);       // OK: レベル1のbが見える
        print(c);       // OK: レベル2のc
    }                   // レベル2終了
    print(c);           // Error: cは見えない
}                       // レベル1終了
```

### rust-analyzerでの重要性

rust-analyzerは以下のためにネストスコープが必要です：
- **関数内のブロック**: if/while/for文のネスト
- **複雑な制御フロー**: 深い条件分岐
- **変数の適切な解決**: どの階層の変数かを特定

## 🎯 今回の目標

**入力**: 深くネストしたAST
```rust
Program {
    statements: [
        LetDeclaration { name: "a", value: Number(1) },
        Block {
            statements: [
                LetDeclaration { name: "b", value: Number(2) },
                Block {
                    statements: [
                        LetDeclaration { name: "c", value: Number(3) },
                        Expression(Identifier("a"))  // 2階層上のaを参照
                    ]
                }
            ]
        }
    ]
}
```

**出力**: 正しい階層的変数解決
```rust
// レベル0: {"a": Symbol}
// レベル1: {"b": Symbol} → parent: レベル0
// レベル2: {"c": Symbol} → parent: レベル1 → parent: レベル0
// Identifier("a")の解決: レベル2→レベル1→レベル0で発見
```

## 🏗️ データ構造の詳細化

### スコープレベルの追跡

```rust
#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize,                    // スコープの深さレベル
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub current_scope: Scope,
    pub scope_level: usize,              // 現在のレベル追跡
}
```

### シンボルにレベル情報を追加

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,              // どのレベルで定義されたか
}
```

## 🔧 拡張されたメソッド

### 1. レベル管理付きのenter_scope()

```rust
impl SymbolTable {
    pub fn enter_scope(&mut self) {
        self.scope_level += 1;  // レベルを増加
        
        let new_scope = Scope {
            level: self.scope_level,
            symbols: HashMap::new(),
            parent: Some(Box::new(self.current_scope.clone())),
        };
        
        self.current_scope = new_scope;
    }
}
```

### 2. レベル管理付きのexit_scope()

```rust
pub fn exit_scope(&mut self) {
    if let Some(parent) = self.current_scope.parent.take() {
        self.scope_level -= 1;  // レベルを減少
        self.current_scope = *parent;
    }
}
```

### 3. レベル情報付きのdefine()

```rust
pub fn define(&mut self, name: String) -> Result<(), String> {
    if self.current_scope.symbols.contains_key(&name) {
        return Err(format!("Variable '{}' already defined in this scope", name));
    }

    let symbol = Symbol {
        name: name.clone(),
        scope_level: self.scope_level,  // 現在のレベルを記録
    };
    
    self.current_scope.symbols.insert(name, symbol);
    Ok(())
}
```

## 🎬 実行例：3レベルネスト

### 入力コード

```rust
let outer = 1;
{
    let middle = 2;
    {
        let inner = 3;
        let sum = outer + middle + inner;  // 3レベル全てから参照
    }
    print(middle);  // OK
    print(inner);   // Error: innerは見えない
}
```

### 解析プロセス

#### Step 1: let outer = 1; (レベル0)
```rust
scope_level: 0
current_scope: {
    level: 0,
    symbols: {"outer": Symbol{scope_level: 0}},
    parent: None
}
```

#### Step 2: 最初のブロック開始 (レベル1)
```rust
enter_scope()実行:
scope_level: 1
current_scope: {
    level: 1,
    symbols: {},
    parent: Some(Box { レベル0のスコープ })
}
```

#### Step 3: let middle = 2; (レベル1)
```rust
current_scope: {
    level: 1,
    symbols: {"middle": Symbol{scope_level: 1}},
    parent: Some(Box { レベル0のスコープ })
}
```

#### Step 4: 二番目のブロック開始 (レベル2)
```rust
enter_scope()実行:
scope_level: 2
current_scope: {
    level: 2,
    symbols: {},
    parent: Some(Box { レベル1のスコープ })
}
```

#### Step 5: let inner = 3; (レベル2)
```rust
current_scope: {
    level: 2,
    symbols: {"inner": Symbol{scope_level: 2}},
    parent: Some(Box { レベル1のスコープ })
}
```

#### Step 6: outer + middle + inner の解析
```rust
resolve("outer"):
レベル2 → なし
レベル1 → なし  
レベル0 → 発見！ Symbol{scope_level: 0}

resolve("middle"):
レベル2 → なし
レベル1 → 発見！ Symbol{scope_level: 1}

resolve("inner"):
レベル2 → 発見！ Symbol{scope_level: 2}
```

#### Step 7: レベル2終了
```rust
exit_scope()実行:
scope_level: 1
current_scope: レベル1のスコープに戻る
```

#### Step 8: print(inner); (エラー)
```rust
resolve("inner"):
レベル1 → なし
レベル0 → なし
→ Error: "Variable 'inner' not defined"
```

## 🔍 実装の詳細

### 1. SymbolTable初期化の更新

```rust
impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            current_scope: Scope {
                level: 0,
                symbols: HashMap::new(),
                parent: None,
            },
            scope_level: 0,
        }
    }
}
```

### 2. Scope構造体の拡張

```rust
impl Scope {
    pub fn new() -> Self {
        Scope {
            level: 0,
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope, level: usize) -> Self {
        Scope {
            level,
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
}
```

### 3. resolve()メソッド（拡張版）

```rust
pub fn resolve(&self, name: &str) -> Option<&Symbol> {
    let mut current = &self.current_scope;
    
    loop {
        if let Some(symbol) = current.symbols.get(name) {
            return Some(symbol);
        }
        
        if let Some(parent) = &current.parent {
            current = parent;
        } else {
            break;
        }
    }
    
    None
}
```

## 🐛 エラーケースの例

### 1. 深いネストでの未定義変数

```rust
{
    {
        {
            print(undefined_var);  // Error: 3レベル全てを検索しても見つからない
        }
    }
}
```

### 2. 中間レベルでの変数隠蔽

```rust
let x = 1;
{
    let x = 2;  // 外側のxを隠蔽
    {
        print(x);  // 2が出力される（レベル1のx）
    }
}
```

## 💡 実装のヒント

### 1. Scope構造体の改良

```rust
#[derive(Debug, Clone)]
pub struct Scope {
    pub level: usize,
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
}
```

### 2. Symbol構造体の改良

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope_level: usize,
}
```

### 3. レベル管理の重要性

- 一貫したレベル追跡
- enter/exitでの正確な増減
- エラー時の状態保持

## ✅ 実装の進め方

1. **Scope構造体を拡張**: levelフィールド追加
2. **Symbol構造体を拡張**: scope_levelフィールド追加
3. **SymbolTable を拡張**: scope_levelフィールド追加
4. **enter_scope/exit_scope を修正**: レベル管理追加
5. **define()を修正**: レベル情報追加
6. **テストで確認**: 4つのテストケース

**実行コマンド**: `cargo test lesson_3::lesson_3_4`

## 🎯 テストケース（4つ）

1. **2レベルネスト**: 基本的な2層構造
2. **3レベルネスト**: より深い3層構造
3. **深いネストでの変数解決**: 複数階層を跨ぐ変数参照
4. **ネスト内でのエラー**: 深い階層での未定義変数

## 🔄 lesson_3_3からの進化

### 追加機能
- ✅ スコープレベルの追跡
- ✅ シンボルのレベル情報
- ✅ より詳細なデバッグ情報

### 継承機能
- ✅ 階層的なスコープ管理
- ✅ 階層的な変数検索
- ✅ ブロックスコープの基本機能

## 🎉 完了後の効果

lesson_3_4が完了すると：
- 任意の深さのネスト対応
- より現実的なプログラム構造の解析
- **lesson_3_5**でシャドウイングの詳細に進む準備完了

**lesson_3_3の基盤があるので、今回は比較的簡単に実装できるはずです！**