//! Configuration types for performance measurement system
//!
//! This module defines configuration structures that control how
//! performance measurements are conducted and reported.

use super::core_types::{MeasurementPrecision, StatisticsLevel};
use std::time::Duration;

/// 測定設定
#[derive(Debug, Clone)]
pub struct MeasurementConfiguration {
    /// 測定間隔
    pub measurement_interval: Duration,

    /// 測定精度
    pub measurement_precision: MeasurementPrecision,

    /// 統計収集レベル
    pub statistics_level: StatisticsLevel,

    /// 自動ベンチマーク実行
    pub auto_benchmark: bool,

    /// 結果出力設定
    pub output_config: OutputConfiguration,

    /// 履歴保持期間
    pub history_retention: Duration,

    /// ウォームアップ設定
    pub warmup_config: WarmupConfiguration,

    /// 異常値検出設定
    pub outlier_detection_config: OutlierDetectionConfiguration,

    /// 比較ベースライン設定
    pub baseline_config: BaselineConfiguration,
}

/// 出力設定
#[derive(Debug, Clone)]
pub struct OutputConfiguration {
    /// 出力フォーマット
    pub format: OutputFormat,

    /// 出力先
    pub destination: OutputDestination,

    /// 詳細レベル
    pub verbosity: VerbosityLevel,

    /// リアルタイム出力
    pub realtime_output: bool,

    /// グラフ生成
    pub generate_graphs: bool,

    /// CSV出力
    pub csv_output: bool,

    /// JSON出力
    pub json_output: bool,
}

/// 出力フォーマット
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// プレーンテキスト
    PlainText,
    /// マークダウン
    Markdown,
    /// HTML
    Html,
    /// JSON
    Json,
    /// CSV
    Csv,
    /// バイナリ
    Binary,
}

/// 出力先
#[derive(Debug, Clone)]
pub enum OutputDestination {
    /// 標準出力
    Stdout,
    /// 標準エラー出力
    Stderr,
    /// ファイル
    File(String),
    /// 複数ファイル
    MultipleFiles { 
        /// ベースパス
        base_path: String, 
        /// ファイルフォーマット
        format: String 
    },
    /// メモリ内
    Memory,
}

/// 詳細レベル
#[derive(Debug, Clone)]
pub enum VerbosityLevel {
    /// 最小出力
    Minimal,
    /// 標準出力
    Standard,
    /// 詳細出力
    Verbose,
    /// デバッグ出力
    Debug,
}

/// ウォームアップ設定
#[derive(Debug, Clone)]
pub struct WarmupConfiguration {
    /// ウォームアップ有効
    pub enabled: bool,

    /// ウォームアップ回数
    pub iterations: usize,

    /// ウォームアップ期間
    pub duration: Option<Duration>,

    /// JITウォームアップ
    pub jit_warmup: bool,

    /// キャッシュウォームアップ
    pub cache_warmup: bool,
}

/// 異常値検出設定
#[derive(Debug, Clone)]
pub struct OutlierDetectionConfiguration {
    /// 検出有効
    pub enabled: bool,

    /// 検出手法
    pub method: OutlierDetectionMethod,

    /// 閾値設定
    pub threshold_config: ThresholdConfiguration,

    /// 異常値処理
    pub outlier_handling: OutlierHandling,
}

/// 異常値検出手法
#[derive(Debug, Clone)]
pub enum OutlierDetectionMethod {
    /// 標準偏差ベース
    StandardDeviation { 
        /// 標準偏差の倍数
        multiplier: f64 
    },
    /// 四分位範囲ベース
    InterquartileRange { 
        /// 四分位範囲の倍数
        multiplier: f64 
    },
    /// 修正Zスコア
    ModifiedZScore { 
        /// Zスコア闾値
        threshold: f64 
    },
    /// パーセンタイルベース
    Percentile { 
        /// 下位パーセンタイル
        lower: f64, 
        /// 上位パーセンタイル
        upper: f64 
    },
}

/// 閾値設定
#[derive(Debug, Clone)]
pub struct ThresholdConfiguration {
    /// 上限閾値
    pub upper_threshold: Option<f64>,

