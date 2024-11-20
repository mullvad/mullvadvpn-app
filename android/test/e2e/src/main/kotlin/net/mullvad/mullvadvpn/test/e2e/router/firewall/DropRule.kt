package net.mullvad.mullvadvpn.test.e2e.router.firewall

import android.annotation.SuppressLint
import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.test.e2e.misc.Networking

@SuppressLint("HardwareIds")
@OptIn(ExperimentalSerializationApi::class)
@Serializable
data class DropRule
constructor(
    @SerialName("src") val source: String,
    @SerialName("dst") val destination: String,
    val protocols: List<NetworkingProtocol>,
    @EncodeDefault
    val label: String = "urn:uuid:${SessionIdentifier.fromDeviceIdentifier()}",
) {
    companion object {
        fun blockUDPTrafficRule(to: String): DropRule {
            val testDeviceIpAddress = Networking.getDeviceIpv4Address()
            return DropRule(
                testDeviceIpAddress,
                to,
                listOf(NetworkingProtocol.UDP),
            )
        }
    }
}
