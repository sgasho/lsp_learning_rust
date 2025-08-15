
# Lesson 1-5: 最初の挨拶を理解しよう！ (Initializeリクエストのパース)

これまでのレッスンで、LSPメッセージの「封筒」と「便箋」を扱い、それぞれの情報を読み取る方法を学びました。いよいよ、これらの知識を組み合わせて、LSPで最も最初に送られてくる重要なメッセージである「`initialize` リクエスト」をパースする練習をします。

## `initialize` リクエストとは？

エディタが言語サーバーを起動すると、まず最初に送るのが `initialize` リクエストです。これは、エディタが言語サーバーに対して「**こんにちは！私はこんなことができますが、あなたは何ができますか？**」と自己紹介し、お互いの能力を確認するための「最初の挨拶」のようなものです。

このリクエストには、エディタの名前やバージョン、開いているプロジェクトのパス、エディタがサポートしている機能（コード補完、定義へ移動など）といった、たくさんの情報が含まれています。

## `lsp-types` クレートの活用

LSPのメッセージは非常に複雑で、たくさんのフィールドを持っています。これらを毎回手動でJSONからRustの `Value` にパースし、必要なフィールドを取り出すのは大変です。

そこで登場するのが `lsp-types` クレートです。このクレートには、LSPのすべてのメッセージやデータ構造が、Rustの `struct` として定義されています。`serde` と組み合わせることで、JSONを直接これらの便利な `struct` に変換できます。

例えば、`initialize` リクエストのパラメータは `lsp_types::InitializeParams` という `struct` に対応しています。

```rust
use lsp_types::InitializeParams;
use serde_json::Value;

// json_value は、すでにパースされたJSONのValue型だとします
let result: Result<InitializeParams, serde_json::Error> = serde_json::from_value(json_value);

match result {
    Ok(params) => { /* InitializeParamsのstructが手に入った！ */ },
    Err(e) => { /* パース失敗 */ },
}
```

## やってみよう！

あなたの今回のミッションは、`parse_initialize_request` 関数を完成させることです。

これまでのレッスンで学んだ関数を組み合わせて、以下のステップで処理を進めてください。

1.  `super::lesson_1_2::parse_lsp_message` を使って、入力された `full_message` をヘッダーとコンテンツに分割します。
2.  `super::lesson_1_3::get_content_length` を使って、ヘッダーから `Content-Length` を取得します。
3.  `super::lesson_1_4::parse_json_content` を使って、コンテンツを `serde_json::Value` にパースします。
4.  最後に、`serde_json::from_value` を使って、`serde_json::Value` を `lsp_types::InitializeParams` に変換します。

途中のどのステップでも失敗する可能性があるため、それぞれの関数の戻り値が `Option` や `Result` であることを考慮し、適切にエラーハンドリング（`?` 演算子や `if let`、`match` など）を行ってください。

すべてが成功したら `Some(InitializeParams)` を、途中で失敗したら `None` を返してください。

`src/lessons/lesson_1_5.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！
