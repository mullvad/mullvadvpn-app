package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
data class DnsOptions(
    val state: DnsState,
    val defaultOptions: DefaultDnsOptions,
    val customOptions: CustomDnsOptions
) {
    companion object
}
