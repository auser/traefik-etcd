use chrono::{DateTime, Local, Utc};

// If you need custom DateTime serialization:
pub mod datetime_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.to_rfc3339())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f")
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

// Convert to UTC if needed
pub fn to_utc(date: DateTime<Local>) -> DateTime<Utc> {
    date.with_timezone(&Utc)
}

pub fn now_utc() -> DateTime<Local> {
    Local::now()
}
