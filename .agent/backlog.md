# Agent Backlog

## Priority 1: IP Split-Tunneling / Whitelisting

Status: not started.

User problem: allow specific IP ranges, hosts, or VPN-related routes to bypass or coexist with
Mullvad so tools such as Netbird, Tailscale, and similar private-network VPN interfaces can keep
working while Mullvad is active.

Initial investigation areas:

- `mullvad-daemon` for settings, daemon commands, persistence, and management-interface hooks.
- `talpid-routing` for platform route management and route ownership.
- `talpid-core/src/firewall` for firewall policy and leak-prevention interactions.
- Existing split-tunneling code for platform conventions and user-safety assumptions.

Design constraints:

- Keep the implementation daemon-focused for the first iteration.
- Prefer fork-owned modules and narrow upstream integration hooks.
- Preserve existing leak protections by default.
- Make planned behavior explicit before changing tunnel or firewall rules.

Open design questions for the implementation phase:

- Which platforms are supported first.
- Whether the first user interface is settings-file only, management-interface driven, CLI driven,
  or a combination.
- Whether whitelist entries are CIDR ranges only or also named interfaces and resolved hostnames.
- How route and firewall behavior should fail when a requested bypass cannot be safely applied.

## Later Backlog

- Add a separate `patched-mullvad-vpn-beta-daemon-bin` AUR package for beta-channel daemon builds.
- Document the upstream sync workflow after the first feature lands.
- Add tests or smoke checks that prove fork patches remain isolated.
- Consider CLI or GUI surfaces only after the daemon behavior is stable.
