package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class TunnelOptions(val wireguard: WireguardTunnelOptions, val dnsOptions: DnsOptions) {
    companion object
}
