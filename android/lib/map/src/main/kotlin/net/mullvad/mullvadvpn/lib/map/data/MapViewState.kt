package net.mullvad.mullvadvpn.lib.map.data

import net.mullvad.mullvadvpn.model.LatLng

class MapViewState(
    val zoom: Float,
    val cameraLatLng: LatLng,
    val locationMarker: Marker?,
    val percent: Float
)
