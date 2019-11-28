# Mullvad VPN app security

This document describes the security properties of the Mullvad VPN app. It describes it for all
platforms and their differences.

This document does not describe *how* we reach and uphold these properties, just what they are.
See the [architecture](architecture.md) document for details on how this security is implemented.

The main purpose of the app is to allow the user to make all network/internet traffic to and
from the device travel via an encrypted VPN tunnel.

## App states

At the core of the app is a state machine called the "tunnel state machine". The following
sub-sections will describe each state and what security properties hold and what network activity
will be blocked and allowed during them.

Except what's described as allowed in this document, all network packets should be blocked.

The following network traffic is always allowed to flow. It's never blocked, regardless of state:

1. All traffic on loopback adapters

1. DHCPv4 and DHCPv6 requests to go out and responses to come in:
   * Outgoing from `*:68` to `255.255.255.255:67` (client to server)
   * Incoming `*:67` to `*:68` (server to client)
   * Outgoing from `[fe80::]/10:546` to `[ff02::1:2]:547` and `[ff05::1:3]:547` (client to server)
   * Incoming from `[fe80::]/10:547` to `[fe80::]/10:546` (server to client)

1. Router solicitation, advertisement and redirects (subset of NDP):
   * Outgoing to `ff02::2``, but only ICMPv6 with type 133 and code 0.
   * Incoming from `[fe80::]/10`, but only ICMPv6 type 134 or 137 and code 0.

1. If the "Allow LAN" setting is enabled, the following is also allowed:
   * Outgoing to, and incoming from, any IP in an unroutable network, that means:
     * `10.0.0.0/8`
     * `172.16.0.0/12`
     * `192.168.0.0/16`
     * `169.254.0.0/16`
     * `fe80::/10`
   * Outgoing to any IP in a local, unroutable, multicast network, meaning these:
     * `224.0.0.0/24` (local subnet IPv4 multicast)
     * `239.255.255.250/32` (SSDP)
     * `239.255.255.251/32` (mDNS)
     * `ff02::/16` (Link-local IPv6 multicast. IPv6 equivalent of `224.0.0.0/24`)
     * `ff05::/16` (Site-local IPv6 multicast. Is routable, but should never leave the "site")
   * Incoming DHCPv4 requests and outgoing responses (be a DHCPv4 server):
     * Incoming from `*:68` to `255.255.255.255:67`
     * Outgoing from `*:67` to `*:68`

### Disconnected

This is the default state that the `mullvad-daemon` starts in when the device boots, unless
"Launch app on start-up" and "Auto-connect" are both active. Then the app will proceed to the
[connecting](#connecting) state immediately.

The disconnected state behaves very differently depending on the value of the
"block when disconnected" setting. If this setting is enabled the disconnected state behaves
like, and has the same security properties as, the [blocked](#blocked) state. If the setting is
disabled (the default), then it is the only state where the app does not enforce any firewall.
rules. This state behaves the same as if the `mullvad-daemon` was not even running. It lets
network traffic flow in and out of the computer freely.

The disconnected state is not active while the app changes server or if the VPN tunnel goes down
unexpectedly. See the [connecting](#connecting) state and [kill switch](#kill-switch)
documentation for these unexpected network issues. The only time this state is active is
initially when the daemon starts and later
when the user explicitly clicks the disconnect/cancel button to intentionally disable the VPN.

### Connecting

This state is active from when the app decides to create a VPN tunnel, until said tunnel has
been established and verified to work. Then it transitions to the [connected](#connected) state.

In this state network traffic to and from the IP and port that the VPN tunnel is established
towards. Meaning the IP of the VPN relay server and the selected OpenVPN or WireGuard port.
In the case where a bridge/proxy is used this IP/port combo becomes the IP of the bridge
and the port of the used proxying service.

If connecting via WireGuard, this state allows ICMP packets to and from the in-tunnel IPs
(both v4 and v6) of the relay server the app is currently connecting to. That means the private
network IPs where the relay will respond inside the tunnel.

### Connected

This state becomes active when [connecting](#connecting) has fully established a VPN tunnel. It
stays active until the user requests a disconnect, quit, server change, change of other setting
that affects the tunnel or until the tunnel goes down unexpectedly.

In this state, all traffic in both directions over the tunnel interface should be allowed. Minus
the DNS requests (TCP and UDP port 53) not to a gateway IP for that interface. Meaning we can
*only* request DNS from the relay server itself.

This state allows traffic to and from the IP and port combo that the tunnel runs over. See the
[connecting](#connecting) state for details.

### Disconnecting

This state becomes active if there is a VPN tunnel active but the app decides to close said
tunnel. This state is active until the tunnel has been properly closed.

This state does not apply its own security policy on the firewall. It just keeps what was already
active. Usually all states transitioning into this state, and all states this state later
transitions into, have their own security policies. This state is just a short transition between
those, while the app waits for a running tunnel to come down and clean up after itself.

### Blocked

This state is only active when there is a problem/error. As described in other sections, the app
will never unlock the firewall and allow network traffic outside the tunnel unless a
disconnect/quit is explicitly requested by a user. At the same time there might be situations
when the app can't establish a tunnel for the device. This includes, but is not limited to:
* Account runs out of time,
* The computer is offline,
* the TAP adapter driver has an error or the adapter can't be found,
* Some internal error parsing or modifying system routing table, DNS settings etc.

In the above cases the app gives up trying to create a tunnel, but it can't go to the
[disconnecte](#disconnected) state, since it can't unlock the firewall. Then it enters this state.
This state locks the firewall so no traffic can flow (except the always active exceptions) and
informs the user what the problem is. The user must then explicitly click disconnect in order
to unlock the firewall and get access to the internet again.

## Kill switch

The app has an always on kill switch that can't be disabled. There is no setting for it.
This means that whenever the app changes server or temporarily loses tunnel connectivity it will
ensure no network traffic leaks out unencrypted.

We usually don't like the term "kill switch". Because it makes it sound like a big red button
that the VPN client pushes when it detects a problem. Maybe that's how the clients who coined
the term implemented it, but this app is much more proactive about it. This app applies
strict firewall rules directly when it leaves the [disconnected](#disconnected) state
and keeps those rules active and enforced until the app comes back to the disconnected state.
Said rules unsure that packets can only leave or enter the computer in a few predefined ways,
most notably to the selected VPN server IP and port. If the tunnel were to come down and your
operating system tries to route packets out via the normal network rather than through the VPN,
these rules would block them from leaving. So rather than failing open, meaning if the tunnel
fails your traffic leaves in other ways, we fail closed, meaning if the packets don't leave
encrypted in the way the app intends, then they can't leave at all.

### Block when disconnected

The "block when disconnected" setting in the app is regularly misunderstood as our kill switch.
This is not the case. The "block when disconnected" setting only changes whether or not the
[disconnected](#disconnected) state should allow traffic to flow freely or to block it. The
disconnected state is not active during intermittent network issues or server changes, when
a kill switch would normally be operating.

The intended use case for this setting is when the user want to only switch between no internet
connectivity at all and using VPN. With this setting active, the device can never communicate
with the internet outside of a VPN tunnel.

## Firewall

The states above should probably explain what can and can't be reached in the different states.
But we might need/want this section in case there is something that does not fit above.



## DNS

Where are DNS requests sent?
