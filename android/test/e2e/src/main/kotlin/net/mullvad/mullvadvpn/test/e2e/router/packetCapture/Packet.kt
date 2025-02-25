package net.mullvad.mullvadvpn.test.e2e.router.packetCapture

import java.time.ZonedDateTime
import kotlinx.serialization.Contextual
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.test.e2e.serializer.PacketSerializer

@Serializable(with = PacketSerializer::class)
sealed interface Packet {
    @SerialName("timestamp") val date: ZonedDateTime
    val fromPeer: Boolean
}

@Serializable
data class RxPacket(@SerialName("timestamp") @Contextual override val date: ZonedDateTime) :
    Packet {
    @SerialName("from_peer") override val fromPeer: Boolean = false
}

@Serializable
data class TxPacket(@SerialName("timestamp") @Contextual override val date: ZonedDateTime) :
    Packet {
    @SerialName("from_peer") override val fromPeer: Boolean = true
}
