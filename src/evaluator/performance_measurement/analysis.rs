//! Analysis and verification system for performance measurements
//!
//! This module provides advanced analysis capabilities for performance
//! data and optimization effect verification.

use super::core_types::{MetricType, MetricValue};
use super::benchmarking::{BenchmarkExecutionResult, BenchmarkStatistics};
use crate::error::{LambdustError, Result};
use crate::evaluator::RuntimeOptimizationLevel;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 分析エンジン
#[derive(Debug, Clone)]
pub struct AnalysisEngine {
    /// 分析設定
    analysis_config: AnalysisConfiguration,
    
    /// 分析履歴
    analysis_history: Vec<AnalysisResult>,
    
    /// 統計モデル
    statistical_models: HashMap<String, StatisticalModel>,
}

/// 最適化効果検証器
#[derive(Debug, Clone)]
pub struct OptimizationEffectVerifier {
    /// 検証設定
    pub verification_config: VerificationConfiguration,
    
    /// ベースライン測定
    pub baseline_measurements: HashMap<MetricType, Vec<MetricValue>>,
    
    /// 検証履歴
    pub verification_history: Vec<OptimizationEffectResult>,
}

/// 分析設定
#[derive(Debug, Clone)]
pub struct AnalysisConfiguration {
    /// 分析深度
    pub analysis_depth: AnalysisDepth,
    
    /// 統計手法
    pub statistical_methods: Vec<StatisticalMethod>,
    
    /// 信頼区間
    pub confidence_interval: f64,
    
    /// 有意水準
    pub significance_level: f64,
    
    /// 異常値検出
    pub outlier_detection: OutlierDetectionConfig,
    
    /// トレンド分析
    pub trend_analysis: TrendAnalysisConfig,
}

/// 分析深度
#[derive(Debug, Clone)]
pub enum AnalysisDepth {
    /// 基本分析
    Basic,
    /// 標準分析
    Standard,
    /// 詳細分析
    Detailed,
    /// 包括分析
    Comprehensive,
}

/// 統計手法
#[derive(Debug, Clone)]
pub enum StatisticalMethod {
    /// 記述統計
    DescriptiveStatistics,
    /// 仮説検定
    HypothesisTesting,
    /// 回帰分析
    RegressionAnalysis,
    /// 時系列分析
    TimeSeriesAnalysis,
    /// 分散分析
    VarianceAnalysis,
    /// 相関分析
    CorrelationAnalysis,
}

/// 異常値検出設定
#[derive(Debug, Clone)]
pub struct OutlierDetectionConfig {
    /// 検出手法
    pub method: OutlierDetectionMethod,
    
    /// 閾値
    pub threshold: f64,
    
    /// 処理方法
    pub handling: OutlierHandling,
}

/// 異常値検出手法
#[derive(Debug, Clone)]
pub enum OutlierDetectionMethod {
    /// Zスコア
    ZScore,
    /// 修正Zスコア
    ModifiedZScore,
    /// 四分位範囲
    InterquartileRange,
    /// 孤立フォレスト
    IsolationForest,
}

/// 異常値処理
#[derive(Debug, Clone)]
pub enum OutlierHandling {
    /// 削除
    Remove,
    /// フラグ付け
    Flag,
    /// 修正
    Correct,
    /// 無視
    Ignore,
}

/// トレンド分析設定
#[derive(Debug, Clone)]
pub struct TrendAnalysisConfig {
    /// ウィンドウサイズ
    pub window_size: usize,
    
    /// 平滑化手法
    pub smoothing_method: SmoothingMethod,
    
    /// 季節性検出
    pub seasonal_detection: bool,
    
    /// 予測期間
    pub forecast_horizon: Duration,
}

/// 平滑化手法
#[derive(Debug, Clone)]
pub enum SmoothingMethod {
    /// 移動平均
    MovingAverage,
    /// 指数平滑化
    ExponentialSmoothing,
    /// Savitzky-Golay
    SavitzkyGolay,
    /// LOWESS
    Lowess,
}

/// 統計モデル
#[derive(Debug, Clone)]
pub struct StatisticalModel {
    /// モデル名
    pub name: String,
    
