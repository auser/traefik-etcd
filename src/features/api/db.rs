use std::str::FromStr;

use anyhow::Context;
use dotenvy::dotenv;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::error::TraefikResult;

#[derive(Debug)]
pub enum DatabaseType {
    Mysql,
}

impl FromStr for DatabaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mysql" => Ok(DatabaseType::Mysql),
            _ => Err("Unsupported database type".to_string()),
        }
    }
}

pub async fn run_migrations(db_type: DatabaseType, pool: &Pool<MySql>) -> TraefikResult<()> {
    match db_type {
        DatabaseType::Mysql => migrate_mysql(pool).await,
    }
}

pub async fn prepare_database(database_url: Option<String>) -> TraefikResult<Pool<MySql>> {
    // prepare connection pool
    let pool = create_pool(database_url).await?;

    // prepare schema in db if it does not yet exist
    run_migrations(DatabaseType::Mysql, &pool).await?;

    Ok(pool)
}

async fn create_pool(database_url: Option<String>) -> Result<Pool<MySql>, sqlx::Error> {
    dotenv().ok();

    let database_url = database_url.unwrap_or_else(|| {
        std::env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")
            .unwrap()
    });

    // Add multiple statements support to the connection URL
    let database_url = if database_url.contains('?') {
        format!("{}&multiple_statements=true", database_url)
    } else {
        format!("{}?multiple_statements=true", database_url)
    };

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

async fn migrate_mysql(pool: &Pool<MySql>) -> TraefikResult<()> {
    // let sql = include_str!("../../../migrations/20241124033729_initial_schema.sql");

    // let mut tx = pool.begin().await?;
    // sqlx::migrate!("./migrations").run(&mut *pool).await?;
    // sqlx::query(sql).execute(&mut *tx).await?;
    // tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mysql_migration() -> TraefikResult<()> {
        let pool = create_pool(None).await?;
        run_migrations(DatabaseType::Mysql, &pool).await?;
        Ok(())
    }
}
