# Issues with macOS getting stuck in the offline state for too long

When macOS is coming back from sleep or connecting to a new WiFi network, it may try to send various
requests over the internet before it publishes a default route to the routing table. Since our
daemon relies on the routing table to obtain a default route to route traffic to relays and bridges,
and since macOS's network reachability seemingly does too, the daemon won't be able to connect to a
relay and thus stay in the blocked state for a prolonged time. The default route is only published
when macOS finishes or times out its captive portal check. The captive portal check involves
looking up specific captive portal domains and issuing an HTTP request to the resolved address, and
by default, if the app is blocking traffic, none of these network operations can take place, so the
timeout is always incurred, which forces the app into the offline error state for a prolonged time.

To not have to wait for macOS to time out its captive portal check, the app should allow the
captive portal check even when it's in a blocking state, whilst still blocking all arbitrary DNS
traffic. However, only a DNS response is required to appease the connectivity check - and it doesn't
even need to be valid. As such, during blocked states the app can run a custom resolver that only
responds to queries for captive portal domains to allow macOS to do its connectivity check. Since no
lookups have to be made, no traffic needs to be leaked.

# Overcoming these issues in the daemon.

To allow the connectivity check to pass when blocking traffic, the daemon runs a custom resolver
that listens only on localhost on an arbitrary port. Traffic to it is only redirected during blocked
states. The resolver only replies to queries for captive portal domains. The resolver won't actually
send any packets besides replying to DNS query that originates from localhost.

### List of currently known captive portal domains

- `captive.apple.com`
- `netcts.cdn-apple.com`

