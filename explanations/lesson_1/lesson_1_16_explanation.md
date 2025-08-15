# Lesson 1-16: ファイルを開いたことを知ろう！ (textDocument/didOpen)

コード補完の基礎を学びましたね。素晴らしいです！

言語サーバーがコードの診断や補完などの機能を提供するためには、まず「**どのファイルがエディタで開かれているか**」を知る必要があります。これは `textDocument/didOpen` という通知を使ってエディタからサーバーに伝えられます。

## `textDocument/didOpen` 通知とは？

エディタが新しいファイルを開いたとき、または既存のファイルをエディタで開いたときに、言語サーバーに送られる通知です。この通知には、開かれたファイルのURI（場所）と、そのファイルの内容が含まれています。

サーバーはこれを受け取ると、そのファイルの内容を自分の「**ドキュメントストア**」という場所に保存します。ドキュメントストアは、サーバーが現在開いているすべてのファイルの内容を管理するための場所です。これにより、サーバーはいつでもファイルの内容にアクセスして、診断を生成したり、補完候補を提供したりできるようになります。

## `params` の構造

`textDocument/didOpen` 通知の `params` フィールドは、以下のような構造を持つオブジェクトです。

```json
{
  "textDocument": {
    "uri": "file:///path/to/file.rs",
    "languageId": "rust",
    "version": 1,
    "text": "fn main() {}\n"
  }
}
```

ここから `uri` と `text` の値を取り出す必要があります。

## RustでのJSON値の抽出と `HashMap`

`serde_json::Value` から特定のフィールドの値を取り出すには、`get()` メソッドを使います。そして、その値が文字列であれば `as_str()` で `&str` に変換できます。`Url::parse()` を使えば、URI文字列を `Url` 型に変換できます。

ドキュメントストアとしては、Rustの標準ライブラリにある `std::collections::HashMap` が便利です。これは「キー」と「値」のペアを保存するもので、今回は `Url` をキーに、ファイルの内容（`String`）を値として保存します。

```rust
use std::collections::HashMap;
use lsp_types::Url;

let mut document_store: HashMap<Url, String> = HashMap::new();
let uri_string = "file:///a.rs";
let file_content = "Hello, world!";

let uri = Url::parse(uri_string).unwrap(); // URI文字列をUrl型に変換
document_store.insert(uri.clone(), file_content.to_string()); // HashMapに保存

// 保存した内容を取り出す
if let Some(content) = document_store.get(&uri) {
    println!("Stored content: {}", content);
}
```

## やってみよう！

あなたの今回のミッションは、`handle_did_open_notification` 関数を完成させることです。

1.  `params_value` から `textDocument` オブジェクトを取得します。
2.  `textDocument` オブジェクトから `uri` と `text` の値を取得します。
3.  `uri` 文字列を `Url` 型にパースします。
4.  パースした `Url` をキー、`text` を値として `document_store` に保存します。
5.  保存した `text` の内容に対して、Lesson 1-13 で作成した `generate_diagnostics` 関数を使って診断を生成し、その結果を `Vec<Diagnostic>` として返します。
6.  もし `params_value` のパースに失敗したり、必要なフィールドが欠けていたり、URIが無効だったりした場合は、空の `Vec<Diagnostic>` を返します。

`src/lessons/lesson_1_16.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

