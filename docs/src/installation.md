# Installation

## Prerequisites

Required:
- [Rust](https://www.rust-lang.org/)
- Git
- QEMU/KVM
- virtiofsd
- Podman

Optional:
- libvirt (for persistent VM features)
  ```bash
  sudo systemctl enable --now libvirtd
  sudo usermod -a -G libvirt $USER
  ```

## Building from Source

```bash
git clone https://github.com/cgwalters/bcvk.git
cd bcvk
cargo build --release
```

Binary location: `target/release/bcvk`

Install to PATH:
```bash
sudo cp target/release/bcvk /usr/local/bin/
```

## Platform Support

- Linux: Supported
- macOS: Not supported, use [podman-bootc](https://github.com/containers/podman-bootc/)
- Windows: Not supported

See the [Quick Start Guide](./quick-start.md) to begin using bcvk.