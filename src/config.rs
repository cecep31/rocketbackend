use std::env;
use std::time::Duration;

pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub db_pool: PoolConfig,
}

pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,
    /// Timeout for acquiring a connection from the pool (in seconds)
    pub connection_timeout: Duration,
    /// Maximum lifetime of a connection (in seconds, 0 = no limit)
    /// Note: Reserved for future use with custom pool manager
    #[allow(dead_code)]
    pub max_lifetime: Option<Duration>,
    /// Idle timeout for connections (in seconds, 0 = no limit)
    /// Note: Reserved for future use with custom pool manager
    #[allow(dead_code)]
    pub idle_timeout: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 20,
            connection_timeout: Duration::from_secs(30),
            max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
            idle_timeout: Some(Duration::from_secs(600)),  // 10 minutes
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "host=localhost user=postgres password=postgres dbname=rocketbackend".to_string()
        });

        let db_pool = PoolConfig {
            max_size: env::var("DB_POOL_MAX_SIZE")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .expect("DB_POOL_MAX_SIZE must be a valid number"),
            connection_timeout: Duration::from_secs(
                env::var("DB_POOL_CONNECTION_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .expect("DB_POOL_CONNECTION_TIMEOUT must be a valid number"),
            ),
            max_lifetime: parse_optional_duration("DB_POOL_MAX_LIFETIME", Some(1800)),
            idle_timeout: parse_optional_duration("DB_POOL_IDLE_TIMEOUT", Some(600)),
        };

        Config {
            port,
            database_url,
            db_pool,
        }
    }
}

/// Parse an optional duration from environment variable
/// If env var is "0", returns None (no limit)
/// If env var is not set, uses default_secs
fn parse_optional_duration(env_key: &str, default_secs: Option<u64>) -> Option<Duration> {
    let value = env::var(env_key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .or(default_secs);

    match value {
        Some(0) => None,
        Some(secs) => Some(Duration::from_secs(secs)),
        None => None,
    }
}