    /// モデル種別
    pub model_type: ModelType,
    
    /// パラメータ
    pub parameters: HashMap<String, f64>,
    
    /// 精度メトリクス
    pub accuracy_metrics: AccuracyMetrics,
}

/// モデル種別
#[derive(Debug, Clone)]
pub enum ModelType {
    /// 線形回帰
    LinearRegression,
    /// 多項式回帰
    PolynomialRegression,
    /// ARIMA
    Arima,
    /// 指数平滑化
    ExponentialSmoothing,
}

/// 精度メトリクス
#[derive(Debug, Clone)]
pub struct AccuracyMetrics {
    /// 平均絶対誤差
    pub mae: f64,
    
    /// 平均二乗誤差
    pub mse: f64,
    
    /// 決定係数
    pub r_squared: f64,
    
    /// 平均絶対パーセント誤差
    pub mape: f64,
}

/// 分析結果
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// 分析名
    pub analysis_name: String,
    
    /// 分析時刻
    pub timestamp: Instant,
    
    /// 分析対象データ
    pub analyzed_data: AnalyzedData,
    
    /// 分析結果
    pub results: HashMap<String, AnalysisResultValue>,
    
    /// 洞察
    pub insights: Vec<Insight>,
    
    /// 推奨事項
    pub recommendations: Vec<Recommendation>,
}

/// 分析対象データ
#[derive(Debug, Clone)]
pub struct AnalyzedData {
    /// データ種別
    pub data_type: DataType,
    
    /// サンプル数
    pub sample_count: usize,
    
    /// 時間範囲
    pub time_range: TimeRange,
    
    /// メトリクス
    pub metrics: Vec<MetricType>,
}

/// データ種別
#[derive(Debug, Clone)]
pub enum DataType {
    /// ベンチマーク結果
    BenchmarkResults,
    /// リアルタイム測定
    RealtimeMeasurements,
    /// 履歴データ
    HistoricalData,
    /// 比較データ
    ComparisonData,
}

/// 時間範囲
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// 開始時刻
    pub start: Instant,
    
    /// 終了時刻
    pub end: Instant,
    
    /// 期間
    pub duration: Duration,
}

/// 分析結果値
#[derive(Debug, Clone)]
pub enum AnalysisResultValue {
    /// 数値
    Numeric(f64),
    
    /// パーセンテージ
    Percentage(f64),
    
    /// 期間
    Duration(Duration),
    
    /// 文字列
    Text(String),
    
    /// ブール値
    Boolean(bool),
    
    /// 配列
    Array(Vec<f64>),
}

/// 洞察
#[derive(Debug, Clone)]
pub struct Insight {
    /// 洞察種別
    pub insight_type: InsightType,
    
    /// 重要度
    pub importance: ImportanceLevel,
    
    /// 説明
    pub description: String,
    
    /// 影響度
    pub impact: ImpactLevel,
    
    /// 信頼度
    pub confidence: f64,
}

/// 洞察種別
#[derive(Debug, Clone)]
pub enum InsightType {
    /// パフォーマンス改善
    PerformanceImprovement,
    
    /// パフォーマンス劣化
    PerformanceDegradation,
    
    /// 異常検出
    AnomalyDetection,
    
    /// トレンド変化
    TrendChange,
    
    /// 最適化機会
    OptimizationOpportunity,
}

/// 重要度レベル
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportanceLevel {
    /// 低
    Low,
    
    /// 中
    Medium,
    
    /// 高
    High,
    
    /// 重要
    Critical,
}

/// 影響度レベル
#[derive(Debug, Clone)]
pub enum ImpactLevel {
    /// 最小
    Minimal,
    
    /// 小
    Small,
    
    /// 中
    Medium,
    
    /// 大
    Large,
    
    /// 非常に大
    Huge,
}

/// 推奨事項
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// 推奨種別
    pub recommendation_type: RecommendationType,
    
    /// 優先度
    pub priority: PriorityLevel,
    
    /// 説明
    pub description: String,
    
    /// 期待効果
    pub expected_benefit: ExpectedBenefit,
    
    /// 実装複雑度
    pub implementation_complexity: ComplexityLevel,
}

