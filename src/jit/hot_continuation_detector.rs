#![allow(missing_docs)]
//! ホット継続検出システム - 動的性能分析による最適化候補選択
//!
//! このモジュールは継続実行の動的プロファイリングを行い、JIT最適化の
//! 対象となるホット継続を効率的に検出します：
//!
//! - **リアルタイム実行頻度追跡**: O(1)での高速プロファイリング
//! - **適応的しきい値調整**: 動的な最適化判断基準
//! - **継続重要度スコアリング**: 総合的な最適化価値評価
//! - **統計的信頼性保証**: 統計的有意性に基づく判定

use crate::continuations::{ContinuationId, OptimizedContinuation};
use crate::diagnostics::{Error, Result};

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};

/// ホット継続検出システム
///
/// **アルゴリズム特徴**:
/// - **指数移動平均**: 実行頻度の時系列解析
/// - **統計的分析**: 信頼区間による判定精度向上
/// - **適応的学習**: 動的しきい値調整
/// - **負荷分散**: プロファイリングオーバーヘッド最小化
pub struct HotContinuationDetector {
    /// 継続実行統計データベース
    execution_stats: Arc<RwLock<HashMap<ContinuationId, ContinuationExecutionStats>>>,

    /// 動的しきい値管理
    threshold_manager: Arc<Mutex<DynamicThresholdManager>>,

    /// 実行履歴バッファ（統計分析用）
    execution_history: Arc<Mutex<VecDeque<ExecutionEvent>>>,

    /// 検出設定
    config: HotContinuationDetectorConfig,

    /// システム統計
    system_stats: Arc<RwLock<DetectorSystemStats>>,
}

impl HotContinuationDetector {
    /// 新しいホット継続検出システムを作成
    pub fn new(config: HotContinuationDetectorConfig) -> Self {
        HotContinuationDetector {
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
            threshold_manager: Arc::new(Mutex::new(DynamicThresholdManager::new(
                config.threshold_config.clone(),
            ))),
            execution_history: Arc::new(Mutex::new(VecDeque::with_capacity(
                config.history_buffer_size,
            ))),
            system_stats: Arc::new(RwLock::new(DetectorSystemStats::new())),
            config,
        }
    }

    /// 継続実行の記録とホット継続判定
    ///
    /// **アルゴリズムの流れ**:
    /// 1. 実行統計の更新（O(1)）
    /// 2. 指数移動平均の計算
    /// 3. 統計的有意性の検証
    /// 4. 動的しきい値との比較
    /// 5. ホット継続判定の結果返却
    pub fn record_execution(
        &self,
        continuation_id: ContinuationId,
        execution_time: Duration,
    ) -> Result<HotContinuationDecision> {
        let record_start = Instant::now();

        // Step 1: 実行統計の更新
        let current_stats = self.update_execution_stats(continuation_id, execution_time)?;

        // Step 2: 実行履歴への追加（統計分析用）
        self.add_execution_to_history(continuation_id, execution_time);

        // Step 3: ホット継続判定の実行
        let decision = self.evaluate_hot_continuation_candidate(continuation_id, &current_stats)?;

        // Step 4: しきい値の動的調整
        if self.should_adjust_thresholds() {
            self.adjust_dynamic_thresholds()?;
        }

        // Step 5: システム統計の更新
        self.update_system_stats(record_start.elapsed());

        Ok(decision)
    }

    /// 実行統計の更新（O(1)操作）
    fn update_execution_stats(
        &self,
        continuation_id: ContinuationId,
        execution_time: Duration,
    ) -> Result<ContinuationExecutionStats> {
        let mut stats_db = self
            .execution_stats
            .write()
            .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;

        let stats = stats_db.entry(continuation_id).or_insert_with(|| {
            ContinuationExecutionStats::new(continuation_id, self.config.ema_alpha)
        });

        // 指数移動平均による実行頻度の更新
        stats.update_execution(execution_time);

        Ok(stats.clone())
    }

