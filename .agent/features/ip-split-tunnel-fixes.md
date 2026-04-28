# IP Split Tunneling Fixes

## Issue 1: Generic CGNAT Range Naming

### Problem
The current implementation uses vendor-specific constant names (`TAILSCALE_CGNAT_RANGE`, `NETBIRD_CGNAT_RANGE`) when both refer to the same CGNAT range (100.64.0.0/10). The example constant `NETBIRD_TENANT_RANGE_EXAMPLE` is also too specific.

### Solution
- Rename `TAILSCALE_CGNAT_RANGE` and `NETBIRD_CGNAT_RANGE` to a single `CGNAT_RANGE` constant
- Rename `NETBIRD_TENANT_RANGE_EXAMPLE` to `TENANT_RANGE_SAMPLE` with a more generic example value
- Update the `ApplyTemplates` command output to reflect generic terminology
- Update tests to use generic naming

### Files to Modify
- `mullvad-cli/src/cmds/fork/ip_split_tunnel.rs`

## Issue 2: System-Wide Firewall Rules at Boot

### Problem
When the Mullvad daemon starts at system boot (via systemctl, before user login), it applies a blocking firewall policy through `early_boot_firewall.rs`. The IP split-tunnel rules are only applied later in the daemon initialization (line 936 in `lib.rs`), but by that time the firewall is already blocking traffic.

**Root cause:** The early boot firewall applies `FirewallPolicy::Blocked` which drops all traffic except allowed endpoints. The IP split-tunnel nftables rules are applied separately and later, so they don't take effect until after the daemon fully initializes.

### Current Flow
1. System boot → systemctl starts mullvad-daemon
2. `early_boot_firewall::initialize_firewall()` applies `FirewallPolicy::Blocked`
3. Daemon continues initialization
4. Line 931-941: IP split-tunnel module loads and applies rules
5. Rules are now active, but only after full daemon initialization

### Solution Approach

**Option A: Apply IP split-tunnel rules in early boot firewall (Recommended)**
- Load IP split-tunnel settings in `early_boot_firewall.rs`
- Apply IP split-tunnel nftables rules before or immediately after the blocking policy
- Ensures split-tunnel ranges bypass the tunnel from the moment the firewall is active

**Option B: Defer firewall blocking until IP split-tunnel is ready**
- Delay applying the blocking policy until after IP split-tunnel initialization
- Risk: Small window where traffic might leak

**Recommendation: Option A** - It maintains security while ensuring split-tunnel rules are active from boot.

### Implementation Plan (Option A)

1. **Create early boot IP split-tunnel loader**
   - Add function in `mullvad-daemon/src/fork/ip_split_tunnel.rs`:
     ```rust
     pub async fn load_and_apply_early(settings_dir: impl AsRef<Path>) -> Result<(), Error>
     ```
   - This function loads ranges from disk and applies nftables rules
   - Does NOT need route manager (only applies firewall marking rules)

2. **Integrate into early boot firewall**
   - Modify `mullvad-daemon/src/early_boot_firewall.rs`
   - Call IP split-tunnel loader before or after applying blocking policy
   - Log errors but don't fail if IP split-tunnel fails (graceful degradation)

3. **Ensure idempotency**
   - When daemon fully initializes, calling `apply()` again should be safe
   - nftables rules are already designed to be reapplied (del + add pattern)

### Files to Modify
- `mullvad-daemon/src/fork/ip_split_tunnel.rs` - Add early boot loader function
- `mullvad-daemon/src/early_boot_firewall.rs` - Call IP split-tunnel early loader
- `talpid-core/src/firewall/linux/ip_split_tunnel.rs` - Ensure idempotent application

### Testing Plan
1. Configure IP split-tunnel ranges (e.g., 100.64.0.0/10)
2. Reboot system
3. Before user login, verify:
   - Firewall is active (blocking policy applied)
   - IP split-tunnel nftables rules exist: `sudo nft list table inet mullvad-ip-split-tunnel`
   - Traffic to 100.64.0.1 is marked and bypasses tunnel
4. After user login, verify:
   - Rules remain active
   - CLI commands work correctly
   - Adding/removing ranges works

### Edge Cases
- Empty IP split-tunnel configuration → No rules applied, normal blocking behavior
- Corrupted settings file → Log error, continue with empty configuration
- nftables not available → Log error, continue without IP split-tunnel

### Security Considerations
- IP split-tunnel rules only mark traffic; they don't bypass the firewall entirely
- The main Mullvad firewall still controls accept/drop decisions
- Leak protection remains active for non-whitelisted traffic
- Early boot application ensures no window where rules are missing

## Additional Consideration: System-Wide Toggle

The user mentioned implementing a "toggle for SYSTEM-WIDE 10.64 ip range allowance". This could mean:

1. **Auto-apply CGNAT range at boot** - A setting that automatically applies 100.64.0.0/10 at system startup
2. **Persistent template** - Make `ApplyTemplates` command persist across reboots

**Recommendation:** The current implementation already persists ranges to disk, so once applied, they survive reboots. The fix for Issue 2 will make them active at boot. No additional toggle needed unless the user wants a specific "enable at boot" setting separate from the range configuration.

If a toggle is desired:
- Add `apply_cgnat_at_boot: bool` to settings
- In early boot, check this setting and auto-apply CGNAT_RANGE if true
- Provide CLI command to enable/disable this setting
