use crate::catalog;
use crate::config::{self, GlobalConfig};
use crate::miner;
use crate::pool;
use crate::wallet::WalletManager;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

const SOCKET_PATH: &str = "/run/strawberryd.sock";

/// Run the daemon — listen on Unix socket, handle requests.
pub async fn run(config: GlobalConfig) -> anyhow::Result<()> {
    let config = Arc::new(config);

    // Remove stale socket if it exists
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)?;
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("Listening on {}", SOCKET_PATH);

    // Set permissions so non-root user can talk to the socket
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o660))?;
    }

    info!("strawberryd v{} ready", env!("CARGO_PKG_VERSION"));
    info!(
        "Telemetry: {}",
        if config.general.telemetry {
            "ON"
        } else {
            "OFF (hardcoded)"
        }
    );

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let cfg = Arc::clone(&config);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, cfg).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

/// Handle a single client connection on the Unix socket.
async fn handle_connection(stream: UnixStream, config: Arc<GlobalConfig>) -> anyhow::Result<()> {
    let (reader_half, mut writer_half) = stream.into_split();
    let mut lines = BufReader::new(reader_half).lines();

    // Send welcome banner
    writer_half
        .write_all(b"strawberryd v0.1.0\n")
        .await?;

    while let Some(line) = lines.next_line().await? {
        let response = process_command(&line, &config).await;
        writer_half.write_all(response.as_bytes()).await?;
        writer_half.write_all(b"\n").await?;
    }

    Ok(())
}

