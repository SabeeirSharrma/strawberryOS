use std::collections::HashMap;
use std::time::Duration;
use tokio::net::TcpStream;
use tracing::debug;

/// A known mining pool entry in the built-in directory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolEntry {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub coins: Vec<String>,
    pub fee_pct: f64,
    pub tls: bool,
    pub notes: String,
}

/// Latency measurement result for a pool.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LatencyResult {
    pub pool: String,
    pub address: String,
    pub port: u16,
    pub latency_ms: Option<u64>,
    pub reachable: bool,
}

/// Built-in pool directory — curated, zero-telemetry.
/// These are well-known public mining pools. No user data is sent here;
/// we only measure TCP connect latency (anonymous).
pub fn built_in_pools() -> Vec<PoolEntry> {
    vec![
        // Monero pools
        PoolEntry {
            name: "P2Pool".to_string(),
            address: "p2pool.io".to_string(),
            port: 3333,
            coins: vec!["monero".to_string()],
            fee_pct: 0.0,
            tls: false,
            notes: "Decentralized pool, no registration needed".to_string(),
        },
        PoolEntry {
            name: "MoneroOcean".to_string(),
            address: "gulf.moneroocean.stream".to_string(),
            port: 10128,
            coins: vec!["monero".to_string()],
            fee_pct: 0.2,
            tls: false,
            notes: "Auto algo-switching, pays in XMR".to_string(),
        },
        PoolEntry {
            name: "SupportXMR".to_string(),
            address: "pool.supportxmr.com".to_string(),
            port: 3333,
            coins: vec!["monero".to_string()],
            fee_pct: 0.6,
            tls: false,
            notes: "Long-running, reliable pool".to_string(),
        },
        PoolEntry {
            name: "XMRPool".to_string(),
            address: "xmrpool.eu".to_string(),
            port: 9999,
            coins: vec!["monero".to_string()],
            fee_pct: 0.0,
            tls: true,
            notes: "EU-based, PPLNS".to_string(),
        },
        // Zcash pools
        PoolEntry {
            name: "Flypool Zcash".to_string(),
            address: "zcash.flypool.org".to_string(),
            port: 4444,
            coins: vec!["zcash".to_string()],
            fee_pct: 1.0,
            tls: true,
            notes: "Popular Equihash pool".to_string(),
        },
        // Ravencoin pools
        PoolEntry {
            name: "Flypool RVN".to_string(),
            address: "rvn.flypool.org".to_string(),
            port: 4444,
            coins: vec!["ravencoin".to_string()],
            fee_pct: 1.0,
            tls: true,
            notes: "KawPow algorithm".to_string(),
        },
        // Ergo pools
        PoolEntry {
            name: "Nanopool ERG".to_string(),
            address: "ergo.nanopool.org".to_string(),
            port: 11111,
            coins: vec!["ergo".to_string()],
            fee_pct: 1.0,
            tls: false,
            notes: "Autolykos2 algorithm".to_string(),
        },
    ]
}

/// Measure TCP connect latency to a pool (anonymous, no data sent).
pub async fn measure_latency(address: &str, port: u16) -> LatencyResult {
    let addr = format!("{}:{}", address, port);
    debug!("Measuring latency to {}", addr);

    let result = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await;

    match result {
        Ok(Ok(_)) => {
            // Measure actual connect time
            let start = std::time::Instant::now();
            let _ = TcpStream::connect(&addr).await;
            let elapsed = start.elapsed().as_millis() as u64;

            LatencyResult {
                pool: String::new(),
                address: address.to_string(),
                port,
                latency_ms: Some(elapsed),
                reachable: true,
            }
        }
        _ => LatencyResult {
            pool: String::new(),
            address: address.to_string(),
            port,
            latency_ms: None,
            reachable: false,
        },
    }
}

/// Score a pool by latency (lower is better). Returns f64 score.
pub fn score_pool(latency_ms: Option<u64>, fee_pct: f64) -> f64 {
    match latency_ms {
        Some(ms) => {
            // Lower latency + lower fee = better score
            // Score: 1000 - latency_ms - (fee_pct * 100)
            1000.0 - (ms as f64) - (fee_pct * 100.0)
        }
        None => -1.0, // Unreachable pools get worst score
    }
}

/// Find pools that support a given coin.
pub fn pools_for_coin<'a>(pools: &'a [PoolEntry], coin: &str) -> Vec<&'a PoolEntry> {
    pools.iter()
        .filter(|p| p.coins.iter().any(|c| c.eq_ignore_ascii_case(coin)))
        .collect()
}

/// Get the built-in directory as a HashMap keyed by name (for config storage).
pub fn pool_directory() -> HashMap<String, PoolEntry> {
    built_in_pools()
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect()
}
