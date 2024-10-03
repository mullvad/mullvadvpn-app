package net.mullvad.mullvadvpn.lib.daemon.grpc

import net.mullvad.mullvadvpn.lib.model.RelayItem

internal object RelayNameComparator : Comparator<RelayItem.Location.Relay> {
    override fun compare(o1: RelayItem.Location.Relay, o2: RelayItem.Location.Relay): Int {
        val partitions1 = o1.name.split(regex)
        val partitions2 = o2.name.split(regex)

        partitions1
            .zip(partitions2)
            .map { (p1, p2) -> compareStringOrInt(p1, p2) }
            .forEach {
                if (it != 0) {
                    // Parts differed, return compare result
                    return it
                }
            }
        return partitions1.size.compareTo(partitions2.size)
    }

    private fun compareStringOrInt(p1: String, p2: String): Int {
        val int1 = p1.toIntOrNull()
        val int2 = p2.toIntOrNull()
        return if (int1 is Int && int2 is Int) {
            // If both are Int we should compare them numbers
            int1.compareTo(int2)
        } else {
            p1.compareTo(p2)
        }
    }

    // Regexp that splits digit and non digit, e.g se-got-wg-101 would be ["se-got-wg-", "101"] so
    // that the number later can be sorted, e.g 9 being listed before 10.
    private val regex = "(?<=\\d)(?=\\D)|(?<=\\D)(?=\\d)".toRegex()
}
