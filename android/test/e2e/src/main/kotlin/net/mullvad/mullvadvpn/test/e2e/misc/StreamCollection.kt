package net.mullvad.mullvadvpn.test.e2e.misc

import java.util.Date
import org.junit.jupiter.api.fail

enum class LeakStatus {
    UNKNOWN,
    LEAK,
    NOLEAK
}

class StreamCollection {
    private var streams: List<Stream> = emptyList()

    constructor(streams: List<Stream>) {
        this.streams = streams
    }

    fun exportStreamCollectionFrom(startDate: Date, endDate: Date): List<Stream> {
        return streams.filter {
            it.startDate in startDate..endDate && it.endDate in startDate..endDate
        }
    }

    fun getConnectedThroughRelayStartEndDate(relayIpAddress: String): Pair<Date, Date> {
        val matchingStreams =
            streams.filter {
                it.destinationAddress == relayIpAddress &&
                    it.transportProtocol == NetworkTransportProtocol.UDP
            }
        var startDate: Date? = null
        var endDate: Date? = null

        if (matchingStreams.isEmpty()) {
            fail("Unexpectedly found no matching streams")
        }

        for (stream in matchingStreams) {
            val matchingPackets = stream.packets.filter { it.fromPeer }.sortedBy { it.date }

            val firstMatchingPacket = matchingPackets.first()
            val lastMatchingPacket = matchingPackets.last()

            if (startDate == null) {
                startDate = firstMatchingPacket.date
            } else {
                if (firstMatchingPacket.date < startDate) {
                    startDate = firstMatchingPacket.date
                }
            }

            if (endDate == null) {
                endDate = lastMatchingPacket.date
            } else {
                if (lastMatchingPacket.date > endDate) {
                    endDate = lastMatchingPacket.date
                }
            }
        }

        if (startDate == null || endDate == null) {
            fail("Unexpectedly found no start and/or end date for UDP communication")
        } else {
            return Pair(startDate, endDate)
        }
    }

    fun extractStreamCollection(from: Date, to: Date): StreamCollection {
        val streamsWithinInterval = streams.map { stream ->
            val packetsWithinInterval = stream.packets.filter {
                it.date in from..to
            }

            Stream(
                stream.sourceAddress,
                stream.destinationAddress,
                stream.flowId,
                stream.transportProtocol,
                packetsWithinInterval
            )
        }

        return StreamCollection(streamsWithinInterval)
    }

    fun dontAllowTrafficFromTestDevice(toHost: String) {
        streams.forEach { stream ->
            stream.packets.forEach { packet ->
                if (packet.fromPeer && stream.destinationAddress == toHost) {
                    packet.leakStatus = LeakStatus.LEAK
                }
            }
        }
    }

    fun verifyDontHaveLeaks(): Boolean {
        return streams.all { stream ->
            stream.packets.all { packet ->
                packet.leakStatus != LeakStatus.LEAK
            }
        }
    }

    fun getLeakCount(): Number {
        return streams.sumOf { stream->
            stream.packets.count { packet ->
                packet.leakStatus == LeakStatus.LEAK
            }
        }
    }
}
