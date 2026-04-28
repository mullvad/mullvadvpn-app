# IP Split Tunneling Mullvad Fork

> [!WARNING]
> This fork works currently only for Linux users, if you're from another platform and wish this worked for you I need to know you exist, star this repo and create an issue for your platform and we can continue developing this fork for more people.

This repository is an unofficial fork of
[mullvadvpn-app](https://github.com/mullvad/mullvadvpn-app).

The goal is to keep the codebase as close to upstream Mullvad as possible while adding a small
set of community-requested daemon features. Changes should be isolated, easy to review, and easy
to carry forward when upstream updates can be merged without major conflicts.

This fork is for advanced users who know exactly why they want these changes. If you want the
official supported Mullvad VPN app, use Mullvad's official releases instead.

## Installation and usage
Install and replace your Mullvad daemon with the one on [Releases](https://github.com/tritaum/ip-split-tunneling-mullvad/releases) , if you're on Arch theres an [AUR package](https://aur.archlinux.org/packages/patched-mullvad-vpn-daemon-bin) for that

After installed your Mullvad CLI will have a new subset of commands on `mullvad split-tunnel ip` there you have a manual explaining everything, but if you want something straightforward simply use `mullvad split-tunnel ip apply-templates` and that will allow CGNAT ranges for you.

## Project Status

Current features:

- [x] Upstream Mullvad VPN app codebase
- [x] Fork planning and agent guidance
- [X] IP split-tunneling / IP whitelist
- [X] Early-boot firewall rules appliance.
- [X] Automatic upstream sync with the original Mullvad repo

Planned features:

- [ ] VPN-interfaces split tunneling for tools like Netbird and Tailscale
- [ ] Daemon-focused patch isolation
- [ ] Cross-platform compatibility
- [ ] Mullvad UI patches
- [ ] Community requested features if any

## Development Direction

This fork should behave like a small patch set over upstream Mullvad:

- Prefer new fork-specific modules and folders over editing upstream code directly.
- Touch upstream files only where an import, call site, or integration hook is required.
- Avoid unrelated refactors, formatting churn, and broad rewrites.
- Keep daemon changes separate from GUI, CLI, and mobile work unless a feature explicitly needs
  those surfaces.
- Keep documentation honest about what is implemented and what is only planned.

See `.agent/` and AI related files for future-agent guidance and the current backlog.
