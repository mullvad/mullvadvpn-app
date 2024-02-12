package net.mullvad.mullvadvpn.lib.map.data

import net.mullvad.mullvadvpn.model.LatLong

class MapViewState(val locationMarker: Marker?, val cameraPosition: CameraPosition)

data class CameraPosition(val latLong: LatLong, val zoom: Float, val bias: Float)
