use http_cache::{CACacheManager, CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;

pub struct MyCacheManager;

type Result<T> = http_cache::Result<T>;

#[async_trait::async_trait]
impl CacheManager for MyCacheManager {
    /// Attempts to pull a cached response and related policy from cache.
    async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
        todo!()
    }
    /// Attempts to cache a response and related policy.
    async fn put(
        &self,
        cache_key: String,
        res: HttpResponse,
        policy: CachePolicy,
    ) -> Result<HttpResponse> {
        todo!()
    }
    /// Attempts to remove a record from cache.
    async fn delete(&self, cache_key: &str) -> Result<()> {
        todo!()
    }
}
