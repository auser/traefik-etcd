use std::time::Duration;

use once_cell::sync::Lazy;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::error::TraefikResult;

// #[cfg(test)]
// pub(crate) mod macs;
#[cfg(test)]
pub(crate) mod test_database;

pub static DB: Lazy<Pool<MySql>> = Lazy::new(|| {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        create_pool(None)
            .await
            .expect("Failed to create database pool")
    })
});

async fn create_pool(database_url: Option<String>) -> Result<Pool<MySql>, sqlx::Error> {
    let database_url = database_url.unwrap_or_else(|| {
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:password@localhost/traefik_config".to_string())
    });

    MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
}

pub async fn prepare_database(database_url: Option<String>) -> TraefikResult<Pool<MySql>> {
    // Initialize the database pool
    let pool = create_pool(database_url).await?;

    //     // Run migrations
    //     let test_migration = r#"
    //     -- test_setup.sql
    // DROP DATABASE IF EXISTS traefik_config_test;
    // CREATE DATABASE traefik_config_test;
    // USE traefik_config_test;"#;
    //     sqlx::query(test_migration).execute(&pool).await?;
    Ok(pool)
}

pub mod operations {
    use super::*;
    pub mod protocols {

        use crate::features::api::DeploymentProtocol;

        use super::*;

        pub async fn get_deployment_protocols(
            pool: &Pool<MySql>,
        ) -> Result<Vec<DeploymentProtocol>, sqlx::Error> {
            sqlx::query_as::<_, DeploymentProtocol>(
                r#"
            SELECT id, name, created_at
            FROM deployment_protocols
            ORDER BY id
            "#,
            )
            .fetch_all(pool)
            .await
        }
    }

    pub mod configs {
        use sqlx::{MySql, Pool};

        use crate::config::traefik_config::TraefikConfigVersion;

        pub async fn get_configs(
            pool: &Pool<MySql>,
        ) -> Result<Vec<TraefikConfigVersion>, sqlx::Error> {
            sqlx::query_as::<_, TraefikConfigVersion>(
                r#"
            SELECT id, name, config, created_at, updated_at, version
            FROM config_versions
            ORDER BY created_at DESC
            "#,
            )
            .fetch_all(pool)
            .await
        }

        pub async fn save_config(
            pool: &Pool<MySql>,
            name: String,
            config: serde_json::Value,
        ) -> Result<TraefikConfigVersion, sqlx::Error> {
            let result = sqlx::query(
                r#"
            INSERT INTO config_versions (name, config)
            VALUES (?, ?)
            "#,
            )
            .bind(name)
            .bind(config)
            .execute(pool)
            .await?;

            let id = result.last_insert_id();

            sqlx::query_as::<_, TraefikConfigVersion>(
                r#"
            SELECT id, name, config, created_at, updated_at, version
            FROM config_versions
            WHERE id = ?
            "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await
        }
    }
}
