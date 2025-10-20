# NAME

bcvk-libvirt-list-volumes - List available bootc volumes with metadata

# SYNOPSIS

**bcvk libvirt list-volumes** [*OPTIONS*]

# DESCRIPTION

List available bootc volumes with metadata

# OPTIONS

<!-- BEGIN GENERATED OPTIONS -->
**--pool**=*POOL*

    Libvirt storage pool name to search

    Default: default

**--json**

    Output as structured JSON instead of table format

**--detailed**

    Show detailed volume information

**--source-image**=*SOURCE_IMAGE*

    Filter by source container image

**--all**

    Show all volumes (not just bootc volumes)

<!-- END GENERATED OPTIONS -->

# EXAMPLES

List all bootc volumes in the default pool:

    bcvk libvirt list-volumes

Show detailed volume information:

    bcvk libvirt list-volumes --detailed

Filter volumes by source container image:

    bcvk libvirt list-volumes --source-image quay.io/fedora/fedora-bootc:42

List all volumes including non-bootc volumes:

    bcvk libvirt list-volumes --all

Output as JSON for scripting:

    bcvk libvirt list-volumes --json

# SEE ALSO

**bcvk**(8)

# VERSION

<!-- VERSION PLACEHOLDER -->
