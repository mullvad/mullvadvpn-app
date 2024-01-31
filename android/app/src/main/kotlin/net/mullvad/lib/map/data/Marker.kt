package net.mullvad.lib.map.data

data class Marker(val coordinate: Coordinate, val type: MarkerType)

enum class MarkerType {
    SECURE,
    UNSECURE
}
