use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use actix_web::web;
use cloudiful_config::{ReadOptions, read};
use cloudiful_server::{CorsConfig, Server, ServerConfig, TlsConfig};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

use crate::http;
use crate::service::AppServices;

#[derive(Clone)]
pub struct AppState {
    pub services: Arc<AppServices>,
}

pub struct BootstrappedApp {
    server: Server<fn(&mut web::ServiceConfig), web::Data<AppState>>,
}

impl BootstrappedApp {
    pub async fn run(self) -> io::Result<()> {
        self.server
            .start()
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerSection,
    pub database: DatabaseSection,
    pub fakesb: FakeSbSection,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerSection::default(),
            database: DatabaseSection::default(),
            fakesb: FakeSbSection::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
    pub ssl: SslSection,
    pub cors_allowed_origins: Vec<String>,
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ssl: SslSection::default(),
            cors_allowed_origins: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SslSection {
    pub enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub client_ca_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSection {
    pub url: String,
    pub max_connections: u32,
}

impl Default for DatabaseSection {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FakeSbSection {
    pub default_target_id: Option<i64>,
    pub request_timeout_ms: u64,
}

impl Default for FakeSbSection {
    fn default() -> Self {
        Self {
            default_target_id: None,
            request_timeout_ms: 10_000,
        }
    }
}

pub async fn bootstrap(app_name: &str) -> io::Result<BootstrappedApp> {
    let mut config: AppConfig = read(app_name, Some(ReadOptions::with_env_prefix("FAKESB_")))?;
    if config.database.url.trim().is_empty() {
        if let Some(database_url) = env_database_url() {
            config.database.url = database_url;
        }
    }

    init_logging()?;

    if config.database.url.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "database.url must not be empty",
        ));
    }

    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections.max(1))
        .connect(&config.database.url)
        .await
        .map_err(|err| io::Error::other(err.to_string()))?;

    let services = AppServices::new(
        pool,
        config.fakesb.default_target_id,
        config.fakesb.request_timeout_ms,
    );
    services
        .initialize()
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
    let state = web::Data::new(AppState {
        services: Arc::new(services),
    });

    let listen_addr = format!("{}:{}", config.server.host, config.server.port);
    let cors = if config.server.cors_allowed_origins.is_empty() {
        CorsConfig::permissive()
    } else {
        CorsConfig::restricted(config.server.cors_allowed_origins.clone())
            .with_allowed_methods(["GET", "POST", "PUT"])
    };

    let mut server_config = ServerConfig::new()
        .with_listen_addr(listen_addr)
        .with_cors(cors)
        .with_app_data(state);

    if config.server.ssl.enabled {
        let cert_path = config.server.ssl.cert_path.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "server.ssl.cert_path is required",
            )
        })?;
        let key_path = config.server.ssl.key_path.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "server.ssl.key_path is required",
            )
        })?;

        let mut tls = TlsConfig::new()
            .with_cert_path(cert_path)
            .with_cert_key_path(key_path);

        if let Some(client_ca) = config.server.ssl.client_ca_path.clone() {
            tls = tls.with_client_ca(client_ca);
        }

        server_config = server_config.with_tls(tls);
    }

    let validated = server_config
        .build()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err.to_string()))?;

    let server = Server::new(validated, http::configure as fn(&mut web::ServiceConfig));

    Ok(BootstrappedApp { server })
}

fn init_logging() -> io::Result<()> {
    let log_dir = PathBuf::from("log");
    std::fs::create_dir_all(log_dir.join("old"))?;

    let pattern = "[{d(%Y-%m-%d %H:%M:%S%.f)} | {l} | {M}:{L}] {m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let roller = FixedWindowRoller::builder()
        .build("log/old/fakESB-{}.log", 100)
        .map_err(|err| io::Error::other(err.to_string()))?;
    let policy = CompoundPolicy::new(
        Box::new(SizeTrigger::new(50 * 1024 * 1024)),
        Box::new(roller),
    );
    let file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(log_dir.join("fakESB.log"), Box::new(policy))
        .map_err(|err| io::Error::other(err.to_string()))?;

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .map_err(|err| io::Error::other(err.to_string()))?;

    log4rs::init_config(config).map_err(|err| io::Error::other(err.to_string()))?;
    Ok(())
}

fn env_database_url() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
}
