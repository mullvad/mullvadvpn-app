package net.mullvad.mullvadvpn.constant

import net.mullvad.mullvadvpn.lib.model.Port

val WIREGUARD_PRESET_PORTS = listOf(Port(51820), Port(53))
val UDP2TCP_PRESET_PORTS = listOf(Port(80), Port(5001))
val SHADOWSOCKS_PRESET_PORTS = listOf(Port(80), Port(443))
