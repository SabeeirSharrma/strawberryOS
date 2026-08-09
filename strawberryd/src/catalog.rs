use crate::config::GlobalConfig;
use std::path::PathBuf;
use tracing::{error, info, warn};

/// Fetch the store catalog from the remote URL and cache it locally.
/// Returns the path to the cached catalog.
pub async fn fetch_and_cache(config: &GlobalConfig) -> PathBuf {
    let cache_path = config.general.catalog_cache.clone();
    let url = &config.general.catalog_url;

    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Try fetching from remote
    match reqwest::get(url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text().await {
                    Ok(body) => {
                        // Validate it's valid JSON
                        if serde_json::from_str::<serde_json::Value>(&body).is_ok() {
                            if let Err(e) = std::fs::write(&cache_path, &body) {
                                error!("Failed to write catalog cache: {}", e);
                            } else {
                                info!("Catalog updated from {}", url);
                            }
                        } else {
                            warn!("Remote catalog is invalid JSON, keeping cached version");
                        }
                    }
                    Err(e) => warn!("Failed to read catalog response: {}", e),
                }
            } else {
                warn!("Remote catalog returned status {}, keeping cached version", resp.status());
            }
        }
        Err(e) => {
            warn!("Failed to fetch catalog from {}: {}", url, e);
        }
    }

    cache_path
}

/// Load the catalog from the local cache.
/// Falls back to the bundled catalog if cache doesn't exist.
pub fn load_cached(config: &GlobalConfig) -> serde_json::Value {
    let cache_path = &config.general.catalog_cache;

    // Try cache first
    if let Ok(content) = std::fs::read_to_string(cache_path) {
        if let Ok(catalog) = serde_json::from_str(&content) {
            return catalog;
        }
    }

    // Fallback to bundled catalog
    let bundled = "/etc/strawberry/store-catalog.json";
    if let Ok(content) = std::fs::read_to_string(bundled) {
        if let Ok(catalog) = serde_json::from_str(&content) {
            info!("Using bundled catalog (cache not available)");
            return catalog;
        }
    }

    warn!("No catalog available");
    serde_json::json!({"miners": [], "wallets": [], "pools": []})
}

/// Background refresh task — runs every `catalog_refresh_secs`.
pub async fn background_refresh(config: GlobalConfig) {
    let interval = std::time::Duration::from_secs(config.general.catalog_refresh_secs);
    loop {
        tokio::time::sleep(interval).await;
        info!("Background catalog refresh");
        fetch_and_cache(&config).await;
    }
}
