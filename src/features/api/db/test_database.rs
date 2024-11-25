use crate::error::{TraefikError, TraefikResult};
use once_cell::sync::Lazy;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::Executor;
use sqlx::{MySql, Pool};
use std::future::Future;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tracing::debug;
use url::Url;

static ROOT_USER_PASSWORD: &str = "mysql";
static TEST_USER_PASSWORD: &str = "testpassword";
static TEST_SQL_HOST: &str = "mysql";
static ATOMIC_COUNTER: Lazy<Mutex<AtomicUsize>> = Lazy::new(|| Mutex::new(AtomicUsize::new(1)));

#[derive(Debug, Clone)]
pub struct TestPoolOptions {
    pub user: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub tracing: bool,
}

impl Default for TestPoolOptions {
    fn default() -> Self {
        Self {
            user: Some("root".to_string()),
            password: Some(ROOT_USER_PASSWORD.to_string()),
            host: Some(TEST_SQL_HOST.to_string()),
            port: Some(3306),
            tracing: false,
        }
    }
}

#[async_trait::async_trait]
pub trait TestDatabaseTrait: Send + Sync {}

#[derive(Debug, Clone)]
pub struct TestDatabase {
    pub uri: String,
    pub mysql_pool: Pool<MySql>,
    pub test_pool: Pool<MySql>,
    pub test_user: String,
}

impl TestDatabase {
    pub async fn new(opts: Option<TestPoolOptions>, test_user: &str) -> TraefikResult<Self> {
        dotenvy::dotenv().ok();
        let opts = opts.unwrap_or_default();
        let (uri, mysql_pool, test_pool, test_user) =
            create_test_pool(Some(opts), test_user).await?;
        debug!("Test database created");
        Ok(Self {
            uri,
            mysql_pool,
            test_pool,
            test_user,
        })
    }

    pub async fn setup<F, Fut, T>(&self, setup_fn: F) -> TraefikResult<T>
    where
        F: FnOnce(Pool<MySql>) -> Fut + Send + Sync,
        Fut: Future<Output = TraefikResult<T>> + Send,
        T: TestDatabaseTrait + Send + 'static,
    {
        grant_permissions(&self.uri, &self.test_user).await?;
        setup_fn(self.mysql_pool.clone()).await
    }
}

pub async fn create_test_pool(
    opts: Option<TestPoolOptions>,
    test_user: &str,
) -> TraefikResult<(String, Pool<MySql>, Pool<MySql>, String)> {
    let opts = opts.unwrap_or_default();
    if opts.tracing {
        let _ = env_logger::try_init();
    }

    let database_name = generate_database_name();
    let root_mysql = generate_database_uri(&opts, "mysql");
    let admin_pool = create_connection_pool(&root_mysql).await?;
    debug!("Admin pool created");
    debug!("Creating database: {:?}", database_name);

    let create_db_sql = format!("CREATE DATABASE IF NOT EXISTS {}", database_name);
    debug!("Creating database: {:?}", create_db_sql);
    let mut txn = admin_pool.begin().await?;
    txn.execute(&*create_db_sql).await?;
    txn.commit().await?;
    debug!("Database created");

    let mysql_uri = generate_database_uri(&opts, &database_name);
    debug!("Creating connection pool: {:?}", mysql_uri);
    let mysql_pool = create_connection_pool(&mysql_uri).await?;
    debug!("Connection pool created");
    // if let Err(e) = sync_drop_database(&mysql_uri) {
    //     tracing::warn!("Failed to drop existing database: {:?}", e);
    // }

    debug!("Creating database: {:?}", mysql_uri);
    create_database(&mysql_uri, &database_name).await?;
    debug!("Database created");

    debug!("Running migrations");
    run_migrations(&mysql_uri, &database_name).await?;
    debug!("Migrations run");

    debug!("Creating test user");
    create_test_user(&mysql_uri, test_user).await?;
    debug!("Test user created");

    debug!("Granting permissions");
    grant_permissions(&mysql_uri, test_user).await?;
    debug!("Permissions granted");

    let test_db_uri = generate_test_db_uri(&opts, &database_name, test_user);
    debug!("Creating test connection pool: {:?}", test_db_uri);
    let test_pool = create_connection_pool(&test_db_uri).await?;
    debug!("Test connection pool created");

    Ok((mysql_uri, mysql_pool, test_pool, test_user.to_string()))
}

pub async fn run_migrations(database_uri: &str, database_name: &str) -> TraefikResult<()> {
    let pool = create_connection_pool(database_uri).await?;
    let mut conn = pool.acquire().await?;
    let sql = format!(
        r#"
    USE {database_name};
    "#
    );
    debug!("Running migrations: {:?}", sql);

    conn.execute(&*sql)
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;

    sqlx::migrate!("./migrations")
        .run(&mut *conn)
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;
    Ok(())
}

async fn create_connection_pool(database_uri: &str) -> TraefikResult<Pool<MySql>> {
    debug!("Creating connection pool: {:?}", database_uri);
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_uri)
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;
    Ok(pool)
}

async fn create_database(database_uri: &str, database_name: &str) -> TraefikResult<()> {
    let pool = create_connection_pool(database_uri).await?;
    // let mut conn = pool.get_conn()?;
    let mut conn = pool.acquire().await?;

    let create_db_sql = format!("CREATE DATABASE IF NOT EXISTS {}", database_name);
    debug!("Creating database: {:?}", create_db_sql);
    conn.execute(&*create_db_sql)
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;

    Ok(())
}

async fn create_test_user(database_uri: &str, test_user: &str) -> TraefikResult<()> {
    let pool = create_connection_pool(database_uri).await?;
    let mut conn = pool.acquire().await?;

    let create_user_sql = format!(
        "CREATE USER IF NOT EXISTS '{}'@'%' IDENTIFIED BY '{}'",
        test_user, TEST_USER_PASSWORD
    );
    debug!("Creating test user: {:?}", create_user_sql);
    conn.execute(&*create_user_sql)
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;

    Ok(())
}

