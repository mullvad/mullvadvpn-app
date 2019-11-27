
# Glossary

- Relay - a server that provides one or multiple tunnel and bridge endpoints, and has a weight
  associated with it
- Endpoint - a combination of a socket address and the transport protocol
- Transport protocol - TCP or UDP
- Tunnel protocol - WireGuard or OpenVPN

# Relay selector

The relay selector's main purpose is to pick a single Mullvad relay from a list of relays taking
into account certain user-configurable criteria.  Relays can be filtered by their _location_
(country, city, hostname) and by the protocols and ports they support (transport protocol, tunnel
protocol, port).  The constraints are user specified and stored in the settings.  The default value
for location constraints restricts relay selection to relays from Sweden. The default protocol
constraints default to _auto_, which implies specific behavior.

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

### Default constraints for tunnel endpoints

Whilst all user selected constraints are always honored, when the user hasn't selected any specific
constraints, following default ones will take effect:

- If no tunnel protocol is specified for tunnel endpoints, then by default only OpenVPN
  endpoints will be selected.

- If the tunnel protocol is specified as WireGuard without any other protocol constraints, then the
  transport protocol is not applicable as only UDP endpoints exist and any port will be matched.

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

Currently, the only explicit constraints for bridges is the location, and the transport protocol is
supposedly inferred by the selected bridge- but for now, the daemon only supports TCP bridges, so
only TCP bridges are being selected. If no location constraint is specified explicitly, then the
relay location will be used.

### Selecting a bridge endpoint between filtered relays

When filtering bridge endpoints by location, if multiple bridge endpoints match the specified
constraints then currently the one which is geographically closest to the selected tunnel relay
would be selected. Ideally, rather than always picking the closest one given the same constraints
and tunnel endpoint, a different but still geographically close bridge endpoint would be selected if
the daemon failed to connect to the first ones initially. If bridge state is set to _On_, then a
bridge is always selected and used. If it's set to _auto_, a bridge will only be tried after 3
failed attempts at connecting without a bridge and only if the relay constraints allow for a bridge
to be selected.

### Bridge caveats

Currently, bridges only support TCP tunnels over TCP bridges. This means that if the bridge state is
set to _On_, the daemon will automatically set the tunnel constraints to _OpenVPN over TCP_. Once we
have bridges that support UDP tunnels over TCP bridges, this behavior should be removed. Conversely,
changing the tunnel constraints to ones that do not support bridges (WireGuard, OpenVPN over UDP)
will indirectly change the bridge state to _Auto_ if it was previously set to _On_.

