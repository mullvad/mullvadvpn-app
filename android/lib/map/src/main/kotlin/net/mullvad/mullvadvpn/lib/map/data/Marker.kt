package net.mullvad.mullvadvpn.lib.map.data

import net.mullvad.mullvadvpn.model.LatLng

data class Marker(val latLng: LatLng, val type: MarkerType, val size: Float = 0.02f)

enum class MarkerType {
    SECURE,
    UNSECURE
}
