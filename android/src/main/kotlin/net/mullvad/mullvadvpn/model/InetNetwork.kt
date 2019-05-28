package net.mullvad.mullvadvpn.model

import java.net.InetAddress

data class InetNetwork(val address: InetAddress, val prefixLength: Short)
