# Mullvad VPN app architecture

This document describes the code architecture and how everything fits together.

For security and anonymity properties, please see [security](security.md).

Some components have specific documentation that go into greater detail:

- [winfw](../windows/winfw/README.md)

## Mullvad vs talpid

Explain the differences between these layers and why the distinction exists.
My thought was that after this section every aspect of the app is explained
under either the Mullvad or the Talpid header. So it's clear which part they
belong to. I yet don't know if this makes sense though.


## Mullvad part of daemon

### Frontend <-> system service communication

### Talking to api.mullvad.net

### Selecting relay and bridge servers
See [this document](relay-selector.md).

### Problem reports


## Talpid part of daemon

### Tunnel state machine

### System DNS management

### Firewall integration

### Detecting device offline

### OpenVPN plugin and communication back to system service


## Frontends

### Desktop Electron app

### Android

### iOS

### CLI
