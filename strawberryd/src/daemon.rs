use crate::config::GlobalConfig;
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
        let response = process_command(&line, &config);
        writer_half.write_all(response.as_bytes()).await?;
        writer_half.write_all(b"\n").await?;
    }

    Ok(())
}

/// Process a single command and return a response string.
fn process_command(cmd: &str, config: &GlobalConfig) -> String {
    let parts: Vec<&str> = cmd.trim().splitn(3, ' ').collect();

    match parts.as_slice() {
        ["ping"] => "pong".to_string(),
        ["version"] => format!("strawberryd {}", env!("CARGO_PKG_VERSION")),
        ["status"] => format!(
            "status: running\ntelemetry: {}\nlog_level: {}",
            config.general.telemetry, config.general.log_level
        ),
        ["config", "show"] => format!("{:#?}", config),
        ["miner", "list"] => {
            "miners: (none installed)\nTip: use the Strawberry Store to install miners.".to_string()
        }
        ["wallet", "list"] => {
            if config.wallets.is_empty() {
                "wallets: (none configured)\nTip: use 'wallet set <coin> <address>' or the Store to add one.".to_string()
            } else {
                let mut out = String::from("wallets:\n");
                for (coin, addr) in &config.wallets {
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
        ["help"] => {
            "Available commands: ping, version, status, config show, miner list, wallet list, help"
                .to_string()
        }
        _ => format!(
            "Unknown command: {}\nType 'help' for available commands.",
            cmd
        ),
    }
}
