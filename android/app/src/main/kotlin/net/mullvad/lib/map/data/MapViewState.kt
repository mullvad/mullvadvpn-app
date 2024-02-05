package net.mullvad.lib.map.data

class MapViewState(
    val zoom: Float,
    val cameraLatLng: LatLng,
    val locationMarker: Marker?,
    val percent: Float
)
