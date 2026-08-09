use crate::config::GlobalConfig;
use tracing::info;

const MINER_DIR: &str = "/opt/strawberry/miners";
const BIN_DIR: &str = "/usr/local/bin";

/// Known miner download sources (Linux x86_64).
pub fn miner_sources() -> Vec<MinerSource> {
    vec![
        MinerSource {
            id: "xmrig".to_string(),
            name: "XMRig".to_string(),
            url: "https://github.com/xmrig/xmrig/releases/download/v6.21.0/xmrig-6.21.0-linux-x64.tar.gz".to_string(),
            bin_inside: "xmrig-6.21.0/xmrig".to_string(),
            wrapper_name: "xmrig".to_string(),
            coins: vec!["monero".to_string()],
            algo_args: MinerArgs::Xmrig,
        },
        MinerSource {
            id: "t-rex".to_string(),
            name: "T-Rex".to_string(),
            url: "https://github.com/trexcoin/T-Rex/releases/download/0.26.8/t-rex-0.26.8-linux.tar.gz".to_string(),
            bin_inside: "t-rex".to_string(),
            wrapper_name: "t-rex".to_string(),
            coins: vec!["monero".to_string(), "ravencoin".to_string()],
            algo_args: MinerArgs::TRex,
        },
        MinerSource {
            id: "lolminer".to_string(),
            name: "lolMiner".to_string(),
            url: "https://github.com/Lolliedieb/lolMiner-releases/releases/download/1.76a/lolMiner_v1.76a_Linux_x64.tar.gz".to_string(),
            bin_inside: "1.76a/lolMiner".to_string(),
            wrapper_name: "lolminer".to_string(),
            coins: vec!["monero".to_string(), "zcash".to_string(), "ergo".to_string()],
            algo_args: MinerArgs::Lolminer,
        },
        MinerSource {
            id: "gminer".to_string(),
            name: "GMiner".to_string(),
            url: "https://github.com/develsoftware/GMinerRelease/releases/download/3.44/gminer_3_44_linux_cuda12.tar.xz".to_string(),
            bin_inside: "miner".to_string(),
            wrapper_name: "gminer".to_string(),
            coins: vec!["monero".to_string(), "zcash".to_string(), "ergo".to_string()],
            algo_args: MinerArgs::Gminer,
        },
        MinerSource {
            id: "srbminer-multi".to_string(),
            name: "SRBMiner Multi".to_string(),
            url: "https://github.com/doktor83/SRBMiner-Multi/releases/download/2.4.4/SRBMiner-Multi-2.4.4-linux-x64.tar.gz".to_string(),
            bin_inside: "SRBMiner-Multi-2.4.4/SRBMiner-Multi".to_string(),
            wrapper_name: "srbminer".to_string(),
            coins: vec!["monero".to_string(), "ergo".to_string()],
            algo_args: MinerArgs::Srbminer,
        },
        MinerSource {
            id: "cpuminer-multi".to_string(),
            name: "cpuminer-multi".to_string(),
            url: "https://github.com/ponga-pool/cpuminer-multi/releases/download/v2.5.1/cpuminer-multi-linux.tar.gz".to_string(),
            bin_inside: "cpuminer-multi".to_string(),
            wrapper_name: "cpuminer".to_string(),
            coins: vec!["monero".to_string()],
            algo_args: MinerArgs::Cpuminer,
        },
        MinerSource {
            id: "nanominer".to_string(),
            name: "NanoMiner".to_string(),
            url: "https://github.com/nanopool/nanominer/releases/download/v3.3.6/nanominer-linux-3.3.6.tar.gz".to_string(),
            bin_inside: "nanominer".to_string(),
            wrapper_name: "nanominer".to_string(),
            coins: vec!["monero".to_string(), "ergo".to_string()],
            algo_args: MinerArgs::Nanominer,
        },
        MinerSource {
            id: "teamredminer".to_string(),
            name: "TeamRedMiner".to_string(),
            url: "https://github.com/todxx/teamredminer/releases/download/v0.10.14/teamredminer-v0.10.14-linux-amd64.tar.gz".to_string(),
            bin_inside: "teamredminer-v0.10.14-linux-amd64/teamredminer".to_string(),
            wrapper_name: "teamredminer".to_string(),
            coins: vec!["monero".to_string()],
            algo_args: MinerArgs::Teamredminer,
        },
    ]
}

