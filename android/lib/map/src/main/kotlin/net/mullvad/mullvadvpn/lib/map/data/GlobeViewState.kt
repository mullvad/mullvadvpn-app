package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

@Immutable
class GlobeViewState(
    val cameraPosition: CameraPosition,
    val markers: List<Marker> = emptyList(),
    val hops: List<Hop> = emptyList(),
    val globeColors: GlobeColors = GlobeColors.default(),
) {
    companion object {
        fun default() =
            GlobeViewState(
                CameraPosition(
                    latLong = LatLong(latitude = Latitude(0f), longitude = Longitude(0f))
                )
            )
    }
}
