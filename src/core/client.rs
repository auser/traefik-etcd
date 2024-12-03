use async_trait::async_trait;

use crate::{error::TraefikResult, features::KeyValue};

/// A trait defining the core operations for interacting with a key-value store.
///
/// This trait provides an async interface for basic CRUD operations on a key-value store,
/// along with additional functionality like prefix-based operations and TTL management.
///
/// # Type Parameters
///
/// The trait is designed to work with any type that can be converted into a byte vector (`Vec<u8>`).
/// This allows flexibility in the types of keys and values that can be stored.
///
/// # Examples
///
/// ```rust,no_run
/// use your_crate::{StoreClientActor, TraefikResult, KeyValue};
///
/// #[async_trait]
/// impl StoreClientActor for MyStore {
///     async fn get(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<KeyValue> {
///         // Implementation
///     }
///     // ... other implementations
/// }
/// ```
///
/// # Methods
///
/// - `get`: Retrieves a single key-value pair
/// - `get_with_prefix`: Retrieves all key-value pairs with a given prefix
/// - `get_keys`: Retrieves keys matching a pattern
/// - `put`: Stores a key-value pair with optional TTL
/// - `delete`: Removes a single key-value pair
/// - `delete_with_prefix`: Removes all key-value pairs with a given prefix
/// - `touch`: Updates the TTL of an existing key
/// - `put_or_touch`: Creates or updates a key-value pair with TTL
#[async_trait]
pub trait StoreClientActor: Send + Sync {
    async fn get(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<KeyValue>;
    async fn get_with_prefix(&self, key: impl Into<Vec<u8>> + Send)
        -> TraefikResult<Vec<KeyValue>>;
    async fn get_keys(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<Vec<KeyValue>>;
    async fn put(
        &self,
        key: impl Into<Vec<u8>> + Send,
        value: impl Into<Vec<u8>> + Send,
        ttl: Option<i64>,
    ) -> TraefikResult<Option<KeyValue>>;
    async fn delete(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<i64>;
    async fn delete_with_prefix(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<i64>;

    async fn touch(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<()>;
    async fn put_or_touch(
        &self,
        key: impl Into<Vec<u8>> + Send,
        value: impl Into<Vec<u8>> + Send,
        ttl: Option<i64>,
    ) -> TraefikResult<()>;
}

#[derive(Debug, Clone)]
pub struct StoreClient<T>
where
    T: StoreClientActor + Send + Sync,
{
    pub actor: T,
}

impl<T> StoreClient<T>
where
    T: StoreClientActor + Send + Sync,
{
    pub fn new(actor: T) -> Self {
        Self { actor }
    }

    pub async fn get(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<KeyValue> {
        self.actor.get(key).await
    }

    pub async fn get_with_prefix(
        &self,
        key: impl Into<Vec<u8>> + Send,
    ) -> TraefikResult<Vec<KeyValue>> {
        self.actor.get_with_prefix(key).await
    }
    pub async fn get_keys(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<Vec<KeyValue>> {
        self.actor.get_keys(key).await
    }
    pub async fn put(
        &self,
        key: impl Into<Vec<u8>> + Send,
        value: impl Into<Vec<u8>> + Send,
        ttl: Option<i64>,
    ) -> TraefikResult<Option<KeyValue>> {
        self.actor.put(key, value, ttl).await
    }
    pub async fn delete(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<i64> {
        self.actor.delete(key).await
    }
    pub async fn delete_with_prefix(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<i64> {
        self.actor.delete_with_prefix(key).await
    }
    pub async fn touch(&self, key: impl Into<Vec<u8>> + Send) -> TraefikResult<()> {
        self.actor.touch(key).await
    }
    pub async fn put_or_touch(
        &self,
        key: impl Into<Vec<u8>> + Send,
        value: impl Into<Vec<u8>> + Send,
        ttl: Option<i64>,
    ) -> TraefikResult<()> {
        self.actor.put_or_touch(key, value, ttl).await
    }
}
