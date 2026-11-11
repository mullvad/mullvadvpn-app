package net.mullvad.mullvadvpn.feature.customvpn.impl

import java.net.InetSocketAddress
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.PeerConfig
import net.mullvad.mullvadvpn.lib.model.TunnelConfig
import net.mullvad.mullvadvpn.lib.model.WireguardKey

const val VALID_NAME = "My Custom Tunnel"
const val VALID_PRIVATE_KEY = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
const val VALID_PUBLIC_KEY = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE="
const val VALID_TUNNEL_IP = "10.64.0.1"
const val VALID_ALLOWED_IP = "0.0.0.0/0"
const val VALID_ENDPOINT_HOST = "192.168.1.1"
const val VALID_ENDPOINT_PORT = 51820

fun createSampleCustomVpnConfig(enabled: Boolean = true): CustomVpnConfig =
    CustomVpnConfig(
        tunnelConfig =
            TunnelConfig(
                privateKey = WireguardKey.from(VALID_PRIVATE_KEY).getOrNull()!!,
                tunnelIps = setOf(VALID_TUNNEL_IP),
            ),
        peerConfig =
            listOf(
                PeerConfig(
                    publicKey = WireguardKey.from(VALID_PUBLIC_KEY).getOrNull()!!,
                    endpoint = InetSocketAddress(VALID_ENDPOINT_HOST, VALID_ENDPOINT_PORT),
                    allowedIps = setOf(VALID_ALLOWED_IP),
                )
            ),
        enabled = enabled,
        name = VALID_NAME,
    )
