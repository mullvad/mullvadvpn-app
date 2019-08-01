package net.mullvad.mullvadvpn.model

import java.net.InetSocketAddress

data class Endpoint(val address: InetSocketAddress, val protocol: TransportProtocol)
