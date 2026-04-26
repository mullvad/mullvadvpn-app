# IP Split Tunneling Mullvad Fork

This repository is an unofficial fork of
[mullvadvpn-app](https://github.com/mullvad/mullvadvpn-app).

The goal is to keep the codebase as close to upstream Mullvad as possible while adding a small
set of community-requested daemon features. Changes should be isolated, easy to review, and easy
to carry forward when upstream updates can be merged without major conflicts.

This fork is for advanced users who know exactly why they want these changes. If you want the
official supported Mullvad VPN app, use Mullvad's official releases instead.

## Project Status

Current features:

- [x] Upstream Mullvad VPN app codebase
- [x] Fork planning and agent guidance

Planned features:

- [ ] IP split-tunneling / IP whitelist
- [ ] VPN-interface compatibility for tools like Netbird and Tailscale
- [ ] Daemon-focused patch isolation

## Development Direction

This fork should behave like a small patch set over upstream Mullvad:

- Prefer new fork-specific modules and folders over editing upstream code directly.
- Touch upstream files only where an import, call site, or integration hook is required.
- Avoid unrelated refactors, formatting churn, and broad rewrites.
- Keep daemon changes separate from GUI, CLI, and mobile work unless a feature explicitly needs
  those surfaces.
- Keep documentation honest about what is implemented and what is only planned.

See `.agent/` for future-agent guidance and the current backlog.
