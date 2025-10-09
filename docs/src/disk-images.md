# Disk Images

The `to-disk` command creates bootable disk images from bootc containers.

## Supported Formats

```bash
# Raw disk image (default)
bcvk to-disk quay.io/fedora/fedora-bootc:42 output.img

# QCOW2 (compressed, for QEMU/KVM)
bcvk to-disk --format qcow2 quay.io/fedora/fedora-bootc:42 output.qcow2

# VHD (Hyper-V, Azure)
bcvk to-disk --format vhd quay.io/fedora/fedora-bootc:42 output.vhd

# VMDK (VMware)
bcvk to-disk --format vmdk quay.io/fedora/fedora-bootc:42 output.vmdk
```

## Configuration Options

```bash
# Disk size
bcvk to-disk --size 50G quay.io/fedora/fedora-bootc:42 output.img

# Filesystem type
bcvk to-disk --filesystem xfs quay.io/fedora/fedora-bootc:42 output.img
bcvk to-disk --filesystem ext4 quay.io/fedora/fedora-bootc:42 output.img

# Partitioning scheme
bcvk to-disk --partition gpt quay.io/fedora/fedora-bootc:42 output.img
bcvk to-disk --partition mbr quay.io/fedora/fedora-bootc:42 output.img
```

## Common Use Cases

### Cloud Deployment

```bash
# AWS/GCP (raw format)
bcvk to-disk --format raw --size 30G quay.io/fedora/fedora-bootc:42 cloud.img

# Azure (VHD format)
bcvk to-disk --format vhd --size 30G quay.io/fedora/fedora-bootc:42 azure.vhd
```

### Bare Metal

```bash
# Create bootable USB/SD card image
bcvk to-disk --size 16G quay.io/fedora/fedora-bootc:42 usb.img
sudo dd if=usb.img of=/dev/sdX bs=4M status=progress
```

### Virtualization

```bash
# QEMU/KVM
bcvk to-disk --format qcow2 quay.io/fedora/fedora-bootc:42 vm.qcow2
qemu-system-x86_64 -hda vm.qcow2 -m 2048 -enable-kvm

# VMware
bcvk to-disk --format vmdk quay.io/fedora/fedora-bootc:42 vm.vmdk

# VirtualBox (requires conversion)
bcvk to-disk --format raw quay.io/fedora/fedora-bootc:42 vm.img
VBoxManage convertfromraw vm.img vm.vdi
```