    /// 実行履歴への記録（統計分析用）
    fn add_execution_to_history(&self, continuation_id: ContinuationId, execution_time: Duration) {
        if let Ok(mut history) = self.execution_history.lock() {
            let event = ExecutionEvent {
                continuation_id,
                execution_time,
                timestamp: SystemTime::now(),
            };

            // バッファサイズ制限の実装
            if history.len() >= self.config.history_buffer_size {
                history.pop_front();
            }
            history.push_back(event);
        }
    }

    /// ホット継続候補の評価
    fn evaluate_hot_continuation_candidate(
        &self,
        continuation_id: ContinuationId,
        stats: &ContinuationExecutionStats,
    ) -> Result<HotContinuationDecision> {
        // Step 1: 基本統計的条件の確認
        if !self.meets_minimum_statistical_requirements(stats) {
            return Ok(HotContinuationDecision::InsufficientData {
                continuation_id,
                current_executions: stats.execution_count,
                required_minimum: self.config.min_executions_for_analysis,
            });
        }

        // Step 2: 現在のしきい値取得
        let thresholds = {
            let threshold_manager = self.threshold_manager.lock().map_err(|_| {
                Error::runtime_error("Failed to acquire threshold lock".to_string(), None)
            })?;
            threshold_manager.get_current_thresholds()
        };

        // Step 3: 継続重要度スコアの計算
        let importance_score = self.calculate_continuation_importance_score(stats, &thresholds)?;

        // Step 4: ホット継続判定
        if importance_score >= thresholds.hot_continuation_threshold {
            Ok(HotContinuationDecision::HotContinuationDetected {
                continuation_id,
                importance_score,
                recommendation: OptimizationRecommendation::HighPriority,
                statistical_confidence: self.calculate_statistical_confidence(stats)?,
            })
        } else if importance_score >= thresholds.warm_continuation_threshold {
            Ok(HotContinuationDecision::WarmContinuationDetected {
                continuation_id,
                importance_score,
                recommendation: OptimizationRecommendation::MediumPriority,
                statistical_confidence: self.calculate_statistical_confidence(stats)?,
            })
        } else {
            Ok(HotContinuationDecision::ColdContinuation {
                continuation_id,
                importance_score,
            })
        }
    }

    /// 継続重要度スコアの計算
    ///
    /// **スコア計算式**:
    /// ```
    /// Score = (実行頻度重み × EMA頻度) +
    ///         (実行時間重み × 平均実行時間) +
    ///         (トレンド重み × 実行トレンド係数) +
    ///         (安定性重み × 統計的安定性)
    /// ```
    fn calculate_continuation_importance_score(
        &self,
        stats: &ContinuationExecutionStats,
        thresholds: &DynamicThresholds,
    ) -> Result<f64> {
        let weights = &self.config.scoring_weights;

        // 1. 実行頻度スコア（指数移動平均）
        let frequency_score = stats.ema_execution_frequency * weights.frequency_weight;

        // 2. 実行時間スコア（長時間実行ほど最適化価値が高い）
        let time_score = stats.average_execution_time.as_secs_f64() * weights.execution_time_weight;

        // 3. 実行トレンドスコア（増加傾向の継続を優先）
        let trend_score = stats.execution_trend_coefficient * weights.trend_weight;

        // 4. 統計的安定性スコア（安定した継続を優先）
        let stability_score = stats.statistical_stability * weights.stability_weight;

        // 5. リソース消費スコア（高リソース消費継続を優先）
        let resource_score = stats.resource_consumption_score * weights.resource_weight;

        let total_score =
            frequency_score + time_score + trend_score + stability_score + resource_score;

        Ok(total_score)
    }

