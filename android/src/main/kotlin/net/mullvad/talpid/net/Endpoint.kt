package net.mullvad.talpid.net

import java.net.InetSocketAddress

data class Endpoint(val address: InetSocketAddress, val protocol: TransportProtocol)
