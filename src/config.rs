//! Tries to create a `Config` from a config file:

use std::sync::OnceLock;

use secrecy::{ExposeSecret, Secret, SecretString};
use serde::Deserialize;
use tracing::debug;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub db_config: DatabaseConfig,
    pub app_port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

/// Allocates a static `OnceLock` containing `AppConfig`.
/// This ensures configuration only gets initialized the first time we call this function.
/// Every other caller gets a &'static ref to AppConfig.
pub fn get_or_init_config() -> &'static AppConfig {
    static CONFIG_INIT: OnceLock<AppConfig> = OnceLock::new();
    CONFIG_INIT.get_or_init(|| {
        debug!(
            "{:<12} - Initializing the configuration",
            "get_or_init_config"
        );
        config::Config::builder()
            .add_source(config::File::new(
                "app_config.toml",
                config::FileFormat::Toml,
            ))
            .build()
            .unwrap_or_else(|er| panic!("Fatal Error: While trying to build AppConfig: {er:?}"))
            .try_deserialize::<AppConfig>()
            .unwrap_or_else(|er| {
                panic!("Fatal Error: While deserializing Config to AppConfig: {er:?}")
            })
    })
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> SecretString {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.db_name
        ))
    }
    pub fn connection_string_without_db(&self) -> SecretString {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
