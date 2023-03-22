package net.mullvad.mullvadvpn.relaylist

internal object RelayNameComparator : Comparator<Relay> {
    override fun compare(o1: Relay, o2: Relay): Int {
        val partitions1 = o1.name.split(regex)
        val partitions2 = o2.name.split(regex)
        return if (partitions1.size > partitions2.size) partitions1 compareWith partitions2
        else -(partitions2 compareWith partitions1)
    }

    private infix fun List<String>.compareWith(other: List<String>): Int {
        this.forEachIndexed { index, s ->
            if (other.size <= index) return 1
            val partsCompareResult = compareStringOrInt(other[index], s)
            if (partsCompareResult != 0) return partsCompareResult
        }
        return 0
    }

    private fun compareStringOrInt(s1: String, s2: String): Int {
        val int1 = s1.toIntOrNull()
        val int2 = s2.toIntOrNull()
        return if (int1 == null || int2 == null || int1 == int2) {
            s2.compareTo(s1)
        } else {
            int2.compareTo(int1)
        }
    }

    private val regex = "(?<=\\d)(?=\\D)|(?<=\\D)(?=\\d)".toRegex()
}
