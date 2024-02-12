package net.mullvad.mullvadvpn.lib.map.data

import net.mullvad.mullvadvpn.model.LatLong

data class Marker(val latLong: LatLong, val type: MarkerType, val size: Float = 0.02f)

enum class MarkerType {
    SECURE,
    UNSECURE
}
