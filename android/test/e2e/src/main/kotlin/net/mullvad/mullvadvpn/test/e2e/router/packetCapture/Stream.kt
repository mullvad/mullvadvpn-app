package net.mullvad.mullvadvpn.test.e2e.router.packetCapture

import java.time.Duration
import java.time.ZonedDateTime
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient
import net.mullvad.mullvadvpn.test.e2e.router.NetworkingProtocol

@Serializable
data class Stream(
    @SerialName("peer_addr") private val sourceAddressAndPort: String,
    @SerialName("other_addr") private val destinationAddressAndPort: String,
    @SerialName("flow_id") val flowId: String?,
    @SerialName("transport_protocol") val transportProtocol: NetworkingProtocol,
    val packets: List<Packet>,
) {
    @Transient val sourceHost = Host.fromString(sourceAddressAndPort)
    @Transient val destinationHost = Host.fromString(destinationAddressAndPort)

    @Transient private val startDate: ZonedDateTime = packets.first().date
    @Transient private val endDate: ZonedDateTime = packets.last().date
    @Transient private val txStartDate: ZonedDateTime? = txPackets().firstOrNull()?.date
    @Transient private val txEndDate: ZonedDateTime? = txPackets().lastOrNull()?.date
    @Transient private val rxStartDate: ZonedDateTime? = rxPackets().firstOrNull()?.date
    @Transient private val rxEndDate: ZonedDateTime? = rxPackets().lastOrNull()?.date

    @Transient val interval = Duration.between(startDate, endDate)

    fun txPackets(): List<TxPacket> = packets.filterIsInstance<TxPacket>()

    fun rxPackets(): List<RxPacket> = packets.filterIsInstance<RxPacket>()

    fun txInterval(): Duration? =
        if (txStartDate != null && txEndDate != null) Duration.between(txStartDate, txEndDate)
        else null

    fun rxInterval(): Duration? =
        if (rxStartDate != null && rxEndDate != null) Duration.between(rxStartDate, rxEndDate)
        else null

    init {
        require(packets.isNotEmpty()) { "Stream must contain at least one packet" }
    }
}
