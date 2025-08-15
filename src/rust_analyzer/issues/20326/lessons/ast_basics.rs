// 📚 AST (Abstract Syntax Tree) の基本を学ぼう
// 
// このファイルでは、rust-analyzer でよく使う AST の概念を
// 実際のコード例で学びます

use std::collections::HashMap;

/// 🌳 AST とは？
/// 
/// AST (Abstract Syntax Tree) は、コードの構造を木構造で表現したものです。
/// rust-analyzer はこの AST を使ってコードを理解し、操作します。
/// 
/// 例: `use std::collections::HashMap;` は以下のような構造になります:
/// 
/// ```
/// Use
/// ├── UseTree
/// │   └── Path
/// │       ├── Path("std")
/// │       ├── Path("collections") 
/// │       └── Name("HashMap")
/// └── Semicolon
/// ```

fn demonstrate_ast_nodes() {
    // 🎯 これらのコード要素は、それぞれ異なる AST ノードになります
    
    // ast::Use ノード
    use std::fs::File;
    
    // ast::Fn ノード (この関数自体)
    // ast::Block ノード (この {} ブロック)
    let map = HashMap::new(); // ast::LetStmt ノード
    
    // ast::IfExpr ノード
    if true {
        // ast::Block ノード (nested)
        println!("Hello");
    }
    
    // ast::Module ノード (mod キーワード)
    mod inner {
        // ast::Use ノード (モジュール内)
        use std::thread;
        
        // ast::Fn ノード (ネストした関数)
        fn nested_function() {
            // ast::Use ノード (関数内) ← これを移動したい！
            use std::sync::Arc;
            let arc = Arc::new(42);
        }
    }
}

/// 🔍 rust-analyzer での AST 操作例
/// 
/// 実際の rust-analyzer のコードで使われるパターンを学びましょう
mod ast_operations_examples {
    // これらは実際の rust-analyzer コードの簡略版です
    
    /// カーソル位置の AST ノードを取得
    fn find_node_example() {
        // 実際のコード:
        // let use_item = ctx.find_node_at_offset::<ast::Use>()?;
        // 
        // これは「カーソル位置にある Use ノードを見つける」という意味
    }
    
    /// 親ノードを探す
    fn find_parent_example() {
        // 実際のコード:
        // let parent_fn = use_item.syntax().ancestors()
        //     .find_map(ast::Fn::cast)?;
        // 
        // これは「Use ノードの祖先を辿って、Function ノードを見つける」という意味
    }
    
    /// AST ノードの操作
    fn modify_ast_example() {
        // 実際のコード:
        // builder.delete(use_item.syntax().text_range());
        // builder.insert(offset, new_text);
        // 
        // これは「Use ノードを削除して、別の場所に挿入する」という意味
    }
}

/// 🎯 Issue #20326 で重要な AST ノード
/// 
/// この issue で主に扱う AST ノードの種類
#[allow(dead_code)]
mod relevant_ast_nodes {
    
    /// ast::Use - use 文のノード
    /// 例: `use std::collections::HashMap;`
    fn example_use_node() {
        use std::collections::HashMap; // ← これが ast::Use
    }
    
    /// ast::Fn - 関数のノード  
    /// 例: `fn example() { ... }`
    fn example_fn_node() { // ← これが ast::Fn
        // この中の use 文を外に出したい
        use std::fs::File;
    }
    
    /// ast::Module - モジュールのノード
    /// 例: `mod inner { ... }`
    mod example_module_node { // ← これが ast::Module
        use std::thread; // モジュール内の use 文
    }
    
    /// ast::SourceFile - ファイル全体のノード
    /// ファイル全体を表現する最上位ノード
}

/// 📍 カーソル位置とAST の関係
/// 
/// エディタでカーソルがある位置と、AST ノードの関係を理解しよう
fn cursor_and_ast_relationship() {
    use std::collections::HashMap;
    //  ^                    ^
    //  カーソルがここ         カーソルがここ
    //  
    // どちらも同じ ast::Use ノードを指している
    // rust-analyzer はカーソル位置から最も近い AST ノードを見つける
    
    let map = HashMap::new();
    //        ^
    //        カーソルがここ = ast::PathExpr ノード
}

/// 🧪 実際の検出ロジックの疑似コード
/// 
/// Issue #20326 で実装する検出ロジックのイメージ
#[allow(dead_code)]
fn detection_logic_pseudocode() {
    // 1. カーソル位置の Use ノードを取得
    // let use_item = ctx.find_node_at_offset::<ast::Use>()?;
    
    // 2. その Use ノードが関数内にあるかチェック
    // let parent_fn = use_item.syntax().ancestors()
    //     .find_map(ast::Fn::cast)?;
    
    // 3. トップレベルでないことを確認
    // if parent_fn.is_some() {
    //     // 移動可能！
    // }
    
    // この関数内の use 文を例にすると...
    use std::thread; // ← parent_fn.is_some() == true (移動可能)
}

// 🏠 ファイルレベル (トップレベル) の use 文
use std::sync::Arc; // ← parent_fn.is_none() (移動不要)

/// 🎨 実装のイメージ図
/// 
/// ```
/// Before:                    After:
/// ┌─────────────────────┐   ┌─────────────────────┐
/// │ use std::sync::Arc; │   │ use std::sync::Arc; │
/// │                     │   │ use std::fs::File;  │ ← 移動！
/// │ fn main() {         │   │                     │
/// │   use std::fs::File │   │ fn main() {         │
/// │   // code...        │   │   // code...        │
/// │ }                   │   │ }                   │
/// └─────────────────────┘   └─────────────────────┘
/// ```

#[cfg(test)]
mod learning_tests {
    /// 🧪 学習用テスト: AST の理解を確認
    /// 
    /// 実際にコードを書いて AST の概念を確認してみよう
    #[test]
    fn understand_ast_structure() {
        // この関数内にある use 文は...
        use std::collections::BTreeMap;
        
        // AST 的には以下のような構造:
        // Fn("understand_ast_structure")
        // └── Block
        //     ├── Use("std::collections::BTreeMap") ← これを移動したい
        //     └── ... (他のステートメント)
        
        let _map = BTreeMap::new();
        
        // テストとしては、「この use 文が関数内にある」ことを検証したい
        assert!(true, "This use statement is inside a function");
    }
}