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


### Temporary DNS leaks while tunnel is being reconfigured on Android

DNS lookups performed directly with the C function `getaddrinfo` can leak for a short period
of time while an android VPN app is being re-configured (reconnecting, force-stopped etc).
These leaks happens even when when the system setting "Block connections without VPN" is
enabled.

We have not found any leaks from apps that only use Android API:s such as [DnsResolver]. The Chrome browser is an example of an app that can use getaddrinfo [directly](https://source.chromium.org/chromium/chromium/src/+/main:android_webview/browser/aw_pac_processor.cc;l=197;drc=133b2d903fa57cfda1317bc589b349cf4c284b7c).

Mullvad is not aware of any mitigation to this leak. It has been reported upstream to Google,
and we wait for their response.

#### Timeline

* April 22, 2024 - We became aware of the leaks, via a [reddit post](https://www.reddit.com/r/mullvadvpn/comments/1c9p96y/dns_leak_with_block_connections_without_vpn_on/)
* April 30, 2024 - We [report the issue](https://issuetracker.google.com/issues/337961996) upstream to Google.
* May 3, 2024 - We [blog](https://mullvad.net/blog/dns-traffic-can-leak-outside-the-vpn-tunnel-on-android) about the findings. This post contains more details.

[DnsResolver]: https://developer.android.com/reference/android/net/DnsResolver


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

* October ????, 2024 - We observe this behavior by accident after a macOS upgrade
* October 16, 2024 - We report the issue upstream to Apple. No public issue tracker is available.
* October 16, 2024 - We [blog](https://mullvad.net/blog/macos-sometimes-leaks-traffic-after-system-updates) 
  about the finding
