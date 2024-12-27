package net.mullvad.mullvadvpn.lib.map.data

import android.os.Parcelable
import androidx.compose.runtime.Immutable
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.LatLong

@Immutable
data class Marker(
    val latLong: LatLong,
    val size: Float = DEFAULT_MARKER_SIZE,
    val colors: LocationMarkerColors,
    val id: GeoLocationId? = null,
) {
    companion object {
        private const val DEFAULT_MARKER_SIZE = 0.02f
    }
}
