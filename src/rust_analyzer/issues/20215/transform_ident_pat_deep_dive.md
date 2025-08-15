# transform_ident_pat メソッド完全解説

## 🎯 メソッドの全体像

```rust
fn transform_ident_pat(
    &self,
    editor: &mut SyntaxEditor,
    ident_pat: &ast::IdentPat,
) -> Option<()> {
    let name = ident_pat.name()?;
    let temp_path = make::path_from_text(&name.text());
    let resolution = self.source_scope.speculative_resolve(&temp_path)?;
    
    match resolution {
        hir::PathResolution::Def(def) if def.as_assoc_item(...).is_none() => {
            let cfg = ImportPathConfig { /* ... */ };
            let found_path = self.target_module.find_path(..., def, cfg)?;
            let res = mod_path_to_ast(&found_path, ...).clone_for_update();
            editor.replace(ident_pat.syntax(), res.syntax());
            Some(())
        }
        _ => None,
    }
}
```

## 🔄 処理フローの詳細

### 全体処理フロー
```mermaid
sequenceDiagram
    participant M as メソッド呼び出し
    participant AST as syntax クレート
    participant HIR as hir クレート
    participant DB as データベース
    participant Edit as SyntaxEditor
    
    M->>AST: ident_pat.name()
    AST-->>M: ast::Name("Unbounded")
    
    M->>AST: make::path_from_text("Unbounded")
    AST-->>M: ast::Path("Unbounded")
    
    M->>HIR: source_scope.speculative_resolve(path)
    HIR->>DB: クエリ実行
    DB-->>HIR: 名前解決結果
    HIR-->>M: PathResolution::Def(variant)
    
    M->>HIR: target_module.find_path(def, config)
    HIR->>DB: パス検索クエリ
    DB-->>HIR: 最適パス
    HIR-->>M: ModPath(["std", "ops", "Bound", "Unbounded"])
    
    M->>AST: mod_path_to_ast(mod_path)
    AST-->>M: ast::Path("std::ops::Bound::Unbounded")
    
    M->>Edit: editor.replace(old, new)
    Edit-->>M: ()
```

---

## 📝 ステップ1: 名前抽出 `ident_pat.name()`

### モジュール依存関係
```mermaid
graph TD
    A["transform_ident_pat<br/>(ide-db/path_transform.rs)"] --> B["ast::IdentPat::name()<br/>(syntax/ast.rs)"]
    B --> C["HasName トレイト<br/>(syntax/ast/traits.rs)"]
    C --> D["SyntaxNode::children_with_tokens()<br/>(rowan)"]
    
    style A fill:#1976d2,color:#fff
    style B fill:#ff9800,color:#fff
    style C fill:#388e3c,color:#fff
    style D fill:#9c27b0,color:#fff
```

### 実際の処理詳細
```rust
// syntax/ast/traits.rs
impl HasName for ast::IdentPat {
    fn name(&self) -> Option<ast::Name> {
        self.syntax()                    // SyntaxNode を取得
            .children_with_tokens()       // 子トークンを走査
            .find(|it| it.kind() == SyntaxKind::NAME)  // NAME トークンを検索
            .and_then(|it| ast::Name::cast(it.as_node()?.clone()))  // ast::Name に変換
    }
}
```

### データ変換の流れ
```mermaid
graph LR
    A["SyntaxNode<br/>(IDENT_PAT)"] --> B["SyntaxToken<br/>(NAME)"] 
    B --> C["ast::Name<br/>wrapper"] --> D["text(): 'Unbounded'"]
    
    style A fill:#e3f2fd,color:#000
    style B fill:#fff3e0,color:#000
    style C fill:#e8f5e8,color:#000
    style D fill:#ffebee,color:#000
```

---

## 🔧 ステップ2: 一時パス作成 `make::path_from_text()`

