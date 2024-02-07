package net.mullvad.lib.map

import android.util.Log
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import kotlinx.coroutines.launch
import net.mullvad.lib.map.data.LatLng
import net.mullvad.lib.map.data.MapViewState
import net.mullvad.lib.map.data.Marker
import net.mullvad.lib.map.data.MarkerType
import net.mullvad.mullvadvpn.compose.util.rememberPrevious

const val animationMinTime = 1300
const val animationMaxTime = 2500
const val unsecureZoom = 1.35f
const val secureZoom = 1.25f
const val completeAngle = 360f
const val movementMultiplier = 1.5f

@Composable
fun Map(
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
                zoom = if (marker?.type == MarkerType.SECURE) secureZoom else unsecureZoom,
                cameraLatLng = cameraLocation,
                locationMarker = marker,
                percent = percent
            )
        }
    MapGLShader(modifier = modifier, mapViewState = mapViewState)
}

@Composable
fun animatedMapViewState(
    targetCameraLocation: LatLng,
    marker: Marker?,
    percent: Float,
): MapViewState {
    val tempPreviousLocation =
        rememberPrevious(
            current = targetCameraLocation,
            shouldUpdate = { prev, curr -> prev != curr }
        ) ?: targetCameraLocation
    val previousLocation = remember(targetCameraLocation) { tempPreviousLocation }

    val distance =
        remember(targetCameraLocation) { targetCameraLocation.distanceTo(previousLocation).toInt() }
    val duration = (distance * 20).coerceIn(animationMinTime, animationMaxTime)

    val longitudeAnimation = remember { Animatable(targetCameraLocation.longitude) }
    val latitudeAnimation = remember { Animatable(targetCameraLocation.latitude) }
    val secureZoomAnimation = remember {
        Animatable(if (marker?.type == MarkerType.SECURE) secureZoom else unsecureZoom)
    }
    val zoomOutMultiplier = remember { Animatable(1f) }

    LaunchedEffect(targetCameraLocation) {
        launch { latitudeAnimation.animateTo(targetCameraLocation.latitude, tween(duration)) }
        launch {
            // We resolve a vector showing us the shortest path to the target longitude, e.g going
            // from 10 to 350 would result in -20 since we can wrap around the globe
            val shortestPathVector =
                shortestPathVector(
                    longitudeAnimation.value,
                    targetCameraLocation.longitude,
                    completeAngle
                )

            Log.d("mullvad", "AAA longitudeAnimationValue ${longitudeAnimation.value}")
            longitudeAnimation.animateTo(
                longitudeAnimation.value + shortestPathVector,
                tween(duration)
            )
        }
        launch {
            zoomOutMultiplier.animateTo(
                targetValue = 1f,
                animationSpec =
                    keyframes {
                        if (duration < 1700) {
                            durationMillis = duration
                            1f at duration with EaseInOut
                        } else {
                            durationMillis = duration
                            1.25f at (duration / 3) with EaseInOut
                            1f at duration with EaseInOut
                        }
                    }
            )
        }
    }

    LaunchedEffect(marker?.type) {
        launch {
            secureZoomAnimation.animateTo(
                targetValue = if (marker?.type == MarkerType.SECURE) secureZoom else unsecureZoom,
                tween(2000)
            )
        }
    }

    return MapViewState(
        zoom = secureZoomAnimation.value * zoomOutMultiplier.value * 0.9f,
        cameraLatLng = LatLng(latitudeAnimation.value, longitudeAnimation.value.mod(completeAngle)),
        locationMarker = marker,
        percent = percent
    )
}

fun shortestPathVector(current: Float, newValue: Float, wrappingSize: Float): Float {
    val diff = newValue - current
    val maxDistance = wrappingSize / 2
    return when {
        diff > maxDistance -> diff - wrappingSize
        diff < -maxDistance -> diff + wrappingSize
        else -> diff
    }
}
