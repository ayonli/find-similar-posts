use napi::{Error, Result};

use crate::feature::IssueFeatures;

fn count_chars(text: Option<&str>) -> usize {
    text.map_or(0, |s| s.chars().count())
}

pub struct FeatureWeights {
    pub operation: f64,
    pub phenomenon: f64,
    pub expected_behavior: f64,
    pub actual_behavior: f64,
}

pub fn get_feature_weights(source: &IssueFeatures) -> Result<FeatureWeights> {
    let operation_chars = count_chars(source.operation.as_deref());
    let phenomenon_chars = count_chars(source.phenomenon.as_deref());
    let expected_behavior_chars = count_chars(source.expected_behavior.as_deref());
    let actual_behavior_chars = count_chars(source.actual_behavior.as_deref());
    let total_chars =
        operation_chars + phenomenon_chars + expected_behavior_chars + actual_behavior_chars;

    if total_chars == 0 {
        Err(Error::from_reason("source is invalid"))
    } else {
        Ok(FeatureWeights {
            operation: operation_chars as f64 / total_chars as f64,
            phenomenon: phenomenon_chars as f64 / total_chars as f64,
            expected_behavior: expected_behavior_chars as f64 / total_chars as f64,
            actual_behavior: actual_behavior_chars as f64 / total_chars as f64,
        })
    }
}
