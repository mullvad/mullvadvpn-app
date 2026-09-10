# Fork note: extra local networks (Tailscale)

This fork adds one thing to upstream Mullvad: a file, `extra-lan-networks.txt`
in the repository root, whose entries are added to the address blocks that
"Local network sharing" allows outside the tunnel. It ships with Tailscale's
IPv4 range `100.64.0.0/10`. Tailscale's IPv6 range (`fd7a:115c:a1e0::/48`)
already falls inside `fc00::/7`, which the unmodified app allows.

The list is compiled into the app. There is no runtime setting, on purpose:
the firewall on every platform keeps working from a constant, and nothing on
the running system can widen the list.

## Using it for your own devices

1. Open `extra-lan-networks.txt`. The comments at the top explain the format.
2. Add one entry per line. For a single device write just its address, such
   as `100.101.102.103`. The `/32` (or `/128` for IPv6) suffix is optional
   and means the same thing. For a whole range use CIDR notation starting at
   the network address, such as `100.81.0.0/24`.
3. Remove or comment out `100.64.0.0/10` if you only want your own devices.
4. Commit, then either run the workflow (below) or build locally.

Find a device's Tailscale address with `tailscale ip -4` on the device or in
the Tailscale admin console. Addresses are stable for the life of the device.

The build refuses anything outside the private (`10/8`, `172.16/12`,
`192.168/16`), link-local, unique-local and shared/CGNAT (`100.64/10`) ranges,
and anything with host bits set behind a prefix, with a message that says what
to write instead. This stops a typo from exempting public internet addresses
from the tunnel.

## How it is wired in

- `talpid-types/build.rs` parses the file at build time and generates the
  `EXTRA_LAN_NETS` constant.
- `talpid-types/src/net/allowed_nets.rs` keeps upstream's six entries as
  `BASE_LAN_NETS` and defines `ALLOWED_LAN_NETS` as base plus extras. Every
  platform firewall (Linux nftables, macOS pf, Android routes, and the
  generated Windows header) reads that constant, so no firewall code changes.
- Tests in `talpid-types` check that every entry in the file is present in
  the compiled list. The Windows header snapshot test uses the base list, so
  it never depends on the file's contents.
- `docs/security.md` and the app's Local network sharing info dialog mention
  the file. The dialog cannot list your entries, since it is static text.

## Things to be aware of

- `100.64.0.0/10` is the carrier-grade NAT range. Some ISPs and most mobile
  carriers hand these addresses to customers. With Local network sharing on
  and Tailscale stopped, traffic to other hosts in that range on such a
  network bypasses the tunnel. Listing only your devices avoids this.
- The same list decides whether a custom DNS server counts as "local" and is
  reachable outside the tunnel. With the full `/10`, Tailscale's MagicDNS
  resolver (`100.100.100.100`) qualifies; with per-device entries it does
  not unless you add it.

## Staying current with upstream

The workflow in `.github/workflows/sync-tailscale-lan.yml` rebases the
fork's commits onto each new upstream stable release, verifies the file
parses and the tests pass, pushes `tailscale-lan/<tag>`, and builds:

- Linux `.deb` and `.rpm` packages inside Mullvad's own build container.
- The Windows installer `.exe` on a GitHub-hosted Windows runner, using the
  same build-environment action as upstream's CI.

Both appear as artifacts on the workflow run. The Windows installer is
unsigned, so SmartScreen shows a warning on first run. The kernel drivers it
installs are Mullvad's own signed binaries from the `dist-assets/binaries`
submodule.

To make it run on its own:

1. Merge this branch into the fork's default branch. GitHub only schedules
   workflows from the default branch.
2. Open the Actions tab once and enable workflows. GitHub disables scheduled
   workflows on forks until the owner does this, and pauses them again after
   60 days without repository activity.
3. Optionally run it by hand from the Actions tab. Inputs let you pick a
   specific upstream tag, force a rebuild of an existing branch, or skip the
   Linux or Windows build.

For macOS, check out `tailscale-lan/<tag>` and build locally as described in
`BuildInstructions.md`. A rebase conflict fails the run and GitHub emails the
repository owner; nothing is pushed in that case.
