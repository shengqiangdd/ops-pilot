//! Anomaly detection using statistical methods.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyScore {
    pub score: f64,
    pub is_anomaly: bool,
    pub severity: String,
}

pub struct AnomalyDetector;

impl AnomalyDetector {
    pub fn new() -> Self { Self }

    pub fn detect_baseline(&self, data: &[f64]) -> Baseline {
        if data.is_empty() {
            return Baseline { mean: 0.0, std_dev: 0.0, min: 0.0, max: 0.0 };
        }
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();
        Baseline {
            mean,
            std_dev,
            min: data.iter().cloned().fold(f64::INFINITY, f64::min),
            max: data.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }

    pub fn check_deviation(&self, value: f64, baseline: &Baseline) -> AnomalyScore {
        if baseline.std_dev == 0.0 {
            return AnomalyScore { score: 0.0, is_anomaly: false, severity: "low".into() };
        }
        let z_score = (value - baseline.mean).abs() / baseline.std_dev;
        let is_anomaly = z_score > 3.0;
        let severity = if z_score > 4.0 { "high" } else if z_score > 3.0 { "medium" } else { "low" }.to_string();
        AnomalyScore { score: z_score, is_anomaly, severity }
    }
}
