package net.mullvad.mullvadvpn.constant

import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange

val WIREGUARD_PRESET_PORTS = listOf(Port(51820), Port(53))
val UDP2TCP_PRESET_PORTS = listOf(Port(80), Port(5001))
val SHADOWSOCKS_PRESET_PORTS = emptyList<Port>()
val SHADOWSOCKS_AVAILABLE_PORTS =
    listOf(PortRange(IntRange(0, 65535))) // Currently we consider all ports to be available
