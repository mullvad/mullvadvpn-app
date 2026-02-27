package net.mullvad.mullvadvpn.lib.model

import java.net.InetAddress
import java.net.InetSocketAddress
import java.time.Instant

sealed interface SetCustomVpnConfigError {
    object Unknown : SetCustomVpnConfigError
}

sealed interface GetCustomVpnConfigError {
    object Unknown : GetCustomVpnConfigError
}

data class CustomVpnConfig(val tunnelConfig: TunnelConfig, val peerConfig: PeerConfig)

data class TunnelConfig(val privateKey: String, val tunnelIp: InetAddress)

data class PeerConfig(val publicKey: String, val endpoint: InetSocketAddress, val allowedIp: String)

data class TunnelStats(
    val rx: Long = 0,
    val tx: Long = 0,
    val lastHandshake: Instant? = null,
)