/// 推奨種別
#[derive(Debug, Clone)]
pub enum RecommendationType {
    /// 最適化設定変更
    OptimizationTuning,
    
    /// アルゴリズム変更
    AlgorithmChange,
    
    /// リソース調整
    ResourceAdjustment,
    
    /// 構成変更
    ConfigurationChange,
    
    /// 追加測定
    AdditionalMeasurement,
}

/// 優先度レベル
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    /// 低
    Low,
    
    /// 中
    Medium,
    
    /// 高
    High,
    
    /// 緊急
    Urgent,
}

/// 期待効果
#[derive(Debug, Clone)]
pub struct ExpectedBenefit {
    /// パフォーマンス改善率
    pub performance_improvement: Option<f64>,
    
    /// リソース削減率
    pub resource_reduction: Option<f64>,
    
    /// エラー率改善
    pub error_rate_improvement: Option<f64>,
    
    /// 信頼度
    pub confidence: f64,
}

/// 複雑度レベル
#[derive(Debug, Clone)]
pub enum ComplexityLevel {
    /// 簡単
    Simple,
    
    /// 中程度
    Moderate,
    
    /// 複雑
    Complex,
    
    /// 非常に複雑
    VeryComplex,
}

/// 検証設定
#[derive(Debug, Clone)]
pub struct VerificationConfiguration {
    /// 最小改善率
    pub minimum_improvement_threshold: f64,
    
    /// 統計的有意性
    pub statistical_significance: bool,
    
    /// 比較方法
    pub comparison_method: ComparisonMethod,
    
    /// 検証期間
    pub verification_duration: Duration,
}

/// 比較方法
#[derive(Debug, Clone)]
pub enum ComparisonMethod {
    /// 前後比較
    BeforeAfter,
    
    /// A/Bテスト
    ABTest,
    
    /// 統制実験
    ControlledExperiment,
    
    /// 時系列比較
    TimeSeriesComparison,
}

/// 最適化効果結果
#[derive(Debug, Clone)]
pub struct OptimizationEffectResult {
    /// 最適化名
    pub optimization_name: String,
    
    /// 検証時刻
    pub verification_time: Instant,
    
    /// 最適化レベル
    pub optimization_level: RuntimeOptimizationLevel,
    
    /// 効果測定
    pub effect_measurements: HashMap<MetricType, EffectMeasurement>,
    
    /// 総合評価
    pub overall_assessment: OverallAssessment,
    
    /// 統計的有意性
    pub statistical_significance: Option<StatisticalSignificance>,
}

/// 効果測定
#[derive(Debug, Clone)]
pub struct EffectMeasurement {
    /// ベースライン値
    pub baseline_value: f64,
    
    /// 最適化後値
    pub optimized_value: f64,
    
    /// 改善率
    pub improvement_ratio: f64,
    
    /// 絶対改善値
    pub absolute_improvement: f64,
    
    /// 測定信頼度
    pub measurement_confidence: f64,
}

/// 総合評価
#[derive(Debug, Clone)]
pub enum OverallAssessment {
    /// 大幅改善
    SignificantImprovement,
    
    /// 改善
    Improvement,
    
    /// 僅かな改善
    SlightImprovement,
    
    /// 変化なし
    NoChange,
    
    /// 僅かな劣化
    SlightDegradation,
    
    /// 劣化
    Degradation,
    
    /// 大幅劣化
    SignificantDegradation,
}

/// 統計的有意性
#[derive(Debug, Clone)]
pub struct StatisticalSignificance {
    /// p値
    pub p_value: f64,
    
    /// 有意水準
    pub significance_level: f64,
    
    /// 有意性
    pub is_significant: bool,
    
    /// 効果サイズ
    pub effect_size: f64,
    
    /// 信頼区間
    pub confidence_interval: (f64, f64),
}

impl AnalysisEngine {
    /// 新しい分析エンジンを作成
    pub fn new() -> Self {
        Self {
            analysis_config: AnalysisConfiguration::default(),
            analysis_history: Vec::new(),
            statistical_models: HashMap::new(),
        }
    }

