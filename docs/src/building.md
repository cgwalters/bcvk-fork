# Building from Source

## Prerequisites

Rust toolchain (install via rustup):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Build dependencies:
```bash
# Fedora/RHEL/CentOS
sudo dnf install gcc pkg-config openssl-devel

# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev
```

Runtime dependencies:
```bash
# Fedora/RHEL/CentOS
sudo dnf install qemu-kvm qemu-img podman virtiofsd

# Ubuntu/Debian
sudo apt install qemu-kvm qemu-utils podman virtiofsd
```

Optional libvirt support:
```bash
# Fedora/RHEL/CentOS
sudo dnf install libvirt libvirt-daemon-kvm
sudo systemctl enable --now libvirtd
```

## Clone and Build

```bash
git clone https://github.com/bootc-dev/bcvk.git
cd bcvk
cargo build --release
```

The binary will be at `target/release/bcvk`.

## Installation

```bash
# Install to ~/.cargo/bin
cargo install --path .

# Or copy to system location
sudo cp target/release/bcvk /usr/local/bin/
```

## Development

```bash
cargo test                    # Run tests
cargo fmt                     # Format code
cargo clippy                  # Run linter
```

Using `just` (if installed):
```bash
just test
just fmt
just clippy
```

See [testing.md](./testing.md) for details.