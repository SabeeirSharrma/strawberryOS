mod config;
mod daemon;
mod pool;
mod wallet;

use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "strawberryd")]
#[command(about = "Strawberry OS system daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the daemon in the foreground (for development)
    Run,
    /// Print the resolved config and exit
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing — respects RUST_LOG env, defaults to info
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "strawberryd=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run) | None => {
            info!("Starting strawberryd v{}", env!("CARGO_PKG_VERSION"));
            let cfg = config::load()?;
            info!("Config loaded from {}", config::config_path().display());
            daemon::run(cfg).await?;
        }
        Some(Commands::Config) => {
            let cfg = config::load()?;
            println!("{:#?}", cfg);
        }
    }

    Ok(())
}