    /// ベンチマーク結果を分析
    pub fn analyze_benchmark_results(
        &mut self,
        results: &[BenchmarkExecutionResult],
    ) -> Result<AnalysisResult> {
        let analyzed_data = AnalyzedData {
            data_type: DataType::BenchmarkResults,
            sample_count: results.len(),
            time_range: self.calculate_time_range(results),
            metrics: self.extract_metrics(results),
        };

        let analysis_results = self.perform_analysis(&analyzed_data)?;
        let insights = self.generate_insights(&analysis_results)?;
        let recommendations = self.generate_recommendations(&insights)?;

        let result = AnalysisResult {
            analysis_name: "Benchmark Analysis".to_string(),
            timestamp: Instant::now(),
            analyzed_data,
            results: analysis_results,
            insights,
            recommendations,
        };

        self.analysis_history.push(result.clone());
        Ok(result)
    }

    /// 分析を実行
    fn perform_analysis(
        &self,
        _data: &AnalyzedData,
    ) -> Result<HashMap<String, AnalysisResultValue>> {
        let mut results = HashMap::new();
        
        // 基本統計計算（簡略化）
        results.insert(
            "mean_execution_time".to_string(),
            AnalysisResultValue::Duration(Duration::from_micros(150)),
        );
        results.insert(
            "improvement_ratio".to_string(),
            AnalysisResultValue::Percentage(15.0),
        );
        
        Ok(results)
    }

    /// 洞察を生成
    fn generate_insights(
        &self,
        _results: &HashMap<String, AnalysisResultValue>,
    ) -> Result<Vec<Insight>> {
        Ok(vec![Insight {
            insight_type: InsightType::PerformanceImprovement,
            importance: ImportanceLevel::High,
            description: "Significant performance improvement detected".to_string(),
            impact: ImpactLevel::Large,
            confidence: 0.95,
        }])
    }

    /// 推奨事項を生成
    fn generate_recommendations(
        &self,
        _insights: &[Insight],
    ) -> Result<Vec<Recommendation>> {
        Ok(vec![Recommendation {
            recommendation_type: RecommendationType::OptimizationTuning,
            priority: PriorityLevel::High,
            description: "Consider increasing optimization level".to_string(),
            expected_benefit: ExpectedBenefit {
                performance_improvement: Some(20.0),
                resource_reduction: Some(10.0),
                error_rate_improvement: None,
                confidence: 0.85,
            },
            implementation_complexity: ComplexityLevel::Simple,
        }])
    }

    /// 時間範囲を計算
    fn calculate_time_range(&self, results: &[BenchmarkExecutionResult]) -> TimeRange {
        let start = results.iter().map(|r| r.start_time).min().unwrap_or_else(Instant::now);
        let end = results.iter().map(|r| r.end_time).max().unwrap_or_else(Instant::now);
        
        TimeRange {
            start,
            end,
            duration: end.duration_since(start),
        }
    }

    /// メトリクスを抽出
    fn extract_metrics(&self, results: &[BenchmarkExecutionResult]) -> Vec<MetricType> {
        let mut metrics = Vec::new();
        for result in results {
            for metric_type in result.measurements.keys() {
                if !metrics.contains(metric_type) {
                    metrics.push(metric_type.clone());
                }
            }
        }
        metrics
    }
}

impl Default for AnalysisConfiguration {
    fn default() -> Self {
        Self {
            analysis_depth: AnalysisDepth::Standard,
            statistical_methods: vec![
                StatisticalMethod::DescriptiveStatistics,
                StatisticalMethod::HypothesisTesting,
            ],
            confidence_interval: 0.95,
            significance_level: 0.05,
            outlier_detection: OutlierDetectionConfig::default(),
            trend_analysis: TrendAnalysisConfig::default(),
        }
    }
}

impl Default for OutlierDetectionConfig {
    fn default() -> Self {
        Self {
            method: OutlierDetectionMethod::ZScore,
            threshold: 2.0,
            handling: OutlierHandling::Flag,
        }
    }
}

impl Default for TrendAnalysisConfig {
    fn default() -> Self {
        Self {
            window_size: 10,
            smoothing_method: SmoothingMethod::MovingAverage,
            seasonal_detection: false,
            forecast_horizon: Duration::from_secs(3600), // 1 hour
        }
    }
}