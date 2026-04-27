# AUR Packaging

This folder contains the Arch User Repository packaging maintained by this fork.

For now the fork publishes one daemon-only package base:

- `patched-mullvad-vpn-daemon-bin`

The package consumes GitHub release `.deb` assets produced by `./build.sh --daemon-only`.
Desktop, source-built, and ARM64 AUR packages can be added later after the daemon feature set is
stable.

Keep `patched-mullvad-vpn-daemon-bin` on Mullvad's latest stable desktop version so the official
GUI does not warn that the daemon is newer than the installed app. Beta builds should use a future
`patched-mullvad-vpn-beta-daemon-bin` package base instead.

## Local Build

On Arch Linux or an Arch container with `base-devel` installed:

```bash
packaging/aur/scripts/build-local patched-mullvad-vpn-daemon-bin
```

The script copies the selected package base into `.build/aur/` before running `makepkg`, so local
build artifacts do not dirty the tracked package recipes.

## Metadata

Regenerate AUR metadata after changing a `PKGBUILD`:

```bash
packaging/aur/scripts/update-srcinfo patched-mullvad-vpn-daemon-bin
```

`makepkg --printsrcinfo > .SRCINFO` is the source of truth for `.SRCINFO`.

## Publishing

Publishing is intentionally a separate step. After the AUR repositories exist and an SSH key has
access, use:

```bash
packaging/aur/scripts/publish patched-mullvad-vpn-daemon-bin
```

The same publish script is intended for future GitHub Actions usage with a deploy key stored as a
repository secret.
