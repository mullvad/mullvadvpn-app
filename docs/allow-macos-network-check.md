# Issues with macOS getting stuck in the offline state for too long

When macOS is coming back from sleep or connecting to a new WiFi network, it may try to send various
requests over the internet before it publishes a default route to the routing table. Since our
daemon relies on the routing table to obtain a default route to route traffic to relays and bridges,
and since macOS's network reachability seemingly does too, the daemon won't be able to connect to a
relay and thus stay in the blocked state for a prolonged time. The default route is only published
when macOS finishes or times out it's captive portal check. The captive portal check involves
looking up `captive.apple.com` and issuing an HTTP request to the resolved address, and by default,
if the app is blocking traffic, none of these network operations can take place, so the timeout is
always incurred, which forces the app into the offline error state for a prolonged time.

To not have to wait for macOS to time out it's captive portal check, the app should allow the
captive portal check even when it's in a blocking state, whilst still blocking all arbitrary DNS
traffic. This necessitates filtering DNS traffic at the application layer rather than the network
layer, so only the request for `captive.apple.com` is leaked, and before the response is returned,
the firewall rules are updated to allow the resolved addresses from the response to be reachable.

# Leaking macOS network-check traffic

To allow macOS's network-check to function, _some_ DNS queries need to leaked during a blocked
state. This can be done via using a resolver that is selectively reacts to some DNS queries and is
able to reach upstream resolvers when the app is in a blocking state. For now, this is achieved by
excluding all traffic from a Mullvad specific group, and having the resolver run as part of the
daemon, which asserts the groups ID on startup. The firewall rules that exclude the resolver traffic
and the resolved IP addresses should only be in effect if the app has been configured to allow macOS
network check. When receiving upstream responses, the DNS server in question should first have the
firewall be reconfigured such that the resolved IP addresses are reachable.

## Requirements from the daemon

To enable the custom resolver, certain conditions in the rest of the daemon need to be met:
- The firewall must allow traffic coming from our resolver (identified via GID) to the configured
  upstream resolvers.  The firewall must have a list of IPs for which traffic will be
  allowed to pass. The list will be populated by the resolved A and AAAA records, and reset when the
  tunnel state machine moves away from the error state.
- The daemon must configure the system to use the filtering resolver.
- The resolver must only reply to queries when it's in an active state and it must only reply to
  allowed queries. For now, only queries for `captive.apple.com` are allowed.

## Filtering resolver's behavior

The functionality of this feature is strongly tied to the states of the app when it's blocking
traffic. These blocking states include the app when it's in the disconnected mode with _always
require vpn_ turned on or in an error state with a blocking reason that isn't related to setting DNS
or starting the filtering resolver. In all other tunnel states, the filtering resolver and firewall
rules shouldn't be affected by this feature.

### State to keep track of

- List of allowed IP addresses as a result of being responses to issues DNS requests, which should
    be cleared when leaving the blocking state.
- The daemon should keep track of *if* the user has enabled the filtering resolver. If the user enables
  the custom resolver but something is already listening on port 53, then this should be reported
  back to the front-ends. The user needs to know that the filtering resolver failed to run.

### When the network-check leak is toggled on

- When in a blocking state:
  1. Exclude the local resolver's traffic from the firewall.
  1. Configure the filtering resolver to bind to port 53.
  1. Read the system's current DNS config and configure the filtering resolver to use it.
  1. Configure the host to use our local resolver
- In all other states, the filtering resolver should bind to port 53.
If any of the above steps fail, the app should report the failure to the frontend that toggled the
setting.

### When the network-check leak is toggled off
- When in a blocking state:
  1. If the host's DNS config is currently using our resolver, this should be reverted.
  1. The firewall should be reset to not allow the resolver traffic and the resolved IP traffic through.
  1. The filtering resolver should be shut down, unbinding from port 53.
- In all other states, the filtering resolver should be shut down, to leave port 53 free.

### When the network-check leak is enabled
#### Behavior when the daemon enters a blocking state
To enable the filtering resolver when entering the error state the daemon should do the following:
1. Exclude the local resolver's traffic from the firewall.
1. Read the system's current DNS config and configure the filtering resolver to use it.
1. Configure the host to use our local resolver.

If any of the above steps fail, and the daemon is not in the disconnected state, it should
transition to an error state and not attempt to start the filtering resolver again.

#### Resolver's behavior when receiving a DNS query
- When the daemon is in a blocking state, and the query is allowed:
  1. The query should be forwarded to the upstream resolvers
  1. When receiving the response, it's `A` and `AAAA` records should be allowed through the firewall.
  1. The response should be forwarded to the original requester.
- Otherwise, if the network-check allowing is enabled, the response should be ignored. If the
  option is disabled, it shouldn't be possible to receive a DNS query.

#### When the daemon leaves a blocking state:
- The host's DNS configuration is reverted to no longer use the filtering resolver.
- The list of IP addresses that are allowed to pass through our firewall are cleared.

