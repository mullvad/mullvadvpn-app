package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable

@Immutable
class MapViewState(
    val cameraPosition: CameraPosition,
    val locationMarker: List<Marker>,
    val globeColors: GlobeColors
)
