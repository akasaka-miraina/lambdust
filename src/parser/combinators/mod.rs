//! Lambdust内製パーサーコンビネータシステム
//!
//! Scheme言語処理に最適化された高性能パーサーコンビネータライブラリ。
//! nomパーサーコンビネータと互換性を保ちながら、S式処理とゼロコピー解析を最適化。
//!
//! # アーキテクチャ概要
//!
//! ```text
//! Core Layer (primitive.rs)
//! ├── 基本パーサープリミティブ
//! ├── ゼロコピー文字列操作
//! ├── エラーハンドリング
//! └── Span情報管理
//! 
//! Combinator Layer (combinator.rs)
//! ├── 基本コンビネータ
//! ├── シーケンス操作
//! ├── 選択・分岐
//! └── 繰り返し処理
//! 
//! Scheme-Specific Layer (scheme.rs)
//! ├── S式パーサー
//! ├── リスト処理
//! ├── アトム解析
//! └── 特殊構文
//! 
//! Integration Layer (nom_compat.rs)
//! ├── nom API互換
//! ├── 移行支援
//! ├── 型変換
//! └── エラー変換
//! ```
//!
//! # 設計原則
//!
//! 1. **ゼロコピー最適化**: 文字列をコピーせずに参照ベースで処理
//! 2. **Scheme特化**: S式、リスト、アトムに最適化されたパーサー
//! 3. **詳細エラー情報**: span情報付きのリッチなエラーレポート
//! 4. **段階的移行**: nom互換APIで既存コードを段階的に移行
//! 5. **高性能**: コンパイル時最適化とSIMD活用
//!
//! # 使用例
//!
//! ```rust
//! use lambdust::parser::combinators::*;
//! 
//! // 基本的なパーサーコンビネータ
//! let parser = tag("define")
//!     .and(whitespace1())
//!     .and(identifier())
//!     .and(expression());
//! 
//! // Scheme特化パーサー
//! let scheme_parser = s_expression()
//!     .or(atom())
//!     .or(list());
//! ```

/// 基本パーサープリミティブ - ゼロコピー操作とコアロジック
pub mod primitive;

/// パーサーコンビネータ - 基本的な組み合わせ操作
pub mod combinator;

/// Scheme特化パーサー - S式、リスト、アトム処理
pub mod scheme;

// nom互換レイヤー - 廃止済み (2025年8月12日)
// pub mod nom_compat;

/// 高レベルパーサービルダー - 使いやすいAPI
pub mod builder;

// Feature flag 切り替えシステム - 廃止済み (2025年8月12日)
// pub mod feature_parser;

// パフォーマンス最適化 - SIMD、並列化、キャッシュ (WIP - コンパイルエラー修正中)
// pub mod optimization;

// Test module temporarily disabled due to JIT system errors
// #[cfg(test)]
// mod tests;

// モジュールのre-export
pub use primitive::*;
pub use combinator::*;
pub use scheme::*;
// pub use nom_compat::*;  // 廃止済み
pub use builder::*;
// pub use feature_parser::*;  // 廃止済み

/// パーサーコンビネータで使用する基本型定義
pub mod types {
    use crate::diagnostics::{Error, Span};
    
    /// パーサー入力 - ゼロコピー文字列スライス
    pub type Input<'a> = &'a str;
    
    /// パーサー結果 - 成功時は残り入力と解析値、失敗時はエラー
    pub type ParseResult<'a, T> = Result<(Input<'a>, T), Box<ParseError>>;
    
    /// パーサー関数型 - 入力を受け取って結果を返す
    pub type Parser<'a, T> = Box<dyn Fn(Input<'a>) -> ParseResult<'a, T>>;
    
    /// パーサーエラー - span情報付き詳細エラー
    #[derive(Debug, Clone)]
    pub struct ParseError {
        /// Error message describing what went wrong
        pub message: String,
        /// Source code span where the error occurred
        pub span: Span,
        /// List of expected tokens or patterns
        pub expected: Vec<String>,
        /// The actual token or input that was found
        pub actual: String,
        /// Context stack showing the parsing context
        pub context: Vec<String>,
    }
    
    impl ParseError {
        /// 新しいパーサーエラーを作成
        pub fn new(message: String, span: Span) -> Self {
            Self {
                message,
                span,
                expected: Vec::new(),
                actual: String::new(),
                context: Vec::new(),
            }
        }
        
        /// 期待される入力を追加
        pub fn with_expected(mut self, expected: String) -> Self {
            self.expected.push(expected);
            self
        }
        
        /// 実際の入力を設定
        pub fn with_actual(mut self, actual: String) -> Self {
            self.actual = actual;
            self
        }
        
        /// コンテキストを追加
        pub fn with_context(mut self, context: String) -> Self {
            self.context.push(context);
            self
        }
    }
}

// 基本型のre-export
pub use types::*;