use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Row;
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigVersion {
    pub id: u64,
    pub name: String,
    pub config: serde_json::Value,
    #[schema(value_type = String, format = DateTime)]
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl FromRow<'_, sqlx::mysql::MySqlRow> for ConfigVersion {
    fn from_row(row: &sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let created_at_offset: OffsetDateTime = row.try_get("created_at")?;
        let updated_at_offset: OffsetDateTime = row.try_get("updated_at")?;

        Ok(ConfigVersion {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            config: row.try_get("config")?,
            created_at: DateTime::from_timestamp(
                created_at_offset.unix_timestamp(),
                created_at_offset.nanosecond(),
            )
            .unwrap(),
            updated_at: DateTime::from_timestamp(
                updated_at_offset.unix_timestamp(),
                updated_at_offset.nanosecond(),
            )
            .unwrap(),
            version: row.try_get("version")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeploymentProtocol {
    pub id: u16,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, sqlx::mysql::MySqlRow> for DeploymentProtocol {
    fn from_row(row: &sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let created_at_offset: OffsetDateTime = row.try_get("created_at")?;

        Ok(DeploymentProtocol {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            created_at: DateTime::from_timestamp(
                created_at_offset.unix_timestamp(),
                created_at_offset.nanosecond(),
            )
            .unwrap(),
        })
    }
}

pub fn datetime_to_offset(dt: DateTime<Utc>) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(dt.timestamp())
        .unwrap()
        .replace_nanosecond(dt.timestamp_subsec_nanos())
        .unwrap()
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaveConfigRequest {
    pub name: String,
    pub config: serde_json::Value,
}
