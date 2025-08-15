# Lesson 1-29: 関数の使い方を教えてもらおう！ (LSPシグネチャヘルプ)

LSPのコンプリーション機能ができるようになりましたね。素晴らしいです！

次に、関数呼び出し時に非常に役立つ「**シグネチャヘルプ（Signature Help）**」機能について学びます。この機能は、関数の引数情報をリアルタイムで表示し、現在入力中のパラメータをハイライトして開発効率を大幅に向上させます。

## シグネチャヘルプ (Signature Help) とは？

シグネチャヘルプ機能は、関数呼び出し中にその関数の引数情報を表示する機能です。例えば：

```rust
println!("Hello, {}", |)  // ← カーソル位置で表示される情報：
                          // println!(format: &str, ...)
                          //          ^^^^^^ ← 現在のパラメータ
```

この機能により、開発者は：
- **正確な引数順序**: パラメータの順番と型を確認
- **引数の省略可能性**: 可変引数や省略可能な引数の理解
- **型情報**: 各パラメータの期待される型の確認
- **現在位置**: 今入力しているパラメータの明確化

エディタは、ユーザーが`(`を入力したり、関数呼び出し内で`,`を入力したりすると、言語サーバーに `textDocument/signatureHelp` リクエストを送ります。サーバーはこれに対して、関数のシグネチャ情報（`SignatureHelp` オブジェクト）を返します。

## `lsp-types::SignatureHelp` の構造

LSPのシグネチャヘルプは `lsp_types::SignatureHelp` という構造体で表現されます。主なフィールドは以下の通りです。

*   `signatures`: 関数シグネチャのリスト（`Vec<SignatureInformation>`）
*   `active_signature`: アクティブなシグネチャのインデックス（通常は0）
*   `active_parameter`: 現在入力中のパラメータのインデックス

### `SignatureInformation` の構造
各シグネチャは以下の情報を持ちます：

*   `label`: シグネチャの文字列表現（例: `"println!(format: &str, ...)"`）
*   `documentation`: シグネチャの説明文（省略可）
*   `parameters`: パラメータ情報のリスト（`Vec<ParameterInformation>`）

### `ParameterInformation` の構造
各パラメータは以下の情報を持ちます：

*   `label`: パラメータ名または範囲（例: `"format: &str"`）
*   `documentation`: パラメータの説明文（省略可）

## 関数呼び出しの検出方法

シグネチャヘルプを提供するには、まずカーソルが関数呼び出し内にあるかを判定する必要があります。

### 関数呼び出し検出のアルゴリズム

```rust
fn find_function_call(line: &str, cursor_pos: usize) -> Option<String> {
    let before_cursor = &line[..cursor_pos];
    
    // 後ろから括弧を探す
    let mut paren_count = 0;
    let mut func_end = None;
    
    for (i, ch) in before_cursor.char_indices().rev() {
        match ch {
            ')' => paren_count += 1,
            '(' => {
                if paren_count == 0 {
                    func_end = Some(i);
                    break;
                } else {
                    paren_count -= 1;
                }
            }
            _ => {}
        }
    }
    
    let func_end = func_end?;
    
    // 関数名を後ろから抽出
    let before_paren = &before_cursor[..func_end];
    let mut func_start = func_end;
    
    for (i, ch) in before_paren.char_indices().rev() {
        if ch.is_alphanumeric() || ch == '_' || ch == '!' {
            func_start = i;
        } else {
            break;
        }
    }
    
    if func_start < func_end {
        Some(before_paren[func_start..].to_string())
    } else {
        None
    }
}
```

## アクティブパラメータの計算

現在入力中のパラメータを特定するには、関数呼び出し内のカンマの数を数えます。

### カンマカウントのアルゴリズム

```rust
fn count_active_parameter(line: &str, func_start: usize, cursor_pos: usize) -> usize {
    let inside_call = &line[func_start..cursor_pos];
    let mut paren_count = 0;
    let mut comma_count = 0;
    
    for ch in inside_call.chars() {
        match ch {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            ',' if paren_count == 1 => comma_count += 1, // 同じレベルのカンマのみカウント
            _ => {}
        }
    }
    
    comma_count // 0番目のパラメータから始まるため、カンマの数 = パラメータインデックス
}
```

## 既知の関数のシグネチャ定義

今回のレッスンでは、Rustの主要なマクロのシグネチャを定義します：

### シグネチャ情報の作成

```rust
fn create_println_signature() -> SignatureInformation {
    SignatureInformation {
        label: "println!(format: &str, ...)".to_string(),
        documentation: Some("Prints to the standard output, with a newline.".to_string()),
        parameters: Some(vec![
            ParameterInformation {
                label: "format: &str".to_string(),
                documentation: Some("Format string".to_string()),
            },
            ParameterInformation {
                label: "...".to_string(),
                documentation: Some("Arguments for formatting".to_string()),
            },
        ]),
        active_parameter: None,
    }
}

fn create_format_signature() -> SignatureInformation {
    SignatureInformation {
        label: "format!(format: &str, ...) -> String".to_string(),
        documentation: Some("Creates a String using interpolation of runtime expressions.".to_string()),
        parameters: Some(vec![
            ParameterInformation {
                label: "format: &str".to_string(),
                documentation: Some("Format string".to_string()),
            },
            ParameterInformation {
                label: "...".to_string(),
                documentation: Some("Arguments for formatting".to_string()),
            },
        ]),
        active_parameter: None,
    }
}
```

## 実装のポイント

### 1. 括弧のネスト処理
```rust
// ネストした関数呼び出しを正しく処理
// 例: println!("Result: {}", other_func(a, b))
//                                      ^ ここでのシグネチャヘルプ
let mut paren_count = 0;
for ch in chars {
    match ch {
        '(' => paren_count += 1,
        ')' => paren_count -= 1,
        ',' if paren_count == 1 => comma_count += 1, // 正しいレベルのカンマのみ
        _ => {}
    }
}
```

### 2. マクロ記法の対応
```rust
// vec![1, 2, 3] と vec!(1, 2, 3) の両方に対応
let is_macro = func_name.ends_with('!');
let bracket_or_paren = if is_square_bracket { '[' } else { '(' };
```

### 3. エラーハンドリング
```rust
// 範囲外アクセスの防止
let line = content?.lines().nth(position.line as usize)?;
let before_cursor = line.get(..position.character as usize)?;
```

## やってみよう！

あなたの今回のミッションは、`get_signature_help` 関数を完成させることです。

1.  `document_store` から `file_uri` に対応するファイルの内容を取得します。見つからなければ `None` を返します。
2.  `position` から該当する行を取得し、カーソル位置より前の部分を抽出します。
3.  後ろから`(`を探して、関数呼び出しパターンを検出します。
4.  関数名を抽出し、既知の関数かチェックします：
    *   `"println!"` → `"println!(format: &str, ...)"`
    *   `"format!"` → `"format!(format: &str, ...) -> String"`
    *   `"vec!"` → `"vec![element, ...] -> Vec<T>"`
    *   `"assert_eq!"` → `"assert_eq!(left: T, right: T)"`
5.  関数呼び出し内のカンマを数えて、アクティブパラメータを決定します。
6.  適切な `SignatureHelp` オブジェクトを作成して返します：
    *   `signatures`: シグネチャ情報のベクター
    *   `active_signature`: `Some(0)`（単一シグネチャの場合）
    *   `active_parameter`: 計算されたパラメータインデックス

`src/lessons/lesson_1_29.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！