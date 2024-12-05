use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize)]
pub enum TraefikConfigSource {
    Database,
    File,
    Default,
    New,
}

#[derive(Serialize, Deserialize)]
pub struct TraefikConfigListItem {
    pub id: i64, // negative for files, 0 for default/new, positive for DB
    pub name: String,
    pub source: TraefikConfigSource,
    pub updated_at: DateTime<Utc>,
}

// impl FromRow<'_, sqlx::mysql::MySqlRow> for TraefikConfigVersion {
//     fn from_row(row: &sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
//         let created_at_offset: OffsetDateTime = row.try_get("created_at")?;
//         let updated_at_offset: OffsetDateTime = row.try_get("updated_at")?;

//         Ok(TraefikConfigVersion {
//             id: row.try_get("id")?,
//             name: row.try_get("name")?,
//             config: row.try_get("config")?,
//             created_at: DateTime::from_timestamp(
//                 created_at_offset.unix_timestamp(),
//                 created_at_offset.nanosecond(),
//             )
//             .unwrap(),
//             updated_at: DateTime::from_timestamp(
//                 updated_at_offset.unix_timestamp(),
//                 updated_at_offset.nanosecond(),
//             )
//             .unwrap(),
//             version: row.try_get("version")?,
//         })
//     }
// }

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SaveConfigRequest {
    pub name: String,
    pub config: serde_json::Value,
}

impl From<SaveConfigRequest> for String {
    fn from(value: SaveConfigRequest) -> Self {
        serde_json::to_string(&value).unwrap()
    }
}

impl From<&SaveConfigRequest> for String {
    fn from(value: &SaveConfigRequest) -> Self {
        serde_json::to_string(value).unwrap()
    }
}
