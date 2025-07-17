//! Core types and data structures for performance measurement system
//!
//! This module defines the fundamental data structures used throughout
//! the performance measurement system.

use crate::ast::Expr;
// Removed unused import: use crate::error::{LambdustError, Result};
use crate::evaluator::EvaluationMode;
use crate::executor::RuntimeOptimizationLevel;
use crate::value::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// メトリクス種別
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MetricType {
    /// 実行時間測定
    ExecutionTime,
    /// メモリ使用量測定
    MemoryUsage,
    /// 最適化効果測定
    OptimizationEffect,
    /// スループット測定
    Throughput,
    /// CPU使用率測定
    CpuUtilization,
    /// 継続プール効率
    ContinuationPoolEfficiency,
    /// インライン評価効果
    InlineEvaluationEffect,
    /// JIT最適化効果
    JitOptimizationEffect,
    /// 末尾呼び出し最適化効果
    TailCallOptimizationEffect,
    /// LLVMバックエンド効果
    LlvmBackendEffect,
    /// キャッシュ効率
    CacheEfficiency,
    /// ガベージコレクション影響
    GarbageCollectionImpact,
    /// 並列化効果
    ParallelizationEffect,
    /// カスタムメトリクス
    Custom(String),
}

/// メトリクス収集器
#[derive(Debug, Clone)]
pub struct MetricCollector {
    /// 収集対象メトリクス
    pub metric_type: MetricType,
    
    /// 収集間隔
    pub collection_interval: Duration,
    
    /// 最後の収集時刻
    pub last_collection: Option<Instant>,
    
    /// サンプリング設定
    pub sampling_config: SamplingConfiguration,
    
    /// 最後の更新時刻
    pub last_update: Option<Instant>,
}

impl Default for MetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricCollector {
    /// 新しいメトリクスコレクターを作成
    #[must_use]
    pub fn new() -> Self {
        Self {
            metric_type: MetricType::ExecutionTime,
            collection_interval: Duration::from_millis(100),
            last_collection: None,
            sampling_config: SamplingConfiguration::default(),
            last_update: None,
        }
    }
    
    /// サンプルを追加（プレースホルダー実装）
    pub fn add_sample(&mut self, _value: MetricValue) {
        self.last_update = Some(Instant::now());
    }
}

/// メトリクスデータ
#[derive(Debug, Clone)]
pub struct MetricData {
    /// メトリクス種別
    pub metric_type: MetricType,
    
    /// 測定値
    pub value: MetricValue,
    
    /// 測定時刻
    pub timestamp: Instant,
    
    /// 測定環境情報
    pub environment: MeasurementEnvironment,
    
    /// 追加メタデータ
    pub metadata: HashMap<String, String>,
}

/// メトリクス値の種別
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// 時間値
    Duration(Duration),
    /// メモリサイズ（バイト）
    MemorySize(usize),
    /// カウント値
    Count(u64),
    /// パーセンテージ（0.0-100.0）
    Percentage(f64),
    /// 比率値（倍率）
    Ratio(f64),
    /// バイト/秒（スループット）
    BytesPerSecond(u64),
    /// 操作/秒（オペレーション数）
    OperationsPerSecond(u64),
    /// カスタム数値
    Custom(f64),
}

/// サンプリング設定
#[derive(Debug, Clone)]
pub struct SamplingConfiguration {
    /// サンプルサイズ
    pub sample_size: usize,
    
    /// サンプリング戦略
    pub strategy: SamplingStrategy,
    
    /// 統計レベル
    pub statistics_level: StatisticsLevel,
}

/// サンプリング戦略
#[derive(Debug, Clone)]
pub enum SamplingStrategy {
    /// 固定間隔サンプリング
    FixedInterval(Duration),
    /// 適応サンプリング
    Adaptive { 
        /// 最小サンプリング間隔
        min_interval: Duration, 
        /// 最大サンプリング間隔
        max_interval: Duration 
    },
    /// 統計的サンプリング
    Statistical { 
        /// 信頼度レベル (0.0-1.0)
        confidence_level: f64 
    },
    /// 負荷ベースサンプリング
    LoadBased { 
        /// 負荷閾値
        threshold: f64 
    },
}

/// 測定精度設定
#[derive(Debug, Clone)]
pub enum MeasurementPrecision {
    /// 低精度（高速）
    Low,
    /// 標準精度
    Standard,
    /// 高精度（詳細）
    High,
    /// 最高精度（包括的）
    Maximum,
}