/// Launch argument patterns per miner type.
pub enum MinerArgs {
    Xmrig,
    TRex,
    Lolminer,
    Gminer,
    Srbminer,
    Cpuminer,
    Nanominer,
    Teamredminer,
}

pub struct MinerSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub bin_inside: String,
    pub wrapper_name: String,
    pub coins: Vec<String>,
    pub algo_args: MinerArgs,
}

/// Check if a miner is installed.
pub fn is_installed(id: &str) -> bool {
    let sources = miner_sources();
    if let Some(src) = sources.iter().find(|s| s.id == id) {
        let wrapper = format!("{}/{}", BIN_DIR, src.wrapper_name);
        let binary = format!("{}/{}/{}", MINER_DIR, id, src.bin_inside.split('/').last().unwrap_or("miner"));
        std::path::Path::new(&wrapper).exists() || std::path::Path::new(&binary).exists()
    } else {
        false
    }
}

/// Get list of installed miner IDs.
pub fn installed_miners() -> Vec<String> {
    miner_sources()
        .iter()
        .filter(|s| is_installed(&s.id))
        .map(|s| s.id.clone())
        .collect()
}

/// Download and install a miner binary, create a launcher wrapper.
pub async fn install_miner(id: &str, config: &GlobalConfig) -> Result<String, String> {
    let sources = miner_sources();
    let src = sources.iter().find(|s| s.id == id)
        .ok_or_else(|| format!("Unknown miner: {}", id))?;

    if is_installed(id) {
        return Ok(format!("{} is already installed. Run '{}' in terminal to launch.", src.name, src.wrapper_name));
    }

    let install_dir = format!("{}/{}", MINER_DIR, id);
    std::fs::create_dir_all(&install_dir).map_err(|e| e.to_string())?;

    // Download archive
    info!("Downloading {} from {}", src.name, src.url);
    let archive_path = format!("{}/archive.tar.gz", install_dir);

    let output = std::process::Command::new("curl")
        .args(["-sSfL", "-o", &archive_path, &src.url])
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Download failed: {}", stderr));
    }

    // Extract
    info!("Extracting to {}", install_dir);
    let extract_output = if src.url.ends_with(".tar.xz") {
        std::process::Command::new("tar")
            .args(["xf", &archive_path, "-C", &install_dir])
            .output()
    } else {
        std::process::Command::new("tar")
            .args(["xzf", &archive_path, "-C", &install_dir])
            .output()
    };

    match extract_output {
        Ok(o) if !o.status.success() => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            return Err(format!("Extraction failed: {}", stderr));
        }
        Err(e) => return Err(format!("Extraction error: {}", e)),
        _ => {}
    }

    // Make binary executable
    let bin_path = format!("{}/{}", install_dir, src.bin_inside);
    let _ = std::process::Command::new("chmod").args(["+x", &bin_path]).output();

    // Create wrapper script
    create_wrapper(src, config)?;

    // Clean up archive
    let _ = std::fs::remove_file(&archive_path);

    Ok(format!("{} installed! Run '{}' to launch.", src.name, src.wrapper_name))
}

/// Uninstall a miner.
pub fn uninstall_miner(id: &str) -> Result<String, String> {
    let sources = miner_sources();
    let src = sources.iter().find(|s| s.id == id)
        .ok_or_else(|| format!("Unknown miner: {}", id))?;

    // Remove wrapper
    let wrapper = format!("{}/{}", BIN_DIR, src.wrapper_name);
    let _ = std::fs::remove_file(&wrapper);

    // Remove install dir
    let install_dir = format!("{}/{}", MINER_DIR, id);
    let _ = std::fs::remove_dir_all(&install_dir);

    Ok(format!("{} uninstalled.", src.name))
}

