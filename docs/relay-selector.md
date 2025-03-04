
# Glossary

- Relay - a server that provides one or multiple tunnel and bridge endpoints, and has a weight
  associated with it
- Endpoint - a combination of a socket address and the transport protocol
- Transport protocol - TCP or UDP
- Tunnel protocol - WireGuard or OpenVPN
- Obfuscation - Putting WireGuard, OpenVPN or API traffic inside a protocol designed to make it
  harder to fingerprint or block the contained traffic. This is used to circumvent censorship.
  Mullvad hosts many different obfuscation protocols. Some are hosted directly on the VPN relays,
  but most are hosted on separate bridge servers. Even if most obfuscation protocols used include
  encryption, that encryption is not to be treated as secure. We only use the obfuscation protocol
  for its obfuscating properties, not for any security properties it might have.
- DAITA - Short for "Defense against AI-guided Traffic Analysis". A technique supported on some
  WireGuard relays that makes website fingerprinting more difficult.

# Relay selector

The relay selector's main purpose is to pick a single Mullvad relay from a list of relays taking
into account certain user-configurable criteria. Relays can be filtered by their _location_
(country, city, hostname), by the protocols and ports they support (transport protocol, tunnel
protocol, port), and by other constraints. The constraints are user specified and stored in the
settings. The default value for location constraints restricts relay selection to relays from Sweden.
The tunnel protocol constraint defaults to Wireguard.

Generally, the filtering process consists of going through each relay in our relay list and
removing relay and endpoint combinations that do not match the constraints outlined above. The
filtering process produces a list of relays that only contain matching endpoints. Of all the relays
that match the constraints, one is selected and a random matching endpoint is selected from that
relay.

The relay selector selects a tunnel endpoint first, and then uses the selected tunnel endpoint to
select a bridge endpoint if necessary - a bridge will only be selected if the bridge state, current
retry attempt and the tunnel protocol allow for it.

## Tunnel endpoint constraints

Endpoints may be filtered by:

- tunnel type (WireGuard or OpenVPN for tunnel endpoints)
- transport protocol (UDP or TCP), not applicable if the tunnel protocol only allows a single one,
  like WireGuard
- entry port
- location (country, city, hostname)
- provider
- ownership (Mullvad-owned or rented)

### Default constraints for tunnel endpoints

Whilst all user selected constraints are always honored, when the user hasn't selected any specific
constraints the following default ones will take effect

#### Tunnel protocol is Wireguard

- The first attempt will connect to a Wireguard relay on a random port
- The second attempt will connect to a Wireguard relay on port 443
- The third attempt will connect to a Wireguard relay over IPv6 (if IPv6 is configured on the host) on a random port
- The fourth attempt will connect to a Wireguard relay on a random port using Shadowsocks for obfuscation
- The fifth attempt will connect to a Wireguard relay on a random port using [UDP2TCP obfuscation](https://github.com/mullvad/udp-over-tcp)
- The sixth attempt will connect to a Wireguard relay over IPv6 on a random port using UDP2TCP obfuscation (if IPv6 is configured on the host)

#### Tunnel protocol is OpenVPN

Note: This is not applicable to Android nor iOS.

- The first attempt will connect to an OpenVPN relay on a random port
- The second attempt will connect to an OpenVPN relay over TCP on port 443
- The third attempt will connect to an OpenVPN relay over a bridge on a random port

### Default constraints for tunnel endpoints on iOS

The iOS platform does not support OpenVPN, or connecting to a relay over IPv6.
As such, the above algorithm is simplified to the following version:
  - The first attempt will connect to a Wireguard relay on a random port
  - The second attempt will connect to a Wireguard relay on port 443
  - The third attempt will connect to a Wireguard relay on a random port using Shadowsocks for obfuscation
  - The fourth attempt will connect to a Wireguard relay on a random port using [UDP2TCP obfuscation](https://github.com/mullvad/udp-over-tcp)

### Random Ports for UDP2TCP and Shadowsocks

- The UDP2TCP random port is **either** 80 **or** 5001
- The Shadowsocks port is random within a certain range of ports defined by the relay list

If no tunnel has been established after exhausting this list of attempts, the relay selector will
loop back to the first default constraint and continue its search from there.

Any default constraint that is incompatible with user specified constraints will simply not be
considered. Conversely, all default constraints which do not conflict with user specified constraints
will be used in the search for a working tunnel endpoint on repeated connection failures.

## Selecting tunnel endpoint between filtered relays

To select a single relay from the set of filtered relays, the relay selector uses a roulette wheel
selection algorithm using the weights that are assigned to each relay. The higher the weight is
relatively to other relays, the higher the likelihood that a given relay will be picked. Once a
relay is picked, then a random endpoint that matches the constraints from the relay is picked.

## Selecting a DAITA-compatible relay

Since not all Wireguard relays deploy DAITA, there are lots of tunnel endpoint constraints that
are fundamentally incompatible with DAITA. As such, if DAITA is enabled the relay selector may select
an alternative entry relay and implicitly use multihop in order to achieve a seamless user experience.
The user's tunnel endpoint constraint is respected for the exit relay.

The user may opt out of this behaviour by toggling the "Direct only" option in the DAITA settings.

## Bridge endpoint constraints

The explicit constraints are:

- location
- provider
- ownership

The transport protocol is supposedly inferred by the selected bridge- but for now, the daemon only
supports TCP bridges, so only TCP bridges are being selected. If no location constraint is specified
explicitly, then the relay location will be used.

### Selecting a bridge endpoint between filtered relays

When filtering bridge endpoints by location, if multiple bridge endpoints match the specified
constraints then endpoints which are geographically closer to the selected tunnel relay are more
likely to be selected. If bridge state is set to _On_, then a bridge is always selected and used.
If it's set to _Auto_, a bridge will only be tried after 3 failed attempts at connecting without a
bridge and only if the relay constraints allow for a bridge to be selected.

### Bridge caveats

Currently, bridges only support TCP tunnels over TCP bridges. This means that if the bridge state is
set to _On_, the daemon will automatically set the tunnel constraints to _OpenVPN over TCP_. Once we
have bridges that support UDP tunnels over TCP bridges, this behavior should be removed. Conversely,
changing the tunnel constraints to ones that do not support bridges (WireGuard, OpenVPN over UDP)
will indirectly change the bridge state to _Auto_ if it was previously set to _On_.


### Obfuscator caveats

There are two type of obfuscators - _udp2tcp_, and _shadowsocks_.
They are used if the obfuscation mode is set _Auto_ and the user has selected WireGuard to be the only tunnel protocol to be used.
