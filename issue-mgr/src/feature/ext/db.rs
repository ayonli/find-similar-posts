use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use napi::Error;
use sqlx::{Connection, MySqlConnection, PgConnection, SqliteConnection, prelude::FromRow};

use crate::feature::{IssueFeatureStore, IssueFeatures};

#[napi(object)]
pub struct DbOptions {
    pub url: String,
    pub table: String,
}

#[derive(Debug, FromRow)]
struct RawIssueFeaturesRecord {
    pub issue_id: String,
    pub operation: Option<String>,
    pub phenomenon: Option<String>,
    pub expected_behavior: Option<String>,
    pub actual_behavior: Option<String>,
}

#[napi]
impl IssueFeatureStore {
    #[napi]
    pub async fn from_db(options: DbOptions) -> napi::Result<Self> {
        let DbOptions { url, table } = options;
        let sql = format!(
            "select issue_id, operation, phenomenon, expected_behavior, actual_behavior from {}",
            &table
        );

        let result = async {
            if url.starts_with("mysql:") {
                let mut db = MySqlConnection::connect(&url).await?;
                let prepare = sqlx::query_as::<_, RawIssueFeaturesRecord>(&sql);
                prepare.fetch_all(&mut db).await
            } else if url.starts_with("postgres:") {
                let mut db = PgConnection::connect(&url).await?;
                let prepare = sqlx::query_as::<_, RawIssueFeaturesRecord>(&sql);
                prepare.fetch_all(&mut db).await
            } else if url.starts_with("sqlite:") {
                let mut db = SqliteConnection::connect(&url).await?;
                let prepare = sqlx::query_as::<_, RawIssueFeaturesRecord>(&sql);
                prepare.fetch_all(&mut db).await
            } else {
                let i = url.find(':').unwrap_or(0);
                let scheme = &url[..i];
                return Err(sqlx::Error::InvalidArgument(format!(
                    "Unsupported database scheme '{}'",
                    scheme
                )));
            }
        }
        .await;
        let rows = match result {
            Ok(rows) => rows,
            Err(e) => {
                return Err(Error::from_reason(format!(
                    "Cannot fetch issue features from database: {}",
                    e
                )));
            }
        };
        let map: HashMap<String, IssueFeatures> = rows
            .into_iter()
            .map(
                |RawIssueFeaturesRecord {
                     issue_id,
                     phenomenon,
                     operation,
                     expected_behavior,
                     actual_behavior,
                 }| {
                    (
                        issue_id.clone(),
                        IssueFeatures {
                            phenomenon,
                            operation,
                            expected_behavior,
                            actual_behavior,
                        },
                    )
                },
            )
            .collect();

        Ok(IssueFeatureStore {
            issue_features_map: Arc::new(RwLock::new(map)),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use dotenv::dotenv;
    use napi::tokio;

    use crate::feature::*;

    use super::*;

    #[tokio::test]
    async fn test_issue_feature_store_load_from_db_mysql() {
        dotenv().ok();
        let store = IssueFeatureStore::from_db(DbOptions {
            url: env::var("MYSQL_URL").unwrap(),
            table: "issue_features".to_string(),
        })
        .await
        .unwrap();

        assert_eq!(
            store.get_record("1".to_string()).unwrap(),
            Some(IssueFeaturesRecord {
                issue_id: "1".to_string(),
                features: IssueFeatures {
                    operation: Some("Turn on the switch".to_string()),
                    phenomenon: None,
                    expected_behavior: Some("The device is turned on".to_string()),
                    actual_behavior: Some("The device is not turned on".to_string()),
                },
            })
        );
        assert_eq!(
            store.get_record("2".to_string()).unwrap(),
            Some(IssueFeaturesRecord {
                issue_id: "2".to_string(),
                features: IssueFeatures {
                    operation: Some("Turn off the switch".to_string()),
                    phenomenon: Some(
                        "The device remains turned on instead if being turned on".to_string(),
                    ),
                    expected_behavior: None,
                    actual_behavior: None,
                },
            })
        );
        assert_eq!(store.get_record("3".to_string()).unwrap(), None);
    }

    #[tokio::test]
    async fn test_issue_feature_store_load_from_db_postgres() {
        dotenv().ok();
        let store = IssueFeatureStore::from_db(DbOptions {
            url: env::var("PG_URL").unwrap(),
            table: "issue_features".to_string(),
        })
        .await
        .unwrap();

        assert_eq!(
            store.get_record("1".to_string()).unwrap(),
            Some(IssueFeaturesRecord {
                issue_id: "1".to_string(),
                features: IssueFeatures {
                    operation: Some("Turn on the switch".to_string()),
                    phenomenon: None,
                    expected_behavior: Some("The device is turned on".to_string()),
                    actual_behavior: Some("The device is not turned on".to_string()),
                },
            })
        );
        assert_eq!(
            store.get_record("2".to_string()).unwrap(),
            Some(IssueFeaturesRecord {
                issue_id: "2".to_string(),
                features: IssueFeatures {
                    operation: Some("Turn off the switch".to_string()),
                    phenomenon: Some(
                        "The device remains turned on instead if being turned on".to_string(),
                    ),
                    expected_behavior: None,
                    actual_behavior: None,
                },
            })
        );
        assert_eq!(store.get_record("3".to_string()).unwrap(), None);
    }

    #[tokio::test]
    async fn test_issue_feature_store_load_from_db_sqlite() {
        let store = IssueFeatureStore::from_db(DbOptions {
            url: "sqlite:./assets/issue_mgr.db".to_string(),
            table: "issue_features".to_string(),
        })
        .await
        .unwrap();

        assert_eq!(
            store.get_record("1".to_string()).unwrap(),
            Some(IssueFeaturesRecord {
                issue_id: "1".to_string(),
                features: IssueFeatures {
                    operation: Some("Turn on the switch".to_string()),
                    phenomenon: None,
                    expected_behavior: Some("The device is turned on".to_string()),
                    actual_behavior: Some("The device is not turned on".to_string()),
                },
            })
        );
        assert_eq!(
            store.get_record("2".to_string()).unwrap(),
            Some(IssueFeaturesRecord {
                issue_id: "2".to_string(),
                features: IssueFeatures {
                    operation: Some("Turn off the switch".to_string()),
                    phenomenon: Some(
                        "The device remains turned on instead if being turned on".to_string(),
                    ),
                    expected_behavior: None,
                    actual_behavior: None,
                },
            })
        );
        assert_eq!(store.get_record("3".to_string()).unwrap(), None);
    }
}
