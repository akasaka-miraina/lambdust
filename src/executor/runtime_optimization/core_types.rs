//! Core types and strategy definitions for runtime optimization integration
//!
//! This module defines the fundamental data structures and optimization
//! strategies used throughout the runtime optimization system.

// Removed unused imports:
// use crate::ast::Expr;
// use crate::error::{LambdustError, Result};
use crate::executor::RuntimeOptimizationLevel;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 最適化戦略定義
#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    /// 戦略名
    pub name: String,

    /// 戦略種別
    pub strategy_type: OptimizationStrategyType,

    /// 適用可能性条件
    pub applicability: ApplicabilityCondition,

    /// 最適化インパクト予測
    pub impact_prediction: OptimizationImpact,

    /// 実行コスト
    pub execution_cost: OptimizationCost,

    /// 実行制約
    pub constraints: Vec<ExecutionConstraint>,

    /// メタデータ
    pub metadata: OptimizationMetadata,
}

/// 最適化戦略種別
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategyType {
    /// 継続プール最適化
    ContinuationPooling {
        /// Size of the continuation pool
        pool_size: usize,
        /// Allocation strategy for the pool
        allocation_strategy: String,
    },

    /// インライン評価最適化
    InlineEvaluation {
        /// Threshold for inlining decisions
        inline_threshold: usize,
        /// Maximum complexity allowed for inlining
        complexity_limit: usize,
    },

    /// JIT最適化
    JitOptimization {
        /// Threshold for JIT compilation
        compilation_threshold: usize,
        /// JIT optimization level
        optimization_level: String,
    },

    /// 末尾呼び出し最適化
    TailCallOptimization {
        /// Depth for tail call detection
        detection_depth: usize,
        /// Strategy for tail call transformation
        transformation_strategy: String,
    },

    /// メモリ最適化
    MemoryOptimization {
        /// Garbage collection strategy
        gc_strategy: String,
        /// Threshold for memory compaction
        compaction_threshold: f64,
    },

    /// 並列化最適化
    ParallelizationOptimization {
        /// Size of the thread pool
        thread_pool_size: usize,
        /// Whether to enable work stealing
        work_stealing: bool,
    },

    /// キャッシュ最適化
    CacheOptimization {
        /// Size of the cache
        cache_size: usize,
        /// Cache eviction policy
        eviction_policy: String,
    },

    /// カスタム最適化
    Custom {
        /// Name of the custom strategy
        strategy_name: String,
        /// Parameters for the custom strategy
        parameters: HashMap<String, OptimizationParameter>,
    },
}

/// 適用可能性条件
#[derive(Debug, Clone)]
pub struct ApplicabilityCondition {
    /// 条件式
    pub conditions: Vec<ConditionType>,

    /// 結合演算子（AND/OR）
    pub combination_operator: LogicalOperator,

    /// 適用閾値
    pub threshold: f64,

    /// 動的評価
    pub dynamic_evaluation: bool,
}

/// 条件種別
#[derive(Debug, Clone)]
pub enum ConditionType {
    /// 式複雑度条件
    ExpressionComplexity {
        /// Predicate for comparison
        predicate: ConditionPredicate,
        /// Threshold value for comparison
        value: f64,
    },

    /// メモリ使用量条件
    MemoryUsage {
        /// Predicate for memory usage comparison
        predicate: ConditionPredicate,
        /// Memory threshold in bytes
        threshold_bytes: usize,
    },

    /// 実行頻度条件
    ExecutionFrequency {
        /// Predicate for frequency comparison
        predicate: ConditionPredicate,
        /// Frequency threshold
        frequency: f64,
    },

    /// パフォーマンス条件
    Performance {
        /// Performance metric name
        metric: String,
        /// Predicate for performance comparison
        predicate: ConditionPredicate,
        /// Performance threshold value
        value: f64,
    },

    /// コンテキスト条件
    Context {
        /// Type of context to check
        context_type: String,
        /// Expected value for the context
        expected_value: String,
    },
}

/// 条件述語
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionPredicate {
    /// より大きい
    GreaterThan,
    /// 以上
    GreaterThanOrEqual,
    /// より小さい
    LessThan,
    /// 以下
    LessThanOrEqual,
    /// 等しい
    Equal,
    /// 等しくない
    NotEqual,
    /// 範囲内
    Between { 
        /// Minimum value of the range
        min: f64, 
        /// Maximum value of the range
        max: f64 
    },
    /// 範囲外
    Outside { 
        /// Minimum value of the excluded range
        min: f64, 
        /// Maximum value of the excluded range
        max: f64 
    },
}

/// 論理演算子
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    /// AND
    And,
    /// OR
    Or,
    /// XOR
    Xor,
    /// NAND
    Nand,
    /// NOR
    Nor,
}

/// 最適化インパクト予測
#[derive(Debug, Clone)]
pub struct OptimizationImpact {
    /// パフォーマンス改善予測（倍率）
    pub performance_improvement: f64,

    /// メモリインパクト
    pub memory_impact: MemoryImpact,

