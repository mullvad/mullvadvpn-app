package net.mullvad.mullvadvpn.lib.model

import arrow.optics.optics
import java.net.InetAddress

@optics
data class CustomDnsOptions(val addresses: List<InetAddress>) {
    companion object
}
