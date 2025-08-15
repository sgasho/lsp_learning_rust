# Lesson 1-12: サーバーの始まりと終わりを理解しよう！ (LSPライフサイクル)

LSPメッセージのパースと生成ができるようになりましたね。素晴らしいです！

しかし、言語サーバーは単にメッセージを処理するだけでなく、エディタとの間で「**ライフサイクル**」と呼ばれる一連の決まった手順を踏みます。これは、サーバーがいつ起動し、いつ初期化され、いつシャットダウンされるのかをエディタと合意するためのものです。

## LSPライフサイクルの主要なイベント

1.  **`initialize` リクエスト**: エディタが言語サーバーを起動すると、まず最初にこのリクエストを送ります。これは「こんにちは！私はこんなことができますが、あなたは何ができますか？」という自己紹介と能力の交換です。サーバーはこれに対して `InitializeResult` を返します。

2.  **`initialized` 通知**: サーバーが `initialize` リクエストに応答した後、エディタはサーバーが完全に準備できたことを示すためにこの通知を送ります。サーバーはこれに対して何も返しません。

3.  **`shutdown` リクエスト**: エディタが言語サーバーを終了する準備ができたときに送られます。サーバーはこれに対して `null` の結果を返します。この時点ではまだサーバーは終了せず、リソースのクリーンアップなどを行う準備をします。

4.  **`exit` 通知**: `shutdown` リクエストに応答した後、エディタはサーバーに終了を指示するためにこの通知を送ります。サーバーはこれに対して何も返さず、プロセスを終了します。

## サーバーの状態管理

言語サーバーは、これらのライフサイクルイベントに応じて内部の状態を管理する必要があります。例えば、`initialize` が完了するまでは、他のほとんどのリクエストを処理すべきではありません。今回のレッスンでは、`initialized_state` というシンプルな真偽値を使って、サーバーが初期化済みかどうかを管理します。

## `LspMessage` の種類を判別しよう！

`handle_lsp_lifecycle` 関数に渡される `message` は、Lesson 1-9 で作った `LspMessage` Enumの形をしていますね。このEnumには、`Request` と `Notification` の2つの種類がありました。

Rustでは、`match` 式を使うと、Enumのどの種類（バリアント）なのかを簡単に判別し、それぞれの種類に応じた処理を書くことができます。まるで、届いたお手紙が「質問」なのか「報告」なのかで、対応を変えるようなものです。

```rust
match message {
    LspMessage::Request { id, method, params } => {
        // これは質問（リクエスト）です！
        // id, method, params の中身を使って、さらに詳しく判別できます。
    },
    LspMessage::Notification { method, params } => {
        // これは報告（通知）です！
        // method, params の中身を使って、さらに詳しく判別できます。
    },
}
```

### 各ライフサイクルイベントの識別方法

`match` 式で `LspMessage` の種類を判別した後、それぞれの `method` フィールドの値を見て、どのライフサイクルイベントなのかを判断します。

*   **`initialize` リクエスト**: `LspMessage::Request` で、かつ `method` が `"initialize"` の場合。
    ```rust
    LspMessage::Request { id, method, params } => {
        if method == "initialize" {
            // initialize リクエストの処理
        }
        // ... その他のリクエストの処理
    }
    ```

*   **`initialized` 通知**: `LspMessage::Notification` で、かつ `method` が `"initialized"` の場合。
    ```rust
    LspMessage::Notification { method, params } => {
        if method == "initialized" {
            // initialized 通知の処理
        }
        // ... その他の通知の処理
    }
    ```

*   **`shutdown` リクエスト**: `LspMessage::Request` で、かつ `method` が `"shutdown"` の場合。
    ```rust
    LspMessage::Request { id, method, params } => {
        // ... initialize の後に
        if method == "shutdown" {
            // shutdown リクエストの処理
        }
        // ... その他のリクエストの処理
    }
    ```

*   **`exit` 通知**: `LspMessage::Notification` で、かつ `method` が `"exit"` の場合。
    ```rust
    LspMessage::Notification { method, params } => {
        // ... initialized の後に
        if method == "exit" {
            // exit 通知の処理
        }
        // ... その他の通知の処理
    }
    ```

## やってみよう！

あなたの今回のミッションは、`handle_lsp_lifecycle` 関数を完成させることです。

この関数は `LspMessage` と、サーバーの初期化状態を表す `initialized_state` （ミュータブルな真偽値）を受け取ります。各LSPメッセージの種類に応じて、以下の処理を行ってください。

*   **`initialize` リクエスト**: `create_lsp_response` を使って `InitializeResult::default()` を結果とする成功応答を生成し、`initialized_state` を `true` に設定して、その応答文字列を `Some(String)` で返します。
*   **`initialized` 通知**: 何もせず `None` を返します。`initialized_state` は変更しません。
*   **`shutdown` リクエスト**: `create_lsp_response` を使って `null` を結果とする成功応答を生成し、その応答文字列を `Some(String)` で返します。`initialized_state` は変更しません。
*   **`exit` 通知**: `initialized_state` を `false` に設定し、`None` を返します。
*   **その他のメッセージ**: 何もせず `None` を返します。

`src/lessons/lesson_1_12.rs` を開いて、挑戦しましょう。

`cargo test` でテストがすべて緑色になったらクリアです！