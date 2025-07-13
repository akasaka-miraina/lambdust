//! Branch Prediction Analysis Module
//!
//! このモジュールは分岐予測システムを実装します。
//! 分岐履歴追跡、適応予測、相関解析を含みます。

use crate::error::Result;
use super::core_types::{
    BranchHistory, TwoLevelAdaptivePredictor, BranchCorrelationAnalyzer, MispredictionCostAnalyzer,
};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Branch prediction analysis system
#[derive(Debug)]
pub struct BranchPredictor {
    /// Branch history table
    pub branch_history: HashMap<String, BranchHistory>,
    
    /// Two-level adaptive predictor
    pub adaptive_predictor: TwoLevelAdaptivePredictor,
    
    /// Branch correlation analysis
    pub correlation_analyzer: BranchCorrelationAnalyzer,
    
    /// Misprediction cost analysis
    pub misprediction_analyzer: MispredictionCostAnalyzer,
}

impl BranchPredictor {
    #[must_use] 
    pub fn new() -> Self { 
        Self { 
            branch_history: HashMap::new(), 
            adaptive_predictor: TwoLevelAdaptivePredictor, 
            correlation_analyzer: BranchCorrelationAnalyzer, 
            misprediction_analyzer: MispredictionCostAnalyzer,
        } 
    }
    
    pub fn record_branch(&mut self, expr_hash: &str, outcome: bool) -> Result<()> {
        let history = self.branch_history.entry(expr_hash.to_string()).or_insert_with(|| {
            BranchHistory {
                outcomes: VecDeque::new(),
                addresses: VecDeque::new(),
                prediction_accuracy: 1.0,
                misprediction_penalty: Duration::from_nanos(10),
            }
        });
        
        history.outcomes.push_back(outcome);
        history.addresses.push_back(expr_hash.to_string());
        
        // Keep history within reasonable bounds
        if history.outcomes.len() > 1000 {
            history.outcomes.pop_front();
            history.addresses.pop_front();
        }
        
        // Update prediction accuracy
        Self::update_prediction_accuracy_static(history);
        
        Ok(())
    }
    
    /// Update prediction accuracy based on recent history
    fn update_prediction_accuracy_static(history: &mut BranchHistory) {
        if history.outcomes.len() < 2 {
            return;
        }
        
        // Simple predictor: predict based on last outcome
        let mut correct_predictions = 0;
        let mut total_predictions = 0;
        
        for window in history.outcomes.iter().collect::<Vec<_>>().windows(2) {
            let predicted = *window[0]; // Predict same as previous
            let actual = *window[1];
            
            if predicted == actual {
                correct_predictions += 1;
            }
            total_predictions += 1;
        }
        
        if total_predictions > 0 {
            history.prediction_accuracy = correct_predictions as f64 / total_predictions as f64;
        }
    }
    
    pub fn calculate_overall_accuracy(&self) -> f64 {
        if self.branch_history.is_empty() {
            return 1.0;
        }
        
        let total_accuracy: f64 = self.branch_history.values()
            .map(|history| history.prediction_accuracy)
            .sum();
        
        total_accuracy / self.branch_history.len() as f64
    }
    
    /// Predict next branch outcome for an expression
    pub fn predict_branch(&self, expr_hash: &str) -> Option<BranchPrediction> {
        let history = self.branch_history.get(expr_hash)?;
        
        if history.outcomes.is_empty() {
            return None;
        }
        
        // Simple prediction strategies
        let last_outcome_prediction = *history.outcomes.back()?;
        let majority_prediction = self.calculate_majority_outcome(history);
        let pattern_prediction = self.detect_pattern_prediction(history);
        
        // Weight different prediction methods
        let final_prediction = if let Some(pattern) = pattern_prediction {
            pattern
        } else if history.prediction_accuracy > 0.7 {
            last_outcome_prediction
        } else {
            majority_prediction
        };
        
        Some(BranchPrediction {
            predicted_outcome: final_prediction,
            confidence: history.prediction_accuracy,
            prediction_method: if pattern_prediction.is_some() {
                PredictionMethod::Pattern
            } else if history.prediction_accuracy > 0.7 {
                PredictionMethod::LastOutcome
            } else {
                PredictionMethod::Majority
            },
        })
    }
    
    /// Calculate the majority outcome in recent history
    fn calculate_majority_outcome(&self, history: &BranchHistory) -> bool {
        let recent_count = 20.min(history.outcomes.len());
        let recent_outcomes: Vec<bool> = history.outcomes.iter()
            .rev()
            .take(recent_count)
            .copied()
            .collect();
        
        let true_count = recent_outcomes.iter().filter(|&&x| x).count();
        true_count > recent_outcomes.len() / 2
    }
    
