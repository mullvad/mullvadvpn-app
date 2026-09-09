# Fork note: Tailscale addresses count as LAN

This fork carries one behavioural change on top of upstream Mullvad: the
Tailscale IPv4 range `100.64.0.0/10` is part of the address blocks that
"Local network sharing" allows outside the tunnel. Tailscale's IPv6 range
(`fd7a:115c:a1e0::/48`) already falls inside `fc00::/7`, so it needed no
change.

The list is a compile-time constant, so this is a source patch, not a
setting. The files the patch touches:

- `talpid-types/src/net/allowed_nets.rs`: the list itself. Every platform
  firewall (Linux nftables, macOS pf, Android routes, and the generated
  Windows header) derives from it.
- `talpid-types/src/bin/snapshots/generate_cpp_lannetworks__tests__cpp_definition.snap`:
  snapshot of the generated Windows header.
- `docs/security.md`: the firewall specification.
- `desktop/packages/mullvad-vpn/.../allow-lan-setting/AllowLanSetting.tsx`:
  the range list shown in the app's info dialog.

## Things to be aware of

- `100.64.0.0/10` is the carrier-grade NAT range. Some ISPs and most mobile
  carriers hand these addresses to customers. With Local network sharing on,
  traffic to other hosts in that range on such a network bypasses the tunnel.
- The same list decides whether a custom DNS server counts as "local" and is
  reachable outside the tunnel. Tailscale's MagicDNS resolver
  (`100.100.100.100`) now qualifies.

## Staying current with upstream

The workflow in `.github/workflows/sync-tailscale-lan.yml` rebases the
fork's commits onto each new upstream stable release, verifies the patch is
still present and the tests pass, pushes `tailscale-lan/<tag>`, and builds
Linux packages as a workflow artifact.

To make it run on its own:

1. Merge this branch into the fork's default branch. GitHub only schedules
   workflows from the default branch.
2. Open the Actions tab once and enable workflows. GitHub disables scheduled
   workflows on forks until the owner does this, and pauses them again after
   60 days without repository activity.
3. Optionally run it by hand from the Actions tab. Inputs let you pick a
   specific upstream tag, force a rebuild of an existing branch, or skip the
   Linux build.

For Windows or macOS, check out `tailscale-lan/<tag>` and build locally as
described in `BuildInstructions.md`. A rebase conflict fails the run and
GitHub emails the repository owner; nothing is pushed in that case.
