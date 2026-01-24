use crate::config::PoolConfig;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;

pub type DbPool = Pool;

/// Create a connection pool from the database URL and pool configuration
///
/// Pool configuration options:
/// - max_size: Maximum number of connections in the pool
/// - connection_timeout: Timeout for acquiring a connection
/// - max_lifetime: Maximum lifetime of a connection before recycling
/// - idle_timeout: How long an idle connection stays in the pool
pub fn create_pool(database_url: &str, pool_config: &PoolConfig) -> Pool {
    let mut cfg = Config::new();
    cfg.url = Some(database_url.to_string());

    // Configure pool settings
    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size: pool_config.max_size,
        timeouts: deadpool::managed::Timeouts {
            wait: Some(pool_config.connection_timeout),
            create: Some(pool_config.connection_timeout),
            recycle: Some(pool_config.connection_timeout),
        },
        ..Default::default()
    });

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create database pool")
}
