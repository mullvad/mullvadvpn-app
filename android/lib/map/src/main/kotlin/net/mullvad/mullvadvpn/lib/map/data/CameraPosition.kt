package net.mullvad.mullvadvpn.lib.map.data

import androidx.compose.runtime.Immutable
import net.mullvad.mullvadvpn.model.LatLong

@Immutable data class CameraPosition(val latLong: LatLong, val zoom: Float, val verticalBias: Float)