    /// 下限閾値
    pub lower_threshold: Option<f64>,

    /// 相対閾値
    pub relative_threshold: Option<f64>,

    /// 動的閾値
    pub dynamic_threshold: bool,
}

/// 異常値処理方法
#[derive(Debug, Clone)]
pub enum OutlierHandling {
    /// 除外
    Exclude,
    /// 記録保持
    KeepAndFlag,
    /// 修正
    Correct,
    /// 警告のみ
    WarnOnly,
}

/// ベースライン設定
#[derive(Debug, Clone)]
pub struct BaselineConfiguration {
    /// ベースライン使用
    pub enabled: bool,

    /// ベースライン種別
    pub baseline_type: BaselineType,

    /// 比較対象
    pub comparison_target: ComparisonTarget,

    /// 許容差異
    pub tolerance: ToleranceConfiguration,
}

/// ベースライン種別
#[derive(Debug, Clone)]
pub enum BaselineType {
    /// 固定ベースライン
    Fixed { 
        /// ベースライン値
        value: f64 
    },
    /// 履歴ベースライン
    Historical { 
        /// 履歴期間
        period: Duration 
    },
    /// 動的ベースライン
    Dynamic { 
        /// アルゴリズム名
        algorithm: String 
    },
    /// 外部ベースライン
    External { 
        /// データソース
        source: String 
    },
}

/// 比較対象
#[derive(Debug, Clone)]
pub enum ComparisonTarget {
    /// 前回測定
    Previous,
    /// 平均値
    Average { 
        /// 平均化期間
        period: Duration 
    },
    /// ベスト値
    Best,
    /// 指定値
    Specified { 
        /// 指定値
        value: f64 
    },
}

/// 許容差異設定
#[derive(Debug, Clone)]
pub struct ToleranceConfiguration {
    /// 許容パーセンテージ
    pub percentage: Option<f64>,

    /// 許容絶対値
    pub absolute: Option<f64>,

    /// 動的許容値
    pub dynamic: bool,
}

impl Default for MeasurementConfiguration {
    fn default() -> Self {
        Self {
            measurement_interval: Duration::from_millis(100),
            measurement_precision: MeasurementPrecision::Standard,
            statistics_level: StatisticsLevel::Detailed,
            auto_benchmark: true,
            output_config: OutputConfiguration::default(),
            history_retention: Duration::from_secs(24 * 60 * 60), // 24 hours
            warmup_config: WarmupConfiguration::default(),
            outlier_detection_config: OutlierDetectionConfiguration::default(),
            baseline_config: BaselineConfiguration::default(),
        }
    }
}

impl Default for OutputConfiguration {
    fn default() -> Self {
        Self {
            format: OutputFormat::PlainText,
            destination: OutputDestination::Stdout,
            verbosity: VerbosityLevel::Standard,
            realtime_output: false,
            generate_graphs: false,
            csv_output: false,
            json_output: false,
        }
    }
}

impl Default for WarmupConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            iterations: 10,
            duration: Some(Duration::from_millis(500)),
            jit_warmup: true,
            cache_warmup: true,
        }
    }
}

impl Default for OutlierDetectionConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            method: OutlierDetectionMethod::StandardDeviation { multiplier: 2.0 },
            threshold_config: ThresholdConfiguration::default(),
            outlier_handling: OutlierHandling::KeepAndFlag,
        }
    }
}

impl Default for ThresholdConfiguration {
    fn default() -> Self {
        Self {
            upper_threshold: None,
            lower_threshold: None,
            relative_threshold: Some(2.0),
            dynamic_threshold: false,
        }
    }
}

impl Default for BaselineConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            baseline_type: BaselineType::Historical {
                period: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            },
            comparison_target: ComparisonTarget::Average {
                period: Duration::from_secs(24 * 60 * 60), // 24 hours
            },
            tolerance: ToleranceConfiguration::default(),
        }
    }
}

impl Default for ToleranceConfiguration {
    fn default() -> Self {
        Self {
            percentage: Some(5.0), // 5% tolerance
            absolute: None,
            dynamic: false,
        }
    }
}