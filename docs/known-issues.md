# Known issues

A collection of known security and privacy issues currently affecting the Mullvad VPN app.

This is not a bug tracker. This is not a collection of post mortems. This is not a historical
record of past issues. This is not a list of issues we plan on solving soon.
This document is for listing issues affecting the app, that cannot be fixed or that we have
decided to not address for some reason. Some example reasons why issues might end up here is:

* The issue is caused by bugs in the operating system, that the app for some reason cannot
  provide a mitigation for
* The only known fixes for the issue comes with other drawbacks, that we consider as bad, or worse
  than the original issue
* We are not able to reliably reproduce the issue. Enough anecdotal evidence exist to indicate
  the issue is real, but Mullvad is unable to reproduce it. As a result, it is really hard to fix.

This document should only contain issues related to security and privacy. This document is a
compliment to the [security documentation](./security.md). Where the security documentation
is a more or less static description of the apps threat model and how the app implements its
security mechanisms, this known issues document is a more dynamic document, describing the current
deviations from said security document.

The goal and motivation for this document is to provide:

* Transparency to our users about shortcomings and problems with the app. For significant
  issues, we will most likely blog about it also. But this is a more
  permanent record, whereas a blog is forgotten fairly quickly
* A resource for developers to find information about known issues and why they are there
  and what is known about it
* A resource for external security auditors. Makes them avoid investigating problems we are
  already aware of and have documented

## Format of issues

Each issue in this document should provide, at least, the following:

* A description of the issue and how a user might be affected by it
* A timeline of events. When it was first discovered and any updates on the issue
* What app versions, platforms and operating systems are affected
* Links to external resources about the issue if there are any.
  Such as upstream bug reports to OS vendors, Mullvad blog posts etc.

## Issues

### Potential leaks just after macOS boot

Due to the inability to specify dependencies of system services in macOS `launchd` there is no way
to ensure that our `mullvad-daemon` is started before any other service or program.
This means that traffic from both system and user programs can potentially leak for a short
period of time after the computer has started up, even if the app has been configured to launch
on start-up and auto-connect.

This affects all app versions and as far as we know all versions of macOS.

There is no good fix or mitigation we know of that we can add to the app for this.
But some things user can do, depending on their threat model:

* Disable the network before shutting the computer down, so it starts up without network.
  This allows `mullvad-daemon` to start before any program has had any chance to leak.
* Do not start any program generating sensitive network traffic until you have verified
  Mullvad is running and has secured the connection.

#### Timeline

* September, 2022 - Mullvad engineers discover leaks during bootup on both Linux and macOS.
  this is discussed during [the audit](../audits/2022-10-14-atredis.md) that takes places just
  after this.
* October, 2022 - This leak is disclosed as part of our audit report and accompanying [blog post].

[blog post]: https://mullvad.net/blog/security-audit-report-for-our-app-available


### iOS is vulnerable to TunnelVision/TunnelCrack LocalNet

We have determined that from a security and privacy standpoint, in relation to the Mullvad VPN
app, TunnelVision (CVE-2024-3661) and TunnelCrack LocalNet (CVE-2023-36672 and CVE-2023-35838)
are virtually identical.

The Mullvad VPN iOS app is unfortunately vulnerable to these attacks. The only solution we know
against these leaks on iOS is to enable a flag called `includeAllNetworks` in iOS VPN terminology.
This flag has not been compatible with our app, so we have not been able to turn it on. But
work is being done in order to change the app so `includeAllNetworks` can be used, and this
attack can be mitigated.

This affects all versions of the iOS app on all versions of iOS.

#### Timeline

* August 9, 2023 - Mullvad [blog about TunnelCrack]
* May 7, 2024 - Mullvad [blog about TunnelVision]

[blog about TunnelCrack]: https://mullvad.net/blog/response-to-tunnelcrack-vulnerability-disclosure
[blog about TunnelVision]: https://mullvad.net/blog/evaluating-the-impact-of-tunnelvision


### DNS requests for excluded applications can go inside the tunnel

Ideally DNS requests from excluded apps would always go outside the tunnel. However, this
is not really possible, or hard to implement on some operating systems. See the
[split tunneling documentation] for details.

[split tunneling documentation]: ./split-tunneling.md#dns


### Temporary leaks while tunnel is being reconfigured on Android

Android may leak for a short period of time while a VPN tunnel is being reconfigured
(reconnecting, force-stopped etc), sending traffic outside the tunnel that is supposed to be inside
the tunnel. Packets sent may have the source IP of the internal tunnel interface. Some of these
leaks can happen even when the system setting "Block connections without VPN" is enabled.

