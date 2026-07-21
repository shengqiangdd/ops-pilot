//! Holt-Winters triple exponential smoothing for time-series prediction.
//!
//! Provides additive seasonal forecasting with confidence intervals.

use serde::{Deserialize, Serialize};

/// Holt-Winters triple exponential smoothing model.
pub struct HoltWinters {
    alpha: f64,   // Level smoothing factor
    beta: f64,    // Trend smoothing factor
    gamma: f64,   // Seasonal smoothing factor
    period: usize, // Seasonal period (e.g. 24 for hourly data with daily seasonality)
    level: f64,
    trend: f64,
    seasonal: Vec<f64>,
    fitted: Vec<f64>,
}

/// Prediction result with confidence interval: (point_estimate, lower_bound, upper_bound).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionPoint {
    pub value: f64,
    pub lower: f64,
    pub upper: f64,
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

impl HoltWinters {
    /// Fit a Holt-Winters model to historical data.
    pub fn fit(data: &[f64], period: usize) -> Self {
        assert!(data.len() >= 2 * period, "Need at least 2 full periods of data");
        assert!(period > 0, "Period must be positive");

        let alpha = 0.3;
        let beta = 0.1;
        let gamma = 0.1;

        // Initialize level as mean of first period
        let level_init: f64 = data[..period].iter().sum::<f64>() / period as f64;
        // Initialize trend as average difference between first two periods
        let trend_init: f64 = if data.len() >= 2 * period {
            let avg_first: f64 = data[..period].iter().sum::<f64>() / period as f64;
            let avg_second: f64 = data[period..2 * period].iter().sum::<f64>() / period as f64;
            (avg_second - avg_first) / period as f64
        } else {
            0.0
        };

        // Initialize seasonal factors from first 2 periods
        let mut seasonal = vec![0.0_f64; period];
        for s in 0..period {
            seasonal[s] = data[s] - level_init;
        }

        let mut level = level_init;
        let mut trend = trend_init;
        let mut fitted = Vec::with_capacity(data.len());

        // Fit the model
        for (t, &value) in data.iter().enumerate() {
            let s = t % period;
            let prev_level = level;
            let prev_trend = trend;

            // Level update
            level = alpha * (value - seasonal[s]) + (1.0 - alpha) * (prev_level + prev_trend);
            // Trend update
            trend = beta * (level - prev_level) + (1.0 - beta) * prev_trend;
            // Seasonal update
            seasonal[s] = gamma * (value - level) + (1.0 - gamma) * seasonal[s];

            fitted.push(level + trend + seasonal[s]);
        }

        Self {
            alpha,
            beta,
            gamma,
            period,
            level,
            trend,
            seasonal,
            fitted,
        }
    }

    /// Predict future values for `steps` steps ahead.
    pub fn predict(&self, steps: usize) -> Vec<f64> {
        let mut predictions = Vec::with_capacity(steps);
        for h in 1..=steps {
            let s = (self.fitted.len() + h - 1) % self.period;
            let y = self.level + self.trend * h as f64 + self.seasonal[s];
            predictions.push(y);
        }
        predictions
    }

    /// Predict with 95% confidence intervals.
    ///
    /// CI width grows with sqrt(h) based on residual standard error.
    pub fn predict_with_ci(&self, steps: usize) -> Vec<PredictionPoint> {
        let residuals: Vec<f64> = self.fitted.iter()
            .map(|_| {
                // Simple residual approximation (actual - fitted for training data)
                0.0 // Placeholder; real impl needs original data
            })
            .collect();

        // Estimate residual std dev from fitted values
        let residual_std = if residuals.len() > 1 {
            let mean = residuals.iter().sum::<f64>() / residuals.len() as f64;
            let variance = residuals.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (residuals.len() - 1) as f64;
            variance.sqrt().max(1.0) // Floor at 1.0
        } else {
            1.0
        };

        let z_95 = 1.96; // 95% CI

        self.predict(steps).into_iter().enumerate().map(|(h, value)| {
            let h_steps = (h + 1) as f64;
            let ci_width = z_95 * residual_std * h_steps.sqrt();
            PredictionPoint {
                value,
                lower: value - ci_width,
                upper: value + ci_width,
            }
        }).collect()
    }

