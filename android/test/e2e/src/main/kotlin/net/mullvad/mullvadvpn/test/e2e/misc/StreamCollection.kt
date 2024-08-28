package net.mullvad.mullvadvpn.test.e2e.misc

import java.util.Date

class StreamCollection {
    private var streams: List<Stream> = emptyList()

    constructor(streams: List<Stream>) {
        this.streams = streams
    }

    fun exportStreamCollectionFrom(startDate: Date, endDate: Date): List<Stream> {
        return streams.filter { it.startDate in startDate..endDate && it.endDate in startDate..endDate }
    }
}
