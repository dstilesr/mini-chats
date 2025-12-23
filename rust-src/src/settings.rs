use std::env;

const VERSION: &'static str = "v0.1.0";

#[derive(Debug)]
pub struct AppSettings {
    pub port: u32,
    pub static_path: String,
    pub log_level: String,
    pub environment: String,
    pub version: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            port: 3501,
            static_path: "assets".to_string(),
            log_level: "INFO".to_string(),
            environment: "dev".to_string(),
            version: VERSION.to_string(),
        }
    }
}

impl AppSettings {
    /// Instantiate the application settings from environment variables
    pub fn new() -> Self {
        let default = Self::default();
        let port_str = env::var("APP_PORT");
        let static_path = env::var("APP_STATIC_PATH").unwrap_or(default.static_path);
        let environment = env::var("APP_ENVIRONMENT").unwrap_or(default.environment);
        let log_level = env::var("APP_LOG_LEVEL").unwrap_or(default.log_level);
        let version = env::var("APP_VERSION").unwrap_or(default.version);

        let port = port_str
            .unwrap_or(format!("{}", default.port))
            .parse()
            .unwrap_or(default.port);

        Self {
            static_path,
            environment,
            port,
            log_level,
            version,
        }
    }
}