### モジュール依存関係
```mermaid
graph TD
    A["transform_ident_pat"] --> B["make::path_from_text<br/>(syntax/ast/make.rs)"]
    B --> C["ast_from_text<br/>(内部関数)"]
    C --> D["SourceFile::parse<br/>(parser クレート)"]
    D --> E["rowan パーサー"]
    
    style A fill:#1976d2,color:#fff
    style B fill:#ff9800,color:#fff
    style C fill:#388e3c,color:#fff
    style D fill:#9c27b0,color:#fff
    style E fill:#607d8b,color:#fff
```

### 内部処理の詳細
```rust
// syntax/ast/make.rs
pub fn path_from_text(text: &str) -> ast::Path {
    ast_from_text(&format!("use {};", text))  // "use Unbounded;" を作成
}

fn ast_from_text<N: AstNode>(text: &str) -> N {
    let parse = SourceFile::parse(text, Edition::CURRENT);  // パース実行
    let file = parse.tree();                                // AST取得
    find_node_at_offset(file.syntax(), TextSize::of("use "))  // useの後を検索
        .unwrap()
}
```

### 変換プロセス
```mermaid
graph LR
    A["'Unbounded'<br/>(文字列)"] --> B["'use Unbounded;'<br/>(Rust コード)"]
    B --> C["パーサー実行"] --> D["ast::Path<br/>(構文ツリー)"]
    
    style A fill:#e3f2fd,color:#000
    style B fill:#fff3e0,color:#000
    style C fill:#e8f5e8,color:#000
    style D fill:#ffebee,color:#000
```

---

## 🧠 ステップ3: 名前解決 `speculative_resolve()`

### モジュール・システム間の依存関係
```mermaid
graph TD
    A["transform_ident_pat<br/>(ide-db)"] --> B["SemanticsScope::speculative_resolve<br/>(hir/semantics.rs)"]
    B --> C["Resolver システム<br/>(hir-def/resolver.rs)"]
    C --> D["DefMap<br/>(hir-def/nameres/mod.rs)"]
    D --> E["Database クエリ<br/>(salsa)"]
    
    subgraph "HIR システム"
        B
        C
        D
    end
    
    subgraph "データベース層"
        E
        F["ModuleDef<br/>(hir/lib.rs)"]
        G["PathResolution<br/>(hir/lib.rs)"]
    end
    
    E --> F --> G
    
    style A fill:#1976d2,color:#fff
    style B fill:#ff9800,color:#fff
    style C fill:#388e3c,color:#fff
    style D fill:#9c27b0,color:#fff
    style E fill:#607d8b,color:#fff
```

### 名前解決の詳細プロセス
```rust
// hir/semantics.rs
impl SemanticsScope<'_> {
    pub fn speculative_resolve(&self, path: &ast::Path) -> Option<PathResolution> {
        let resolver = self.resolver()?;
        let resolved = resolver.resolve_path_in_value_ns(path.clone())?;
        Some(PathResolution::from(resolved))
    }
}
```

### 解決ステップの詳細
```mermaid
sequenceDiagram
    participant Scope as SemanticsScope
    participant Resolver as Resolver
    participant DefMap as DefMap
    participant DB as Database
    
    Note over Scope: "Unbounded" を解決
    
    Scope->>Resolver: resolve_path_in_value_ns("Unbounded")
    Resolver->>DefMap: lookup_path_in_scope("Unbounded")
    DefMap->>DB: クエリ: 現在のスコープ
    DB-->>DefMap: スコープ情報
    
    DefMap->>DB: クエリ: use 文の解析
    DB-->>DefMap: インポート情報
    
    DefMap->>DB: クエリ: prelude の確認
    DB-->>DefMap: std::ops::Bound::Unbounded を発見
    
    DefMap-->>Resolver: ModuleDef::Variant(Unbounded)
    Resolver-->>Scope: PathResolution::Def(variant)
```

---

## 🎯 ステップ4: 解決結果の判定

