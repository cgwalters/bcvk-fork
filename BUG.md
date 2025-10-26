# Issue #90: Nested Virtualization Boot Hang Investigation

**Status:** ✅ **FIXED** - VMs boot successfully in nested virtualization
**Open Issue:** SSH connectivity timeouts in Codespaces environment (under investigation)

---

## Problem Summary

When running `bcvk ephemeral run` in nested virtualization environments (KVM-on-KVM, e.g., Microsoft Azure Codespaces or cloud VMs with nested virt), VMs would hang during boot and never reach a usable state.

### Root Cause

**systemd-journald livelock on virtiofs in nested virtualization**

systemd-journald writing persistent logs to the virtiofs-backed root filesystem causes infinite filesystem lookup loops (5,000+ req/sec) in nested KVM. The system appears hung but is actually trapped in virtiofsd request processing.

Key characteristics:
- Multiple systemd processes repeatedly scan the root directory at extremely high rates (1,000-6,000 req/sec)
- 26.9% of all filesystem operations target the root directory (inode 1)
- All filesystem operations succeed - this is a userspace/systemd bug, not a virtiofs bug
- Only occurs in nested virtualization (KVM-on-KVM)

### Solution

**Kernel argument:** `systemd.journald.storage=volatile`

Forces journald to use tmpfs for log storage instead of writing to virtiofs.

### Results

- ✅ VMs boot successfully in 13 seconds
- ✅ Reach graphical.target reliably
- ✅ 92% reduction in filesystem requests
- ✅ virtiofs now works correctly in nested virt
- ✅ Non-SSH integration tests pass (4/4)

### Commits

1. **Status monitor timeout fix** (0abe636)
   - Added 10-second timeout to inotify event loop
   - Prevents indefinite blocking when vsock is disabled in nested virt
   - Enables faster SSH fallback (10s vs 240s)

2. **Journald livelock fix** (87be05d)
   - Added `systemd.journald.storage=volatile` kernel argument
   - Fixes boot livelock in nested KVM
   - Location: `crates/kit/src/run_ephemeral.rs:978`

---

## Open Issue: SSH Connectivity in Nested Virt

**Status:** 🔍 **Under Investigation**

### Symptoms

- All 7 SSH-based integration tests timeout after 240s in Codespaces
- All 4 non-SSH integration tests pass
- VMs boot successfully and reach graphical.target
- SSH polling shows all connection attempts fail silently

### Test Results

#### Local Environment (Codespaces - Nested KVM)
```
running 11 tests
test run_ephemeral_ssh_cleanup                    ... FAILED (240s timeout)
test run_ephemeral_ssh_exit_code                  ... FAILED (exit code 1 vs 42)
test run_ephemeral_ssh_system_command             ... FAILED (240s timeout)
test run_ephemeral_ssh_command                    ... FAILED (240s timeout)
test run_ephemeral_ssh_cross_distro_compatibility ... FAILED (240s timeout)
test run_ephemeral_container_ssh_access           ... FAILED (ssh failed)
test run_ephemeral_correct_kernel                 ... ok
test run_ephemeral_poweroff                       ... ok
test run_ephemeral_with_memory_limit              ... ok
test run_ephemeral_with_vcpus                     ... ok
```

#### GHA Environment (Native KVM)
- Tests running on: https://github.com/cgwalters/bcvk-fork/actions
- Run 18819440259: Pending
- Run 18819452281: Pending

### Hypotheses

1. **vsock-related** (primary suspect)
   - vsock should be disabled in nested virt but may not be
   - SSH key injection via SMBIOS credentials may depend on vsock
   - Status monitoring already has vsock fallback (10s timeout)

2. **Networking differences**
   - User-mode networking (SLIRP) behavior in nested KVM
   - Port forwarding configuration (tcp::2222-:22)
   - NAT/firewall rules in Codespaces environment

3. **Environment-specific**
   - Nested KVM limitations in Azure Codespaces
   - Different KVM/QEMU versions or configurations
   - Resource constraints or isolation policies

### Investigation Tasks

- [ ] Check if vsock is being properly disabled in nested virt
- [ ] Examine SSH key injection mechanism (SMBIOS vs vsock)
- [ ] Compare QEMU command-line args between GHA and Codespaces
- [ ] Test SSH connectivity with direct QEMU command
- [ ] Verify sshd is actually running in guest VMs
- [ ] Check if authorized_keys file is being created correctly
- [ ] Monitor GHA test results to confirm SSH works in native KVM

### Debug Commands

```bash
# Check running VMs
podman ps --filter "label=bcvk.ephemeral=1"

# Inspect VM logs
podman logs <container-id>

# Check SSH from host
podman exec <container-id> ssh -v -i /run/tmproot/var/lib/bcvk/ssh root@127.0.0.1 -p 2222 -- true

# Check QEMU process
podman exec <container-id> ps aux | grep qemu

# View guest console (if --console used)
./target/release/bcvk ephemeral run --console quay.io/fedora/fedora-bootc:42
```

---

## Timeline

- **2025-10-26 Early**: Investigated boot hang, identified journald as root cause
- **2025-10-26 Afternoon**: Implemented journald workaround, VMs now boot successfully
- **2025-10-26 Evening**: Discovered SSH connectivity issue separate from boot hang
- **2025-10-26 Now**: Investigating SSH failures, testing in GHA to compare environments

---

## Technical Details

### Filesystem Request Analysis (cache=never vs cache=metadata)

From virtiofsd debug logs:

| Metric | cache=never | cache=metadata | Reduction |
|--------|-------------|----------------|-----------|
| Total requests | 184,758 | 13,821 | 92.5% |
| Root (/) lookups | 49,746 (26.9%) | Not measured | - |
| Top process (PID 763) | 17,158 req | - | - |
| Request rate | 5,000+ req/s | <100 req/s | ~98% |

### Key Processes Involved

| PID | Process | Requests | Rate | Activity |
|-----|---------|----------|------|----------|
| 1 | systemd (init) | 32,268 | 1,113/s | Root directory lookups |
| 763 | systemd service | 17,158 | 4,290/s | /var lookups |
| 728 | systemd service | 9,084 | 1,514/s | Root lookups |
| 652 | Path scanner | 5,935 | 1,484/s | Root loop (78% on /) |

### Nested Virtualization Detection

Location: `crates/kit/src/virtualization.rs`

Detection methods:
- CPU flags check (vmx/svm)
- /sys/module/kvm_*/parameters/nested
- Hypervisor vendor detection (Microsoft, QEMU, KVM)

Current behavior:
- Nested virt detected correctly in Codespaces
- Auto-selects virtiofs filesystem
- vsock status: **TBD** (needs verification)

---

## References

- Issue: https://github.com/bootc-dev/bcvk/issues/90
- Related analysis: GUEST_SIDE_ANALYSIS_REPORT.md
- Test fork: https://github.com/cgwalters/bcvk-fork
