package net.mullvad.mullvadvpn.test.e2e.router.packetCapture

import java.time.ZonedDateTime
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.test.e2e.serializer.NanoSecondsTimestampSerializer
import net.mullvad.mullvadvpn.test.e2e.serializer.PacketSerializer

@Serializable(with = PacketSerializer::class)
sealed interface Packet {
    @SerialName("timestamp") val date: ZonedDateTime
    val fromPeer: Boolean
}

@Serializable
data class RxPacket(
    @Serializable(with = NanoSecondsTimestampSerializer::class)
    @SerialName("timestamp")
    override val date: ZonedDateTime
) : Packet {
    @SerialName("from_peer") override val fromPeer: Boolean = false
}

@Serializable
data class TxPacket(
    @Serializable(with = NanoSecondsTimestampSerializer::class)
    @SerialName("timestamp")
    override val date: ZonedDateTime
) : Packet {
    @SerialName("from_peer") override val fromPeer: Boolean = true
}
