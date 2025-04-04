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
    @SerialName("block_wireguard") val blockWireGuard: Boolean = false,
    val protocols: List<NetworkingProtocol>,
    @EncodeDefault val label: String = "urn:uuid:${SessionIdentifier.fromDeviceIdentifier()}",
) {
    companion object {
        fun blockUDPTrafficRule(to: String): DropRule {
            val testDeviceIpAddress = Networking.getDeviceIpv4Address()
            return DropRule(
                source = testDeviceIpAddress,
                destination = to,
                protocols = listOf(NetworkingProtocol.UDP),
            )
        }

        fun blockWireGuardTrafficRule(to: String): DropRule =
            blockUDPTrafficRule(to).copy(blockWireGuard = true)
    }
}
