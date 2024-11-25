use std::{future::Future, sync::Mutex};

use once_cell::sync::Lazy;

use super::test_database::{TestDatabase, TestPoolOptions};
use std::sync::atomic::{AtomicUsize, Ordering};

static WITH_TEST_USER_ATOMIC_COUNTER: Lazy<Mutex<AtomicUsize>> =
    Lazy::new(|| Mutex::new(AtomicUsize::new(1)));

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

// pub async fn with_configured_test_db<F, Fut>(opts: TestPoolOptions, test: F)
// where
//     F: for<'a> FnOnce(TestDatabase) -> Fut + Send + 'static,
//     Fut: Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static,
// {
//     // To catch panics
//     catch_panics();

//     let exec = TestDatabase::new(Some(opts))
//         .await
//         .expect("unable to get test db");
//     if let Err(err) = test(exec).await {
//         panic!("test failed: {:?}", err);
//     }
// }

fn catch_panics() {
    // To catch panics
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        // TODO: close down databases?
    }));
}

fn get_next_user_count() -> usize {
    let counter = WITH_TEST_USER_ATOMIC_COUNTER.lock().unwrap();
    counter.fetch_add(1, Ordering::SeqCst)
}
