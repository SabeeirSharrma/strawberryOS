use std::path::{Path, PathBuf};
use std::process::Command;

/// Wallet management for Strawberry OS.
/// Non-custodial: seeds and keys stay local, never transmitted.
///
/// Supported coins:
/// - Monero (XMR): via `monero-wallet-cli` from the `monero` package

const WALLET_DIR: &str = "/var/lib/strawberry/wallets";

/// Supported coin definitions
pub struct CoinDef {
    pub id: &'static str,
    pub name: &'static str,
    pub binary: &'static str,
    pub network_flag: &'static str, // e.g. "" for mainnet, "--testnet" for testnet
}

pub const SUPPORTED_COINS: &[CoinDef] = &[CoinDef {
    id: "monero",
    name: "Monero (XMR)",
    binary: "monero-wallet-cli",
    network_flag: "",
}];

pub struct WalletManager {
    data_dir: PathBuf,
}

impl WalletManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Get the wallet directory for a specific coin
    fn wallet_dir(&self, coin: &str) -> PathBuf {
        self.data_dir.join("wallets").join(coin)
    }

    /// Generate a new wallet for the given coin.
    /// Returns (address, seed) on success.
    pub fn generate(&self, coin: &str) -> anyhow::Result<(String, String)> {
        let coin_def = SUPPORTED_COINS
            .iter()
            .find(|c| c.id == coin)
            .ok_or_else(|| anyhow::anyhow!("Unsupported coin: {}", coin))?;

        // Check if the binary is available
        let binary_path = format!("/usr/bin/{}", coin_def.binary);
        if !Path::new(&binary_path).exists() {
            anyhow::bail!(
                "{} not found at {}. Install it with: pacman -S {}",
                coin_def.binary,
                binary_path,
                coin
            );
        }

        // Create wallet directory
        let wallet_dir = self.wallet_dir(coin);
        std::fs::create_dir_all(&wallet_dir)?;

        // Generate wallet filename from timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let wallet_name = format!("{}_{}", coin, timestamp);
        let wallet_path = wallet_dir.join(&wallet_name);

        // Run monero-wallet-cli to generate the wallet
        let output = Command::new(&binary_path)
            .arg(format!("--generate-new-wallet={}", wallet_path.display()))
            .arg("--password")
            .arg("")
            .arg("--mnemonic-language")
            .arg("English")
            .arg("--offline") // Don't need daemon for generation
            .arg("--command")
            .arg("quit")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            anyhow::bail!(
                "Failed to generate wallet:\nstdout: {}\nstderr: {}",
                stdout,
                stderr
            );
        }

        // Parse the address from stdout
        let address = parse_address(&stdout)
            .ok_or_else(|| anyhow::anyhow!("Could not parse address from output:\n{}", stdout))?;

        // Parse the mnemonic seed from stdout
        let seed = parse_seed(&stdout)
            .unwrap_or_else(|| "Seed not captured — check wallet file".to_string());

        tracing::info!("Generated {} wallet: {}", coin, address);

        Ok((address, seed))
    }

    /// List configured wallets from the config wallets section.
    /// Returns list of (coin, address) pairs.
    pub fn list(&self, wallets: &std::collections::HashMap<String, String>) -> Vec<(String, String)> {
        wallets
            .iter()
            .map(|(coin, addr)| (coin.clone(), addr.clone()))
            .collect()
    }

    /// Set an existing wallet address for a coin.
    pub fn set_address(
        &self,
        coin: &str,
        address: &str,
    ) -> anyhow::Result<()> {
        // Validate the coin is supported
        SUPPORTED_COINS
            .iter()
            .find(|c| c.id == coin)
            .ok_or_else(|| anyhow::anyhow!("Unsupported coin: {}", coin))?;

        // Basic address validation (Monero addresses start with 4 and are 95 chars)
        if coin == "monero" && (!address.starts_with('4') || address.len() != 95) {
            anyhow::bail!(
                "Invalid Monero address. Expected 95 chars starting with '4', got {} chars starting with '{}'",
                address.len(),
                &address[..1.min(address.len())]
            );
        }

        tracing::info!("Setting {} address: {}", coin, address);
        Ok(())
    }

    /// Check if a wallet exists for the given coin
    pub fn exists(&self, coin: &str, wallets: &std::collections::HashMap<String, String>) -> bool {
        wallets.contains_key(coin)
    }
}

/// Parse Monero address from monero-wallet-cli output
fn parse_address(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        // Address line looks like: "4<94 chars>" or is labeled "address"
        if line.starts_with('4') && line.len() == 95 {
            return Some(line.to_string());
        }
        // Also check for "Generated new wallet: <address>" pattern
        if let Some(addr) = line.strip_prefix("Generated new wallet: ") {
            let addr = addr.trim().to_string();
            if addr.starts_with('4') && addr.len() == 95 {
                return Some(addr);
            }
        }
    }
    None
}

/// Parse mnemonic seed from monero-wallet-cli output
/// Seeds are 25 words, may span multiple lines
fn parse_seed(output: &str) -> Option<String> {
    // Collect all words between the NOTE line and the closing stars
    let lines: Vec<&str> = output.lines().collect();
    let mut seed_words: Vec<String> = Vec::new();
    let mut in_seed = false;

    for line in lines {
        let trimmed = line.trim();
        // Start capturing after the "NOTE:" line about recovery words
        if trimmed.contains("the following 25 words") {
            in_seed = true;
            continue;
        }
        // Stop at the closing separator
        if in_seed && trimmed.starts_with('*') {
            break;
        }
        if in_seed && !trimmed.is_empty() {
            for word in trimmed.split_whitespace() {
                // Filter out non-word tokens
                if word.chars().all(|c| c.is_alphabetic()) && word.len() > 1 {
                    seed_words.push(word.to_string());
                }
            }
        }
    }

    if seed_words.len() == 25 {
        Some(seed_words.join(" "))
    } else if !seed_words.is_empty() {
        Some(seed_words.join(" "))
    } else {
        // Fallback: look for 25 words on a single line
        for line in output.lines() {
            let line = line.trim().to_string();
            if line.split_whitespace().count() == 25 {
                return Some(line);
            }
        }
        None
    }
}
