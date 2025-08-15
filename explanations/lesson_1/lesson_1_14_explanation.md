

# Lesson 1-14: コードの問題をエディタに伝えよう！ (診断の公開)

LSPの診断メッセージを生成できるようになりましたね。素晴らしいです！

しかし、診断メッセージを生成するだけでは、エディタはコードの問題を知ることができません。生成した診断メッセージをエディタに「**公開 (Publish)**」する必要があります。これは `textDocument/publishDiagnostics` という通知を使って行われます。

## `textDocument/publishDiagnostics` 通知とは？

この通知は、言語サーバーがエディタに対して「**このファイルに、こんな問題が見つかりましたよ**」と伝えるためのものです。エディタはこれを受け取ると、該当するファイルを開いているユーザーに、問題の箇所を視覚的に表示したり、問題リストを更新したりします。

これは「通知」なので、サーバーはエディタからの返事を期待しません。一方的に情報を送るだけです。

## `textDocument/publishDiagnostics` の構造

この通知の `params` フィールドは、以下の構造を持つオブジェクトです。

```json
{
  "uri": <ドキュメントのURI>,
  "diagnostics": <診断メッセージの配列>
}
```

*   `uri`: 診断を公開するドキュメントのURIです。`lsp_types::Url` を `to_string()` して使います。
*   `diagnostics`: `lsp_types::Diagnostic` オブジェクトの配列です。Lesson 1-13で生成した診断メッセージの `Vec<Diagnostic>` をそのまま使えます。

## Rustで通知メッセージを生成する

これまでのレッスンで学んだ `serde_json::json!` マクロと `format!` マクロを組み合わせて、通知メッセージを生成します。

```rust
use serde_json::{json, Value};
use lsp_types::{Diagnostic, Url};

let file_uri = Url::parse("file:///a.rs").unwrap();
let diagnostics: Vec<Diagnostic> = vec![/* ... */];

let notification_content = json!({
    "jsonrpc": "2.0",
    "method": "textDocument/publishDiagnostics",
    "params": {
        "uri": file_uri.to_string(),
        "diagnostics": diagnostics,
    },
});

let notification_str = serde_json::to_string(&notification_content).unwrap();
let full_message = format!("Content-Length: {}\r\n\r\n{}", notification_str.len(), notification_str);

// full_message がエディタに送られるLSP通知メッセージになります。
```

## やってみよう！

あなたの今回のミッションは、`create_publish_diagnostics_notification` 関数を完成させることです。

1.  `file_uri` と `diagnostics` を使って、LSPの `textDocument/publishDiagnostics` 通知のJSONコンテンツを構築します。
2.  構築したJSONコンテンツを文字列に変換します。
3.  その文字列のバイト長を計算し、`Content-Length` ヘッダーを付与して、完全なLSP通知メッセージ文字列を返します。

`src/lessons/lesson_1_14.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

```