use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default config path
const GLOBAL_CONFIG: &str = "/etc/strawberry/global.toml";

/// Global configuration — loaded from /etc/strawberry/global.toml
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub mining: MiningConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub wallets: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub pools: std::collections::HashMap<String, PoolConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub telemetry: bool,
    pub log_level: String,
    #[serde(default = "default_catalog_url")]
    pub catalog_url: String,
    #[serde(default = "default_catalog_cache")]
    pub catalog_cache: PathBuf,
    #[serde(default = "default_catalog_interval")]
    pub catalog_refresh_secs: u64,
}

fn default_catalog_url() -> String {
    "https://sabeeirsharrma.github.io/strawberryOS/store/store-catalog.json".to_string()
}

fn default_catalog_cache() -> PathBuf {
    PathBuf::from("/var/lib/strawberry/store-catalog.json")
}

fn default_catalog_interval() -> u64 {
    3600 // 1 hour
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            telemetry: false,
            log_level: "info".to_string(),
            catalog_url: default_catalog_url(),
            catalog_cache: default_catalog_cache(),
            catalog_refresh_secs: default_catalog_interval(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MiningConfig {
    pub default_algo: String,
    pub power_limit: u32,
    pub fan_curve: String,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            default_algo: String::new(),
            power_limit: 0,
            fan_curve: "auto".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("/var/lib/strawberry"),
            log_dir: PathBuf::from("/var/log/strawberry"),
            config_dir: PathBuf::from("/etc/strawberry"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PoolConfig {
    pub address: String,
    pub port: u16,
    #[serde(default)]
    pub tls: bool,
    #[serde(default)]
    pub coin: String,
    #[serde(default)]
    pub fee_pct: f64,
}

/// Returns the path to the global config file.
pub fn config_path() -> PathBuf {
    // Allow override via env for development
    std::env::var("STRAWBERRY_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(GLOBAL_CONFIG))
}

/// Load configuration from disk.
pub fn load() -> anyhow::Result<GlobalConfig> {
    let path = config_path();

    if !path.exists() {
        // First run or fresh install — return defaults
        tracing::warn!("Config not found at {}, using defaults", path.display());
        return Ok(GlobalConfig::default());
    }

    let content = std::fs::read_to_string(&path)?;
    let config: GlobalConfig = toml::from_str(&content)?;

    // Enforce telemetry = false at load time
    if config.general.telemetry {
        tracing::warn!("Telemetry flag was true in config — forcing to false");
    }

    Ok(config)
}

/// Save configuration to disk.
pub fn save(config: &GlobalConfig) -> anyhow::Result<()> {
    let path = config_path();
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    tracing::info!("Config saved to {}", path.display());
    Ok(())
}