### パターンマッチの詳細
```rust
match resolution {
    // ✅ 通常の定義かつ非関連アイテム
    hir::PathResolution::Def(def) if def.as_assoc_item(self.source_scope.db).is_none() => {
        // パス変換処理へ
    }
    
    // ❌ その他のケース
    hir::PathResolution::Def(def) => None,         // 関連アイテム
    hir::PathResolution::Local(_) => None,         // ローカル変数
    hir::PathResolution::TypeParam(_) => None,     // 型パラメータ
    _ => None,                                     // その他
}
```

### 判定フローチャート
```mermaid
flowchart TD
    A["PathResolution"] --> B{解決結果の種類}
    
    B -->|Def| C{関連アイテム?}
    B -->|Local| D["❌ ローカル変数<br/>変換対象外"]
    B -->|TypeParam| E["❌ 型パラメータ<br/>変換対象外"]
    B -->|その他| F["❌ その他<br/>変換対象外"]
    
    C -->|Yes| G["❌ 関連アイテム<br/>変換対象外"]
    C -->|No| H["✅ 通常の定義<br/>変換実行"]
    
    style H fill:#388e3c,color:#fff
    style D fill:#d32f2f,color:#fff
    style E fill:#d32f2f,color:#fff
    style F fill:#d32f2f,color:#fff
    style G fill:#d32f2f,color:#fff
```

---

## 🗺 ステップ5: パス検索 `find_path()`

### モジュール依存関係
```mermaid
graph TD
    A["transform_ident_pat"] --> B["Module::find_path<br/>(hir/lib.rs)"]
    B --> C["find_path 内部ロジック<br/>(hir-def/find_path.rs)"]
    C --> D["ImportMap<br/>(ide-db/imports/import_map.rs)"]
    D --> E["DefDatabase<br/>(hir-def/db.rs)"]
    
    subgraph "設定システム"
        F["ImportPathConfig<br/>(ide-db/imports/mod.rs)"]
    end
    
    B --> F
    F --> C
    
    style A fill:#1976d2,color:#fff
    style B fill:#ff9800,color:#fff
    style C fill:#388e3c,color:#fff
    style D fill:#9c27b0,color:#fff
    style E fill:#607d8b,color:#fff
    style F fill:#795548,color:#fff
```

### ImportPathConfig の詳細
```rust
let cfg = ImportPathConfig {
    prefer_no_std: false,      // std の代わりに core を使うか
    prefer_prelude: true,      // prelude 項目は短縮するか
    prefer_absolute: false,    // 絶対パス vs 相対パス
    allow_unstable: true,      // unstable 機能を許可するか
};
```

### パス検索アルゴリズム
```mermaid
flowchart TD
    A["find_path 開始"] --> B{prelude チェック}
    
    B -->|prelude に含まれる| C["短縮パスを返却<br/>例: Option"]
    B -->|含まれない| D{可視性チェック}
    
    D -->|非公開| E["❌ None を返却"]
    D -->|公開| F["パス探索実行"]
    
    F --> G{相対パス可能?}
    G -->|Yes| H["相対パス生成<br/>例: super::Foo"]
    G -->|No| I["絶対パス生成<br/>例: std::ops::Bound::Unbounded"]
    
    style C fill:#388e3c,color:#fff
    style H fill:#4caf50,color:#fff
    style I fill:#2196f3,color:#fff
    style E fill:#d32f2f,color:#fff
```

---

## 🏗 ステップ6: AST変換 `mod_path_to_ast()`

### データ変換の流れ
```mermaid
sequenceDiagram
    participant Method as transform_ident_pat
    participant ModPath as ModPath
    participant Make as make モジュール
    participant AST as ast::Path
    
    Method->>ModPath: ModPath { segments: ["std", "ops", "Bound", "Unbounded"] }
    ModPath->>Make: mod_path_to_ast(mod_path, edition)
    
    Make->>Make: segments.iter().map(make::path_segment)
    Make->>Make: make::path_from_segments(segments)
    Make->>AST: ast::Path 構築
    AST->>AST: clone_for_update() 実行
    AST-->>Method: 編集可能な ast::Path
```

