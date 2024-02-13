package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable
import net.mullvad.mullvadvpn.model.LatLong

@Immutable
data class Marker(val latLong: LatLong, val size: Float = 0.02f, val colors: LocationMarkerColors)
