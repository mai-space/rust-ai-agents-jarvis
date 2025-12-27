use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Metrics for tracking agent handoff health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffMetrics {
    /// Total number of handoffs
    pub total_handoffs: usize,
    /// Handoffs per agent pair (from -> to)
    pub handoff_pairs: HashMap<String, usize>,
    /// Average handoff chain length
    pub avg_chain_length: f64,
    /// Maximum handoff chain length observed
    pub max_chain_length: usize,
    /// Number of loop incidents detected
    pub loop_incidents: usize,
    /// Number of human interventions triggered
    pub human_interventions: usize,
    /// Successful task completions
    pub successful_completions: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Timestamp of last update
    pub last_updated: DateTime<Utc>,
}

impl Default for HandoffMetrics {
    fn default() -> Self {
        Self {
            total_handoffs: 0,
            handoff_pairs: HashMap::new(),
            avg_chain_length: 0.0,
            max_chain_length: 0,
            loop_incidents: 0,
            human_interventions: 0,
            successful_completions: 0,
            failed_tasks: 0,
            last_updated: Utc::now(),
        }
    }
}

impl HandoffMetrics {
    /// Record a handoff from one agent to another
    pub fn record_handoff(&mut self, from: &str, to: &str) {
        self.total_handoffs += 1;
        let pair = format!("{} -> {}", from, to);
        *self.handoff_pairs.entry(pair).or_insert(0) += 1;
        self.last_updated = Utc::now();
    }
    
    /// Record a loop incident
    pub fn record_loop_incident(&mut self) {
        self.loop_incidents += 1;
        self.last_updated = Utc::now();
    }
    
    /// Record a human intervention
    pub fn record_human_intervention(&mut self) {
        self.human_interventions += 1;
        self.last_updated = Utc::now();
    }
    
    /// Record a successful task completion with the chain length
    pub fn record_success(&mut self, chain_length: usize) {
        self.successful_completions += 1;
        if chain_length > self.max_chain_length {
            self.max_chain_length = chain_length;
        }
        // Update running average
        let total = self.successful_completions + self.failed_tasks;
        self.avg_chain_length = (self.avg_chain_length * (total - 1) as f64 + chain_length as f64) / total as f64;
        self.last_updated = Utc::now();
    }
    
    /// Record a failed task with the chain length
    pub fn record_failure(&mut self, chain_length: usize) {
        self.failed_tasks += 1;
        if chain_length > self.max_chain_length {
            self.max_chain_length = chain_length;
        }
        // Update running average
        let total = self.successful_completions + self.failed_tasks;
        self.avg_chain_length = (self.avg_chain_length * (total - 1) as f64 + chain_length as f64) / total as f64;
        self.last_updated = Utc::now();
    }
    
    /// Get the most common handoff pairs (top N)
    pub fn top_handoff_pairs(&self, n: usize) -> Vec<(String, usize)> {
        let mut pairs: Vec<_> = self.handoff_pairs.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.into_iter().take(n).collect()
    }
    
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_completions + self.failed_tasks;
        if total == 0 {
            0.0
        } else {
            self.successful_completions as f64 / total as f64 * 100.0
        }
    }
    
    /// Generate a summary report
    pub fn summary(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Agent Handoff Metrics ===\n");
        report.push_str(&format!("Total Handoffs: {}\n", self.total_handoffs));
        report.push_str(&format!("Successful Completions: {}\n", self.successful_completions));
        report.push_str(&format!("Failed Tasks: {}\n", self.failed_tasks));
        report.push_str(&format!("Success Rate: {:.1}%\n", self.success_rate()));
        report.push_str(&format!("Average Chain Length: {:.1}\n", self.avg_chain_length));
        report.push_str(&format!("Max Chain Length: {}\n", self.max_chain_length));
        report.push_str(&format!("Loop Incidents: {}\n", self.loop_incidents));
        report.push_str(&format!("Human Interventions: {}\n", self.human_interventions));
        
        let top_pairs = self.top_handoff_pairs(5);
        if !top_pairs.is_empty() {
            report.push_str("\nTop Handoff Pairs:\n");
            for (pair, count) in top_pairs {
                report.push_str(&format!("  {}: {} times\n", pair, count));
            }
        }
        
        report.push_str(&format!("\nLast Updated: {}\n", self.last_updated.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str("==============================\n");
        report
    }
    
    /// Export metrics as JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Import metrics from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

/// Trait for persisting metrics
#[async_trait::async_trait]
pub trait MetricsPersistence: Send + Sync {
    async fn save_metrics(&self, metrics: &HandoffMetrics) -> Result<()>;
    async fn load_metrics(&self) -> Result<Option<HandoffMetrics>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_handoff_metrics_basic() {
        let mut metrics = HandoffMetrics::default();
        
        metrics.record_handoff("ProductOwner", "RequirementsEngineer");
        metrics.record_handoff("RequirementsEngineer", "SeniorDeveloper");
        metrics.record_handoff("SeniorDeveloper", "QATester");
        
        assert_eq!(metrics.total_handoffs, 3);
        assert_eq!(metrics.handoff_pairs.len(), 3);
    }
    
    #[test]
    fn test_success_rate_calculation() {
        let mut metrics = HandoffMetrics::default();
        
        metrics.record_success(5);
        metrics.record_success(6);
        metrics.record_failure(10);
        
        assert_eq!(metrics.successful_completions, 2);
        assert_eq!(metrics.failed_tasks, 1);
        assert!((metrics.success_rate() - 66.67).abs() < 0.1);
    }
    
    #[test]
    fn test_average_chain_length() {
        let mut metrics = HandoffMetrics::default();
        
        metrics.record_success(4);
        metrics.record_success(6);
        metrics.record_success(8);
        
        assert_eq!(metrics.avg_chain_length, 6.0);
    }
    
    #[test]
    fn test_top_handoff_pairs() {
        let mut metrics = HandoffMetrics::default();
        
        metrics.record_handoff("A", "B");
        metrics.record_handoff("A", "B");
        metrics.record_handoff("A", "B");
        metrics.record_handoff("B", "C");
        metrics.record_handoff("C", "D");
        
        let top = metrics.top_handoff_pairs(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "A -> B");
        assert_eq!(top[0].1, 3);
    }
    
    #[test]
    fn test_json_serialization() {
        let mut metrics = HandoffMetrics::default();
        metrics.record_handoff("A", "B");
        metrics.record_success(5);
        
        let json = metrics.to_json().unwrap();
        let restored = HandoffMetrics::from_json(&json).unwrap();
        
        assert_eq!(restored.total_handoffs, metrics.total_handoffs);
        assert_eq!(restored.successful_completions, metrics.successful_completions);
    }
}
