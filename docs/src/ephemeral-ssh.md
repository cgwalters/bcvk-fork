# SSH Access

## Overview

bcvk provides SSH access to ephemeral VMs with automatic key management. SSH sessions can be direct (VM created on-demand) or connect to existing named VMs.

## Key Management

Keys are generated automatically per VM and injected during creation. You can also provide your own public keys if needed.

## Usage Examples

### Quick debugging

```bash
# Create VM, SSH directly, cleanup on exit
bcvk ephemeral ssh quay.io/myapp/debug:latest
```

### Development environment

```bash
# Create development VM
bcvk ephemeral run -d --name dev-env \
  --bind ~/code:/workspace \
  quay.io/myapp/dev:latest

# Connect when needed
bcvk ephemeral ssh dev-env
```

### Running commands

```bash
# Execute commands in VM
bcvk ephemeral ssh test-vm "systemctl status myapp"
```

## Integration

Standard SSH tools work with bcvk VMs:

- **scp/sftp**: File transfer
- **ssh -L/-R**: Port forwarding
- **IDE remote development**: VS Code, JetBrains, etc.

## See Also

- [bcvk-ephemeral-ssh(8)](./man/bcvk-ephemeral-ssh.md) - Command reference
- [Ephemeral VM Concepts](./ephemeral-run.md) - VM lifecycle