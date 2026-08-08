# Strawberry OS

![Splash](splash.png)

**A minimal, zero-telemetry, Arch-based Linux distro purpose-built for cryptocurrency mining.**

## Vision

- Boot straight into a mining-ready environment — no bloat, no accounts, no telemetry.
- Curated "app store" experience for miners, wallets, and pool configs.
- Support both solo hobbyist rigs and multi-rig/headless farms.
- Every wallet is accountless — non-custodial, local keys/seed only.

## Quick Start

### Build the ISO (on an Arch Linux system)

```bash
# Install archiso
sudo pacman -S archiso

# Build
./scripts/build-iso.sh
```

### Run strawberryd locally (for development)

```bash
cd strawberryd
cargo build --release
sudo ./target/release/strawberryd
```

### Verify zero telemetry

```bash
./scripts/telemetry-audit.sh
```

## License

Custom Source Available License (BSL-based), see [LICENSE](LICENSE).
