package net.mullvad.mullvadvpn.lib.model

data class ConnectionPath(
    val offlineLocation: LatLong? = null,
    val entry: LatLong? = null,
    val exit: LatLong? = null,
) {
    fun toHops(): List<Pair<LatLong, LatLong>> =
        buildList {
                if (offlineLocation != null) {
                    add(offlineLocation)
                }
                if (entry != null) {
                    add(entry)
                }
                if (exit != null) {
                    add(exit)
                }
            }
            .zipWithNext()
}