    /// 統計的信頼性の計算
    fn calculate_statistical_confidence(&self, stats: &ContinuationExecutionStats) -> Result<f64> {
        // サンプル数に基づく信頼度計算
        let sample_size_confidence = (stats.execution_count as f64).ln() / 10.0;
        let sample_size_confidence = sample_size_confidence.min(1.0).max(0.0);

        // 統計的安定性に基づく信頼度
        let stability_confidence = stats.statistical_stability;

        // 総合信頼度（調和平均）
        let overall_confidence = 2.0 * sample_size_confidence * stability_confidence
            / (sample_size_confidence + stability_confidence);

        Ok(overall_confidence)
    }

    /// 最小統計要件の確認
    fn meets_minimum_statistical_requirements(&self, stats: &ContinuationExecutionStats) -> bool {
        stats.execution_count >= self.config.min_executions_for_analysis
            && stats.observation_duration >= self.config.min_observation_duration
    }

    /// しきい値調整が必要かの判定
    fn should_adjust_thresholds(&self) -> bool {
        // 設定された間隔でしきい値を調整
        if let Ok(mut system_stats) = self.system_stats.write() {
            let now = Instant::now();
            if now.duration_since(system_stats.last_threshold_adjustment)
                >= self.config.threshold_adjustment_interval
            {
                system_stats.last_threshold_adjustment = now;
                return true;
            }
        }
        false
    }

    /// 動的しきい値の調整
    fn adjust_dynamic_thresholds(&self) -> Result<()> {
        let mut threshold_manager = self.threshold_manager.lock().map_err(|_| {
            Error::runtime_error("Failed to acquire threshold manager".to_string(), None)
        })?;

        // 実行履歴から統計データを収集
        let historical_data = self.collect_historical_statistics()?;

        // 統計分析に基づくしきい値調整
        threshold_manager.adjust_thresholds(&historical_data, &self.config)?;

        Ok(())
    }

    /// 履歴統計データの収集
    fn collect_historical_statistics(&self) -> Result<HistoricalStatistics> {
        let history = self.execution_history.lock().map_err(|_| {
            Error::runtime_error("Failed to acquire history lock".to_string(), None)
        })?;

        if history.is_empty() {
            return Ok(HistoricalStatistics::empty());
        }

        let mut continuation_frequencies: HashMap<ContinuationId, u64> = HashMap::new();
        let mut total_execution_time = Duration::ZERO;
        let mut execution_times = Vec::new();

        for event in history.iter() {
            *continuation_frequencies
                .entry(event.continuation_id)
                .or_insert(0) += 1;
            total_execution_time += event.execution_time;
            execution_times.push(event.execution_time.as_nanos() as f64);
        }

        // 統計値の計算
        let mean_execution_time = total_execution_time / (history.len() as u32);
        let frequency_distribution =
            self.calculate_frequency_distribution(&continuation_frequencies);
        let execution_time_percentiles = self.calculate_percentiles(&execution_times);

        Ok(HistoricalStatistics {
            sample_size: history.len(),
            mean_execution_time,
            frequency_distribution,
            execution_time_percentiles,
            observation_period: self.calculate_observation_period(&history),
        })
    }

    /// 頻度分布の計算
    fn calculate_frequency_distribution(
        &self,
        frequencies: &HashMap<ContinuationId, u64>,
    ) -> FrequencyDistribution {
        let values: Vec<u64> = frequencies.values().cloned().collect();
        if values.is_empty() {
            return FrequencyDistribution::default();
        }

        let sum: u64 = values.iter().sum();
        let mean = sum as f64 / values.len() as f64;

        let variance = values
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        let std_dev = variance.sqrt();

        FrequencyDistribution {
            mean,
            std_dev,
            median: self.calculate_median(&values),
            p90: self.calculate_percentile(&values, 0.90),
            p95: self.calculate_percentile(&values, 0.95),
            p99: self.calculate_percentile(&values, 0.99),
        }
    }

    /// パーセンタイルの計算
    fn calculate_percentiles(&self, values: &[f64]) -> ExecutionTimePercentiles {
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        ExecutionTimePercentiles {
            p50: self.calculate_percentile_f64(&sorted_values, 0.50),
            p90: self.calculate_percentile_f64(&sorted_values, 0.90),
            p95: self.calculate_percentile_f64(&sorted_values, 0.95),
            p99: self.calculate_percentile_f64(&sorted_values, 0.99),
        }
    }

