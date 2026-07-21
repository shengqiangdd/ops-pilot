//! Anomaly detection using Z-Score, EWMA smoothing, dual-threshold,
//! seasonality decomposition, and trend detection.

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
    /// EWMA-smoothed value
    pub smoothed_value: f64,
    /// Raw Z-score
    pub z_score: f64,
}

/// Trend direction detected by linear regression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDirection {
    pub direction: String, // "up", "down", "stable"
    pub slope: f64,
    pub r_squared: f64,
}

/// Seasonality decomposition components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decomposition {
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residual: Vec<f64>,
}

/// Residual analysis statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualStats {
    pub mean: f64,
    pub std_dev: f64,
    pub max_residual: f64,
    pub outlier_count: usize,
    pub outlier_ratio: f64,
}

pub struct AnomalyDetector {
    pub ewma_alpha: f64,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self { ewma_alpha: 0.3 }
    }

    /// Compute EWMA (Exponentially Weighted Moving Average).
    pub fn ewma(prev: f64, current: f64, alpha: f64) -> f64 {
        alpha * current + (1.0 - alpha) * prev
    }

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

    /// Improved check_deviation with EWMA smoothing and dual-threshold.
    pub fn check_deviation(&self, value: f64, baseline: &Baseline) -> AnomalyScore {
        let std_safe = baseline.std_dev.max(1e-10);
        let z_score = (value - baseline.mean).abs() / std_safe;

        // EWMA smoothing
        let smoothed = Self::ewma(baseline.mean, value, self.ewma_alpha);

        // Dual threshold
        let (is_anomaly, severity) = if z_score > 3.5 {
            (true, "critical".to_string())
        } else if z_score > 2.0 {
            (true, "warning".to_string())
        } else {
            (false, "low".to_string())
        };

        AnomalyScore {
            score: z_score,
            is_anomaly,
            severity,
            smoothed_value: smoothed,
            z_score,
        }
    }

    /// Simple seasonal decomposition: trend (moving average) + seasonal (period average) + residual.
    pub fn seasonality_decompose(data: &[f64], period: usize) -> Decomposition {
        let n = data.len();
        if n < period || period == 0 {
            return Decomposition {
                trend: vec![0.0; n],
                seasonal: vec![0.0; n],
                residual: data.to_vec(),
            };
        }

        // Trend: centered moving average with window = period
        let mut trend = vec![0.0_f64; n];
        let half = period / 2;
        for i in half..(n - half) {
            let window_sum: f64 = data[(i - half)..=(i + half)].iter().sum();
            trend[i] = window_sum / period as f64;
        }
        // Pad edges with nearest trend value
        if half > 0 && half < n {
            for i in 0..half {
                trend[i] = trend[half];
            }
            for i in (n - half)..n {
                trend[i] = trend[n - half - 1];
            }
        }

        // Seasonal: average detrended values at each position in the period
        let detrended: Vec<f64> = data.iter().zip(trend.iter()).map(|(d, t)| d - t).collect();
        let mut seasonal = vec![0.0_f64; period];
        for (s, slot) in seasonal.iter_mut().enumerate() {
            let mut sum = 0.0;
            let mut count = 0;
            for i in (s..n).step_by(period) {
                sum += detrended[i];
                count += 1;
            }
            if count > 0 {
                *slot = sum / count as f64;
            }
        }
        // Normalize seasonal to sum to zero
        let seasonal_mean: f64 = seasonal.iter().sum::<f64>() / period as f64;
        for s in seasonal.iter_mut() {
            *s -= seasonal_mean;
        }

        // Residual = original - trend - seasonal
        let residual: Vec<f64> = data.iter().enumerate()
            .map(|(i, d)| d - trend[i] - seasonal[i % period])
            .collect();

        Decomposition { trend, seasonal, residual }
    }

    /// Detect trend direction using simple linear regression.
    pub fn detect_trend(data: &[f64]) -> TrendDirection {
        let n = data.len() as f64;
        if n < 2.0 {
            return TrendDirection { direction: "stable".into(), slope: 0.0, r_squared: 0.0 };
        }

        let mean_x = (n - 1.0) / 2.0;
        let mean_y: f64 = data.iter().sum::<f64>() / n;

        let mut ss_xy = 0.0_f64;
        let mut ss_xx = 0.0_f64;
        let mut ss_yy = 0.0_f64;

        for (i, y) in data.iter().enumerate() {
            let x = i as f64;
            ss_xy += (x - mean_x) * (y - mean_y);
            ss_xx += (x - mean_x).powi(2);
            ss_yy += (y - mean_y).powi(2);
        }

        let slope = if ss_xx > 0.0 { ss_xy / ss_xx } else { 0.0 };
        let r_squared = if ss_xx > 0.0 && ss_yy > 0.0 {
            (ss_xy / (ss_xx * ss_yy).sqrt()).powi(2)
        } else {
            0.0
        };

        let direction = if slope.abs() < 0.01 {
            "stable"
        } else if slope > 0.0 {
            "up"
        } else {
            "down"
        };

        TrendDirection {
            direction: direction.into(),
            slope,
            r_squared,
        }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_baseline(mean: f64, std_dev: f64) -> Baseline {
        Baseline { mean, std_dev, min: mean - std_dev, max: mean + std_dev }
    }

    #[test]
    fn test_zscore_normal() {
        let det = AnomalyDetector::new();
        let baseline = make_baseline(100.0, 10.0);
        let score = det.check_deviation(102.0, &baseline);
        assert!(score.z_score < 1.0, "z_score={}", score.z_score);
        assert!(!score.is_anomaly);
        assert_eq!(score.severity, "low");
    }

    #[test]
    fn test_zscore_outlier() {
        let det = AnomalyDetector::new();
        let baseline = make_baseline(100.0, 10.0);
        let score = det.check_deviation(150.0, &baseline);
        assert!(score.z_score > 3.5, "z_score={}", score.z_score);
        assert!(score.is_anomaly);
        assert_eq!(score.severity, "critical");
    }

    #[test]
    fn test_ewma_smoothing() {
        let result = AnomalyDetector::ewma(100.0, 120.0, 0.3);
        // 0.3 * 120 + 0.7 * 100 = 36 + 70 = 106
        assert!((result - 106.0).abs() < 0.01);
    }

    #[test]
    fn test_double_threshold_warning() {
        let det = AnomalyDetector::new();
        let baseline = make_baseline(100.0, 10.0);
        // Z = |122 - 100| / 10 = 2.2 → warning
        let score = det.check_deviation(122.0, &baseline);
        assert!(score.is_anomaly);
        assert_eq!(score.severity, "warning");
    }

    #[test]
    fn test_double_threshold_critical() {
        let det = AnomalyDetector::new();
        let baseline = make_baseline(100.0, 10.0);
        // Z = |140 - 100| / 10 = 4.0 → critical
        let score = det.check_deviation(140.0, &baseline);
        assert!(score.is_anomaly);
        assert_eq!(score.severity, "critical");
    }

    #[test]
    fn test_seasonality_decompose_dimensions() {
        let data: Vec<f64> = (0..24).map(|i| 50.0 + (i % 6) as f64 * 5.0 + i as f64 * 0.5).collect();
        let decomp = AnomalyDetector::seasonality_decompose(&data, 6);
        assert_eq!(decomp.trend.len(), 24);
        assert_eq!(decomp.seasonal.len(), 6);
        assert_eq!(decomp.residual.len(), 24);
    }

    #[test]
    fn test_seasonality_decompose_short_data() {
        let data = vec![1.0, 2.0, 3.0];
        let decomp = AnomalyDetector::seasonality_decompose(&data, 4);
        // n < period: early return with n-length vectors
        assert_eq!(decomp.trend.len(), 3);
        assert_eq!(decomp.seasonal.len(), 3);
        assert_eq!(decomp.residual.len(), 3);
    }

    #[test]
    fn test_trend_detection_up() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let trend = AnomalyDetector::detect_trend(&data);
        assert_eq!(trend.direction, "up");
        assert!(trend.slope > 0.0);
        assert!(trend.r_squared > 0.9);
    }

    #[test]
    fn test_trend_detection_down() {
        let data = vec![50.0, 40.0, 30.0, 20.0, 10.0];
        let trend = AnomalyDetector::detect_trend(&data);
        assert_eq!(trend.direction, "down");
        assert!(trend.slope < 0.0);
    }

    #[test]
    fn test_trend_detection_stable() {
        let data = vec![100.0, 100.0, 100.0, 100.0];
        let trend = AnomalyDetector::detect_trend(&data);
        assert_eq!(trend.direction, "stable");
        assert!((trend.slope).abs() < 0.01);
    }

    #[test]
    fn test_trend_single_point() {
        let data = vec![42.0];
        let trend = AnomalyDetector::detect_trend(&data);
        assert_eq!(trend.direction, "stable");
    }
}
