//! # Issue #20264 学習モジュール
//! 
//! マクロ内での `&field` 補完欠落問題を解決するための
//! 学習リソースを提供します。
//!
//! ## 学習順序（推奨）
//! 
//! 1. [`macro_basics`] - Rustマクロの基礎概念
//! 2. [`macro_expansion`] - rust-analyzerでのマクロ展開処理
//! 3. [`completion_context`] - 補完エンジンの文脈解析
//! 4. [`token_tree_analysis`] - TokenTreeの構造と解析
//!
//! ## 各モジュールの内容
//!
//! ### [`macro_basics`]
//! - 宣言的マクロと手続き的マクロの違い
//! - TokenTreeの構造理解
//! - マクロ展開の基本メカニズム
//! - `dbg!` マクロの詳細分析
//!
//! ### [`macro_expansion`] 
//! - rust-analyzerでのマクロ展開フロー
//! - `hir-expand` crateの役割
//! - SpanMappingとトークン位置の対応
//! - MacroCallからExpandResultまでの処理
//!
//! ### [`completion_context`]
//! - CompletionContextの構造と役割
//! - 期待型推論のメカニズム
//! - SyntaxNodeとSemanticModelの連携
//! - マクロ内補完での課題
//!
//! ### [`token_tree_analysis`]
//! - TokenTreeの走査アルゴリズム
//! - 位置ベースのトークン検索
//! - マクロ引数の解析方法
//! - カーソル位置と対応するトークンの特定

pub mod macro_basics;
pub mod macro_expansion;
pub mod completion_context;
pub mod token_tree_analysis;

/// Issue #20264 の核心問題を実演するデモ関数
/// 
/// この関数を使って実際の問題を体験し、
/// 解決前後の挙動を比較できます。
pub fn demonstrate_issue() {
    println!("🔮 Issue #20264 デモンストレーション");
    println!();
    
    struct NamedField {
        out: Vec<u8>,
    }
    
    let s = NamedField { out: Vec::new() };
    
    println!("✅ 通常のコード（期待通りに動作）:");
    println!("   str::from_utf8(s.); // <- &out が補完候補に表示される");
    
    println!();
    println!("❌ マクロ内のコード（問題が発生）:");
    println!("   dbg!(s.); // <- &out が補完候補に表示されない");
    
    println!();
    println!("🎯 目標: マクロ内でも &out が補完候補に表示されるようにする");
}

/// 学習の進捗を追跡するための構造体
#[derive(Debug, Default)]
pub struct LearningProgress {
    pub macro_basics_completed: bool,
    pub macro_expansion_completed: bool, 
    pub completion_context_completed: bool,
    pub token_tree_analysis_completed: bool,
}

impl LearningProgress {
    /// 新しい学習進捗を作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 完了率を計算
    pub fn completion_rate(&self) -> f32 {
        let completed = [
            self.macro_basics_completed,
            self.macro_expansion_completed,
            self.completion_context_completed,
            self.token_tree_analysis_completed,
        ].iter().filter(|&&x| x).count();
        
        completed as f32 / 4.0
    }
    
    /// 次に学習すべきモジュールを推奨
    pub fn next_module(&self) -> Option<&'static str> {
        if !self.macro_basics_completed {
            Some("macro_basics")
        } else if !self.macro_expansion_completed {
            Some("macro_expansion")
        } else if !self.completion_context_completed {
            Some("completion_context")
        } else if !self.token_tree_analysis_completed {
            Some("token_tree_analysis")
        } else {
            None // すべて完了
        }
    }
    
    /// 学習状況を表示
    pub fn display_status(&self) {
        println!("📊 学習進捗状況:");
        println!("   🧩 macro_basics: {}", if self.macro_basics_completed { "✅" } else { "⏳" });
        println!("   🔄 macro_expansion: {}", if self.macro_expansion_completed { "✅" } else { "⏳" });
        println!("   💭 completion_context: {}", if self.completion_context_completed { "✅" } else { "⏳" });
        println!("   🌳 token_tree_analysis: {}", if self.token_tree_analysis_completed { "✅" } else { "⏳" });
        println!();
        println!("   📈 完了率: {:.1}%", self.completion_rate() * 100.0);
        
        if let Some(next) = self.next_module() {
            println!("   🎯 次のステップ: {}", next);
        } else {
            println!("   🎉 すべての学習が完了しました！");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_learning_progress() {
        let mut progress = LearningProgress::new();
        assert_eq!(progress.completion_rate(), 0.0);
        assert_eq!(progress.next_module(), Some("macro_basics"));
        
        progress.macro_basics_completed = true;
        assert_eq!(progress.completion_rate(), 0.25);
        assert_eq!(progress.next_module(), Some("macro_expansion"));
        
        progress.macro_expansion_completed = true;
        progress.completion_context_completed = true;
        progress.token_tree_analysis_completed = true;
        assert_eq!(progress.completion_rate(), 1.0);
        assert_eq!(progress.next_module(), None);
    }
    
    #[test]
    fn test_demonstrate_issue() {
        // パニックしないことを確認
        demonstrate_issue();
    }
}