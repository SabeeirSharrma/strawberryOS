use std::path::PathBuf;

/// Miner process management — Phase 0 stub.
///
/// In later phases, strawberryd will:
/// - Launch miners as supervised child processes (systemd user units)
/// - Monitor hashrate via process output
/// - Auto-restart on crash
/// - Watchdog: restart if hashrate drops to 0 or GPU falls off bus

pub struct MinerManager {
    data_dir: PathBuf,
}

impl MinerManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// List installed miners (reads from data_dir/miners/)
    pub fn list_installed(&self) -> Vec<String> {
        // Stub — will scan data_dir/miners/ in Phase 1
        vec![]
    }

    /// Start a miner by name
    pub fn start(&self, _name: &str) -> anyhow::Result<()> {
        tracing::warn!("Miner start not yet implemented (Phase 1)");
        Ok(())
    }

    /// Stop a miner by name
    pub fn stop(&self, _name: &str) -> anyhow::Result<()> {
        tracing::warn!("Miner stop not yet implemented (Phase 1)");
        Ok(())
    }
}
