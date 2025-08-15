# Lesson 1-36: コードに情報を表示しよう！ (LSPコードレンズ)

Selection Range機能ができるようになりましたね。今度は、**コードの上に実行可能なアクションを表示**する機能を学びます。

## コードレンズ (Code Lens) とは？

### 📖 概念の詳細説明

コードレンズは、**関数やクラスの上に表示される小さなボタン**のような機能です。

**従来の開発体験**:
- 関数を実行したい → 手動でコマンド実行
- テストを実行したい → 別のツールを起動
- 参照を調べたい → 検索コマンドを使用

**✨ コードレンズ（便利な体験）**:
- 関数の上に「▶ Run test」ボタンが表示
- クリック一つでアクション実行
- **コードと連動した直感的なUI**

### 🎯 実際の表示例

```rust
▶ Run function                    // ← このボタンがコードレンズ
fn main() {
    println!("Hello World");
}

🔍 Show references              // ← このボタンもコードレンズ  
fn calculate(x: i32) -> i32 {
    x * 2
}

🧪 Run test                     // ← テスト専用のコードレンズ
#[test]
fn test_example() {
    assert_eq!(1, 1);
}
```

各関数の**役割に応じた適切なアクション**が提供される！

## CodeLens構造体の詳細

```rust
pub struct CodeLens {
    pub range: Range,              // レンズを表示する位置
    pub command: Option<Command>,  // クリック時に実行するコマンド
    pub data: Option<serde_json::Value>, // 追加データ
}

pub struct Command {
    pub title: String,             // ボタンに表示するテキスト
    pub command: String,           // 実行するコマンドID
    pub arguments: Option<Vec<serde_json::Value>>, // コマンドの引数
}
```

### 🔍 フィールドの意味

- **`range`**: コードレンズを表示する位置（通常は関数名の範囲）
- **`command`**: ボタンをクリックしたときの動作
- **`title`**: ユーザーに表示されるテキスト（"Run test"など）
- **`command`**: エディタが実行するコマンドのID

### 🏗️ 関数タイプとアクションの対応

```rust
#[derive(Debug, PartialEq)]
enum FunctionType {
    Test,    // #[test] 付きのテスト関数
    Main,    // main関数（エントリポイント）
    Regular, // その他の通常関数
}

// 関数タイプごとの適切なアクション
match function_type {
    FunctionType::Test => "Run test",        // テスト実行
    FunctionType::Main => "Run function",    // プログラム実行
    FunctionType::Regular => "Show references", // 参照表示
}
```

## 実装のポイント

### 1. 関数検出の基本パターン
- `fn ` で始まる行を検出
- 関数名を正確に抽出
- 前行の`#[test]`アノテーションをチェック

### 2. 位置計算
- 関数名の正確な範囲を計算
- コードレンズは関数定義行に配置

### 3. Command作成
```rust
Command {
    title: "Run test".to_string(),
    command: "rust.test.run".to_string(),
    arguments: Some(vec![
        serde_json::Value::String(function_name.to_string())
    ]),
}
```

## やってみよう！

`provide_code_lenses`関数を実装してください：

1. 関数定義を検出（`find_function_definitions`）
2. 関数タイプを判定（`determine_function_type`）  
3. 適切なCommandを作成（`create_command_for_function`）
4. CodeLensオブジェクトを作成して返す

`cargo test lesson_1_36`でテストがすべて緑色になったらクリアです！