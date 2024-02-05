package net.mullvad.lib.map.data

data class Marker(val latLng: LatLng, val type: MarkerType)

enum class MarkerType {
    SECURE,
    UNSECURE
}
