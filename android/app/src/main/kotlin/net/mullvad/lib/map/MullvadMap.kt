package net.mullvad.lib.map

import android.util.Log
import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import net.mullvad.lib.map.data.LatLng
import net.mullvad.lib.map.data.MapViewState
import net.mullvad.lib.map.data.Marker
import net.mullvad.lib.map.data.MarkerType
import net.mullvad.mullvadvpn.compose.util.rememberPrevious

const val animationMinTime = 1300
const val animationMaxTime = 2500
const val unsecureZoom = 1.35f
const val secureZoom = 1.25f
const val movementMultiplier = 1.5f

@Composable
fun MullvadMap(
    modifier: Modifier,
    animate: Boolean,
    cameraLocation: LatLng,
    marker: Marker?,
    percent: Float,
) {
    val mapViewState =
        if (animate) {
            animatedMapViewState(cameraLocation, marker, percent)
        } else {
            MapViewState(
                zoom = secureZoom,
                cameraLatLng = cameraLocation,
                locationMarker = marker,
                percent = percent
            )
        }
    MapGLShader(modifier = modifier, mapViewState = mapViewState)
}

@Composable
fun animatedMapViewState(
    cameraLocation: LatLng,
    marker: Marker?,
    percent: Float,
): MapViewState {
    val tempPreviousLocation =
        rememberPrevious(current = cameraLocation, shouldUpdate = { prev, curr -> prev != curr })
            ?: cameraLocation

    val previousLocation = remember(cameraLocation) { tempPreviousLocation }
    val distance = remember(cameraLocation) { cameraLocation.distanceTo(previousLocation).toInt() }

    val duration = (distance * 20).coerceIn(animationMinTime, animationMaxTime)

    val animatedLat by
        animateFloatAsState(
            targetValue = cameraLocation.latitude,
            tween(durationMillis = duration),
            label = "latitude"
        )
    Log.d("MullvadMap", distance.toString())
    Log.d("MullvadMap", duration.toString())

    val correctedAnimatedLat =
        if (animatedLat - cameraLocation.latitude < 180) animatedLat else 360f - animatedLat

    val animatedLon =
        animateFloatAsState(
            targetValue = cameraLocation.longitude,
            tween(durationMillis = duration),
            label = "longitude"
        )

    val securityZoom =
        animateFloatAsState(
            targetValue = if (marker?.type == MarkerType.SECURE) secureZoom else unsecureZoom,
            animationSpec = tween(duration),
            label = "secureZoom"
        )
    val movementMultiplierAnimation =
        if (duration < 1700) {
            remember { mutableFloatStateOf(1f) }
        } else {
            animateFloatAsState(
                targetValue = if (previousLocation != cameraLocation) 1f else 1.001f,
                animationSpec =
                    keyframes {
                        durationMillis = duration
                        movementMultiplier at (duration / 3) with EaseInOut
                        1f at duration with EaseInOut
                    },
                label = "zoomMultiplier"
            )
        }

    return MapViewState(
        zoom = secureZoom * movementMultiplierAnimation.value,
        cameraLatLng = LatLng(animatedLat, animatedLon.value),
        locationMarker = marker,
        percent = percent
    )
}
