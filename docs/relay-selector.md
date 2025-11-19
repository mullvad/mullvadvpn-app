
# Glossary

- Relay - a server that provides one or multiple tunnel endpoints, and has a weight
  associated with it
- Endpoint - a combination of a socket address and the transport protocol
- Transport protocol - TCP or UDP
- Obfuscation - Putting WireGuard or API traffic inside a protocol designed to make it
  harder to fingerprint or block the contained traffic. This is used to circumvent censorship.
  Mullvad hosts many different obfuscation protocols. Even if most obfuscation protocols used include
  encryption, that encryption is not to be treated as secure. We only use the obfuscation protocol
  for its obfuscating properties, not for any security properties it might have.
- Anti-censorhip - Umbrella term for methods and protocols used to circumvent network censorship.
  Obfuscation is a type of Anti-censorhip measurement that Mullvad use for this effort.
- DAITA - Short for "Defense against AI-guided Traffic Analysis". A technique supported on some
  WireGuard relays that makes website fingerprinting more difficult.

# Relay selector

The relay selector's main purpose is to pick a configuration of one or more Mullvad relays taking
into account certain user-configurable criteria. Relays can be filtered by their _location_
(country, city, hostname), by the protocols and ports they support and by other constraints.
The constraints are user specified and stored in the settings. The default value for location
constraints restricts relay selection to relays from Sweden.

Generally, the filtering process consists of going through each relay in our relay list and
removing relay and endpoint combinations that do not match the constraints outlined above. The
filtering process produces a list of relays that only contain matching endpoints. Of all the relays
that match the constraints, one is selected and a random matching endpoint is selected from that
relay.

## Tunnel endpoint constraints

Endpoints may be filtered by:

- entry port
- location (country, city, hostname)
- provider
- ownership (Mullvad-owned or rented)

### Default constraints for tunnel endpoints

Whilst all user selected constraints are always honored, when the user hasn't selected any specific
constraints the following default ones will take effect

- The first attempt will connect to a relay on a random port
- The second attempt will connect to a relay over IPv6 (if IPv6 is configured on the host) on a random port
- The third attempt will connect to a relay on a random port using Shadowsocks for obfuscation
- The fourth attempt will connect to a relay using QUIC for obfuscation
- The fifth attempt will connect to a relay on a random port using [UDP2TCP obfuscation](https://github.com/mullvad/udp-over-tcp)
- The sixth attempt will connect to a relay over IPv6 on a random port using UDP2TCP obfuscation (if IPv6 is configured on the host)
- The seventh attempt will connect to a relay using LWO

### Default constraints for tunnel endpoints on iOS

The iOS platform does not support connecting to a relay over IPv6.
As such, the above algorithm is simplified to the following version:

- The first attempt will connect to a relay on a random port
- The second attempt will connect to a relay on a random port using Shadowsocks for obfuscation
- The third attempt will connect to a relay using QUIC for obfuscation
- The fourth attempt will connect to a relay on a random port between port 80 and 443 using [UDP2TCP obfuscation](https://github.com/mullvad/udp-over-tcp)

### Random Ports for UDP2TCP and Shadowsocks

- The UDP2TCP random port is **either** 80 **or** 5001
- The Shadowsocks port is random within a certain range of ports defined by the relay list

### Ports for QUIC

QUIC will use port 443.

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

Since not all relays deploy DAITA, there are lots of tunnel endpoint constraints that
are fundamentally incompatible with DAITA. As such, if DAITA is enabled the relay selector may select
an alternative entry relay and implicitly use multihop in order to achieve a seamless user experience.
The user's tunnel endpoint constraint is respected for the exit relay.

The user may opt out of this behaviour by toggling the "Direct only" option in the DAITA settings.

### Obfuscator caveats

There are four types of obfuscators - _udp2tcp_, _shadowsocks_, _quic_, and _lwo_.
Any of them may be used if the anti-censorship method mode is set _Automatic_.