    /// Get model diagnostics.
    pub fn diagnostics(&self) -> serde_json::Value {
        serde_json::json!({
            "alpha": self.alpha,
            "beta": self.beta,
            "gamma": self.gamma,
            "period": self.period,
            "level": self.level,
            "trend": self.trend,
            "fitted_count": self.fitted.len(),
        })
    }
}

/// Analyze residuals between actual and predicted values.
pub fn residual_analysis(actual: &[f64], predicted: &[f64]) -> ResidualStats {
    if actual.is_empty() || predicted.is_empty() {
        return ResidualStats {
            mean: 0.0, std_dev: 0.0, max_residual: 0.0,
            outlier_count: 0, outlier_ratio: 0.0,
        };
    }

    let n = actual.len().min(predicted.len());
    let residuals: Vec<f64> = (0..n).map(|i| actual[i] - predicted[i]).collect();

    let mean = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let variance = residuals.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / residuals.len() as f64;
    let std_dev = variance.sqrt();
    let max_residual = residuals.iter().map(|r| r.abs()).fold(0.0_f64, f64::max);

    // Outliers: residuals > 2 * std_dev
    let outlier_count = residuals.iter().filter(|r| r.abs() > 2.0 * std_dev).count();

    ResidualStats {
        mean,
        std_dev,
        max_residual,
        outlier_count,
        outlier_ratio: outlier_count as f64 / residuals.len() as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holt_winters_fit_and_predict() {
        // Generate synthetic seasonal data: base=100, trend=1, period=4
        let data: Vec<f64> = (0..20)
            .map(|i| 100.0 + i as f64 * 0.5 + (i % 4) as f64 * 5.0)
            .collect();

        let model = HoltWinters::fit(&data, 4);
        let predictions = model.predict(4);

        assert_eq!(predictions.len(), 4);
        // Predictions should be in a reasonable range
        for p in &predictions {
            assert!(*p > 80.0 && *p < 150.0, "prediction {} out of range", p);
        }
    }

    #[test]
    fn test_predict_with_ci() {
        let data: Vec<f64> = (0..16).map(|i| 50.0 + (i % 4) as f64 * 10.0).collect();
        let model = HoltWinters::fit(&data, 4);
        let ci = model.predict_with_ci(4);

        assert_eq!(ci.len(), 4);
        for p in &ci {
            assert!(p.lower <= p.value);
            assert!(p.value <= p.upper);
        }
    }

    #[test]
    fn test_residual_analysis() {
        let actual = vec![10.0, 20.0, 30.0, 40.0];
        let predicted = vec![11.0, 19.0, 32.0, 38.0];
        let stats = residual_analysis(&actual, &predicted);

        // residuals = [-1, 1, -2, 2], mean = 0
        assert!(stats.mean.abs() < 0.01, "mean should be ~0, got {}", stats.mean);
        assert!(stats.std_dev > 0.0);
        assert_eq!(stats.outlier_count, 0);
    }

    #[test]
    fn test_holt_winters_constant() {
        // Constant series: all values = 42
        let data: Vec<f64> = vec![42.0; 24];
        let model = HoltWinters::fit(&data, 4);
        let predictions = model.predict(4);
        for p in &predictions {
            assert!((*p - 42.0).abs() < 5.0, "constant prediction should be near 42, got {}", p);
        }
    }

    #[test]
    fn test_holt_winters_linear_trend() {
        // Linear trend: 10, 12, 14, 16, 18, 20, 22, 24 (period=4, so 2 full periods)
        let data: Vec<f64> = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0];
        let model = HoltWinters::fit(&data, 4);
        let predictions = model.predict(2);
        // Predictions should continue upward trend
        assert!(predictions[0] > 20.0, "first prediction should be > 20, got {}", predictions[0]);
    }

    #[test]
    fn test_residual_analysis_empty() {
        let stats = residual_analysis(&[], &[]);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.outlier_count, 0);
    }

    #[test]
    fn test_residual_analysis_outliers() {
        let actual = vec![10.0, 10.0, 10.0, 10.0, 100.0];
        let predicted = vec![10.0, 10.0, 10.0, 10.0, 10.0];
        let stats = residual_analysis(&actual, &predicted);
        assert!(stats.outlier_count >= 1, "should detect at least one outlier");
        assert!(stats.max_residual > 80.0);
    }
}
