//! init.rs
//!
//! requires tracing-subscriber
//!
//! dotenvy = "0.15.6"
//! tracing = "0.1.37"
//! tracing-subscriber = {version = "0.3.16", features=[ "std", "registry", "env-filter", "fmt"]}
//! strum={ version= "0.25.0", features=["derive"]}  # https://stackoverflow.com/questions/69015213/how-can-i-display-an-enum-in-lowercase
//! strum_macros = "0.25.1"
use std::str::FromStr;
use strum::{Display, EnumString};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ConfigLocation {
    Docker,
    NotDocker,
}


///
pub fn init(package_name: &str) {
    // load .env
    // if docker, load .env from the root directory, otherwise use the cargo workspace directory
    let config_location: ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");
    tracing::debug!("[init] config_location: {}", &config_location);

    let dot_env_path = match config_location {
        ConfigLocation::Docker => ".env".to_string(),
        ConfigLocation::NotDocker => {
            // backend/.env
            // frontend/.env
            // etc
            format!("{}/.env", &package_name)
        }
    };

    match dotenvy::from_filename(&dot_env_path) {
        Ok(_) => tracing::debug!("[init] .env found"),
        _ => tracing::debug!("[init] .env not found (need RUST_LOG=debug for tracing)"),
    }
    let env_file_version = std::env::var("ENV_FILE_VERSION").unwrap_or_else(|_| ".env not loaded".to_string());
    tracing::debug!("[init] dot_env_path: {}; env_file_version: {}", &dot_env_path, &env_file_version);

    // start tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("[init] .env file: {}", &dot_env_path);
    tracing::debug!("[init] done");
}