/// 統計レベル
#[derive(Debug, Clone)]
pub enum StatisticsLevel {
    /// 基本統計（平均・最大・最小）
    Basic,
    /// 詳細統計（分散・標準偏差含む）
    Detailed,
    /// 包括統計（パーセンタイル・分布含む）
    Comprehensive,
}

/// 測定環境情報
#[derive(Debug, Clone)]
pub struct MeasurementEnvironment {
    /// 最適化レベル
    pub optimization_level: RuntimeOptimizationLevel,
    
    /// 評価モード
    pub evaluation_mode: EvaluationMode,
    
    /// システム情報
    pub system_info: SystemInfo,
    
    /// 環境変数
    pub environment_variables: HashMap<String, String>,
}

/// システム情報
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// CPU情報
    pub cpu_info: CpuInfo,
    
    /// メモリ情報
    pub memory_info: MemoryInfo,
    
    /// OS情報
    pub os_info: String,
    
    /// Rustバージョン
    pub rust_version: String,
}

/// CPU情報
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// CPU名
    pub name: String,
    
    /// コア数
    pub cores: usize,
    
    /// 論理プロセッサ数
    pub logical_processors: usize,
    
    /// ベースクロック（MHz）
    pub base_clock_mhz: u32,
}

/// メモリ情報
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// 総メモリサイズ（バイト）
    pub total_memory_bytes: u64,
    
    /// 利用可能メモリ（バイト）
    pub available_memory_bytes: u64,
    
    /// ページサイズ（バイト）
    pub page_size_bytes: usize,
}

/// リアルタイム統計
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct RealtimeStatistics {
    /// アクティブメトリクス数
    pub active_metrics_count: usize,
    
    /// 総測定回数
    pub total_measurements: u64,
    
    /// 最後の測定時刻
    pub last_measurement_time: Option<Instant>,
    
    /// 平均測定間隔
    pub average_measurement_interval: Option<Duration>,
    
    /// 測定オーバーヘッド
    pub measurement_overhead: Option<Duration>,
}

/// 測定対象
#[derive(Debug, Clone)]
pub struct MeasurementTarget {
    /// 対象名
    pub name: String,
    
    /// 対象種別
    pub target_type: MeasurementTargetType,
    
    /// 測定対象式
    pub expression: Option<Expr>,
    
    /// 期待値
    pub expected_value: Option<Value>,
    
    /// 測定回数
    pub iteration_count: usize,
}

/// 測定対象種別
#[derive(Debug, Clone)]
pub enum MeasurementTargetType {
    /// 式評価
    ExpressionEvaluation,
    /// 関数呼び出し
    FunctionCall,
    /// マクロ展開
    MacroExpansion,
    /// 最適化処理
    OptimizationProcess,
    /// システム全体
    SystemWide,
}

impl Default for SamplingConfiguration {
    fn default() -> Self {
        Self {
            sample_size: 100,
            strategy: SamplingStrategy::FixedInterval(Duration::from_millis(100)),
            statistics_level: StatisticsLevel::Basic,
        }
    }
}

impl Default for MeasurementEnvironment {
    fn default() -> Self {
        Self {
            optimization_level: RuntimeOptimizationLevel::Balanced,
            evaluation_mode: EvaluationMode::Semantic,
            system_info: SystemInfo::default(),
            environment_variables: HashMap::new(),
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu_info: CpuInfo::default(),
            memory_info: MemoryInfo::default(),
            os_info: std::env::consts::OS.to_string(),
            rust_version: std::env::var("RUSTC_VERSION")
                .unwrap_or_else(|_| "Unknown".to_string()),
        }
    }
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            name: "Unknown CPU".to_string(),
            cores: 4, // Default assumption when num_cpus unavailable
            logical_processors: 4, // Default assumption
            base_clock_mhz: 2400, // Default assumption
        }
    }
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            total_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB default
            available_memory_bytes: 4 * 1024 * 1024 * 1024, // 4GB default
            page_size_bytes: 4096, // 4KB default
        }
    }
}

impl MetricType {
    /// Create metric type from measurement data
    #[must_use] pub fn from_measurement_data(data: &MetricData) -> Self {
        // Infer metric type from measurement data characteristics
        if data.metadata.contains_key("execution_time") {
            MetricType::ExecutionTime
        } else if data.metadata.contains_key("memory_usage") {
            MetricType::MemoryUsage
        } else if data.metadata.contains_key("cpu_usage") {
            MetricType::CpuUtilization
        } else if data.metadata.contains_key("throughput") {
            MetricType::Throughput
        } else {
            // Default to execution time for unspecified measurements
            MetricType::ExecutionTime
        }
    }
}

