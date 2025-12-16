package net.mullvad.mullvadvpn.test.e2e.router.firewall

import android.annotation.SuppressLint
import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.test.e2e.misc.Networking
import net.mullvad.mullvadvpn.test.e2e.router.NetworkingProtocol

@SuppressLint("HardwareIds")
@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class DropRule(
    @SerialName("src") val source: String,
    @SerialName("dst") val destination: String,
    val protocols: List<NetworkingProtocol>,
    @EncodeDefault val label: String = "urn:uuid:${SessionIdentifier.fromDeviceIdentifier()}",
    @SerialName("block_all_except_dst") val blockAllExceptDestination: Boolean = false,
) {
    companion object {

        fun blockUDPTrafficRule(toIpv4: String): List<DropRule> =
            blockTrafficToRule(protocols = listOf(NetworkingProtocol.UDP), toIpv4 = toIpv4)

        fun blockWireGuardTrafficRule(toIpv4: String): List<DropRule> =
            blockTrafficToRule(protocols = listOf(NetworkingProtocol.WireGuard), toIpv4 = toIpv4)

        fun blockAllTrafficExceptToDestinationRule(toIpv4: String): List<DropRule> =
            blockTrafficToRule(
                protocols = emptyList(),
                toIpv4 = toIpv4,
                blockAllExceptDestination = true,
            )

        private fun blockTrafficToRule(
            protocols: List<NetworkingProtocol>,
            toIpv4: String,
            blockAllExceptDestination: Boolean = false,
        ): List<DropRule> {
            val (sourceIpv4, sourceIpv6) = Networking.getDeviceIpAddrs()

            val ipv4Rule =
                DropRule(
                    source = sourceIpv4,
                    destination = toIpv4,
                    protocols = protocols,
                    blockAllExceptDestination = blockAllExceptDestination,
                )

            val ipv6Rules =
                sourceIpv6.map {
                    DropRule(
                        source = it.split('%')[0], // remove link-local suffix
                        destination = toIpv4,
                        protocols = protocols,
                        blockAllExceptDestination = blockAllExceptDestination,
                    )
                }

            return ipv6Rules + ipv4Rule
        }
    }
}
