# Mullvad VPN app security

This document describes the security properties of the Mullvad VPN app. It describes it for all
platforms and their differences. Individual platforms might have slightly different properties and
allow or block network traffic a bit differently, but all such deviations are described here.

This document does not describe in detail *how* we reach and uphold these properties, just what
they are. See the [architecture](architecture.md) document for details on how the firewall
integration is implemented.

The main purpose of the app is to allow the user to make all network/internet traffic to and
from the device travel via an encrypted VPN tunnel.

## Desktop vs mobile

For desktop operating systems, the security is ensured via tight integration with the default
system firewall. This means WFP on Windows, PF on macOS and nftables on Linux. All changes to
the rules are applied as atomic transactions. Meaning there is no time window of inconsistent or
invalid rules during changes.

On mobile, Android and iOS, it is not possible for apps to filter network traffic by manipulating
firewall rules. There we employ various other techniques to try to reach similar security
properties as on desktop.

### Android

On Android, the only way an app can filter network traffic is essentially via the VPN service API.
This API allows all traffic to and from the phone to flow though a third party app. This API is of
course what the app uses for the tunnel itself as well, but apart from that it is also what the
leak protection is built on.

An app with permission to act as a VPN service can request to open a VPN tunnel on the device and
provide a set of IP networks it would like to have routed via itself. Doing so and specifying
the routes `0/0` and `::0/0` forces all traffic to go via the app. That is what this app does both
when it has a VPN tunnel up, but also when in a state where it would like to block all network
traffic. Such as the [connecting], [disconnecting] and [blocked] states.

### iOS

TODO

## App states

At the core of the app is a state machine called the "tunnel state machine". The following
sub-sections will describe each state and what security properties hold and what network activity
will be blocked and allowed during them.

Except what is described as allowed in this document, all network packets should be blocked.

The following network traffic is allowed or blocked independent of state:

1. All traffic on loopback adapters is always allowed.

1. DHCPv4 and DHCPv6 requests are always allowed to go out and responses to come in:
   * Outgoing UDP from `*:68` to `255.255.255.255:67` (client to server)
   * Incoming UDP `*:67` to `*:68` (server to client)
   * Outgoing UDP from `[fe80::]/10:546` to `[ff02::1:2]:547` and `[ff05::1:3]:547` (client to
     server)
   * Incoming UDP from `[fe80::]/10:547` to `[fe80::]/10:546` (server to client)

1. Router solicitation, advertisement and redirects (subset of NDP) is always allowed:
   * Outgoing to `ff02::2`, but only ICMPv6 with type 133 and code 0 (Router solicitation)
   * Incoming from `[fe80::]/10`, but only ICMPv6 type 134 and code 0 (Router advertisement)
   * Incoming from `[fe80::]/10`, but only ICMPv6 type 137 and code 0 (Redirect)

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
     * Incoming UDP from `*:68` to `255.255.255.255:67`
     * Outgoing UDP from `*:67` to `*:68`

#### macOS deviations

* The app does not look at ICMPv6 type and code headers. So all ICMPv6 is allowed between the
  specified IP networks.

### Disconnected

This is the default state that the `mullvad-daemon` starts in when the device boots, unless
"Launch app on start-up" and "Auto-connect" are **both** active. Then the app will proceed to the
[connecting] state immediately.

The disconnected state behaves very differently depending on the value of the
"block when disconnected" setting. If this setting is enabled, the disconnected state behaves
like and has the same security properties as, the [blocked] state. If the setting is
disabled (the default), then it is the only state where the app does not enforce any firewall
rules. It then behaves the same as if the `mullvad-daemon` was not even running. It lets
network traffic flow in and out of the computer freely.

