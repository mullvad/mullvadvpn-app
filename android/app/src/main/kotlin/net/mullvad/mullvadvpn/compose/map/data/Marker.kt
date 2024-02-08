package net.mullvad.mullvadvpn.compose.map.data

data class Marker(val latLng: LatLng, val type: MarkerType, val size: Float = 0.02f)

enum class MarkerType {
    SECURE,
    UNSECURE
}
