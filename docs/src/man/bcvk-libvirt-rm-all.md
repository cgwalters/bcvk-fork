# NAME

bcvk-libvirt-rm-all - Remove multiple libvirt domains and their resources

# SYNOPSIS

**bcvk libvirt rm-all** [*OPTIONS*]

# DESCRIPTION

Remove multiple libvirt domains and their resources

# OPTIONS

<!-- BEGIN GENERATED OPTIONS -->
**-f**, **--force**

    Force removal without confirmation

**--stop**

    Remove domains even if they're running

**--label**=*LABEL*

    Filter domains by label (only remove domains with this label)

<!-- END GENERATED OPTIONS -->

# EXAMPLES

Remove all stopped libvirt VMs (will prompt for confirmation):

    bcvk libvirt rm-all

Remove all VMs without confirmation:

    bcvk libvirt rm-all --force

Remove all VMs including running ones:

    bcvk libvirt rm-all --stop --force

Remove all VMs with a specific label:

    bcvk libvirt rm-all --label environment=test --force

Clean up test environment workflow:

    # Create some test VMs
    bcvk libvirt run --name test1 --label purpose=testing quay.io/fedora/fedora-bootc:42
    bcvk libvirt run --name test2 --label purpose=testing quay.io/fedora/fedora-bootc:42

    # Clean up only the test VMs
    bcvk libvirt rm-all --label purpose=testing -f

# SEE ALSO

**bcvk**(8)

# VERSION

<!-- VERSION PLACEHOLDER -->
