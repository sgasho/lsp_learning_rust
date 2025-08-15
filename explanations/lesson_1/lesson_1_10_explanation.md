
# Lesson 1-10: 質問に答えよう！ (LSP応答の作成)

これまでのレッスンで、エディタから送られてくるLSPメッセージを完全に解析できるようになりましたね。素晴らしいです！

しかし、言語サーバーはメッセージを解析するだけではありません。エディタからの「リクエスト」に対しては、必ず「**応答 (Response)**」を返さなければなりません。

## LSP応答の構造

LSPの応答メッセージも、リクエストや通知と同じくJSON-RPC 2.0の形式に従います。成功応答の場合、基本的な構造は以下のようになります。

```json
{
  "jsonrpc": "2.0",
  "id": <リクエストのID>,
  "result": <リクエストの結果>
}
```

*   `jsonrpc`: 必ず `"2.0"` です。
*   `id`: 応答するリクエストの `id` と同じ値です。これにより、エディタはどの質問に対する回答なのかを判断します。
*   `result`: リクエストが成功した場合の「結果」です。これは、リクエストの種類によって様々なJSONの形をとります（オブジェクト、配列、文字列、数値など）。

### エラー応答

もしリクエストの処理中にエラーが発生した場合は、`result` の代わりに `error` フィールドを含めます。これについては、後のレッスンで詳しく学びます。

## Rustで応答メッセージを生成する

`serde_json::json!` マクロを使うと、Rustのコード内で簡単にJSONの構造を構築できます。そして、それを `serde_json::to_string` で文字列に変換し、最後に `Content-Length` ヘッダーを付与すれば、完全なLSP応答メッセージが完成します。

```rust
use serde_json::{json, Value};

let id = json!(1);
let result = json!({ "message": "Hello from server!" });

let response_content = json!({
    "jsonrpc": "2.0",
    "id": id,
    "result": result,
});

let response_str = serde_json::to_string(&response_content).unwrap();
let full_message = format!("Content-Length: {}\r\n\r\n{}", response_str.len(), response_str);

// full_message がエディタに送られるLSP応答メッセージになります。
```

## やってみよう！

あなたの今回のミッションは、`create_lsp_response` 関数を完成させることです。

1.  `id` と `result` を使って、LSPの成功応答のJSONコンテンツを構築します。
2.  構築したJSONコンテンツを文字列に変換します。
3.  その文字列のバイト長を計算し、`Content-Length` ヘッダーを付与して、完全なLSP応答メッセージ文字列を返します。

`src/lessons/lesson_1_10.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！