    /// Detect repeating patterns in branch outcomes
    fn detect_pattern_prediction(&self, history: &BranchHistory) -> Option<bool> {
        if history.outcomes.len() < 4 {
            return None;
        }
        
        let outcomes: Vec<bool> = history.outcomes.iter().copied().collect();
        
        // Check for simple alternating pattern
        if outcomes.len() >= 4 {
            let last_four = &outcomes[outcomes.len()-4..];
            if last_four[0] != last_four[1] && 
               last_four[1] != last_four[2] && 
               last_four[2] != last_four[3] &&
               last_four[0] == last_four[2] {
                // Alternating pattern detected
                return Some(!last_four[3]);
            }
        }
        
        // Check for repeating sequence of length 2
        if outcomes.len() >= 6 {
            let last_six = &outcomes[outcomes.len()-6..];
            if last_six[0] == last_six[2] && last_six[2] == last_six[4] &&
               last_six[1] == last_six[3] && last_six[3] == last_six[5] {
                // Pattern of length 2 detected
                return Some(last_six[0]);
            }
        }
        
        None
    }
    
    /// Get branch statistics for all expressions
    pub fn get_branch_statistics(&self) -> BranchStatistics {
        let total_expressions = self.branch_history.len();
        let total_branches: usize = self.branch_history.values()
            .map(|history| history.outcomes.len())
            .sum();
        
        let average_accuracy = self.calculate_overall_accuracy();
        
        let misprediction_rate = 1.0 - average_accuracy;
        
        let total_penalty: Duration = self.branch_history.values()
            .map(|history| history.misprediction_penalty)
            .sum();
        
        let average_misprediction_penalty = if total_expressions > 0 {
            total_penalty / total_expressions as u32
        } else {
            Duration::ZERO
        };
        
        BranchStatistics {
            total_expressions_tracked: total_expressions,
            total_branches_predicted: total_branches,
            overall_prediction_accuracy: average_accuracy,
            overall_misprediction_rate: misprediction_rate,
            average_misprediction_penalty,
        }
    }
    
    /// Identify branch hotspots (expressions with many branches and low accuracy)
    pub fn identify_branch_hotspots(&self, min_branches: usize, max_accuracy: f64) -> Vec<BranchHotspot> {
        let mut hotspots = Vec::new();
        
        for (expr_hash, history) in &self.branch_history {
            if history.outcomes.len() >= min_branches && history.prediction_accuracy <= max_accuracy {
                hotspots.push(BranchHotspot {
                    expression: expr_hash.clone(),
                    total_branches: history.outcomes.len(),
                    prediction_accuracy: history.prediction_accuracy,
                    misprediction_penalty: history.misprediction_penalty,
                    optimization_potential: self.calculate_optimization_potential(history),
                });
            }
        }
        
        // Sort by optimization potential (descending)
        hotspots.sort_by(|a, b| b.optimization_potential.partial_cmp(&a.optimization_potential)
                              .unwrap_or(std::cmp::Ordering::Equal));
        hotspots
    }
    
    /// Calculate optimization potential for a branch
    fn calculate_optimization_potential(&self, history: &BranchHistory) -> f64 {
        let frequency_factor = (history.outcomes.len() as f64).ln() / 10.0; // Log scale
        let accuracy_factor = 1.0 - history.prediction_accuracy;
        let penalty_factor = history.misprediction_penalty.as_nanos() as f64 / 1_000_000.0; // Convert to ms
        
        frequency_factor * accuracy_factor * penalty_factor
    }
}

/// Branch prediction result
#[derive(Debug, Clone)]
pub struct BranchPrediction {
    /// Predicted branch outcome
    pub predicted_outcome: bool,
    
    /// Confidence in prediction (0.0-1.0)
    pub confidence: f64,
    
    /// Method used for prediction
    pub prediction_method: PredictionMethod,
}

/// Branch prediction methods
#[derive(Debug, Clone, PartialEq)]
pub enum PredictionMethod {
    /// Predict based on last outcome
    LastOutcome,
    
    /// Predict based on majority of recent outcomes
    Majority,
    
    /// Predict based on detected pattern
    Pattern,
    
    /// Predict using correlation with other branches
    Correlation,
    
    /// Two-level adaptive prediction
    TwoLevel,
}

/// Branch prediction statistics
#[derive(Debug, Clone)]
pub struct BranchStatistics {
    /// Total expressions with branch tracking
    pub total_expressions_tracked: usize,
    
    /// Total branches predicted
    pub total_branches_predicted: usize,
    
    /// Overall prediction accuracy
    pub overall_prediction_accuracy: f64,
    
    /// Overall misprediction rate
    pub overall_misprediction_rate: f64,
    
    /// Average penalty for mispredictions
    pub average_misprediction_penalty: Duration,
}

/// Branch hotspot information
#[derive(Debug, Clone)]
pub struct BranchHotspot {
    /// Expression identifier
    pub expression: String,
    
    /// Total number of branches
    pub total_branches: usize,
    
    /// Current prediction accuracy
    pub prediction_accuracy: f64,
    
    /// Misprediction penalty
    pub misprediction_penalty: Duration,
    
    /// Optimization potential score
    pub optimization_potential: f64,
}

impl Default for BranchPredictor {
    fn default() -> Self {
        Self::new()
    }
}