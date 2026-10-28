# Relay selector

The relay selector is the part of the VPN client that filters relays and decides their configuration,
endpoints, and features.

## Glossary

- **Relay** - A VPN server which all traffic is routed through. Provides one or multiple tunnel
  endpoints and has a weight that affects the likelihood of it being selected.
- **Endpoint** - A combination of a socket address and the transport protocol (TCP or UDP).
- **Anti-censorship** - Methods for reaching the Mullvad services while under censorship or other
  network restrictions by making the traffic harder to distinguish from non-VPN traffic.
  Can be configured under the settings page of the same name.
- **WireGuard** - The VPN protocol used by Mullvad. Changing the WireGuard port is an anti-censorship
  method.
- **Obfuscation protocol** - Circumvents censorship by putting WireGuard or API traffic inside another
  protocol - designed to make it harder to fingerprint. There are currently four obfuscation protocols:
  _udp2tcp_, _shadowsocks_, _quic_, and _lwo_.
- **DAITA** - Short for "Defense against AI-guided Traffic Analysis". A technique supported on some
  WireGuard relays that makes website fingerprinting more difficult.

## Relay selection

Relays are filtered by:

- location (country, city, hostname)
- provider
- ownership (Mullvad-owned or rented)
- support for features such as DAITA and obfuscation methods
- device IP version and port

To select a single relay from the set of filtered relays, the relay selector uses a roulette wheel
selection algorithm using the weights that are assigned to each relay. The higher the weight is
relative to other relays, the higher the likelihood that a given relay will be picked. Once a
relay is picked, a random endpoint that matches the constraints from the relay is picked.

If multihop is enabled, two relays are selected and the user connection is routed through them in
sequence. Anti-censorship, DAITA, IP version, and port settings only apply to the entry relay.

### Automatic anti-censorship selection

If the anti-censorship setting is set to _Automatic_, the relay selector will first attempt to connect
without an obfuscation method. If that fails, it will simultaneously try all obfuscation methods
that are available on the relay to find a working tunnel endpoint. Both IPv4 and IPv6 will be
tried if the device IP version is set to automatic. The first working obfuscation method will be
selected and used for the remainder of the session.

If a working endpoint is not found, subsequent connection attempts use other relays, chosen so that
obfuscation methods not supported by the previously tried relays are attempted next. Once every
enabled method has been tried, the sequence restarts from the plain WireGuard attempt.

### Default constraints for tunnel endpoints on iOS

The iOS platform supports connecting to a relay over IPv6 but will use only
either IPv4 or IPv6, depending on user settings. As such, the above algorithm
is simplified to the following version:

1. The first attempt will connect to a relay on a random port.
2. The second attempt will connect to a relay on a random port using Shadowsocks for obfuscation.
3. The third attempt will connect to a relay using QUIC for obfuscation.
4. The fourth attempt will connect to a relay on a random port using
   [UDP2TCP obfuscation](https://github.com/mullvad/udp-over-tcp).
5. The fifth attempt will connect to a relay using LWO.

### Ports

Wireguard and the obfuscation protocols have different valid port ranges. If a specific anti-censorship
method is selected, then a port can also be specified. Otherwise, the port is randomly selected in the
valid ranges.

Valid ports for each anti-censorship method are:

- **WireGuard** - 53, 123, 4000-33433, 33565-51820 and 52001-60000.
- **LWO** - The same ports as WireGuard.
- **Shadowsocks** - 51900-51949 is supported on all relays (using the same IP address
  as the WireGuard endpoint). Some relays also have dedicated shadowsocks IP addresses
  which support any port. Setting a port outside this range will therefore reduce the
  number of available relays.
- **QUIC** - Always port 443.
- **UDP2TCP** - 80, 443, or 5001 on desktop. On iOS, only 80 or 443.

The valid port ranges are provided by the relay list and may change from what's listed here.