async fn grant_permissions(database_uri: &str, test_user: &str) -> TraefikResult<()> {
    let pool = create_connection_pool(database_uri).await?;
    let mut conn = pool.acquire().await?;

    let parsed_url = Url::parse(database_uri).map_err(|e| TraefikError::ParsingError(e.into()))?;
    let database_name = parsed_url.path().trim_start_matches('/');

    // GRANT ALL PRIVILEGES ON *.* TO 'root'@'localhost' IDENTIFIED BY 'password';
    let grant_sql = format!(
        "GRANT ALL PRIVILEGES ON {}.* TO '{}'@'%'",
        database_name, test_user
    );
    debug!("Granting permissions: {:?}", grant_sql);
    conn.execute(&*grant_sql)
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;

    conn.execute("FLUSH PRIVILEGES")
        .await
        .map_err(|e| TraefikError::DatabaseError(e.to_string()))?;

    Ok(())
}

// pub async fn connect_to_db(database_uri: &str) -> TraefikResult<Pool<MySql>> {
//     let pool = create_connection_pool(database_uri).await?;
//     debug!("Connecting to database: {:?}", database_uri);

//     for attempt in 0..3 {
//         match pool.acquire().await {
//             Ok(_) => {
//                 debug!("Connected to {database_uri}");
//                 return Ok(pool);
//             }
//             Err(e) => {
//                 if attempt == 2 {
//                     return Err(TraefikError::DatabaseError(format!(
//                         "Failed to connect after 3 attempts: {}",
//                         e
//                     )));
//                 }
//                 tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//             }
//         }
//     }
//     unreachable!()
// }

pub fn sync_drop_database(database_uri: &str) -> TraefikResult<()> {
    let parsed = Url::parse(database_uri)?;
    let database_name = parsed.path().trim_start_matches('/');
    let test_user = parsed.username();

    // Execute all commands in a single MySQL session
    let mut cmd = Command::new("mysql");
    cmd.arg("-h")
        .arg(parsed.host_str().unwrap_or("localhost"))
        .arg("-u")
        .arg("root")
        .arg(if let Some(pass) = parsed.password() {
            format!("-p{}", pass)
        } else {
            "-p".to_string()
        })
        .arg("-e")
        .arg(format!(
            r#"
            SELECT CONCAT('KILL ', id, ';') 
            FROM INFORMATION_SCHEMA.PROCESSLIST 
            WHERE db = '{database_name}';
            DROP DATABASE IF EXISTS {database_name};
            DROP USER IF EXISTS '{test_user}'@'localhost';
            "#
        ));

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(TraefikError::DatabaseDropFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if let Err(e) = sync_drop_database(&self.uri) {
            tracing::error!("Failed to drop database: {:?}", e);
        }
    }
}
// pub fn sync_drop_database(database_uri: &str) -> Result<(), TraefikError> {
//     let parsed = Url::parse(database_uri).map_err(|e| TraefikError::DatabaseError(e.to_string()))?;
//     let database_name = parsed.path().trim_start_matches('/').to_string();
//     let test_user = parsed.username().to_string();
//     let database_uri = database_uri.to_string();

//     tokio::task::spawn_blocking(move || {
//         let rt = tokio::runtime::Runtime::new().map_err(|e| TraefikError::DatabaseError(e.to_string()))?;
//         rt.block_on(async move {
//             let pool = create_connection_pool(&database_uri).await.map_err(|e| TraefikError::DatabaseError(e.to_string()))?;
//             let mut conn = pool.acquire().await.map_err(|e| TraefikError::DatabaseError(e.to_string()))?;

//             conn.execute(&*format!(
//                 "SELECT CONCAT('KILL ', id, ';') FROM INFORMATION_SCHEMA.PROCESSLIST WHERE db = '{}'",
//                 database_name
//             ))
//             .await
//             .map_err(|e| TraefikError::DatabaseDropFailed(e.to_string()));

//             conn.execute(&*format!("DROP DATABASE IF EXISTS {}", database_name))
//                 .await
//                 .map_err(|e| TraefikError::DatabaseDropFailed(e.to_string()));

//             conn.execute(&*format!("DROP USER IF EXISTS '{}'@'{TEST_SQL_HOST}'", test_user))
//                 .await
//                 .map_err(|e| TraefikError::DatabaseDropFailed(e.to_string()));

//             Ok(())
//         })
//         .expect("Failed to drop database");
//         Ok(())
//     })
//     .map_err(|e| TraefikError::DatabaseError(e.to_string()))
// }

fn generate_database_name() -> String {
    format!("test_traefikctl_{}", get_next_count())
}

fn generate_database_uri(opts: &TestPoolOptions, database_name: &str) -> String {
    let port = opts.port.unwrap_or(3306);
    let host = opts.host.as_deref().unwrap_or("mysql");
    let user = opts.user.as_deref().unwrap_or("root");
    let password = opts.password.as_deref().unwrap_or("mysql");

    format!("mysql://{user}:{password}@{host}:{port}/{database_name}")
}

fn generate_test_db_uri(opts: &TestPoolOptions, database_name: &str, test_user: &str) -> String {
    let port = opts.port.unwrap_or(3306);
    let host = opts.host.as_deref().unwrap_or("localhost");
    format!("mysql://{test_user}:{TEST_USER_PASSWORD}@{host}:{port}/{database_name}")
}

fn get_next_count() -> usize {
    let counter = ATOMIC_COUNTER.lock().unwrap();
    counter.fetch_add(1, Ordering::SeqCst)
}