/// Create a launcher wrapper script for a miner.
fn create_wrapper(src: &MinerSource, config: &GlobalConfig) -> Result<(), String> {
    let wrapper_path = format!("{}/{}", BIN_DIR, src.wrapper_name);
    let miner_bin = format!("{}/{}/{}", MINER_DIR, src.id, src.bin_inside.split('/').last().unwrap_or("miner"));

    // Get wallet and pool for this miner's primary coin
    let primary_coin = src.coins.first().map(|s| s.as_str()).unwrap_or("monero");
    let wallet = config.wallets.get(primary_coin).map(|s| s.as_str()).unwrap_or("YOUR_WALLET_ADDRESS");
    let pool = config.pools.get(primary_coin)
        .map(|p| format!("{}:{}", p.address, p.port))
        .unwrap_or_else(|| "pool.supportxmr.com:3333".to_string());

    let script = match &src.algo_args {
        MinerArgs::Xmrig => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
exec "{bin}" --url "{pool}" --wallet "{wallet}" --coin monero --donate-level 0 "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::TRex => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
COIN="${{1:-monero}}"
case "$COIN" in
    monero|xmr) ALGO="randomx" ;;
    ravencoin|rvn) ALGO="kawpow" ;;
    *) ALGO="randomx" ;;
esac
exec "{bin}" -a "$ALGO" -o "{pool}" -u "{wallet}.1" -p x "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::Lolminer => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
COIN="${{1:-monero}}"
case "$COIN" in
    monero|xmr) ALGO="randomx" ;;
    zcash|zec) ALGO="equihash" ;;
    ergo|erg) ALGO="autolykos2" ;;
    *) ALGO="randomx" ;;
esac
exec "{bin}" --algo "$ALGO" --pool "{pool}" --user "{wallet}" "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::Gminer => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
COIN="${{1:-monero}}"
case "$COIN" in
    monero|xmr) ALGO="randomx" ;;
    zcash|zec) ALGO="equihash" ;;
    ergo|erg) ALGO="autolykos2" ;;
    *) ALGO="randomx" ;;
esac
exec "{bin}" -a "$ALGO" -s "{pool}" -u "{wallet}" -p x "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::Srbminer => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
COIN="${{1:-monero}}"
case "$COIN" in
    monero|xmr) ALGO="randomx" ;;
    ergo|erg) ALGO="autolykos2" ;;
    *) ALGO="randomx" ;;
esac
exec "{bin}" --algorithm "$ALGO" --pool "{pool}" --wallet "{wallet}" --password x "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::Cpuminer => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
exec "{bin}" -a randomx -o "{pool}" -u "{wallet}" -p x "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::Nanominer => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
COIN="${{1:-monero}}"
case "$COIN" in
    monero|xmr) ALGO="randomx" ;;
    ergo|erg) ALGO="autolykos2" ;;
    *) ALGO="randomx" ;;
esac
# NanoMiner uses a config file approach — create temp config
TMPDIR=$(mktemp -d)
cat > "$TMPDIR/config.ini" << INNEREOF
${{ALGO}}="{pool}"
wallet="{wallet}"
INNEREOF
exec "{bin}" -c "$TMPDIR/config.ini" "$@"
rm -rf "$TMPDIR"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),

        MinerArgs::Teamredminer => format!(r#"#!/bin/bash
# Strawberry OS — {name} launcher
# Wallet: {wallet}
# Pool: {pool}
exec "{bin}" -a randomx -o "{pool}" -u "{wallet}" -p x "$@"
"#, name = src.name, wallet = wallet, pool = pool, bin = miner_bin),
    };

    std::fs::write(&wrapper_path, script).map_err(|e| e.to_string())?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&wrapper_path, std::fs::Permissions::from_mode(0o755));
    }

    info!("Created wrapper script: {}", wrapper_path);
    Ok(())
}
