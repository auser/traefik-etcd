use chrono::{Local, Offset};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::{format_description::well_known, UtcOffset};
use tracing_subscriber::{fmt::format, fmt::time::OffsetTime, prelude::*, EnvFilter};

use super::error::TraefikResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub max_level: String,
    pub filter: String,
    pub rolling_file_path: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_level: "debug".to_owned(),
            filter: "traefik-ctl=debug".to_owned(),
            rolling_file_path: Default::default(),
        }
    }
}

pub fn init_tracing(name: &str, log_config: &LogConfig) -> TraefikResult<()> {
    // log
    let mut logfile = None;
    let mut stdout = None;
    if let Some(rolling_file_path) = &log_config.rolling_file_path {
        // logfile
        logfile = Some(tracing_appender::rolling::daily(rolling_file_path, name));
    } else {
        // stdout
        stdout = Some(
            std::io::stdout
                .with_max_level(tracing::Level::from_str(&log_config.max_level).unwrap()),
        );
    }

    // set timer
    let local_offset_sec = Local::now().offset().fix().local_minus_utc();
    let utc_offset = UtcOffset::from_whole_seconds(local_offset_sec)
        .unwrap_or(UtcOffset::from_hms(8, 0, 0).unwrap());
    let timer = OffsetTime::new(utc_offset, well_known::Rfc3339);

    if let Some(stdout) = stdout {
        tracing_subscriber::registry()
            .with(EnvFilter::new(&log_config.filter))
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(format().compact())
                    .with_ansi(false)
                    .with_timer(timer)
                    .with_writer(stdout),
            )
            .try_init()?;
    } else {
        tracing_subscriber::registry()
            .with(EnvFilter::new(&log_config.filter))
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(format().compact())
                    .with_ansi(false)
                    .with_timer(timer)
                    .with_writer(logfile.unwrap()),
            )
            .try_init()?;
    }

    Ok(())
}
