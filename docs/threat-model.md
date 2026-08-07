# Mullvad VPN app threat model

This document defines what the Mullvad VPN app protects, whom it protects against, and what it
does not attempt to protect against. It names the assets worth protecting, the attackers the app
defends against, and the security properties that must hold as a result.

It is the source of truth for what the app is trying to achieve, and against whom. It keeps
technical design decisions consistent, tells auditors and reviewers what the app intends to do,
and gives anyone analysing the app a stated intent to check against.

This is the core of the threat model. It is deliberately compact and will be extended
incrementally. Sections that are planned but not yet written are listed under
[Planned extensions](#planned-extensions).

## How this document relates to the others

- **This document** - what is at stake, from whom, and what the app defends against.
- **[Security design specification](security.md)** - the concrete rules that follow: what traffic
  is allowed and blocked in each state on each platform, and how the app's components, privileges,
  interfaces and stored data are protected.
- **[Known issues](known-issues.md)** - where the app currently falls short of these properties
  and rules.
- **[Update mechanism threat model](../mullvad-update/threat-model.md)** - a separate model for
  downloading, verifying and installing app updates.
- **[Architecture](architecture.md)** - how the app is built.

Security properties here are stated as absolute invariants. They describe what the app must
achieve, not necessarily what today's implementation achieves: any violation is a bug - whether
the bug is in our code or in the operating system. A platform limitation that prevents reaching a
property does not weaken the property; it is recorded in the known issues document instead.
Defining properties down to the weakest platform would lower the bar for every platform.

## Scope

In scope is the app as shipped and running on a user's device: Windows, macOS, Linux, Android and
iOS. That includes the component that enforces the network security policy, the frontends, tunnel
establishment and operation, communication with Mullvad infrastructure, and what the app stores on
the device.

It also includes the cryptographic code Mullvad has written itself: our WireGuard implementation,
and the construction that combines post-quantum key exchange results into a WireGuard pre-shared
key together with the protocol that negotiates it. The update signature scheme is ours as well,
but is covered by the update mechanism threat model.

Mullvad's infrastructure is outside the boundary. This document does not model how relays or the
API are built and operated. It states what the app guarantees while they behave as expected, and
what a compromise of them could achieve.

Also outside:

- The update mechanism, which has its own threat model.
- Mullvad's development practices and supply chain.

One document covers all platforms. The threats are largely the same everywhere, but the defences
are not: where a platform makes a stronger position achievable, the app takes it. The clearest
case is software already installed on the device, which is an attacker on mobile and trusted on
desktop. Differences like that are stated where they occur.

## System overview

One component enforces the network security policy and manages tunnels. On desktop it is a system
service running as administrator or root, started at boot and running whether or not a user
interface is open. On Android it is the VPN service and on iOS the packet tunnel provider.

Users control that component through frontends: a graphical interface on every platform, plus a
command line interface on desktop. On desktop and Android the frontends reach it over a local
management interface. On iOS the app communicates with the packet tunnel provider through the
operating system's network extension framework instead.

Tunnels are WireGuard connections to Mullvad VPN relays, either to a single relay or through
multiple relays in sequence (Multihop). Anti-censorship measures can wrap the WireGuard
packets in another protocol.

Independently of tunnel traffic, the app communicates with the Mullvad API for account data,
device registration, relay lists and version information.

The device stores the app's settings, the account number and the device's WireGuard keys.

See the [architecture](architecture.md) document for details.

## Assumptions

The properties in this document rest on the following assumptions. If one of them is violated,
the guarantees derived from it do not hold.

**Cryptography.** The TLS 1.3 specification and rustls, the implementation the app uses, are
assumed sound. The WireGuard protocol specification is assumed sound, as are the third-party
WireGuard implementations in use. Cryptographic code Mullvad has written itself is not covered by
this assumption - it is in scope, as listed above.

**Quantum adversaries.** We assume no attacker currently possesses a cryptographically relevant
quantum computer, so the classical cryptography above holds today. Harvest-now-decrypt-later is an
explicit exception: an attacker may record encrypted tunnel traffic now and decrypt it once such a
computer exists. Quantum-resistant tunnels address that threat.

**Operating system.** The operating system is assumed not to be compromised or malicious. On a
compromised system no guarantee holds. This does not extend to operating system *defects*. Those
still cause bugs, and are recorded in known issues.

**Hardware.** Device hardware and firmware are assumed trusted. Hardware compromise voids all
guarantees, as operating system compromise does.

**Binary authenticity.** The running binary is assumed to be the one Mullvad built and signed.
Delivery and update integrity are covered by the update mechanism threat model. Build pipeline and
supply chain security are out of scope for this document.

**What is deliberately not assumed.** Mullvad's own infrastructure is *not* assumed
uncompromised. Compromise of a VPN relay or of the API is an in-scope threat, analysed under
[Trust boundaries](#trust-boundaries).

## REST OF THE THREAT MODEL

...

