use std::{
    collections::HashMap,
    fs::File,
    sync::{Arc, RwLock},
};

use csv::{Reader, Writer};
use napi::{Env, Error, Result, Task, bindgen_prelude::AsyncTask};

use crate::feature::{
    IssueFeatureStore, IssueFeatures, IssueFeaturesRecord, ext::RawIssueFeaturesRecord,
};

#[napi]
impl IssueFeatureStore {
    #[napi(ts_return_type = "Promise<IssueFeatureStore>")]
    pub fn from_csv(path: String) -> AsyncTask<AsyncLoader> {
        AsyncTask::new(AsyncLoader { path })
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn to_csv(&self, path: String) -> AsyncTask<AsyncDumper> {
        AsyncTask::new(AsyncDumper {
            path,
            issue_features_map: self.issue_features_map.clone(),
        })
    }
}

pub struct AsyncLoader {
    pub path: String,
}

#[napi]
impl Task for AsyncLoader {
    type Output = IssueFeatureStore;
    type JsValue = Self::Output;

    fn compute(&mut self) -> Result<Self::Output> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(e) => return Err(Error::from_reason(format!("Cannot open CSV file: {}", e))),
        };
        let mut rdr = Reader::from_reader(file);
        let mut records: Vec<IssueFeaturesRecord> = Vec::new();

        for record in rdr.deserialize() {
            let RawIssueFeaturesRecord {
                issue_id,
                operation,
                phenomenon,
                expected_behavior,
                actual_behavior,
            } = match record {
                Ok(record) => record,
                Err(e) => {
                    return Err(Error::from_reason(format!(
                        "Cannot parse CSV record: {}",
                        e
                    )));
                }
            };

            records.push(IssueFeaturesRecord {
                issue_id,
                features: IssueFeatures {
                    operation,
                    phenomenon,
                    expected_behavior,
                    actual_behavior,
                },
            });
        }

        Ok(IssueFeatureStore::new(Some(records)))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

pub struct AsyncDumper {
    pub path: String,
    pub issue_features_map: Arc<RwLock<HashMap<String, IssueFeatures>>>,
}

#[napi]
impl Task for AsyncDumper {
    type Output = ();
    type JsValue = Self::Output;

    fn compute(&mut self) -> Result<Self::Output> {
        let file = match File::create(&self.path) {
            Ok(file) => file,
            Err(e) => return Err(Error::from_reason(format!("Cannot create CSV file: {}", e))),
        };
        let mut wtr = Writer::from_writer(file);
        let map = match self.issue_features_map.read() {
            Ok(lock) => lock,
            Err(e) => {
                return Err(Error::from_reason(format!(
                    "Cannot acquire read lock on issue features map: {}",
                    e
                )));
            }
        };

        for (issue_id, features) in map.iter() {
            let raw_record = RawIssueFeaturesRecord {
                issue_id: issue_id.clone(),
                operation: features.operation.clone(),
                phenomenon: features.phenomenon.clone(),
                expected_behavior: features.expected_behavior.clone(),
                actual_behavior: features.actual_behavior.clone(),
            };

            if let Err(e) = wtr.serialize(raw_record) {
                return Err(Error::from_reason(format!(
                    "Cannot serialize CSV record: {}",
                    e
                )));
            }
        }

        wtr.flush()
            .map_err(|e| Error::from_reason(format!("Cannot flush CSV writer: {}", e)))?;
        Ok(())
    }

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}
