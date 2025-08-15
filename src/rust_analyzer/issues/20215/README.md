# Issue #20215: パターンマッチで名前が正しく変換されないバグ

## 何が起きているの？

rust-analyzerの**「Implement default member」**という便利機能を使うと、生成されるコードにバグがあります。

### 「Implement default member」って何？

トレイトを実装するとき、「このトレイトにはデフォルト実装があるメソッドがありますよ！追加しますか？」と提案してくれる機能です。

```rust
// こんなコードで...
impl RangeBounds<usize> for MyRange {
    fn start_bound(&self) -> Bound<&usize> { todo!() }
    fn end_bound(&self) -> Bound<&usize> { todo!() }
    // ← ここでrust-analyzerが「is_emptyメソッドも追加できるよ！」と提案
}
```

## 何がバグなの？

**問題**: 生成されたコードがコンパイルエラーになる

### 実際に試してみよう
1. 以下のコードをVSCodeに書く：
```rust
struct MyRange;

impl core::ops::RangeBounds<usize> for MyRange {
    fn start_bound(&self) -> core::ops::Bound<&usize> {
        todo!()
    }

    fn end_bound(&self) -> core::ops::Bound<&usize> {
        todo!()
    }
}
```

2. `impl`の上にマウスを置いて、「Implement default member」をクリック

3. すると、こんなコードが生成される：
```rust
fn is_empty(&self) -> bool {
    !match (self.start_bound(), self.end_bound()) {
        (Unbounded, _) | (_, Unbounded) => true,  // ← エラー！Unboundedって何？
        // ...
    }
}
```

4. **コンパイルエラー**：`cannot find value 'Unbounded' in this scope`

### 正しくはこうあるべき
```rust
fn is_empty(&self) -> bool {
    !match (self.start_bound(), self.end_bound()) {
        (std::ops::Bound::Unbounded, _) | (_, std::ops::Bound::Unbounded) => true,  // ← 正しい！
        // ...
    }
}
```

## なぜこのバグが起きるの？

### 簡単な説明
rust-analyzerがコードを生成するとき、「名前を正しいパス（フルパス）に変換する」処理をします。

- `Included` → `std::ops::Bound::Included` ✅ (これはうまくいく)
- `Unbounded` → `std::ops::Bound::Unbounded` ❌ (これが失敗する)

### なぜ `Unbounded` だけ失敗するの？

技術的な理由：
- `Included(value)` は「パス + 引数」の形なので、rust-analyzerが「あ、これはパスだ！」と認識できる
- `Unbounded` は単体の名前なので、「パス」として認識されない
- rust-analyzerの`PathTransform`という部分が、単体の名前を見落としている

## どこを直せばいいの？

**修正対象のファイル**: `crates/ide-db/src/path_transform.rs` (310行目あたり)

**やること**: 
1. 現在は「パス」だけを変換している
2. 「パターン内の単体の名前」も変換するように拡張する

## 初心者でも取り組める理由

1. **明確な問題**: 何が起きているかはっきりしている
2. **再現が簡単**: すぐに問題を確認できる
3. **修正箇所が特定済み**: コメントで具体的な場所が指摘されている
4. **テストが簡単**: 修正後すぐに動作確認できる

## 次にやること

1. `crates/ide-db/src/path_transform.rs` を読んでみる
2. 現在のコードがどういう処理をしているか理解する
3. 「パターン内の単体名前」も処理するよう拡張する

これは rust-analyzer の貢献を始めるのにちょうど良い問題です！