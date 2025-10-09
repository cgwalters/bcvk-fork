# Image Management

## Listing Images

bcvk identifies bootc images by the `containers.bootc=1` label.

```bash
# List all bootc images
bcvk images list
```

## Finding bootc Images

Common bootc images:

- `quay.io/fedora/fedora-bootc:42`
- `quay.io/centos-bootc/centos-bootc:stream10`
- `registry.redhat.io/rhel9/rhel-bootc:latest`

## Pulling Images

```bash
podman pull quay.io/fedora/fedora-bootc:42
```

## Building Custom Images

```dockerfile
FROM quay.io/fedora/fedora-bootc:42
LABEL containers.bootc=1

RUN dnf install -y httpd && dnf clean all
RUN systemctl enable httpd
```

Build and test:
```bash
podman build -t localhost/my-bootc:latest .
bcvk ephemeral run-ssh localhost/my-bootc:latest
```