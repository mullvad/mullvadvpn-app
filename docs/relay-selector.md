
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

# Relay selector

The relay selector's main purpose is to pick a single Mullvad relay from a list of relays taking
into account certain user-configurable criteria.  Relays can be filtered by their _location_
(country, city, hostname), by the protocols and ports they support (transport protocol, tunnel
protocol, port), and by other constraints.  The constraints are user specified and stored in the
settings. The default value for location constraints restricts relay selection to relays from Sweden.
The default protocol constraints default to _Auto_, which implies specific behavior.

Generally, the filtering process consists of going through each relay in our relay list and
removing relay and endpoint combinations that do not match the constraints outlined above. The
filtering process produces a list of relays that only contain matching endpoints.  Of all the relays
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
constraints, following default ones will take effect:

- If no tunnel protocol is specified, the first three connection attempts will use WireGuard. All
  remaining attempts will use OpenVPN. If no specific constraints are set:
  - The first two attempts will connect to a Wireguard server, first on a random port, and then port
    53.
  - The third attempt will connect to a Wireguard server on port 80 with _udp2tcp_.
  - Remaining attempts will connect to OpenVPN servers, first over UDP on two random ports, and then
    over TCP on port 443. Remaining attempts alternate between TCP and UDP on random ports.

- If the tunnel protocol is specified as WireGuard and obfuscation mode is set to _Auto_:
  - First two attempts will be used without _udp2tcp_, using a random port on first attempt, and
    port 53 on second attempt.
  - Next two attempts will use _udp2tcp_ on ports 80 and 5001 respectively.
  - The above steps repeat ad infinitum.

  If obfuscation is turned on, connections will alternate between port 80 and port 5001 using
  _udp2tcp_ all of the time.

  If obfuscation is turned _off_, WireGuard connections will first alternate between using
  a random port and port 53, e.g. first attempt using port 22151, second 53, third
  26107, fourth attempt using port 53, and so on.

  If the user has specified a specific port for either _udp2tcp_ or WireGuard, it will override the
  port selection, but it will not change the connection type described above (WireGuard or WireGuard
  over _udp2tcp_).

- If no OpenVPN tunnel constraints are specified, then the first two attempts at selecting a tunnel
  will try to select UDP endpoints on any port, and the third and fourth attempts will filter for
  TCP endpoints on port 443. Any subsequent filtering attempts will alternate between TCP and UDP on
  any port.

## Selecting tunnel endpoint between filtered relays

To select a single relay from the set of filtered relays, the relay selector uses a roulette wheel
selection algorithm using the weights that are assigned to each relay.  The higher the weight is
relatively to other relays, the higher the likelihood that a given relay will be picked. Once a
relay is picked, then a random endpoint that matches the constraints from the relay is picked.

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

Currently, there is only a single type of obfuscator - _udp2tcp_, and it's only used if it's mode is
set to _On_ or _Auto_ and the user has selected WireGuard to be the only tunnel protocol to be used.