The disconnected state is not active while the app changes server or if the VPN tunnel goes down
unexpectedly. See the [connecting] state and [kill switch](#kill-switch) documentation for these
unexpected network issues. The only time this state is active is initially when the daemon
starts and later when the user explicitly clicks the disconnect/cancel button to intentionally
disable the VPN.

### Connecting

This state is active from when the app decides to create a VPN tunnel, until said tunnel has
been established and verified to work. Then it transitions to the [connected] state.

In this state, network traffic to the IP+port+protocol combination used for the first hop of the
VPN tunnel is allowed on all interfaces, together with responses to this outgoing traffic.
First hop means the bridge server if one is used, otherwise the VPN server directly.
Examples:
1. No bridge is used and the tunnel protocol is OpenVPN trying to connect with UDP to a VPN
  server at IP `a.b.c.d` port `1301` - Allow traffic to `a.b.c.d:1301/UDP` and incoming matching
  traffic.
1. Connecting to the same VPN server, but via a bridge. The bridge is at IP `e.f.g.h` and the
  proxy service listens on TCP port `443` - Allow traffic to `e.f.g.h:443/TCP` and incoming matching
  traffic. Do not allow any direct communication with the VPN server.

If connecting via WireGuard, this state allows ICMP packets to and from the in-tunnel IPs
(both v4 and v6) of the relay server the app is currently connecting to. That means the private
network IPs where the relay will respond inside the tunnel. It allows this on all interfaces,
since with the current architecture we don't know which network interface is the tunnel interface
at this point.

### Connected

This state becomes active when [connecting] has fully established a VPN tunnel. It
stays active until the user requests a disconnect, quit, server change, change of other setting
that affects the tunnel or until the tunnel goes down unexpectedly.

In this state, all traffic in both directions over the tunnel interface is allowed. Minus DNS
requests (TCP and UDP destination port 53) not to a gateway IP on the tunnel interface.
Meaning we can *only* request DNS inside the tunnel and *only* from the relay server itself.

This state allows traffic on all interfaces to and from the IP+port+protocol combination that
the tunnel runs over. See the [connecting] state for details on this rule.

### Disconnecting

This state becomes active if there is a VPN tunnel active but the app decides to close said
tunnel. This state is active until the tunnel has been properly closed.

This state does not apply its own security policy on the firewall. It just keeps what was already
active. All states transitioning into this state, and all states this state later
transitions to, have their own security policies. This state is just a short transition between
those, while the app waits for a running tunnel to come down and clean up after itself.

### Blocked

This state is only active when there is a problem/error. As described in other sections, the app
will never unlock the firewall and allow network traffic outside the tunnel unless a
disconnect/quit is explicitly requested by the user. At the same time there might be situations
when the app can't establish a tunnel for the device. This includes, but is not limited to:
* Account runs out of time
* The computer is offline
* the TAP adapter driver has an error or the adapter can't be found (Windows)
* Some internal error parsing or modifying system routing table, DNS settings etc.

In the above cases the app gives up trying to create a tunnel, but it can't go to the
[disconnected] state, since it should not unlock the firewall. Then it enters this state.
This state locks the firewall so no traffic can flow (except the always active exceptions) and
informs the user what the problem is. The user must then explicitly click disconnect in order
to unlock the firewall and get access to the internet again.

## Kill switch

The app has an always on kill switch that can't be disabled. There is no setting for it.
This means that whenever the app changes server or temporarily loses tunnel connectivity it will
ensure no network traffic leaks out unencrypted.

We usually don't like the term "kill switch". Because it makes it sound like a big red button
that the VPN client pushes when it detects a problem. This in turn gives the impression there
might be a time window of insecurity between when the problem occurs and the app manages to "push"
this virtual red button. Maybe that is how the clients who coined the term implemented it,
but this app is much more proactive about stopping leaks.
This app applies [strict firewall rules](#app-states) directly when it leaves the [disconnected]
state and keeps those rules active and enforced until the app comes back to the [disconnected]
state via an explicit user request again. Said strict firewall rules unsure that packets can only
leave or enter the computer in a few predefined ways, most notably to the selected VPN server of
course. Changes to the firewall are done in atomic transactions. Meaning there is no time window
where no or invalid rules are active on the device.

If the tunnel were to come down and your operating system tries to route packets out via the
normal network rather than through the VPN, these rules would block them from leaving.
So rather than failing open, meaning if the tunnel fails your traffic leaves in other ways,
we fail closed, meaning if the packets don't leave encrypted in the way the app intends,
then they can't leave at all.

Essentially, one can say that the app's "kill switch" is the fact that the [connecting],
[disconnecting] and [blocked] states prevent leaks via firewall rules.

### Block when disconnected

The "block when disconnected" setting in the app is regularly misunderstood as the kill switch.
This is not the case. The "block when disconnected" setting only changes whether or not the
[disconnected] state should allow traffic to flow freely or to block it. The
disconnected state is not active during intermittent network issues or server changes, when
a kill switch would normally be operating.

The intended use case for this setting is when the user want to only switch between no internet
connectivity at all and using VPN. With this setting active, the device can never communicate
with the internet outside of a VPN tunnel.

## DNS

DNS is treated a bit differently from other protocols. Since a user's DNS history can give a
detailed view of what they are doing, it is important to not leak it.
Since an invalid or missing DNS response prevents the user from going where they want to go,
it is important that it works and gives correct replies, from an anti-censorship point of view.
Poisoned DNS replies is a very common way of censoring the network in many places.

With the above as background, the app makes sure that every DNS request from the device goes
inside the VPN tunnel and to exactly one place, the VPN relay server the device is currently
connected to. That ensures the request reaches the Mullvad infrastructure and does so safely
(encrypted). From there the Mullvad servers are responsible for delivering a correct and
uncensored reply.

The above holds during the [connected] state. In the [disconnected]
state the app does nothing with DNS, meaning the default one is used, probably from the ISP.
In the other states DNS is simply blocked.

## Android


[disconnected]: #disconnected
[connecting]: #connecting
[connected]: #connected
[disconnecting]: #disconnecting
[blocked]: #blocked