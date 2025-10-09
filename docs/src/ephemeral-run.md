# Ephemeral VMs

Ephemeral VMs are temporary virtual machines that start quickly from bootc container images and automatically clean up when stopped.

## Basic Usage

```bash
# Quick test with auto-cleanup
bcvk ephemeral run --rm quay.io/fedora/fedora-bootc:42

# Background VM
bcvk ephemeral run -d --name myvm quay.io/fedora/fedora-bootc:42

# With SSH
bcvk ephemeral run-ssh quay.io/fedora/fedora-bootc:42
```

## Resource Configuration

```bash
# Custom resources
bcvk ephemeral run --memory 4096 --cpus 4 --name myvm quay.io/fedora/fedora-bootc:42
```

## Use Cases

- Quick testing of bootc images
- Development environments
- CI/CD integration
- Isolated experimentation

See [ephemeral-ssh](./ephemeral-ssh.md) for SSH access details.