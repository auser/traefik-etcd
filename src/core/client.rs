use async_trait::async_trait;

use crate::{error::TraefikResult, features::KeyValue};

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
}