/// Process a single command and return a response string.
async fn process_command(cmd: &str, config: &GlobalConfig) -> String {
    let parts: Vec<&str> = cmd.trim().splitn(5, ' ').collect();

    match parts.as_slice() {
        ["ping"] => "pong".to_string(),
        ["version"] => format!("strawberryd {}", env!("CARGO_PKG_VERSION")),
        ["status"] => format!(
            "status: running\ntelemetry: {}\nlog_level: {}",
            config.general.telemetry, config.general.log_level
        ),
        ["config", "show"] => format!("{:#?}", config),
        ["miner", "list"] => {
            let installed = miner::installed_miners();
            if installed.is_empty() {
                "miners: (none installed)\nTip: use 'strawberry-cli store install <miner>' or the Store app.".to_string()
            } else {
                let catalog = catalog::load_cached(config);
                let miners_arr = catalog.get("miners").and_then(|m| m.as_array());
                let mut out = String::from("Installed miners:\n");
                for id in &installed {
                    let name = miners_arr
                        .and_then(|arr| arr.iter().find(|m| m.get("id").and_then(|i| i.as_str()) == Some(id.as_str())))
                        .and_then(|m| m.get("name").and_then(|n| n.as_str()))
                        .unwrap_or(id);
                    out.push_str(&format!("  {} — run '{}' to launch\n", name, id));
                }
                out
            }
        }
        ["wallet", "list"] => {
            // Reload config from disk to pick up any changes
            let fresh_config = config::load().unwrap_or_else(|_| config.clone());
            if fresh_config.wallets.is_empty() {
                "wallets: (none configured)\nTip: use 'wallet set <coin> <address>' or the Store to add one.".to_string()
            } else {
                let mut out = String::from("wallets:\n");
                for (coin, addr) in &fresh_config.wallets {
                    let truncated = if addr.len() >= 8 {
                        format!("{}...{}", &addr[..4], &addr[addr.len() - 4..])
                    } else {
                        addr.clone()
                    };
                    out.push_str(&format!("  {}: {}\n", coin, truncated));
                }
                out
            }
        }
        ["wallet", "new", coin] => {
            let wm = WalletManager::new(config.storage.data_dir.clone());
            match wm.generate(coin) {
                Ok((address, seed)) => {
                    // Auto-save the address to config
                    let mut cfg = config.clone();
                    cfg.wallets.insert(coin.to_string(), address.clone());
                    let _ = config::save(&cfg);
                    format!(
                        "New {} wallet generated!\nAddress: {}\nSeed: {}\n\nIMPORTANT: Write down the seed and store it safely. It cannot be recovered.",
                        coin, address, seed
                    )
                }
                Err(e) => format!("Failed to generate wallet: {}", e),
            }
        }
        ["wallet", "set", coin, address] => {
            let wm = WalletManager::new(config.storage.data_dir.clone());
            match wm.set_address(coin, address) {
                Ok(()) => {
                    // Persist to config file
                    let mut cfg = config.clone();
                    cfg.wallets.insert(coin.to_string(), address.to_string());
                    if let Err(e) = config::save(&cfg) {
                        format!("Address set but failed to save config: {}", e)
                    } else {
                        format!("{} address set to: {}", coin, address)
                    }
                }
                Err(e) => format!("Failed to set address: {}", e),
            }
        }
        ["pool", "list"] => {
            let pools = pool::built_in_pools();
            let mut out = String::from("Pool Directory:\n");
            for p in &pools {
                out.push_str(&format!(
                    "  {} ({}:{}) [{}] fee={}%\n    Coins: {}\n    {}\n",
                    p.name, p.address, p.port,
                    if p.tls { "TLS" } else { "no-TLS" },
                    p.fee_pct,
                    p.coins.join(", "),
                    p.notes
                ));
            }
            out
        }
        ["pool", "list", coin] => {
            let all_pools = pool::built_in_pools();
            let pools = pool::pools_for_coin(&all_pools, coin);
            if pools.is_empty() {
                format!("No pools found for {}", coin)
            } else {
                let mut out = format!("Pools for {}:\n", coin);
                for p in &pools {
                    out.push_str(&format!(
                        "  {} ({}:{}) fee={}%\n",
                        p.name, p.address, p.port, p.fee_pct
                    ));
                }
                out
            }
        }
        ["pool", "ping", name] => {
            let pools = pool::pool_directory();
            match pools.get(*name) {
                Some(entry) => {
                    let result = pool::measure_latency(&entry.address, entry.port).await;
                    if result.reachable {
                        format!(
                            "{} ({}:{}) — {}ms",
                            entry.name, entry.address, entry.port,
                            result.latency_ms.unwrap_or(0)
                        )
                    } else {
                        format!("{} ({}:{}) — unreachable", entry.name, entry.address, entry.port)
                    }
                }
                None => format!("Unknown pool: {}", name),
            }
        }
        ["pool", "set", coin, name] => {
            let pools = pool::pool_directory();
            match pools.get(*name) {
                Some(entry) => {
                    let mut cfg = config.clone();
                    cfg.pools.insert(
                        coin.to_string(),
                        config::PoolConfig {
                            address: entry.address.clone(),
                            port: entry.port,
                            tls: entry.tls,
                            coin: coin.to_string(),
                            fee_pct: entry.fee_pct,
                        },
                    );
                    if let Err(e) = config::save(&cfg) {
                        format!("Pool set but failed to save config: {}", e)
                    } else {
                        format!("{} pool set to {} ({}:{})", coin, entry.name, entry.address, entry.port)
                    }
                }
                None => format!("Unknown pool: {}", name),
            }
        }
        ["pool", "set", coin, addr, port] => {
            let mut cfg = config.clone();
            cfg.pools.insert(
                coin.to_string(),
                config::PoolConfig {
                    address: addr.to_string(),
                    port: port.parse().unwrap_or(3333),
                    tls: false,
                    coin: coin.to_string(),
                    fee_pct: 0.0,
                },
            );
            if let Err(e) = config::save(&cfg) {
                format!("Pool set but failed to save config: {}", e)
            } else {
                format!("{} pool set to {}:{}", coin, addr, port)
            }
        }
        ["store", "install", "miner", id] => {
            if miner::is_installed(id) {
                let sources = miner::miner_sources();
                let src = sources.iter().find(|s| s.id == *id).unwrap();
                format!("{} is already installed. Run '{}' to launch.", src.name, src.wrapper_name)
            } else {
                match miner::install_miner(id, config).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Install failed: {}", e),
                }
            }
        }
        ["store", "uninstall", "miner", id] => {
            match miner::uninstall_miner(id) {
                Ok(msg) => msg,
                Err(e) => format!("Uninstall failed: {}", e),
            }
        }
        ["store", "list", "miners"] => {
            let catalog = catalog::load_cached(config);
            if let Some(miners) = catalog.get("miners").and_then(|m| m.as_array()) {
                let mut out = String::from("Available miners:\n");
                for m in miners {
                    let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let ver = m.get("version").and_then(|v| v.as_str()).unwrap_or("?");
                    let hw: Vec<String> = m.get("hardware").and_then(|h| h.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_uppercase())).collect())
                        .unwrap_or_default();
                    out.push_str(&format!("  {} v{} [{}]\n", name, ver, hw.join(", ")));
                }
                out
            } else {
                "No miners in catalog".to_string()
            }
        }
        ["store", "list", "wallets"] => {
            let catalog = catalog::load_cached(config);
            if let Some(wallets) = catalog.get("wallets").and_then(|w| w.as_array()) {
                let mut out = String::from("Available wallets:\n");
                for w in wallets {
                    let name = w.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let coin = w.get("coin").and_then(|c| c.as_str()).unwrap_or("?");
                    out.push_str(&format!("  {} ({})\n", name, coin));
                }
                out
            } else {
                "No wallets in catalog".to_string()
            }
        }
        ["store", "list", "pools"] => {
            let catalog = catalog::load_cached(config);
            if let Some(pools) = catalog.get("pools").and_then(|p| p.as_array()) {
                let mut out = String::from("Available pools:\n");
                for p in pools {
                    let name = p.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let coins: Vec<String> = p.get("coins").and_then(|c| c.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    out.push_str(&format!("  {} [{}]\n", name, coins.join(", ")));
                }
                out
            } else {
                "No pools in catalog".to_string()
            }
        }
        ["store", "update"] => {
            let result = catalog::fetch_and_cache(config).await;
            format!("Catalog cached to {}", result.display())
        }
        ["help"] => {
            "Available commands:\n  ping\n  version\n  status\n  config show\n  miner list\n  wallet list\n  wallet new <coin>\n  wallet set <coin> <address>\n  pool list [coin]\n  pool ping <name>\n  pool set <coin> <name|address port>\n  store install <type> <id>\n  store uninstall <type> <id>\n  store list <type>\n  store update\n  help".to_string()
        }
        _ => format!(
            "Unknown command: {}\nType 'help' for available commands.",
            cmd
        ),
    }
}
