# Lesson 3-5: シャドウイング

lesson_3_4でネストしたスコープができるようになりましたね。今度は、**シャドウイング（変数の隠蔽）**の詳細な動作を学びます。

## 📚 シャドウイングとは？

### 🤔 変数の隠蔽メカニズム

**シャドウイング**は、内側のスコープで外側と同名の変数を定義することで、外側の変数を「影に隠す」現象です：

```rust
let x = 1;        // 外側のx
{
    let x = 2;    // 内側のx（外側のxをシャドウ）
    print(x);     // → 2 (内側のxが見える)
}
print(x);         // → 1 (外側のxが復活)
```

### rust-analyzerでの重要性

rust-analyzerは以下のためにシャドウイングの理解が必要です：
- **正確な変数解決**: どの変数を参照しているかの特定
- **ホバー情報**: 正しい変数の情報表示
- **リネーム機能**: シャドウされた変数の適切な処理
- **コード補完**: スコープに応じた候補表示

## 🎯 今回の目標

**入力**: シャドウイングを含むAST
```rust
Program {
    statements: [
        LetDeclaration { name: "x", value: Number(1) },  // 外側のx
        Block {
            statements: [
                LetDeclaration { name: "x", value: Number(2) },  // 内側のx
                Expression(Identifier("x"))  // → 内側のxを参照
            ]
        },
        Expression(Identifier("x"))  // → 外側のxを参照
    ]
}
```

**出力**: 正確なシャドウイング解決
```rust
// ブロック内: 内側のx (scope_level: 1)
// ブロック外: 外側のx (scope_level: 0)
```

## 🔍 シャドウイングの種類

### 1. 基本的なシャドウイング

```rust
let x = 1;
{
    let x = 2;  // xをシャドウ
    print(x);   // 2
}
print(x);       // 1
```

### 2. 多層シャドウイング

```rust
let x = 1;
{
    let x = 2;
    {
        let x = 3;  // さらにシャドウ
        print(x);   // 3
    }
    print(x);       // 2
}
print(x);           // 1
```

### 3. 部分的シャドウイング

```rust
let x = 1;
let y = 2;
{
    let x = 10;     // xだけシャドウ
    print(x);       // 10 (シャドウされた)
    print(y);       // 2  (シャドウされていない)
}
```

## 🔧 resolve()メソッドの動作詳細

### シャドウイング解決の仕組み

```rust
pub fn resolve(&self, name: &str) -> Option<&Symbol> {
    let mut current = &self.current_scope;
    
    loop {
        // 現在のスコープで検索（最も内側から）
        if let Some(symbol) = current.symbols.get(name) {
            return Some(symbol);  // 見つかった瞬間に返す
        }
        
        // 親スコープに移動
        if let Some(parent) = &current.parent {
            current = parent;
        } else {
            break;
        }
    }
    
    None
}
```

### なぜシャドウイングが発生するのか？

**検索順序**が重要です：
1. **最も内側のスコープ**から検索開始
2. 見つかった瞬間に**即座に返す**
3. 外側のスコープは**検索されない**

## 🎬 実行例：複雑なシャドウイング

### 入力コード

```rust
let x = 1;
let y = 100;
{
    let x = 2;  // xをシャドウ
    {
        let x = 3;      // xをさらにシャドウ
        let z = x + y;  // x=3, y=100を使用
        print(z);       // 103
    }
    print(x);           // 2 (レベル1のx)
}
print(x);               // 1 (レベル0のx)
```

### 解析プロセス

#### Step 1: グローバル変数定義
```rust
scope_level: 0
symbols: {"x": Symbol{scope_level: 0}, "y": Symbol{scope_level: 0}}
```

#### Step 2: 最初のブロック内でxをシャドウ
```rust
scope_level: 1
symbols: {"x": Symbol{scope_level: 1}}
parent: グローバルスコープ
```

#### Step 3: 二番目のブロック内でxをさらにシャドウ
```rust
scope_level: 2
symbols: {"x": Symbol{scope_level: 2}, "z": Symbol{scope_level: 2}}
parent: レベル1スコープ
```

#### Step 4: 変数解決
```rust
resolve("x"): レベル2で発見 → Symbol{scope_level: 2}
resolve("y"): レベル2→レベル1→レベル0で発見 → Symbol{scope_level: 0}
```

## 🔄 今回の実装（変更なし）

実は、**lesson_3_4の実装で既にシャドウイングは正しく動作**しています！

```rust
// 既存のresolve()メソッドがシャドウイングを正しく処理
pub fn resolve(&self, name: &str) -> Option<&Symbol> {
    let mut current = &self.current_scope;
    
    loop {
        if let Some(symbol) = current.symbols.get(name) {
            return Some(symbol);  // 内側から外側の順で検索
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

### なぜ既に動作するのか？

1. **内側優先検索**: 最も内側のスコープから検索
2. **即座リターン**: 見つかった瞬間に返す
3. **階層構造**: 親スコープへの自然な移動

## 🎯 今回のタスク

### 実装内容（ほぼ変更なし）

今回は**理解を深める**ことが主目的です：

1. **define()での重複チェック修正**: 同一スコープのみチェック
2. **テストケースの追加**: シャドウイングの様々なパターン
3. **動作確認**: 既存実装の正確性確認

### define()の小さな修正

```rust
pub fn define(&mut self, name: String) -> Result<(), String> {
    // 同一スコープでの重複のみチェック（シャドウイングは許可）
    if self.current_scope.symbols.contains_key(&name) {
        return Err(format!("Variable '{}' already defined in this scope", name));
    }

    let symbol = Symbol {
        name: name.clone(),
        scope_level: self.scope_level,
    };
    
    self.current_scope.symbols.insert(name, symbol);
    Ok(())
}
```

## 🐛 シャドウイング関連のエラー

### 1. 同一スコープでの重複定義（エラー）

```rust
{
    let x = 1;
    let x = 2;  // Error: 同一スコープでの重複
}
```

### 2. 異なるスコープでのシャドウイング（OK）

```rust
let x = 1;
{
    let x = 2;  // OK: 異なるスコープなのでシャドウイング
}
```

## 💡 実装のポイント

### 1. 重複チェックの範囲

- **同一スコープ内**: 重複定義エラー
- **異なるスコープ**: シャドウイング（許可）

### 2. 変数解決の順序

- **内側から外側**への検索
- **最初に見つかった変数**を使用

### 3. スコープレベルの活用

- デバッグ時にどのレベルの変数かを確認
- テストでの正確性検証

## ✅ 実装の進め方

1. **基本コピー**: lesson_3_4をベースにコピー
2. **テストケース追加**: シャドウイング特有のテスト
3. **動作確認**: 既存実装の正確性確認
4. **理解深化**: シャドウイングの仕組みを理解

**実行コマンド**: `cargo test lesson_3::lesson_3_5`

## 🎯 テストケース（3つ）

1. **基本シャドウイング**: 単純な変数の隠蔽と復活
2. **多レベルシャドウイング**: 複数階層での変数隠蔽
3. **部分シャドウイング**: 一部変数のみの隠蔽

## 🎉 学習効果

lesson_3_5が完了すると：
- シャドウイングの仕組みを完全理解
- 変数解決アルゴリズムの深い理解
- rust-analyzerの変数管理の核心を把握

**今回は理解重視**なので、リラックスして取り組んでください！