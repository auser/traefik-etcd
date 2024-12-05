use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::Row;
use time::OffsetDateTime;
use utoipa::ToSchema;

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
