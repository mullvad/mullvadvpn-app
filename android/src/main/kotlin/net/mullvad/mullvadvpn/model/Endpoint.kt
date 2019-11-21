package net.mullvad.mullvadvpn.model

import java.net.InetSocketAddress
import net.mullvad.talpid.net.TransportProtocol

data class Endpoint(val address: InetSocketAddress, val protocol: TransportProtocol)
