package net.mullvad.mullvadvpn.test.e2e.router.packetCapture

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

    @Transient private val startDate: DateTime = packets.first().date
    @Transient private val endDate: DateTime = packets.last().date
    @Transient private val txStartDate: DateTime? = txPackets().firstOrNull()?.date
    @Transient private val txEndDate: DateTime? = txPackets().lastOrNull()?.date
    @Transient private val rxStartDate: DateTime? = rxPackets().firstOrNull()?.date
    @Transient private val rxEndDate: DateTime? = rxPackets().lastOrNull()?.date

    @Transient val interval = Interval(startDate, endDate)

    fun txPackets(): List<TxPacket> = packets.filterIsInstance<TxPacket>()

    fun rxPackets(): List<RxPacket> = packets.filterIsInstance<RxPacket>()

    fun txInterval(): Interval? =
        if (txStartDate != null && txEndDate != null) Interval(txStartDate, txEndDate) else null

    fun rxInterval(): Interval? =
        if (rxStartDate != null && rxEndDate != null) Interval(rxStartDate, rxEndDate) else null

    init {
        require(packets.isNotEmpty()) { "Stream must contain at least one packet" }
    }
}
