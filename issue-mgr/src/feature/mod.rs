use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use napi::{bindgen_prelude::AsyncTask, Env, Error, Result, Task};
use rapidfuzz::distance::levenshtein::normalized_similarity;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::feature::util::get_feature_weights;

mod util;

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueFeatures {
    pub operation: Option<String>,
    pub phenomenon: Option<String>,
    pub expected_behavior: Option<String>,
    pub actual_behavior: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueFeaturesRecord {
    pub issue_id: String,
    pub features: IssueFeatures,
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarIssueFeaturesRecord {
    pub issue_id: String,
    pub features: IssueFeatures,
    /// Similarity score `0 - 1`, higher is more similar.
    pub score: f64,
}

#[napi]
pub struct IssueFeatureStore {
    issue_features_map: Arc<RwLock<HashMap<String, IssueFeatures>>>,
}

#[napi]
impl IssueFeatureStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        IssueFeatureStore {
            issue_features_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[napi]
    pub fn preload(&self, records: Vec<IssueFeaturesRecord>) -> Result<()> {
        let _map: HashMap<String, IssueFeatures> = records
            .into_iter()
            .filter(|item| {
                !item.issue_id.is_empty()
                    && (item.features.operation.is_some()
                        || item.features.phenomenon.is_some()
                        || item.features.expected_behavior.is_some()
                        || item.features.actual_behavior.is_some())
            })
            .map(|map| (map.issue_id, map.features))
            .collect();

        match self.issue_features_map.write() {
            Ok(mut map) => {
                *map = _map;
                Ok(())
            }
            Err(e) => Err(Error::from_reason(format!(
                "Failed to preload issue features: {}",
                e
            ))),
        }
    }

    #[napi]
    pub fn add_record(&self, record: IssueFeaturesRecord) -> Result<()> {
        if record.issue_id.is_empty() {
            return Err(Error::from_reason("issue_id must not be empty"));
        } else if record.features.operation.is_none()
            && record.features.phenomenon.is_none()
            && record.features.expected_behavior.is_none()
            && record.features.actual_behavior.is_none()
        {
            return Err(Error::from_reason("features must not be empty"));
        }

        match self.issue_features_map.write() {
            Ok(mut map) => {
                map.insert(record.issue_id, record.features);
                Ok(())
            }
            Err(e) => Err(Error::from_reason(format!(
                "Failed to add issue feature record: {}",
                e
            ))),
        }
    }

    #[napi]
    pub fn get_record(&self, issue_id: String) -> Result<Option<IssueFeaturesRecord>> {
        match self.issue_features_map.read() {
            Ok(map) => {
                if let Some(feature) = map.get(&issue_id) {
                    Ok(Some(IssueFeaturesRecord {
                        issue_id,
                        features: feature.clone(),
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(Error::from_reason(format!(
                "Failed to get issue feature record: {}",
                e
            ))),
        }
    }

    #[napi]
    pub fn remove_record(&self, issue_id: String) -> Result<bool> {
        match self.issue_features_map.write() {
            Ok(mut map) => {
                if map.remove(&issue_id).is_some() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(Error::from_reason(format!(
                "Failed to remove issue feature record: {}",
                e
            ))),
        }
    }

    #[napi(ts_return_type = "Promise<Array<SimilarIssueFeaturesRecord>>")]
    pub fn find_similar_records(
        &self,
        features: IssueFeatures,
        top_n: Option<u32>,
    ) -> AsyncTask<AsyncFindSimilarRecords> {
        AsyncTask::new(AsyncFindSimilarRecords {
            features,
            issue_feature_map: self.issue_features_map.clone(),
            top_n: top_n.unwrap_or(5),
        })
    }
}

pub struct AsyncFindSimilarRecords {
    features: IssueFeatures,
    issue_feature_map: Arc<RwLock<HashMap<String, IssueFeatures>>>,
    top_n: u32,
}

#[napi]
impl Task for AsyncFindSimilarRecords {
    type Output = Vec<SimilarIssueFeaturesRecord>;
    type JsValue = Vec<SimilarIssueFeaturesRecord>;

    fn compute(&mut self) -> Result<Self::Output> {
        match self.issue_feature_map.read() {
            Ok(map) => find_similar_records_in_parallel(&self.features, &map, self.top_n),
            Err(e) => Err(Error::from_reason(format!("Failed to read posts: {}", e))),
        }
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

fn find_similar_records_in_parallel(
    features: &IssueFeatures,
    candidates: &HashMap<String, IssueFeatures>,
    top_n: u32,
) -> Result<Vec<SimilarIssueFeaturesRecord>> {
    let weights = get_feature_weights(features)?;
    let mut matches: Vec<SimilarIssueFeaturesRecord> = candidates
        .par_iter()
        .filter_map(|(issue_id, _features)| {
            let operation_score = {
                if features.operation.is_some() && _features.operation.is_some() {
                    let operand1 = features.operation.as_ref().unwrap();
                    let operand2 = _features.operation.as_ref().unwrap();
                    normalized_similarity(operand1.chars(), operand2.chars()) * weights.operation
                } else {
                    0.0
                }
            };
            let phenomenon_score = {
                if features.phenomenon.is_some() && _features.phenomenon.is_some() {
                    let operand1 = features.phenomenon.as_ref().unwrap();
                    let operand2 = _features.phenomenon.as_ref().unwrap();
                    normalized_similarity(operand1.chars(), operand2.chars()) * weights.phenomenon
                } else {
                    0.0
                }
            };
            let expected_behavior_score = {
                if features.expected_behavior.is_some() && _features.expected_behavior.is_some() {
                    let operand1 = features.expected_behavior.as_ref().unwrap();
                    let operand2 = _features.expected_behavior.as_ref().unwrap();
                    normalized_similarity(operand1.chars(), operand2.chars())
                        * weights.expected_behavior
                } else {
                    0.0
                }
            };
            let actual_behavior_score = {
                if features.actual_behavior.is_some() && _features.actual_behavior.is_some() {
                    let operand1 = features.actual_behavior.as_ref().unwrap();
                    let operand2 = _features.actual_behavior.as_ref().unwrap();
                    normalized_similarity(operand1.chars(), operand2.chars())
                        * weights.actual_behavior
                } else {
                    0.0
                }
            };
            let mut score = operation_score
                + phenomenon_score
                + expected_behavior_score
                + actual_behavior_score;
            score = score.clamp(0.0, 1.0);

            if score > 0.5 {
                Some(SimilarIssueFeaturesRecord {
                    issue_id: issue_id.clone(),
                    features: features.clone(),
                    score,
                })
            } else {
                None
            }
        })
        .collect();

    matches.sort_by(|a, b| {
        let order = b.score.partial_cmp(&a.score);
        order.unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_n_matches = if matches.len() <= top_n as usize {
        matches
    } else {
        matches.into_iter().take(top_n as usize).collect()
    };

    Ok(top_n_matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_feature_store_preload() {
        let store = IssueFeatureStore::new();

        let record1 = IssueFeaturesRecord {
            issue_id: "1".to_string(),
            features: IssueFeatures {
                operation: Some("Turn on the switch".to_string()),
                phenomenon: None,
                expected_behavior: Some("The device is turned on".to_string()),
                actual_behavior: Some("The device is not turned on".to_string()),
            },
        };
        let record2 = IssueFeaturesRecord {
            issue_id: "2".to_string(),
            features: IssueFeatures {
                operation: Some("Turn off the switch".to_string()),
                phenomenon: Some(
                    "The device remains turned on instead if being turned on".to_string(),
                ),
                expected_behavior: None,
                actual_behavior: None,
            },
        };
        let records = vec![
            record1.clone(),
            record2.clone(),
            IssueFeaturesRecord {
                // This one is invalid will be ignored
                issue_id: "3".to_string(),
                features: IssueFeatures {
                    operation: None,
                    phenomenon: None,
                    expected_behavior: None,
                    actual_behavior: None,
                },
            },
        ];

        store.preload(records).unwrap();

        assert_eq!(store.get_record("1".to_string()).unwrap(), Some(record1));
        assert_eq!(store.get_record("2".to_string()).unwrap(), Some(record2));
        assert_eq!(store.get_record("3".to_string()).unwrap(), None);
        assert_eq!(store.get_record("4".to_string()).unwrap(), None);
    }

    #[test]
    fn test_issue_feature_store_add_record() {
        let store = IssueFeatureStore::new();

        let record = IssueFeaturesRecord {
            issue_id: "1".to_string(),
            features: IssueFeatures {
                operation: Some("Turn on the switch".to_string()),
                phenomenon: None,
                expected_behavior: Some("The device is turned on".to_string()),
                actual_behavior: Some("The device is not turned on".to_string()),
            },
        };

        store.add_record(record.clone()).unwrap();
        assert_eq!(store.get_record("1".to_string()).unwrap(), Some(record));
        assert_eq!(store.get_record("2".to_string()).unwrap(), None);
    }

    #[test]
    fn test_issue_feature_store_remove_record() {
        let store = IssueFeatureStore::new();

        let record1 = IssueFeaturesRecord {
            issue_id: "1".to_string(),
            features: IssueFeatures {
                operation: Some("Turn on the switch".to_string()),
                phenomenon: None,
                expected_behavior: Some("The device is turned on".to_string()),
                actual_behavior: Some("The device is not turned on".to_string()),
            },
        };
        let record2 = IssueFeaturesRecord {
            issue_id: "2".to_string(),
            features: IssueFeatures {
                operation: Some("Turn off the switch".to_string()),
                phenomenon: Some(
                    "The device remains turned on instead if being turned on".to_string(),
                ),
                expected_behavior: None,
                actual_behavior: None,
            },
        };

        store.add_record(record1.clone()).unwrap();
        store.add_record(record2.clone()).unwrap();

        assert_eq!(store.get_record("1".to_string()).unwrap(), Some(record1));
        assert_eq!(store.get_record("2".to_string()).unwrap(), Some(record2));

        assert_eq!(store.remove_record("1".to_string()).unwrap(), true);
        assert_eq!(store.get_record("1".to_string()).unwrap(), None);
        assert_eq!(store.remove_record("3".to_string()).unwrap(), false);
    }

    #[test]
    fn test_find_similar_records_in_parallel() {
        let store = IssueFeatureStore::new();

        let record1 = IssueFeaturesRecord {
            issue_id: "1".to_string(),
            features: IssueFeatures {
                operation: Some("Turn on the switch".to_string()),
                phenomenon: None,
                expected_behavior: Some("The device is turned on".to_string()),
                actual_behavior: Some("The device is not turned on".to_string()),
            },
        };
        let record2 = IssueFeaturesRecord {
            issue_id: "2".to_string(),
            features: IssueFeatures {
                operation: Some("Turn off the switch".to_string()),
                phenomenon: Some(
                    "The device remains turned on instead if being turned on".to_string(),
                ),
                expected_behavior: None,
                actual_behavior: None,
            },
        };

        store.add_record(record1.clone()).unwrap();
        store.add_record(record2.clone()).unwrap();

        let features = IssueFeatures {
            operation: Some("Turn on the switch".to_string()),
            phenomenon: None,
            expected_behavior: Some("The device turns on".to_string()),
            actual_behavior: Some("The device does not turn on".to_string()),
        };
        let matches = find_similar_records_in_parallel(
            &features,
            &store.issue_features_map.read().unwrap(),
            5,
        )
        .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].issue_id, "1");
        assert!(matches[0].score > 0.8);
    }
}
