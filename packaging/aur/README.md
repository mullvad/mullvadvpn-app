# AUR Packaging

This folder contains the Arch User Repository packaging maintained by this fork.

Two AUR package bases are used:

- `patched-mullvad-vpn-bin`
  - `patched-mullvad-vpn-bin`
  - `patched-mullvad-vpn-daemon-bin`
- `patched-mullvad-vpn`
  - `patched-mullvad-vpn`
  - `patched-mullvad-vpn-daemon`

The `-bin` package base consumes GitHub release `.deb` assets from this fork. The source package
base builds from this fork's Git tags.

## Local Build

On Arch Linux or an Arch container with `base-devel` installed:

```bash
packaging/aur/scripts/build-local patched-mullvad-vpn-bin
packaging/aur/scripts/build-local patched-mullvad-vpn
```

The script copies the selected package base into `.build/aur/` before running `makepkg`, so local
build artifacts do not dirty the tracked package recipes.

## Metadata

Regenerate AUR metadata after changing a `PKGBUILD`:

```bash
packaging/aur/scripts/update-srcinfo patched-mullvad-vpn-bin
packaging/aur/scripts/update-srcinfo patched-mullvad-vpn
```

`makepkg --printsrcinfo > .SRCINFO` is the source of truth for `.SRCINFO`.

## Publishing

Publishing is intentionally a separate step. After the AUR repositories exist and an SSH key has
access, use:

```bash
packaging/aur/scripts/publish patched-mullvad-vpn-bin
packaging/aur/scripts/publish patched-mullvad-vpn
```

The same publish script is intended for future GitHub Actions usage with a deploy key stored as a
repository secret.
