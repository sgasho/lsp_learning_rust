
# Lesson 1-18: ファイルを閉じたことを知ろう！ (textDocument/didClose)

ファイルが開かれたとき (`didOpen`) や変更されたとき (`didChange`) の通知を処理できるようになりましたね。素晴らしいです！

言語サーバーが管理しているファイルは、エディタで閉じられることもあります。不要になったファイルの内容をいつまでもドキュメントストアに保存しておくのは、メモリの無駄遣いです。そこで、エディタがファイルを閉じたときに送られてくる `textDocument/didClose` 通知を処理し、ドキュメントストアから該当するファイルの内容を削除する必要があります。

## `textDocument/didClose` 通知とは？

ユーザーがエディタでファイルを閉じたときに、言語サーバーに送られる通知です。この通知には、閉じられたファイルのURI（場所）が含まれています。

サーバーはこれを受け取ると、自分の「ドキュメントストア」から該当するファイルの内容を削除します。これにより、メモリを解放し、リソースを効率的に管理できます。

## `params` の構造

`textDocument/didClose` 通知の `params` フィールドは、以下のような構造を持つオブジェクトです。

```json
{
  "textDocument": {
    "uri": "file:///path/to/file.rs"
  }
}
```

ここから `uri` の値を取り出す必要があります。

## Rustでの `HashMap` からの削除

`HashMap` からキーに対応するエントリを削除するには、`remove()` メソッドを使います。`remove()` は、削除された値（`String`）を `Option<String>` として返します。もしキーが存在しなければ `None` を返します。

```rust
use std::collections::HashMap;
use lsp_types::Url;

let mut document_store: HashMap<Url, String> = HashMap::new();
let uri = Url::parse("file:///a.rs").unwrap();

document_store.insert(uri.clone(), "内容".to_string());

// HashMapから削除
document_store.remove(&uri);

// 削除されたか確認
if !document_store.contains_key(&uri) {
    println!("Document removed!");
}
```

## やってみよう！

あなたの今回のミッションは、`handle_did_close_notification` 関数を完成させることです。

1.  `params_value` から `textDocument` オブジェクトを取得します。
2.  `textDocument` オブジェクトから `uri` を取得し、`Url` 型にパースします。
3.  パースした `Url` をキーとして、`document_store` から該当するドキュメントを削除します。
4.  もし `params_value` のパースに失敗したり、必要なフィールドが欠けていたり、URIが無効だったりした場合は、何もせずに関数を終了します。

`src/lessons/lesson_1_18.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！