    /// CPU使用率変化
    pub cpu_usage_change: f64,

    /// エネルギー効率変化
    pub energy_efficiency_change: f64,

    /// リスクレベル
    pub risk_level: RiskLevel,

    /// 予測信頼度
    pub confidence_level: f64,
}

/// メモリインパクト
#[derive(Debug, Clone)]
pub enum MemoryImpact {
    /// メモリ削減
    Reduction { 
        /// Estimated bytes to be reduced
        estimated_bytes: usize 
    },
    /// メモリ増加
    Increase { 
        /// Estimated bytes to be increased
        estimated_bytes: usize 
    },
    /// メモリ使用量不変
    Neutral,
    /// 動的変化
    Dynamic { 
        /// Function describing the dynamic memory change
        function: String 
    },
}

/// リスクレベル
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RiskLevel {
    /// 非常に低い
    VeryLow,
    /// 低い
    Low,
    /// 中程度
    Medium,
    /// 高い
    High,
    /// 非常に高い
    VeryHigh,
}

/// 最適化実行コスト
#[derive(Debug, Clone)]
pub struct OptimizationCost {
    /// 初期化コスト
    pub initialization_cost: Duration,

    /// 実行コスト（式あたり）
    pub execution_cost_per_expr: Duration,

    /// メモリオーバーヘッド
    pub memory_overhead: usize,

    /// CPU使用率増加
    pub cpu_overhead: f64,

    /// スケーラビリティ係数
    pub scalability_factor: f64,
}

/// 実行制約
#[derive(Debug, Clone)]
pub struct ExecutionConstraint {
    /// 制約種別
    pub constraint_type: ConstraintType,

    /// 制約値
    pub constraint_value: ConstraintValue,

    /// 制約の重要度
    pub importance: ConstraintImportance,

    /// 制約違反時の処理
    pub violation_handling: ViolationHandling,
}

/// 制約種別
#[derive(Debug, Clone)]
pub enum ConstraintType {
    /// 最大実行時間
    MaxExecutionTime,
    /// 最大メモリ使用量
    MaxMemoryUsage,
    /// 最大CPU使用率
    MaxCpuUsage,
    /// 最小パフォーマンス改善
    MinPerformanceImprovement,
    /// スレッドセーフティ
    ThreadSafety,
    /// メモリセーフティ
    MemorySafety,
    /// デッドロック防止
    DeadlockPrevention,
    /// カスタム制約
    Custom { 
        /// Name of the custom constraint
        name: String 
    },
}

/// 制約値
#[derive(Debug, Clone)]
pub enum ConstraintValue {
    /// 期間
    Duration(Duration),
    /// メモリサイズ（バイト）
    MemorySize(usize),
    /// パーセンテージ
    Percentage(f64),
    /// ブール値
    Boolean(bool),
    /// 文字列
    String(String),
    /// 数値
    Numeric(f64),
}

/// 制約の重要度
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ConstraintImportance {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 重要
    Critical,
}

/// 制約違反時の処理
#[derive(Debug, Clone)]
pub enum ViolationHandling {
    /// 最適化をスキップ
    SkipOptimization,
    /// 警告を出力
    Warning,
    /// エラーを発生
    Error,
    /// 代替戦略を試行
    FallbackStrategy { 
        /// Name of the fallback strategy
        strategy_name: String 
    },
    /// 最適化を無効化
    DisableOptimization,
}

/// 最適化メタデータ
#[derive(Debug, Clone)]
pub struct OptimizationMetadata {
    /// 作成者
    pub author: String,

    /// バージョン
    pub version: String,

    /// 説明
    pub description: String,

    /// 互換性情報
    pub compatibility: CompatibilityInfo,

    /// 依存関係
    pub dependencies: Vec<String>,

    /// タグ
    pub tags: Vec<String>,

    /// 実験的フラグ
    pub experimental: bool,
}

/// 互換性情報
#[derive(Debug, Clone)]
pub struct CompatibilityInfo {
    /// 対応最適化レベル
    pub supported_levels: Vec<RuntimeOptimizationLevel>,

    /// 式タイプ制限
    pub expression_type_restrictions: Vec<ExpressionType>,

    /// 他の最適化との競合
    pub conflicts_with: Vec<String>,

    /// 必須の前提条件
    pub prerequisites: Vec<String>,
}

/// 式タイプ
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ExpressionType {
    /// リテラル
    Literal,
    /// 変数
    Variable,
    /// 関数呼び出し
    FunctionCall,
    /// Lambda式
    Lambda,
    /// 条件式
    Conditional,
    /// ループ
    Loop,
    /// 束縛式
    Binding,
    /// 算術演算
    Arithmetic,
    /// 比較演算
    Comparison,
    /// 論理演算
    Logical,
    /// リスト操作
    ListOperation,
    /// マクロ展開
    MacroExpansion,
    /// カスタム
    Custom(String),
}

