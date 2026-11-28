# Mullvad VPN without an account (WireGuard config only)

This is a fork of `mullvadvpn-app` at commit `e8fd32` (version `2026.5-dev`). The app does not log
in to a Mullvad account any more. It connects with a WireGuard config generated at mullvad.net, or
with a private key plus its tunnel address, and everything else works as in the regular app:
relay selection, multihop, DAITA, post-quantum key exchange, the anti-censorship methods
(UDP-over-TCP, Shadowsocks, QUIC, LWO, WireGuard port), the kill switch and lockdown mode, split
tunneling and the API access methods.

## Using it

GUI: on the first start the app shows "Import WireGuard config". Paste the config or pick the
file. The button in the header opens the config view with the public key and the tunnel
addresses, where the config can be replaced or removed.

CLI:

```
mullvad config import wg-mullvad-xx.conf      # "-" reads from standard input
mullvad config set --private-key <KEY> --address 10.66.1.2/32 --address fc00:bbbb:bbbb:bb01::2:102/128
mullvad config show                           # public key and addresses, never the private key
mullvad config clear                          # disconnects and deletes the key
mullvad connect
```

Only `PrivateKey` and `Address` from the `[Interface]` section are used. The relay, DNS, endpoint
and peer key in the file are ignored, the app chooses them like the regular app does.

The tunnel `Address` is required. Mullvad assigns it to the public key on its servers when the key
is registered, it cannot be derived from the private key. It is in the `Address` line of every
config generated at mullvad.net.

## What was changed

- Daemon: the device and account handling (`device/`, account history, voucher, login, the
  account related migrations) is removed. `identity.rs` holds the imported key and addresses in
  `wireguard-identity.json` in the settings directory (mode 0600 on Unix, on Windows the daemon's
  settings directory under the SYSTEM profile). Without an identity the daemon stays disconnected and refuses to connect.
  The relay list is still downloaded from the API, which needs no authentication.
- `mullvad-api`: all account, device and partner endpoints are removed.
- Management interface: the account and device RPCs are replaced by `ImportWireguardConfig`,
  `GetWireguardIdentity` and `ClearWireguardIdentity`, plus the `wireguard_identity` daemon event.
  Error messages never contain key material.
- CLI: `mullvad config ...` replaces `mullvad account` and `mullvad device`.
- GUI: the login, account, device management, voucher and payment views are removed. The import
  and config views are new. Update notifications are off by default because they pointed at the
  official release channel (`MULLVAD_ENABLE_UPDATE_NOTIFICATION=1` turns them on). The in-app
  upgrade is not built.
- Installers and uninstallers: the `remove-device` calls are gone.
- `factory-reset` also forgets the key, `reset-settings` keeps it. The key is never rotated, it
  stays what the config says.
- Not touched: the tunnel, firewall, routing, DNS, split tunneling, DAITA, post-quantum and
  obfuscation code.

## What was verified

- Rust: 143 unit tests pass in `mullvad-types`, `mullvad-daemon`, `mullvad-cli`,
  `mullvad-management-interface`, `mullvad-api` and `mullvad-setup`, among them new ones for the
  config parser and the identity store. Clippy is clean and the daemon, CLI and setup tool build
  for Linux and for `x86_64-pc-windows-msvc`.
- Linux end-to-end (`evidence/linux-e2e.txt`, 36 checks): real daemon and CLI against a WireGuard
  peer in a network namespace. Import, handshake and ping through the tunnel, UDP-over-TCP,
  replacing the config, restart persistence, `clear`, invalid configs, and that the private key
  appears nowhere but the identity file (not in settings, logs, cache, status or error output).
- GUI against that real daemon (`evidence/gui/`): import view, rejected config, connect from the
  button, config view, removal.
- GUI unit tests, type check, eslint and the mocked Playwright suite (136 passed, the skipped ones
  are platform specific).
- The installer: its contents were inspected (the four executables are byte-identical to the
  build output, the drivers, `relays.json` and the new GUI are in it). The executables start under
  Wine as far as Wine allows: CLI, daemon start, settings loading and the management pipe
  (`evidence/wine-smoke.txt`).

## What was not verified

- Nothing was run on real Windows: drivers (WireGuard-NT, Wintun, split tunnel), the firewall
  (WFP), the service and the installer itself could not be exercised on this build host.
- Post-quantum key exchange, DAITA, multihop, Shadowsocks, QUIC and LWO were not run end to end,
  they need real Mullvad relays. Their code is unchanged, only the key now comes from the config.
- The VM based integration tests in `test/` still use accounts and no longer build.

## Limitations

- The installer is not code signed, so Windows SmartScreen will warn.
- It uses the same service name, install directory, registry keys and installer GUID as the
  official app. Do not install both on one machine.
- Windows x64 only. The mobile and macOS targets were not adapted or built, the Android and iOS
  crates are removed from the workspace.
- The bundled `relays.json` is a snapshot, the daemon refreshes it from the API.

## Building the Windows installer on Linux

```
export XWIN_ACCEPT_LICENSE=1     # Microsoft CRT/SDK license for cargo-xwin
FAST=1 ./build-windows-on-linux.sh
```

The prerequisites are listed at the top of `build-windows-on-linux.sh`.
