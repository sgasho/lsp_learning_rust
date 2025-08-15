# Lesson 4-4: 到達不可能コード検出

lesson_4_3で未使用関数検出ができるようになりましたね。今度は、**到達不可能コード検出**を学びます。

## 🎯 なぜ到達不可能コード検出？

到達不可能コード検出は**rust-analyzerの高度な診断機能**です：

- **論理エラーの発見**: 意図しないreturn文などの検出
- **コードの整理**: デッドコードの明確な特定
- **保守性向上**: 実行されないコードの除去
- **高度な解析**: 制御フローの理解が必要

### 🔍 検出例

```rust
fn example() -> i32 {
    return 42;         // ここでreturn
    
    let x = 10;        // ← 警告: 到達不可能コード
    println!("{}", x); // ← 警告: 到達不可能コード
    
    x // この行も実行されない
}

fn complex_example(condition: bool) -> i32 {
    if condition {
        return 1;      // then分岐でreturn
    } else {
        return 2;      // else分岐でもreturn
    }
    
    let unreachable = 42; // ← 警告: どちらの分岐でもreturnするため到達不可能
}
```

## 🏗️ 実装アーキテクチャ

### 📦 制御フローASTの導入

lesson_4_4では、return文やif文を含む制御フローを扱います：

```rust
// 制御フロー文を含む拡張された式
#[derive(Debug, Clone, PartialEq)]
pub enum FlowExpr {
    Number(i64, Span),
    Boolean(bool, Span),
    String(String, Span),
    Identifier(String, Span),
    Return {              // 新規追加: return式
        value: Option<Box<FlowExpr>>,
        span: Span,
    },
}

// 制御フロー文を含む拡張された文
#[derive(Debug, Clone, PartialEq)]
pub enum FlowStmt {
    LetDeclaration { name: String, value: FlowExpr, span: Span },
    Expression(FlowExpr),
    Block {               // 新規追加: ブロック文
        statements: Vec<FlowStmt>,
        span: Span,
    },
    IfStatement {         // 新規追加: if文  
        condition: FlowExpr,
        then_branch: Box<FlowStmt>,
        else_branch: Option<Box<FlowStmt>>,
        span: Span,
    },
    Return {              // 新規追加: return文
        value: Option<FlowExpr>,
        span: Span,
    },
}
```

### 🔧 2つのフェーズ（新しい構造）

```rust
impl UnreachableCodeChecker {
    pub fn check(&mut self, program: &FlowProgram) -> Vec<Diagnostic> {
        // Phase 1: 文の到達可能性を解析
        self.analyze_reachability(&program.statements, true);
        
        // Phase 2: 到達不可能コードの診断を生成  
        self.generate_unreachable_diagnostics();  // ← あなたが実装
    }
}
```

## 💡 実装のポイント

### 🎯 実装箇所: 診断生成

**場所**: `generate_unreachable_diagnostics()` メソッド

```rust
fn generate_unreachable_diagnostics(&mut self) {
    // todo!("到達不可能コードの診断を実装してください")
    // ヒント：
    // 1. self.reachable_statements をイテレート
    // 2. is_reachable が false の文を見つける
    // 3. 到達不可能コードの警告を生成
    // 4. self.diagnostics に追加
    // 5. DiagnosticCategory::TypeError を使用（専用カテゴリがないため）
    // 6. メッセージは "unreachable code" を使用
}
```

**考えるポイント**: 
- `ReachabilityInfo`の`is_reachable`フラグをチェック
- `Diagnostic::warning()`の使用
- `DiagnosticCategory::TypeError`の使用（到達不可能性は型エラーの一種として扱う）
- エラーコードの追加（`with_code("unreachable_code".to_string())`）

## 🧠 到達可能性解析の仕組み

### 🔍 基本的な解析ロジック

```rust
fn analyze_reachability(&mut self, statements: &[FlowStmt], mut is_reachable: bool) -> bool {
    let mut has_early_return = false;

    for stmt in statements {
        // 1. 現在の文の到達可能性を記録
        let reachability = ReachabilityInfo::new(stmt.clone(), is_reachable);
        self.reachable_statements.push(reachability);

        // 2. 文の種類に応じた解析
        match stmt {
            FlowStmt::Return { .. } => {
                // return文以降は到達不可能
                has_early_return = true;
                is_reachable = false;  // 後続の文は到達不可能
            }
            FlowStmt::IfStatement { then_branch, else_branch, .. } => {
                // if文の両分岐を解析
                let then_returns = self.analyze_stmt_reachability(then_branch, true);
                let else_returns = if let Some(else_stmt) = else_branch {
                    self.analyze_stmt_reachability(else_stmt, true)
                } else {
                    false  // else節がない場合は、returnしない
                };

                // 両方の分岐でreturnする場合、後続は到達不可能
                if then_returns && else_returns {
                    is_reachable = false;
                    has_early_return = true;
                }
            }
            // その他の文...
        }
    }

    has_early_return
}
```