/// 最適化パラメータ
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationParameter {
    /// 整数値
    Integer(i64),
    /// 浮動小数点値
    Float(f64),
    /// ブール値
    Boolean(bool),
    /// 文字列
    String(String),
    /// 期間
    Duration(Duration),
    /// 配列
    Array(Vec<OptimizationParameter>),
    /// オブジェクト
    Object(HashMap<String, OptimizationParameter>),
}

/// 動的戦略調整システム
#[derive(Debug, Clone)]
pub struct DynamicStrategyAdjustment {
    /// 調整可能パラメータ
    pub adjustable_parameters: HashMap<String, ParameterRange>,

    /// 調整戦略
    pub adjustment_strategy: AdjustmentStrategy,

    /// 学習率
    pub learning_rate: f64,

    /// 調整履歴
    pub adjustment_history: Vec<AdjustmentRecord>,

    /// フィードバックループ
    pub feedback_loop: FeedbackConfiguration,
}

/// パラメータ範囲
#[derive(Debug, Clone)]
pub struct ParameterRange {
    /// 最小値
    pub min_value: f64,
    /// 最大値
    pub max_value: f64,
    /// デフォルト値
    pub default_value: f64,
    /// ステップサイズ
    pub step_size: f64,
}

/// 調整戦略
#[derive(Debug, Clone)]
pub enum AdjustmentStrategy {
    /// 勾配降下法
    GradientDescent,
    /// 遺伝的アルゴリズム
    GeneticAlgorithm,
    /// ベイズ最適化
    BayesianOptimization,
    /// 強化学習
    ReinforcementLearning,
    /// ランダムサーチ
    RandomSearch,
    /// グリッドサーチ
    GridSearch,
}

/// 調整記録
#[derive(Debug, Clone)]
pub struct AdjustmentRecord {
    /// タイムスタンプ
    pub timestamp: Instant,
    /// 調整パラメータ
    pub parameters: HashMap<String, f64>,
    /// パフォーマンス結果
    pub performance_result: f64,
    /// 調整理由
    pub reason: String,
}

/// フィードバック設定
#[derive(Debug, Clone)]
pub struct FeedbackConfiguration {
    /// フィードバック間隔
    pub feedback_interval: Duration,
    /// 最小改善閾値
    pub min_improvement_threshold: f64,
    /// 最大調整回数
    pub max_adjustments: usize,
    /// 安定性要求
    pub stability_requirement: f64,
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        Self {
            name: "DefaultStrategy".to_string(),
            strategy_type: OptimizationStrategyType::ContinuationPooling {
                pool_size: 100,
                allocation_strategy: "FIFO".to_string(),
            },
            applicability: ApplicabilityCondition::default(),
            impact_prediction: OptimizationImpact::default(),
            execution_cost: OptimizationCost::default(),
            constraints: Vec::new(),
            metadata: OptimizationMetadata::default(),
        }
    }
}

impl Default for ApplicabilityCondition {
    fn default() -> Self {
        Self {
            conditions: Vec::new(),
            combination_operator: LogicalOperator::And,
            threshold: 0.5,
            dynamic_evaluation: false,
        }
    }
}

impl Default for OptimizationImpact {
    fn default() -> Self {
        Self {
            performance_improvement: 1.0,
            memory_impact: MemoryImpact::Neutral,
            cpu_usage_change: 0.0,
            energy_efficiency_change: 0.0,
            risk_level: RiskLevel::Low,
            confidence_level: 0.8,
        }
    }
}

impl Default for OptimizationCost {
    fn default() -> Self {
        Self {
            initialization_cost: Duration::from_micros(100),
            execution_cost_per_expr: Duration::from_nanos(50),
            memory_overhead: 1024, // 1KB
            cpu_overhead: 0.05,    // 5%
            scalability_factor: 1.0,
        }
    }
}

impl Default for OptimizationMetadata {
    fn default() -> Self {
        Self {
            author: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            description: "Default optimization strategy".to_string(),
            compatibility: CompatibilityInfo::default(),
            dependencies: Vec::new(),
            tags: Vec::new(),
            experimental: false,
        }
    }
}

impl Default for CompatibilityInfo {
    fn default() -> Self {
        Self {
            supported_levels: vec![
                RuntimeOptimizationLevel::Conservative,
                RuntimeOptimizationLevel::Balanced,
            ],
            expression_type_restrictions: Vec::new(),
            conflicts_with: Vec::new(),
            prerequisites: Vec::new(),
        }
    }
}

impl Default for DynamicStrategyAdjustment {
    fn default() -> Self {
        Self {
            adjustable_parameters: HashMap::new(),
            adjustment_strategy: AdjustmentStrategy::GradientDescent,
            learning_rate: 0.01,
            adjustment_history: Vec::new(),
            feedback_loop: FeedbackConfiguration::default(),
        }
    }
}

impl Default for FeedbackConfiguration {
    fn default() -> Self {
        Self {
            feedback_interval: Duration::from_secs(60),
            min_improvement_threshold: 0.05, // 5%
            max_adjustments: 100,
            stability_requirement: 0.95, // 95% stability
        }
    }
}