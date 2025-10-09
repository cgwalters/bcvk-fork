# Libvirt Integration

The `bcvk libvirt run` command creates persistent virtual machines from bootc containers. It generates a disk image using `bcvk to-disk` and provisions a VM managed through libvirt.

## Basic Usage

```bash
# Create a VM
bcvk libvirt run quay.io/myapp/server:latest

# With specific resources
bcvk libvirt run \
  --name production-api \
  --memory 8192 \
  --cpus 4 \
  --autostart \
  quay.io/myapp/api:v1.0

# Development setup with SSH
bcvk libvirt run \
  --name dev-environment \
  --memory 4096 \
  --cpus 2 \
  --ssh \
  quay.io/myapp/dev:latest
```

## Host Directory Mounting

Share host directories with the VM using virtiofs:

```bash
bcvk libvirt run \
  --volume /home/chris/projects/foo:src \
  --volume /home/chris/data:data \
  --ssh \
  quay.io/myapp/dev:latest
```

Format: `--volume HOST_PATH:TAG` where TAG is the virtiofs mount tag.

Mount in the guest:

```bash
mkdir -p /mnt/src
mount -t virtiofs src /mnt/src
```

## Container Storage Integration

Access host container storage for bootc upgrades:

```bash
bcvk libvirt run \
  --bind-storage-ro \
  quay.io/fedora/fedora-bootc:42
```

This provisions a virtiofs mount named `hoststorage`. Mount in the guest:

```bash
mkdir -p /run/virtiofs-mnt-hoststorage
mount -t virtiofs hoststorage /run/virtiofs-mnt-hoststorage
```

Use with bootc:

```bash
env STORAGE_OPTS=additionalimagestore=/run/virtiofs-mnt-hoststorage \
  bootc switch --transport containers-storage localhost/bootc

env STORAGE_OPTS=additionalimagestore=/run/virtiofs-mnt-hoststorage \
  bootc upgrade
```

## Secure Boot

Prerequisites:
```bash
sudo dnf install -y edk2-ovmf python3-virt-firmware openssl
```

Enable Secure Boot with existing keys:

```bash
bcvk libvirt run --firmware uefi-secure --secure-boot-keys /path/to/keys quay.io/myimage:latest
```

Generate keys:

```bash
mkdir -p ./my-secure-boot-keys
cd ./my-secure-boot-keys

openssl req -newkey rsa:4096 -nodes -keyout PK.key -new -x509 -sha256 -days 3650 -subj '/CN=Platform Key/' -out PK.crt
openssl req -newkey rsa:4096 -nodes -keyout KEK.key -new -x509 -sha256 -days 3650 -subj '/CN=Key Exchange Key/' -out KEK.crt
openssl req -newkey rsa:4096 -nodes -keyout db.key -new -x509 -sha256 -days 3650 -subj '/CN=Signature Database key/' -out db.crt
uuidgen > GUID.txt
```

Required files: PK.crt, KEK.crt, db.crt, GUID.txt

Firmware options:
- `--firmware uefi-secure` (default)
- `--firmware uefi-insecure`
- `--firmware bios`

Verify in the guest:
```bash
mokutil --sb-state
```

## See Also

- [bcvk-libvirt-run(8)](./man/bcvk-libvirt-run.md) - Command reference
- [Advanced Workflows](./libvirt-advanced.md) - Complex deployment patterns