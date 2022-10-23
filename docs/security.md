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
the rules are applied as atomic transactions. This means that there is no time window of
inconsistent or invalid rules during changes.

On mobile, Android and iOS, it is not possible for apps to directly access and manipulate the
firewall, routing table or DNS settings. There we employ other techniques to keep the system as
secure as possible with the limitations of the OS APIs.

### Android

On Android, the only way an app can filter network traffic is essentially via the VPN service API.
This API allows all traffic, except some [exempt by the system](#exempt-traffic), to and from the
phone to flow though a third party app. This API is of course what the app uses for the tunnel
itself as well, but apart from that it is also what the leak protection is built on.

An app with permission to act as a VPN service can request to open a VPN tunnel on the device and
provide a set of IP networks it would like to have routed via itself. Doing so and specifying
the routes `0/0` and `::0/0` forces all traffic, except some
[exempt by the system](#exempt-traffic), to go via the app. That is what this app does both when it
has a VPN tunnel up, but also when in a state where it would like to block all network traffic. Such
as the [connecting], [disconnecting] and [error] states. In these states, all outgoing packets are
simply dropped, but incoming traffic is still allowed due to the limitations of Android.

#### Exempt traffic

Even though not being properly documented by Google, some traffic is exempt by the system from using
the VPN, which means that the traffic will leak and therefore potentially impact user privacy. This
applies even if *Block connections without VPN* is enabled. The exempt traffic includes:
* Connectivity checks (DNS lookups and HTTP(S) connections)
* Network provided time (NTP)

The following issues have been reported by Mullvad in the Android issue tracker in order to improve
documentation and user privacy:
* [Incorrect VPN lockdown documentation](https://issuetracker.google.com/issues/249990229)
* [Add option to disable connectivity checks when VPN lockdown is enabled](https://issuetracker.google.com/issues/250529027)

### iOS

On iOS a designated packet tunnel process handles the network packet flow. iOS implementation
delegates the traffic handling to wireguard-go, which works directly with the tun interface.
The network configuration set up by the packet tunnel extension specifies the routing rules
that all traffic should flow through the tunnel, the same way it works on Android.

The iOS app currently does not support blocking in the app's blocked state.

## App states

At the core of the app is a state machine called the "tunnel state machine". The following
sub-sections will describe each state and what security properties hold and what network activity
will be blocked and allowed in each state.

Except what is described as allowed in this document, all network packets should be blocked.

The following network traffic is allowed or blocked independent of state:

1. All traffic on loopback adapters is always allowed.

1. DHCPv4 and DHCPv6 requests are always allowed to go out and responses to come in:
   * Outgoing UDP from `*:68` to `255.255.255.255:67` (client to server)
   * Incoming UDP `*:67` to `*:68` (server to client)
   * Outgoing UDP from `[fe80::]/10:546` to `[ff02::1:2]:547` and `[ff05::1:3]:547` (client to
     server)
   * Incoming UDP from `[fe80::]/10:547` to `[fe80::]/10:546` (server to client)

1. A subset of NDP is allowed:
   * Outgoing to `ff02::2`, but only ICMPv6 with type 133 and code 0 (Router solicitation)
   * Incoming from `fe80::/10`, but only ICMPv6 type 134 and code 0 (Router advertisement)
   * Incoming from `fe80::/10`, but only ICMPv6 type 137 and code 0 (Redirect)
   * Outgoing to `ff02::1:ff00:0/104` and `fe80::/10`, but only ICMPv6 with type 135 and code 0 (Neighbor solicitation).
   * Incoming from `fe80::/10`, but only ICMPv6 with type 135 and code 0 (Neighbor solicitation).
   * Outgoing to `fe80::/10`, but only ICMPv6 with type 136 and code 0 (Neighbor advertisement).
   * Incoming from `*`, but only ICMPv6 with type 136 and code 0 (Neighbor advertisement).

1. If the "Allow LAN" setting is enabled, the following is also allowed:
   * Outgoing to, and incoming from, any IP in an unroutable network, that means:
     * `10.0.0.0/8`
     * `172.16.0.0/12`
     * `192.168.0.0/16`
     * `169.254.0.0/16` (Link-local IPv4 range)
     * `fe80::/10` (Link-local IPv6 range)
     * `fc00::/7` (Unique local address (ULA) range)
   * Outgoing to any IP in globally unroutable multicast networks, meaning these:
     * `224.0.0.0/24` (Local subnet IPv4 multicast)
     * `239.0.0.0/8` (Administratively scoped IPv4 multicast. E.g. SSDP and mDNS)
     * `255.255.255.255/32` (Broadcasts to the local network)
     * `ff01::/16` (Interface-local multicast. Local to a single interface on a node.)
     * `ff02::/16` (Link-local IPv6 multicast. IPv6 equivalent of `224.0.0.0/24`)
     * `ff03::/16` (Realm-local IPv6 multicast)
     * `ff04::/16` (Admin-local IPv6 multicast)
     * `ff05::/16` (Site-local IPv6 multicast)
   * Incoming DHCPv4 requests and outgoing responses (be a DHCPv4 server):
     * Incoming UDP from `*:68` to `255.255.255.255:67`
     * Outgoing UDP from `*:67` to `*:68`

#### Packet forwarding

On Linux, any situation that permits incoming or outgoing traffic also allows that traffic to be
forwarded. All other forward traffic is rejected.

#### Mullvad API

The firewall allows traffic to the API regardless of tunnel state, so the daemon is able to update
keys, fetch account data, etc. In the [Connected] state, API traffic is only allowed inside the tunnel.
For the other states, API traffic will bypass the firewall. On Windows, only the Mullvad service and
problem report tool are able to communicate with the API in any of the blocking states. On macOS and
Linux all applications runnning as root are able to reach the API in blocking states.

### Disconnected

This is the default state that the `mullvad-daemon` starts in when the device boots, unless
"Launch app on start-up" and "Auto-connect" are **both** active. Then the app will proceed to the
[connecting] state immediately.

The disconnected state behaves very differently depending on the value of the
"always require VPN" setting. If this setting is enabled, the disconnected state behaves
like and has the same security properties as, the [error] state. If the setting is
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
This IP+port+protocol combination should only be allowed for the process establishing the
VPN tunnel, or only administrator level processes, depending on what the platform firewall
allows restricting. On Windows the rule only allows processes from binaries in certain paths. macOS
the rule only allows packets from processes running as `root`. On Linux, the rule only allows
packets that have the mark `0x6d6f6c65` set: setting a firewall mark on traffic requires elevated
privileges when using tunnels that support marking traffic, otherwise the rule is the same as on
macOS: the packet needs to originate from a process running as `root`.
This process/user check is important to not allow unprivileged programs
to leak packets to this IP outside the tunnel, as those packets can be fingerprinted.

Examples:
1. No bridge is used and the tunnel protocol is OpenVPN trying to connect with UDP to a VPN
  server at IP `a.b.c.d` port `1301` - Allow traffic to `a.b.c.d:1301/UDP` for `openvpn.exe`
  or any process running as `root`, and incoming matching traffic.
1. Connecting to the same VPN server, but via a bridge. The bridge is at IP `e.f.g.h` and the
  proxy service listens on TCP port `443` - Allow traffic to `e.f.g.h:443/TCP` for
  `mullvad-daemon.exe` or any process running as `root`, and incoming matching
  traffic. Do not allow any direct communication with the VPN server.
1. Connecting to `a.b.c.d` port `1234` using WireGuard: Allow `a.b.c.d:1234/UDP` for
  `mullvad-daemon.exe` or any process running as `root`.

When using WireGuard, traffic inside the tunnel is permitted immediately after the tunnel device
has been created. See the [connected] state for details on this.

### Connected

This state becomes active when [connecting] has fully established a VPN tunnel. It
stays active until the user requests a disconnect, quit, server change, change of other setting
that affects the tunnel or until the tunnel goes down unexpectedly.

In this state, all traffic in both directions over the tunnel interface is allowed. Minus DNS
requests (TCP and UDP destination port 53) not to a gateway IP on the tunnel interface or
one of the defined custom DNS servers.
We can *only* request DNS inside the tunnel and *only* from the relay server itself,
unless one or more custom DNS servers are provided. If custom servers are specified, DNS requests
can only be made to them.

This state allows traffic on all interfaces to and from the IP+port+protocol combination that
the tunnel runs over. See the [connecting] state for details on this rule.

### Disconnecting

This state becomes active if there is a VPN tunnel active but the app decides to close said
tunnel. This state is active until the tunnel has been properly closed.

This state does not apply its own security policy on the firewall. It just keeps what was already
active. All states transitioning into this state, and all states this state later
transitions to, have their own security policies. This state is just a short transition between
those, while the app waits for a running tunnel to come down and clean up after itself.

### Error

This state is only active when there is a problem/error. As described in other sections, the app
will never unlock the firewall and allow network traffic outside the tunnel unless a
disconnect/quit is explicitly requested by the user. At the same time there might be situations
when the app can't establish a tunnel for the device. This includes, but is not limited to:
* Account runs out of time
* The computer is offline
* Some internal error parsing or modifying system routing table, DNS settings etc.

In the above cases the app gives up trying to create a tunnel, but it can't go to the
[disconnected] state, since it should not unlock the firewall. Then it enters this state.
This state locks the firewall so no traffic can flow (except the always active exceptions) and
informs the user what the problem is. The user must then explicitly click disconnect in order
to unlock the firewall and get access to the internet again.

If the firewall integration fails, so this state fails to block traffic. Then it is not much
left the app can do to prevent leaks. It then informs the user of the seriousness of the
situation.

## Kill switch

The app has an always on "kill switch" that can't be disabled. There is no setting for it.
This means that whenever the app changes server or temporarily loses tunnel connectivity it will
ensure no network traffic leaks out unencrypted.

The app avoids the term "kill switch". Because it sounds like a red button
that has to be *engaged when a problem arises*. This app is much more proactive and applies
[strict firewall rules](#app-states) directly when it leaves the [disconnected]
state and keeps those rules active and enforced until the app comes back to the [disconnected]
state via an explicit user request again. Said strict firewall rules ensure that packets can only
leave or enter the computer in a few predefined ways, most notably to the selected VPN server of
course. Changes to the firewall are done in atomic transactions. Meaning there is no time window
where no or invalid rules are active on the device.

If the tunnel were to come down and your operating system tries to route packets out via the
normal network rather than through the VPN, these rules would block them from leaving.
So rather than failing open, meaning if the tunnel fails your traffic leaves in other ways,
we fail closed, meaning if the packets don't leave encrypted in the way the app intends,
then they can't leave at all.

Essentially, one can say that the app's "kill switch" is the fact that the [connecting],
[disconnecting] and [error] states prevent leaks via firewall rules.

### Always require VPN

The "always require VPN" setting in the app is regularly misunderstood as the kill switch.
This is not the case. The "always require VPN" setting only changes whether or not the
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

By default, the app makes sure that every DNS request from the device goes inside the VPN tunnel
and only to the VPN relay server that the device is currently connected to. If custom DNS servers
are provided, requests are always made inside the tunnel unless the address belongs to a private
address range (such as 192.168.0.0/16) or a loopback address.

The above holds during the [connected] state. In the [disconnected]
state the app does nothing with DNS, meaning the default one is used, probably from the ISP.
In the other states DNS is simply blocked.


## Desktop system service

On all desktop platforms the VPN tunnel and the device security is handled by a system
service called `mullvad-daemon`. This service is installed as the administrator/root user
during app install and is then always running in the background, even when the user
quits the GUI and when no tunnels are running.

This system service can be controlled via a management interface, exposed locally
via unix domain sockets (UDS) on Linux and macOS and via named pipes on Windows.
This management interface can be reached by any process running on the device.
Locally running malicious programs are outside of the app's threat model.

The `mullvad-daemon` transition to the [disconnected] state before exiting. To
limit leaks during computer shutdown, it will maintain the blocking firewall
rules upon exit in the following scenarios:
- _Always require VPN_ is enabled
- A user didn't explicitly request for the `mullvad-daemon` to be shut down and
  either or both of the following are true
    - The daemon is currently in one of the blocking states ([connected],
      [connecting], or [error])
    - _auto-connect_ is enabled

In other cases, when the daemon process stops normally, firewall rules will be
removed.


### Windows

On Windows, persistent firewall filters may be added when the service exits, in case the service
decides to continue to enforce a blocking policy. These filters block any traffic occurring before
the service has started back up again during boot, including before the BFE service has started.

As with "Always require VPN", enabling "Auto-connect" in the service will cause it to
enforce the blocking policy before being stopped.

### Linux

Due to the dependence on various other services, the `mullvad-daemon` is not
started early enough to prevent leaks. To prevent this, another system unit is
started during early boot that applies a blocking policy that persists until the
`mullvad-daemon` is started.


### macOS

Due to the inability to specify dependencies of system services in `launchd` there is no way to
ensure that our daemon is started before any other service  or program is started.  Thus, whilst our
daemon will start as soon as it possibly can, there's nothing that can be done about the order in
which launch daemons get started, so some leaks may still occur.

## Desktop Electron GUI

The graphical frontend for the app on desktop is an Electron app. This app only ever loads
local resources in the form of html, CSS and Javascript directly from the installation
directory of the app, and never from remote sources.

The GUI only communicates with the system service (`mullvad-daemon`), it makes no other
network connections. Except when the user sends a problem report, then it spawn the
`mullvad-problem-report` tool, which in turn communicate over TLS with our API.


[disconnected]: #disconnected
[connecting]: #connecting
[connected]: #connected
[disconnecting]: #disconnecting
[error]: #error
[GUI]: #desktop-electron-gui