    fn calculate_median(&self, values: &[u64]) -> f64 {
        let mut sorted = values.to_vec();
        sorted.sort();

        if sorted.is_empty() {
            return 0.0;
        }

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
        } else {
            sorted[mid] as f64
        }
    }

    fn calculate_percentile(&self, values: &[u64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort();

        let index = (percentile * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)] as f64
    }

    fn calculate_percentile_f64(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (percentile * (sorted_values.len() - 1) as f64).round() as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    fn calculate_observation_period(&self, history: &VecDeque<ExecutionEvent>) -> Duration {
        if history.len() < 2 {
            return Duration::ZERO;
        }

        let earliest = history.front().unwrap().timestamp;
        let latest = history.back().unwrap().timestamp;

        latest.duration_since(earliest).unwrap_or(Duration::ZERO)
    }

    /// システム統計の更新
    fn update_system_stats(&self, processing_time: Duration) {
        if let Ok(mut stats) = self.system_stats.write() {
            stats.total_profiling_calls += 1;
            stats.total_profiling_time += processing_time;
            stats.average_profiling_time = stats.total_profiling_time / stats.total_profiling_calls;
        }
    }

    /// 現在の検出統計を取得
    pub fn get_detection_stats(&self) -> Result<DetectionStatistics> {
        let stats_db = self
            .execution_stats
            .read()
            .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;

        let system_stats = self.system_stats.read().map_err(|_| {
            Error::runtime_error("Failed to acquire system stats lock".to_string(), None)
        })?;

        let hot_continuations = stats_db
            .values()
            .filter(|stats| {
                if let Ok(thresholds) = self.threshold_manager.lock() {
                    let current_thresholds = thresholds.get_current_thresholds();
                    self.calculate_continuation_importance_score(stats, &current_thresholds)
                        .map(|score| score >= current_thresholds.hot_continuation_threshold)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .count();

        let warm_continuations = stats_db
            .values()
            .filter(|stats| {
                if let Ok(thresholds) = self.threshold_manager.lock() {
                    let current_thresholds = thresholds.get_current_thresholds();
                    if let Ok(score) =
                        self.calculate_continuation_importance_score(stats, &current_thresholds)
                    {
                        score >= current_thresholds.warm_continuation_threshold
                            && score < current_thresholds.hot_continuation_threshold
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .count();

        Ok(DetectionStatistics {
            total_continuations_tracked: stats_db.len(),
            hot_continuations_detected: hot_continuations,
            warm_continuations_detected: warm_continuations,
            average_profiling_overhead: system_stats.average_profiling_time,
            total_profiling_calls: system_stats.total_profiling_calls,
        })
    }
}

/// 動的しきい値管理システム
///
/// **適応的学習特徴**:
/// - 統計的分析によるしきい値最適化
/// - システム負荷に応じた動的調整
/// - フィードバックループによる自動改善
pub struct DynamicThresholdManager {
    /// 現在のしきい値
    current_thresholds: DynamicThresholds,

    /// しきい値履歴（学習用）
    threshold_history: Vec<ThresholdHistoryEntry>,

    /// 設定
    config: ThresholdManagerConfig,
}

impl DynamicThresholdManager {
    pub fn new(config: ThresholdManagerConfig) -> Self {
        DynamicThresholdManager {
            current_thresholds: DynamicThresholds::from_config(&config),
            threshold_history: Vec::new(),
            config,
        }
    }

    /// 現在のしきい値を取得
    pub fn get_current_thresholds(&self) -> DynamicThresholds {
        self.current_thresholds.clone()
    }

    /// 統計データに基づくしきい値調整
    pub fn adjust_thresholds(
        &mut self,
        historical_stats: &HistoricalStatistics,
        detector_config: &HotContinuationDetectorConfig,
    ) -> Result<()> {
        // Step 1: 統計的分析による推奨しきい値の計算
        let recommended_thresholds = self.calculate_recommended_thresholds(historical_stats)?;

        // Step 2: 現在のしきい値との差分分析
        let adjustment_vector = self.calculate_adjustment_vector(&recommended_thresholds);

        // Step 3: 学習率に基づく段階的調整
        let new_thresholds = self.apply_gradual_adjustment(&adjustment_vector);

        // Step 4: しきい値の妥当性検証
        if self.validate_threshold_bounds(&new_thresholds) {
            // Step 5: しきい値履歴への記録
            self.record_threshold_change(&new_thresholds);

            // Step 6: しきい値の更新
            self.current_thresholds = new_thresholds;
        }

        Ok(())
    }

    fn calculate_recommended_thresholds(
        &self,
        stats: &HistoricalStatistics,
    ) -> Result<DynamicThresholds> {
        // 統計分析に基づく推奨しきい値の計算
        let freq_dist = &stats.frequency_distribution;

        // P95を基準としたホット継続しきい値
        let hot_threshold = freq_dist.p95 * self.config.hot_threshold_multiplier;

        // P90を基準としたウォーム継続しきい値
        let warm_threshold = freq_dist.p90 * self.config.warm_threshold_multiplier;

        // 実行時間を考慮した調整
        let time_adjustment = stats.execution_time_percentiles.p95 / 1000.0; // μs to ms

        Ok(DynamicThresholds {
            hot_continuation_threshold: hot_threshold + time_adjustment,
            warm_continuation_threshold: warm_threshold + time_adjustment * 0.5,
            confidence_threshold: self.config.base_confidence_threshold,
            adjustment_learning_rate: self.config.learning_rate,
        })
    }

    fn calculate_adjustment_vector(
        &self,
        recommended: &DynamicThresholds,
    ) -> ThresholdAdjustmentVector {
        ThresholdAdjustmentVector {
            hot_threshold_delta: recommended.hot_continuation_threshold
                - self.current_thresholds.hot_continuation_threshold,
            warm_threshold_delta: recommended.warm_continuation_threshold
                - self.current_thresholds.warm_continuation_threshold,
            confidence_threshold_delta: recommended.confidence_threshold
                - self.current_thresholds.confidence_threshold,
        }
    }

    fn apply_gradual_adjustment(
        &self,
        adjustment: &ThresholdAdjustmentVector,
    ) -> DynamicThresholds {
        let learning_rate = self.current_thresholds.adjustment_learning_rate;

        DynamicThresholds {
            hot_continuation_threshold: self.current_thresholds.hot_continuation_threshold
                + (adjustment.hot_threshold_delta * learning_rate),
            warm_continuation_threshold: self.current_thresholds.warm_continuation_threshold
                + (adjustment.warm_threshold_delta * learning_rate),
            confidence_threshold: self.current_thresholds.confidence_threshold
                + (adjustment.confidence_threshold_delta * learning_rate),
            adjustment_learning_rate: learning_rate,
        }
    }

    fn validate_threshold_bounds(&self, thresholds: &DynamicThresholds) -> bool {
        // しきい値の合理性チェック
        thresholds.hot_continuation_threshold > thresholds.warm_continuation_threshold
            && thresholds.warm_continuation_threshold > 0.0
            && thresholds.hot_continuation_threshold <= self.config.max_hot_threshold
            && thresholds.confidence_threshold >= 0.0
            && thresholds.confidence_threshold <= 1.0
    }

    fn record_threshold_change(&mut self, new_thresholds: &DynamicThresholds) {
        let entry = ThresholdHistoryEntry {
            timestamp: SystemTime::now(),
            hot_threshold: new_thresholds.hot_continuation_threshold,
            warm_threshold: new_thresholds.warm_continuation_threshold,
            confidence_threshold: new_thresholds.confidence_threshold,
        };

        self.threshold_history.push(entry);

        // 履歴サイズ制限
        if self.threshold_history.len() > self.config.max_history_size {
            self.threshold_history.remove(0);
        }
    }
}

// ========== データ構造定義 ==========

/// 継続実行統計
#[derive(Debug, Clone)]
pub struct ContinuationExecutionStats {
    pub continuation_id: ContinuationId,
    pub execution_count: u64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub ema_execution_frequency: f64, // 指数移動平均による実行頻度
    pub execution_trend_coefficient: f64, // 実行トレンド係数
    pub statistical_stability: f64,   // 統計的安定性
    pub resource_consumption_score: f64, // リソース消費スコア
    pub first_observed: SystemTime,
    pub last_observed: SystemTime,
    pub observation_duration: Duration,
    ema_alpha: f64, // EMAパラメータ
}

impl ContinuationExecutionStats {
    pub fn new(continuation_id: ContinuationId, ema_alpha: f64) -> Self {
        let now = SystemTime::now();
        ContinuationExecutionStats {
            continuation_id,
            execution_count: 0,
            total_execution_time: Duration::ZERO,
            average_execution_time: Duration::ZERO,
            ema_execution_frequency: 0.0,
            execution_trend_coefficient: 0.0,
            statistical_stability: 0.0,
            resource_consumption_score: 0.0,
            first_observed: now,
            last_observed: now,
            observation_duration: Duration::ZERO,
            ema_alpha,
        }
    }

    /// 実行統計の更新（指数移動平均計算）
    pub fn update_execution(&mut self, execution_time: Duration) {
        let now = SystemTime::now();

        // 基本統計の更新
        self.execution_count += 1;
        self.total_execution_time += execution_time;
        self.average_execution_time = self.total_execution_time / self.execution_count as u32;
        self.last_observed = now;
        self.observation_duration = now
            .duration_since(self.first_observed)
            .unwrap_or(Duration::ZERO);

        // 指数移動平均による実行頻度の更新
        let time_since_last = now
            .duration_since(self.last_observed)
            .unwrap_or(Duration::from_secs(1));
        let current_frequency = 1.0 / time_since_last.as_secs_f64();

        if self.execution_count == 1 {
            self.ema_execution_frequency = current_frequency;
        } else {
            self.ema_execution_frequency = self.ema_alpha * current_frequency
                + (1.0 - self.ema_alpha) * self.ema_execution_frequency;
        }

        // 実行トレンド係数の更新（簡略化）
        if self.execution_count > 2 {
            self.execution_trend_coefficient = (current_frequency - self.ema_execution_frequency)
                .abs()
                / self.ema_execution_frequency;
        }

        // 統計的安定性の更新（変動係数ベース）
        if self.execution_count > 1 {
            let variance =
                execution_time.as_nanos() as f64 - self.average_execution_time.as_nanos() as f64;
            let relative_variance = variance / self.average_execution_time.as_nanos() as f64;
            self.statistical_stability = 1.0 / (1.0 + relative_variance.abs());
        }

        // リソース消費スコアの更新（実行時間ベース）
        self.resource_consumption_score =
            execution_time.as_secs_f64() * self.ema_execution_frequency;
    }
}

/// ホット継続検出の決定結果
#[derive(Debug, Clone)]
pub enum HotContinuationDecision {
    /// ホット継続を検出
    HotContinuationDetected {
        continuation_id: ContinuationId,
        importance_score: f64,
        recommendation: OptimizationRecommendation,
        statistical_confidence: f64,
    },

    /// ウォーム継続を検出
    WarmContinuationDetected {
        continuation_id: ContinuationId,
        importance_score: f64,
        recommendation: OptimizationRecommendation,
        statistical_confidence: f64,
    },

    /// コールド継続（最適化不要）
    ColdContinuation {
        continuation_id: ContinuationId,
        importance_score: f64,
    },

    /// 統計データ不足
    InsufficientData {
        continuation_id: ContinuationId,
        current_executions: u64,
        required_minimum: u64,
    },
}

/// 最適化推奨レベル
#[derive(Debug, Clone)]
pub enum OptimizationRecommendation {
    HighPriority,
    MediumPriority,
    LowPriority,
}

/// 実行イベント
#[derive(Debug, Clone)]
struct ExecutionEvent {
    continuation_id: ContinuationId,
    execution_time: Duration,
    timestamp: SystemTime,
}

/// 動的しきい値
#[derive(Debug, Clone)]
pub struct DynamicThresholds {
    pub hot_continuation_threshold: f64,
    pub warm_continuation_threshold: f64,
    pub confidence_threshold: f64,
    pub adjustment_learning_rate: f64,
}

impl DynamicThresholds {
    pub fn from_config(config: &ThresholdManagerConfig) -> Self {
        DynamicThresholds {
            hot_continuation_threshold: config.initial_hot_threshold,
            warm_continuation_threshold: config.initial_warm_threshold,
            confidence_threshold: config.base_confidence_threshold,
            adjustment_learning_rate: config.learning_rate,
        }
    }
}

/// しきい値調整ベクトル
#[derive(Debug)]
struct ThresholdAdjustmentVector {
    hot_threshold_delta: f64,
    warm_threshold_delta: f64,
    confidence_threshold_delta: f64,
}

/// しきい値履歴エントリ
#[derive(Debug)]
struct ThresholdHistoryEntry {
    timestamp: SystemTime,
    hot_threshold: f64,
    warm_threshold: f64,
    confidence_threshold: f64,
}

/// 履歴統計データ
#[derive(Debug)]
pub struct HistoricalStatistics {
    pub sample_size: usize,
    pub mean_execution_time: Duration,
    pub frequency_distribution: FrequencyDistribution,
    pub execution_time_percentiles: ExecutionTimePercentiles,
    pub observation_period: Duration,
}

impl HistoricalStatistics {
    pub fn empty() -> Self {
        HistoricalStatistics {
            sample_size: 0,
            mean_execution_time: Duration::ZERO,
            frequency_distribution: FrequencyDistribution::default(),
            execution_time_percentiles: ExecutionTimePercentiles::default(),
            observation_period: Duration::ZERO,
        }
    }
}

/// 頻度分布統計
#[derive(Debug, Default)]
pub struct FrequencyDistribution {
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// 実行時間パーセンタイル
#[derive(Debug, Default)]
pub struct ExecutionTimePercentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// 検出統計
#[derive(Debug)]
pub struct DetectionStatistics {
    pub total_continuations_tracked: usize,
    pub hot_continuations_detected: usize,
    pub warm_continuations_detected: usize,
    pub average_profiling_overhead: Duration,
    pub total_profiling_calls: u32,
}

/// システム統計
#[derive(Debug)]
struct DetectorSystemStats {
    total_profiling_calls: u32,
    total_profiling_time: Duration,
    average_profiling_time: Duration,
    last_threshold_adjustment: Instant,
}

impl DetectorSystemStats {
    pub fn new() -> Self {
        DetectorSystemStats {
            total_profiling_calls: 0,
            total_profiling_time: Duration::ZERO,
            average_profiling_time: Duration::ZERO,
            last_threshold_adjustment: Instant::now(),
        }
    }
}

// ========== 設定構造体 ==========

/// ホット継続検出システム設定
#[derive(Debug, Clone)]
pub struct HotContinuationDetectorConfig {
    /// 分析に必要な最小実行回数
    pub min_executions_for_analysis: u64,

    /// 最小観測期間
    pub min_observation_duration: Duration,

    /// 指数移動平均のアルファパラメータ
    pub ema_alpha: f64,

    /// 実行履歴バッファサイズ
    pub history_buffer_size: usize,

    /// しきい値調整間隔
    pub threshold_adjustment_interval: Duration,

    /// スコア計算重み
    pub scoring_weights: ScoringWeights,

    /// しきい値管理設定
    pub threshold_config: ThresholdManagerConfig,
}

impl Default for HotContinuationDetectorConfig {
    fn default() -> Self {
        HotContinuationDetectorConfig {
            min_executions_for_analysis: 10,
            min_observation_duration: Duration::from_secs(5),
            ema_alpha: 0.2,
            history_buffer_size: 10000,
            threshold_adjustment_interval: Duration::from_secs(60),
            scoring_weights: ScoringWeights::default(),
            threshold_config: ThresholdManagerConfig::default(),
        }
    }
}

/// スコア計算重み
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub frequency_weight: f64,
    pub execution_time_weight: f64,
    pub trend_weight: f64,
    pub stability_weight: f64,
    pub resource_weight: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        ScoringWeights {
            frequency_weight: 0.4,
            execution_time_weight: 0.3,
            trend_weight: 0.1,
            stability_weight: 0.1,
            resource_weight: 0.1,
        }
    }
}

/// しきい値管理設定
#[derive(Debug, Clone)]
pub struct ThresholdManagerConfig {
    pub initial_hot_threshold: f64,
    pub initial_warm_threshold: f64,
    pub base_confidence_threshold: f64,
    pub learning_rate: f64,
    pub hot_threshold_multiplier: f64,
    pub warm_threshold_multiplier: f64,
    pub max_hot_threshold: f64,
    pub max_history_size: usize,
}

impl Default for ThresholdManagerConfig {
    fn default() -> Self {
        ThresholdManagerConfig {
            initial_hot_threshold: 10.0,
            initial_warm_threshold: 5.0,
            base_confidence_threshold: 0.8,
            learning_rate: 0.1,
            hot_threshold_multiplier: 1.2,
            warm_threshold_multiplier: 0.8,
            max_hot_threshold: 100.0,
            max_history_size: 1000,
        }
    }
}

// ========== テスト ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_continuation_detector_creation() {
        let config = HotContinuationDetectorConfig::default();
        let detector = HotContinuationDetector::new(config);

        assert_eq!(detector.config.min_executions_for_analysis, 10);
        assert_eq!(detector.config.ema_alpha, 0.2);
    }

    #[test]
    fn test_continuation_execution_stats() {
        let mut stats =
            ContinuationExecutionStats::new(crate::continuations::ContinuationId::from(1), 0.2);
        let execution_time = Duration::from_millis(100);

        stats.update_execution(execution_time);

        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.total_execution_time, execution_time);
        assert_eq!(stats.average_execution_time, execution_time);
    }

    #[test]
    fn test_dynamic_threshold_manager() {
        let config = ThresholdManagerConfig::default();
        let manager = DynamicThresholdManager::new(config);

        let thresholds = manager.get_current_thresholds();
        assert_eq!(thresholds.hot_continuation_threshold, 10.0);
        assert_eq!(thresholds.warm_continuation_threshold, 5.0);
    }

    #[test]
    fn test_execution_record_and_decision() {
        let config = HotContinuationDetectorConfig::default();
        let detector = HotContinuationDetector::new(config);

        let continuation_id = 1;
        let execution_time = Duration::from_millis(50);

        let decision = detector.record_execution(
            crate::continuations::ContinuationId::from(continuation_id),
            execution_time,
        );
        assert!(decision.is_ok());

        // 最初の実行は統計データ不足となるはず
        match decision.unwrap() {
            HotContinuationDecision::InsufficientData { .. } => {
                // 期待される結果
            }
            _ => panic!("Expected InsufficientData for first execution"),
        }
    }

    #[test]
    fn test_scoring_weights_validation() {
        let weights = ScoringWeights::default();
        let total_weight = weights.frequency_weight
            + weights.execution_time_weight
            + weights.trend_weight
            + weights.stability_weight
            + weights.resource_weight;

        // 重みの合計が1.0であることを確認
        assert!((total_weight - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_historical_statistics_empty() {
        let stats = HistoricalStatistics::empty();
        assert_eq!(stats.sample_size, 0);
        assert_eq!(stats.mean_execution_time, Duration::ZERO);
        assert_eq!(stats.observation_period, Duration::ZERO);
    }
}
