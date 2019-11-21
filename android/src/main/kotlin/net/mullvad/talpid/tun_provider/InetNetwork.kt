package net.mullvad.talpid.tun_provider

import java.net.InetAddress

data class InetNetwork(val address: InetAddress, val prefixLength: Short)
