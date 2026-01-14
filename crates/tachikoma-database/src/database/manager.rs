use super::pool::{DatabasePool, PoolConfig, PoolError};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Global database pool manager
pub struct PoolManager {
    pools: RwLock<Vec<Arc<DatabasePool>>>,
    primary: RwLock<Option<Arc<DatabasePool>>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(Vec::new()),
            primary: RwLock::new(None),
        }
    }

    /// Initialize primary database pool
    pub async fn init_primary(&self, config: PoolConfig) -> Result<Arc<DatabasePool>, PoolError> {
        let pool = Arc::new(DatabasePool::new(config).await?);

        let mut pools = self.pools.write().await;
        pools.push(pool.clone());

        let mut primary = self.primary.write().await;
        *primary = Some(pool.clone());

        info!("Primary database pool initialized");
        Ok(pool)
    }

    /// Get primary pool
    pub async fn primary(&self) -> Option<Arc<DatabasePool>> {
        self.primary.read().await.clone()
    }

    /// Add additional pool
    pub async fn add_pool(&self, config: PoolConfig) -> Result<Arc<DatabasePool>, PoolError> {
        let pool = Arc::new(DatabasePool::new(config).await?);

        let mut pools = self.pools.write().await;
        pools.push(pool.clone());

        Ok(pool)
    }

    /// Health check all pools
    pub async fn health_check_all(&self) -> Vec<Result<(), PoolError>> {
        let pools = self.pools.read().await;
        let mut results = Vec::new();

        for pool in pools.iter() {
            results.push(pool.health_check().await);
        }

        results
    }

    /// Close all pools
    pub async fn close_all(&self) {
        let pools = self.pools.read().await;
        for pool in pools.iter() {
            pool.close().await;
        }
        info!("All database pools closed");
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref POOL_MANAGER: PoolManager = PoolManager::new();
}