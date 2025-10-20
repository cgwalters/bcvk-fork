# NAME

bcvk-ephemeral-rm-all - Remove all ephemeral VM containers

# SYNOPSIS

**bcvk ephemeral rm-all** [*OPTIONS*]

# DESCRIPTION

Remove all ephemeral VM containers

# OPTIONS

<!-- BEGIN GENERATED OPTIONS -->
**-f**, **--force**

    Force removal without confirmation

<!-- END GENERATED OPTIONS -->

# EXAMPLES

Remove all ephemeral VMs (will prompt for confirmation):

    bcvk ephemeral rm-all

Remove all ephemeral VMs without confirmation:

    bcvk ephemeral rm-all --force

Clean up after testing workflow:

    # Run some ephemeral test VMs
    bcvk ephemeral run -d --rm --name test1 quay.io/fedora/fedora-bootc:42
    bcvk ephemeral run -d --rm --name test2 quay.io/fedora/fedora-bootc:42

    # Clean up all at once
    bcvk ephemeral rm-all -f

# SEE ALSO

**bcvk**(8)

# VERSION

<!-- VERSION PLACEHOLDER -->