### AST構築の詳細
```rust
// syntax/ast/make.rs
pub fn mod_path_to_ast(path: &ModPath, edition: Edition) -> ast::Path {
    let segments = path.segments.iter().map(|name| {
        let name_ref = make::name_ref(&name.display(edition).to_string());
        make::path_segment(name_ref)
    });
    
    make::path_from_segments(segments, path.kind == PathKind::Plain)
}
```

---

## ✏️ ステップ7: ノード置換 `editor.replace()`

### SyntaxEditor の動作
```mermaid
graph TD
    A["SyntaxEditor<br/>(syntax/ted.rs)"] --> B["replace(old, new)"]
    B --> C["Edit 記録"]
    C --> D["finish() で適用"]
    
    subgraph "編集操作"
        E["IdentPat ノード"] --> F["Path ノード"]
        G["'Unbounded'"] --> H["'std::ops::Bound::Unbounded'"]
    end
    
    B --> E
    F --> D
    G --> H
    
    style A fill:#1976d2,color:#fff
    style C fill:#ff9800,color:#fff
    style D fill:#388e3c,color:#fff
    style F fill:#4caf50,color:#fff
    style H fill:#2196f3,color:#fff
```

### 型安全な置換
```rust
// 置換前: ast::IdentPat
let old_node: &SyntaxNode = ident_pat.syntax();  // IDENT_PAT ノード

// 置換後: ast::Path  
let new_node: SyntaxNode = res.syntax().clone(); // PATH ノード

// SyntaxEditor による安全な置換
editor.replace(old_node, new_node);  // 型は実行時にチェック
```

---

## 📊 全体的なモジュール依存マップ

```mermaid
graph TB
    subgraph "エントリポイント"
        A["transform_ident_pat<br/>(ide-db/path_transform.rs)"]
    end
    
    subgraph "syntax クレート"
        B["ast::IdentPat::name()"]
        C["make::path_from_text()"]
        D["mod_path_to_ast()"]
        E["SyntaxEditor::replace()"]
    end
    
    subgraph "hir クレート"
        F["SemanticsScope::speculative_resolve()"]
        G["Module::find_path()"]
    end
    
    subgraph "hir-def クレート"  
        H["Resolver システム"]
        I["DefMap"]
        J["find_path 内部ロジック"]
    end
    
    subgraph "rowan (基盤)"
        K["SyntaxNode"]
        L["パーサー"]
    end
    
    subgraph "salsa (クエリシステム)"
        M["Database"]
        N["クエリ実行"]
    end
    
    A --> B --> K
    A --> C --> L
    A --> F --> H --> I --> M
    A --> G --> J --> N
    A --> D --> K
    A --> E --> K
    
    style A fill:#1976d2,color:#fff
    style B fill:#ff9800,color:#fff
    style F fill:#388e3c,color:#fff
    style H fill:#9c27b0,color:#fff
    style M fill:#607d8b,color:#fff
```

---

## 🎯 まとめ

### 処理の要点
1. **AST操作** (syntax): 構文レベルでの名前抽出・変換・置換
2. **セマンティック解析** (hir): 意味的な名前解決とパス検索  
3. **データベース** (salsa): 効率的なクエリ実行とキャッシュ
4. **エディター** (ted): 安全なAST変更操作

### 設計の優秀さ
- **レイヤー分離**: 各クレートが明確な責任を持つ
- **型安全性**: コンパイル時・実行時の両方で安全性を確保
- **拡張性**: 新しいノード型への対応が容易
- **効率性**: クエリシステムによる最適化されたパフォーマンス

この`transform_ident_pat`メソッドは、rust-analyzerの設計哲学を体現した優れた実装例です。