### 🌊 制御フローの複雑なケース

#### ケース1: 単純なreturn
```rust
return 42;        // 到達可能
let x = 10;       // 到達不可能（return後）
```

#### ケース2: if文の部分return
```rust
if condition {
    return 1;     // then分岐でreturn
}
// else節なし → 分岐を通らない可能性がある

let x = 42;       // 到達可能（else節を通る場合がある）
```

#### ケース3: if文の完全return
```rust
if condition {
    return 1;     // then分岐でreturn
} else {
    return 2;     // else分岐でもreturn
}

let x = 42;       // 到達不可能（どちらの分岐でもreturn）
```

#### ケース4: ネストしたブロック
```rust
{
    return 42;    // ブロック内でreturn
    let x = 1;    // 到達不可能
}
let y = 2;        // 到達不可能（ブロック内でreturn済み）
```

## 🔍 lesson_4_1-4_3からの進化

### 共通パターン（継承された部分）
- ✅ **診断生成**: 同じ`Diagnostic::warning()`パターン
- ✅ **テスト構造**: 複数のテストケース
- ✅ **エラーハンドリング**: 適切な診断メッセージ

### 大きな変化（新しい複雑さ）
- 🔄 **解析アプローチ**: 使用状況追跡 → 制御フロー解析
- 🔄 **データ構造**: HashMap管理 → 順次解析
- 🔄 **複雑性**: 単純な存在チェック → 複雑な論理解析
- 🔄 **状態管理**: 静的情報 → 動的な到達可能性状態

### アルゴリズムの違い

```rust
// lesson_4_1-4_3: 使用状況ベース
1. 定義を収集
2. 使用箇所を追跡  
3. 未使用を検出

// lesson_4_4: 制御フローベース
1. 文を順次解析
2. 到達可能性を追跡
3. 到達不可能を検出
```

## ⚙️ ReachabilityInfoの役割

```rust
#[derive(Debug, Clone)]
pub struct ReachabilityInfo {
    pub statement: FlowStmt,    // 解析対象の文
    pub is_reachable: bool,     // 到達可能かどうか
    pub span: Span,             // 位置情報
}
```

この構造体により、各文の到達可能性を順次記録し、後で到達不可能な文を特定できます。

## ✅ 実装手順

1. **lesson_4_4.rs** の `todo!()` を実装
2. **テスト実行**: `cargo test lesson_4::lesson_4_4`
3. **4つのテスト**をすべてパス

## 🎯 テストケース

1. **return後の到達不可能**: 単純なreturn文後のコード
2. **到達可能のみ**: 全てのコードが到達可能なケース
3. **if文の完全return**: 両分岐でreturnする場合
4. **if文の部分return**: 一方の分岐のみreturnする場合

## 📚 実際のrust-analyzerでの例

```rust
// よくある到達不可能コードのケース

fn example1() -> i32 {
    if true {
        return 42;
    }
    
    // この部分は到達不可能（常にtrue）
    let dead_code = 10;  // ← 警告
    dead_code
}

fn example2(x: i32) -> i32 {
    match x {
        1 => return 1,
        2 => return 2,
        _ => return 3,
    }
    
    // 全てのケースでreturnするため到達不可能
    println!("This never prints");  // ← 警告
}

fn example3() -> Result<i32, &'static str> {
    return Ok(42);
    
    // return後なので到達不可能
    if some_condition() {  // ← 警告
        Err("error")
    } else {
        Ok(0)
    }
}
```

## 🎉 完了後の効果

lesson_4_4が完了すると：
- **制御フロー解析**: 高度な静的解析スキル
- **複雑な論理**: if文とreturn文の組み合わせ理解  
- **実用的診断**: より sophisticated な診断機能
- **rust-analyzer準備**: 実際の制御フロー解析に近い経験

**lesson_4_1-4_4で、診断機能の基礎から応用まで完成！**

## 🔄 学習の進化

```
lesson_4_1: 単純な存在チェック（変数）
    ↓
lesson_4_2: 単純な存在チェック（インポート）
    ↓  
lesson_4_3: 複雑な追跡（関数呼び出し）
    ↓
lesson_4_4: 制御フロー解析（到達可能性）← より高度な静的解析
```

この実装により、rust-analyzerの診断機能の理解が大幅に深まります！