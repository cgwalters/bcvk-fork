# VM Lifecycle Management

## Basic Operations

```bash
# Create and start VM
bcvk libvirt run --name myvm quay.io/fedora/fedora-bootc:42

# Manage state
bcvk libvirt start myvm
bcvk libvirt stop myvm
bcvk libvirt restart myvm

# Remove VM
bcvk libvirt rm myvm

# List VMs
bcvk libvirt list
```

## Resource Configuration

```bash
# Configure memory, CPU, and disk
bcvk libvirt run --name myvm \
  --memory 8192 \
  --cpus 4 \
  --disk-size 50G \
  quay.io/fedora/fedora-bootc:42
```

## SSH Access

```bash
bcvk libvirt ssh myvm
```

See the [libvirt run guide](./libvirt-run.md) for more details.