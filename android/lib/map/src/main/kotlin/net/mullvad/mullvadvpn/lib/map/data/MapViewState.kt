package net.mullvad.mullvadvpn.lib.map.data

import net.mullvad.mullvadvpn.model.LatLng

class MapViewState(val locationMarker: Marker?, val cameraPosition: CameraPosition)

data class CameraPosition(val latLng: LatLng, val zoom: Float, val bias: Float)
