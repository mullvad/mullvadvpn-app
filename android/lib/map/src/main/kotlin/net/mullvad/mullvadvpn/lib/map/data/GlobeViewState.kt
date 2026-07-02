package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable

@Immutable
class GlobeViewState(
    val cameraPosition: CameraPosition,
    val markers: List<Marker> = emptyList(),
    val hops: List<Hop> = emptyList(),
    val globeColors: GlobeColors,
)
