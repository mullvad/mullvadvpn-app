#IPv6 connections to relays
We want to allow users to connect over IPv6 in the iOS app. To do this, the
following components need changing:

##General changes
- Settings structure must change to allow for saving a setting for IPv6, whether the app should automatically choose IP
  version, or use IPv4 or IPv6 exclusively. - doneish
- The relay selector must use the setting to select relays that have an IPv6 address for a given obfuscation protocol. -
  doneish
- Change selectedRelay/MullvadEnpoint to not hold both IPv4 and IPv6 - pick one and stick with it. Also consider moving
  to a different type - maybe we don't need to pass it down everywhere - a new type might make a lot more sense.
- Change relay selector to pick obfuscation and port far sooner in the process. Or at least produce candidates that can
  be culled later far sooner. Maybe an M:N model, where for ever relay, there's N possible connection modes, where
  connection mode is {
    address: [AddressRange]
    port: [PortRange]
    obfuscation: { .none, .quic, .shadowsocks, .lwo, .udp2tcp }
  } - we will need this anyway later when connecting via may connection modes at once


##UI changes
- The user needs to be able to change that settings value in the VPN settings view. - doneish
- When connecting via IPv6, the IPv6 feature indicator should be used - doneish

##Packet tunnel changes
- If no obfuscation is used, the packet tunnel should still use an IPv6 entry address
- IPv6 should only be used to reach the entry relay in multihop cases, exit config must never use IPv6
- If the IP version is left to the `Automatic` setting, the app should default to just using IPv4 all the time.

#Testing
An IPv6 end-to-end test must be added to RelayTests.


## Work done so far
- Connections work in all manners

## Work not done so fwmark
QUIC IPv6 only works with extra IPs, so have to filter for that in relay selector.

Relay selector is a bit messy, should be reworked.

Tests bork


