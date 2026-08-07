use std::path::PathBuf;

/// Wallet key/seed management — Phase 1 stub.
///
/// Design constraints (from spec §4, §8):
/// - Non-custodial: seeds generated and stored locally only
/// - No Strawberry backend ever sees funds or addresses
/// - Seeds encrypted at rest, decrypted only in memory
/// - Passphrase-gated access

pub struct WalletManager {
    data_dir: PathBuf,
}

impl WalletManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// List configured wallets (reads from config wallets section)
    pub fn list(&self) -> Vec<(String, String)> {
        // Stub — will read from config + on-disk wallets in Phase 1
        vec![]
    }

    /// Generate a new wallet seed for the given coin
    pub fn generate(&self, _coin: &str) -> anyhow::Result<String> {
        tracing::warn!("Wallet generation not yet implemented (Phase 1)");
        Ok(String::new())
    }
}
