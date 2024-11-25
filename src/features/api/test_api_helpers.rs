use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Mutex,
};

use axum::Router;

use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use tracing::debug;

use crate::error::{TraefikError, TraefikResult};
#[cfg(test)]
use crate::features::api::db::test_database::{TestDatabase, TestPoolOptions};

use super::ServerConfig;
use std::sync::atomic::{AtomicUsize, Ordering};

static USER_ATOMIC_COUNTER: Lazy<Mutex<AtomicUsize>> =
    Lazy::new(|| Mutex::new(AtomicUsize::new(1)));

// pub struct TestEnv {
//     test_user: String,
// }
// impl TestDatabaseTrait for TestEnv {}
// impl TestEnv {
//     pub fn new(test_user: String) -> Self {
//         Self { test_user }
//     }
// }

pub async fn prepare_test_database(
    opts: TestPoolOptions,
    test_user: &str,
) -> TraefikResult<TestDatabase> {
    debug!("Preparing test database");
    let db = TestDatabase::new(Some(opts), test_user).await?;
    debug!("Test database created");
    // db.setup(|_| async { Ok(TestEnv::new("test_user".to_string())) })
    //     .await?;
    debug!("Test database setup");
    Ok(db)
}

async fn create_test_app(
    config: &ServerConfig,
    test_user: &str,
) -> TraefikResult<(Router, TestDatabase)> {
    debug!("Loading environment variables");
    dotenvy::dotenv().ok();

    let pool = prepare_test_database(TestPoolOptions::default(), test_user).await?;

    debug!("Building application");
    let app = crate::features::api::server::create_app(config, pool.mysql_pool.clone()).await?;
    Ok((app, pool))
}

pub(crate) async fn create_listener(config: &ServerConfig) -> TraefikResult<TcpListener> {
    let host = config.host.clone();
    let port = config.port;
    debug!("Running server on {}:{}", host, port);
    let addr = SocketAddr::from((Ipv4Addr::from_str(&host).unwrap(), port));
    TcpListener::bind(addr).await.map_err(TraefikError::from)
}

pub struct TestServer {
    pub app: Router,
    pub client: reqwest::Client,
    pub addr: String,
    pub db: TestDatabase,
    pub test_user: String,
    _server: tokio::task::JoinHandle<()>,
}

impl TestServer {
    pub async fn new() -> TraefikResult<Self> {
        let server_config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            database_url: None,
            hmac_key: "".to_string(),
        };
        let test_user_id = get_next_user_count();
        let test_user = format!("test_user_{}", test_user_id);
        let (app, db) = create_test_app(&server_config, &test_user).await?;
        let listener = create_listener(&server_config).await?;
        let addr = format!("http://{}", listener.local_addr().unwrap());

        let test_app = app.clone();
        let server = tokio::spawn(async move {
            axum::serve(listener, test_app.into_make_service())
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();

        Ok(Self {
            app,
            client,
            addr,
            db,
            test_user,
            _server: server,
        })
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        let url = format!("{}{}", self.addr, path);
        self.client.get(url).send().await.unwrap()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self._server.abort();
    }
}

pub async fn setup_test_server() -> TraefikResult<TestServer> {
    let test_server = TestServer::new().await?;
    Ok(test_server)
}

pub async fn with_test_db<F, Fut, T>(test: F)
where
    F: FnOnce(TestDatabase) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>> + Send + 'static,
    T: Send + 'static,
{
    catch_panics();

    let test_user_id = get_next_user_count();
    let test_user = format!("test_user_{}", test_user_id);

    match TestDatabase::new(None, &test_user).await {
        Ok(db) => {
            if let Err(err) = test(db).await {
                panic!("Test failed: {:?}", err);
            }
        }
        Err(e) => {
            panic!("Failed to create test database: {:?}", e);
        }
    }
}

fn catch_panics() {
    // To catch panics
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
    }));
}

fn get_next_user_count() -> usize {
    let counter = USER_ATOMIC_COUNTER.lock().unwrap();
    counter.fetch_add(1, Ordering::SeqCst)
}
