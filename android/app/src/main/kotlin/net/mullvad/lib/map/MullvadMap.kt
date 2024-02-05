package net.mullvad.lib.map

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import net.mullvad.lib.map.data.LatLng
import net.mullvad.lib.map.data.MapViewState
import net.mullvad.lib.map.data.Marker

@Composable
fun MullvadMap(
    modifier: Modifier,
    cameraLocation: LatLng,
    marker: Marker?,
    percent: Float,
) {
    val animatedLat =
        animateFloatAsState(
            targetValue = cameraLocation.latitude,
            tween(durationMillis = 1000),
            label = "latitude"
        )
    val animatedLon =
        animateFloatAsState(
            targetValue = cameraLocation.longitude,
            tween(durationMillis = 1000),
            label = "longitude"
        )

    val unsecureZoom = 1.35f
    val secureZoom = 1.25f

    val zoom =
        animateFloatAsState(
            targetValue = unsecureZoom, // if (tunnelState.isSecured()) connectedZoom else
            // disconnectedZoom,,
            tween(durationMillis = 1000),
            label = "zoom"
        )

    val mapViewState =
        MapViewState(
            zoom = zoom.value,
            cameraLatLng = LatLng(animatedLat.value, animatedLon.value),
            locationMarker = marker,
            percent = percent
        )
    MapGLShader(modifier = modifier, mapViewState = mapViewState)
}
