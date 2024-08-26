package net.mullvad.mullvadvpn.test.e2e.util

import net.mullvad.mullvadvpn.test.e2e.model.Stream
import org.joda.time.Interval

fun splitStreamList(streamList: List<Stream>, interval: Interval): List<Stream> {
    return streamList.filter { interval.contains(it.interval) }
}