The known leaks include, but may not be limited to, the following type of traffic:
- Any traffic sent by the current VPN app (e.g API requests).
- DNS lookups performed directly with the C function `getaddrinfo`.
- Private DNS traffic (e.g DNS-over-TLS).
- [OS connectivity checks](https://issuetracker.google.com/issues/250529027).

Multiple reports with variants of this behaviour have surfaced over the years, however the problems
still persist. Mullvad is not aware of any mitigation to these leaks.

- [A few packets leak to the public network at VPN reconnection](https://issuetracker.google.com/issues/37343051)
- [Android's VPN does not provide a seamless routing transition across VPN reconfigurations.](https://issuetracker.google.com/issues/117288570)
- [Android 10 Private DNS breaks VPN](https://issuetracker.google.com/issues/141674015)
- [Packets leak to the public network when VPN reconnection using seamless handover](https://issuetracker.google.com/issues/172141171)
- [VPN leaks DNS traffic outside the tunnel](https://issuetracker.google.com/issues/337961996)

#### Timeline

* April 22, 2024 - Mullvad became aware that Android could leak DNS when `getaddrinfo` was being used.
* April 30, 2024 - Mullvad [report the issue](https://issuetracker.google.com/issues/337961996) upstream to Google.
* May 3, 2024 - Mullvad [blog](https://mullvad.net/blog/dns-traffic-can-leak-outside-the-vpn-tunnel-on-android) about the findings. This post contains more details.
* Mar 12, 2025 - Mullvad realize the leaks are about much more than just DNS. This document is updated accordingly.


### Broadcast traffic to the LAN bypass the VPN on Android

A longstanding issue in Android makes it so that broadcast and multicast traffic to the local
network that the device is on, bypasses the VPN and is sent outside the tunnel.

This has been known for a long time, but has not been fixed in Android. Mullvad is not aware of
any way that a VPN app can mitigate this issue. It has to be solved upstream in Android.

#### Timeline

* December 18, 2019 - Someone [reports the issue to google](https://issuetracker.google.com/issues/146484540)


### Possible leaks on macOS on first start after upgrade

We have found that traffic could be leaking on macOS after system updates. In this scenario the
macOS firewall does not seem to function correctly and is disregarding firewall rules.
Most traffic will still go inside the VPN tunnel since the routing table specifies that it should.
Unfortunately apps are not required to respect the routing table and can send traffic outside
the tunnel if they try to.

Some examples of apps that can leak are Apple’s own apps and services between macOS 14.6,
up until a macOS 15.1 beta. This can also affect any other app that explicitly bind its sockets
directly to the local network interface.

To our current knowledge a reboot resolves the issue. We have only observed this behavior
sporadically, on the first start after a system upgrade. Since this is hard to reproduce
we have not been able to locate the source of the issue, and as a result not figured out
any mitigation neither.

Since this seems to be an operating system bug, it affects all versions of the Mullvad VPN
app. We have observed it on macOS 14.6 and newer, but it could very well have existed much earlier.

#### Timeline

* September 30, 2024 - Mullvad observe this behavior internally after a macOS upgrade
* October 16, 2024 - Mullvad report the issue upstream to Apple. No public issue tracker is available
* October 16, 2024 - Mullvad [blog](https://mullvad.net/blog/macos-sometimes-leaks-traffic-after-system-updates)
  about the finding


### Hyper-V virtual networking cause leaks on Windows

The Hyper-V Virtual Ethernet Adapter passes traffic to and from guests without letting the
host’s firewall inspect the packets in the same way normal packets are inspected.
The forwarded (NATed) packets are seen in the lower layers of WFP (OSI layer 2) as
Ethernet frames only. This means that all the normal firewall rules inserted by the Mullvad app
to stop leaks are circumvented.

This problem affects all virtual machines, containers and software running on a Hyper-V virtual
network.

The app mitigates the issue by blocking all Hyper-V traffic in secured states using Hyper-V-specific
filters, i.e. a firewall that applies specifically to the Hyper-V hypervisor. The connected state is
exempted since the routing table will ensure that traffic is tunneled in that case, at least for WSL
(see details below).

There are certain limitations to this mitigation. First, the Hyper-V firewall is only available on
*Windows 11 version 22H2 and above*, so it has no effect on earlier versions of Windows.
Additionally, LAN traffic will never be blocked while connected, regardless of whether "Local
network sharing" is enabled. Moreover, DNS leaks are more likely to occur.

Your [WSL config] needs to enable the `firewall` setting for the Hyper-V firewall to be enabled.
It is enabled by default.

#### Linux under WSL2

Network traffic from a Linux guest running under WSL2 always goes out the default route of
the host machine without being inspected by the normal layers of WFP (the firewall on the
Windows host that Mullvad use to prevent leaks). This means that if there is a VPN tunnel
up and running, the Linux guest’s traffic will be sent via the VPN with no leaks!
In the other states, the mitigation above is used to prevent leaks.

#### Windows Sandbox

The Hyper-V-specific firewall rules unfortunately do not apply to traffic from Windows Sandbox.
As a consequence, traffic from Windows Sandbox will leak during secured states, e.g. while switching
servers, or when disconnected with "lockdown mode" enabled. Another known issue with Windows Sandbox
is that DNS traffic is blocked when connected on the host OS, unless local network sharing is
enabled.

We suggest installing and running the VPN inside Windows Sandbox, which solves both the issues of
leaks and DNS. It is possible to also run an instance of the VPN on the host without any conflicts,
as long as there are enough devices available on the account.

#### Edge using Application guard

When running the Microsoft Edge browser with Microsoft Defender Application Guard activated,
the browser uses Hyper-V networking underneath. This makes the network traffic generated
by the browser ignore the Mullvad firewall rules. On top of this, it even ignores the routing
table, and *always* send the traffic directly on the physical network interface
instead of the tunnel interface. Hence, the mitigation above is ineffective when the VPN tunnel is
active.

This affects all app versions and all versions of Edge on Application Guard as far as we know.
Since [Application Guard is deprecated] we are not going to put much effort into solving this.
We recommend users to not use Application Guard.

[Application Guard is deprecated]: https://learn.microsoft.com/en-us/deployedge/microsoft-edge-security-windows-defender-application-guard
[WSL config]: https://learn.microsoft.com/en-us/windows/wsl/wsl-config#main-wsl-settings

#### Other VPN software

We have tested a few other VPN clients from competitors and found that all of them leak in
the same way. Therefore, this is not a problem with Mullvad VPN specifically, but rather an
industry-wide issue. The way Microsoft has implemented virtual networking guests makes
it very difficult to properly secure them.

#### Timeline

* August 12, 2020 - A user report the Linux under WSL2 leak to our support
* September 30, 2020 - Mullvad blog about [Linux under WSL2 leaking]
* May 15, 2024 - A user notify us that Edge under Application Guard cause leaks

[Linux under WSL2 leaking]: https://mullvad.net/en/blog/linux-under-wsl2-can-be-leaking


### Android exposes in-tunnel VPN IPs to network adjacent attackers via ARP
<a id="MLLVD-CR-24-03"></a>

By default the kernel parameter [`arp_ignore`] is set to `0` on Android. This makes the device reply
to ARP requests for any local target IP address, configured on any interface. This means that any
network adjacent attacker (same local network) can figure out the IP address configured on the VPN
tunnel interface by sending an ARP request for every private IPv4 address to the device.

This can be used by an adversary on the same local network to make a qualified guess if the device
is using Mullvad VPN. Furthermore, since the in-tunnel IP only changes monthly, the adversary can
also possibly identify a device over time.

Android apps, including Mullvad VPN, do not have the permission to change kernel parameters such as
`arp_ignore`. All Android devices that we know of are affected, as it is the default behavior of the
OS. We have reported this issue [upstream to Google], and recommended that they change the kernel
parameter to prevent the device from disclosing the VPN tunnel IP to the local network in this way.
See the report for more details.

We don't consider this a critical leak since the in-tunnel IP does not tell a great deal about the
user. However, users that are worried can log out and back in to the app, as this gives them a new
tunnel IP.

#### Timeline

* November 6, 2024 - Auditors from X41 D-Sec reported this issue as part of the [2024 app audit].
  The issue was given the identifier [`MLLVD-CR-24-03`].
* November 14, 2024 - We reported the issue [upstream to Google].

[`arp_ignore`]: https://www.kernel.org/doc/Documentation/networking/ip-sysctl.txt
[2024 app audit]: ../audits/2024-12-10-X41-D-Sec.md
[`MLLVD-CR-24-03`]: ../audits/2024-12-10-X41-D-Sec.md#MLLVD-CR-24-03
[upstream to Google]: https://issuetracker.google.com/issues/378814597
