package net.mullvad.mullvadvpn.lib.map

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
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.data.MarkerType
import net.mullvad.mullvadvpn.lib.map.internal.MapGLShader
import net.mullvad.mullvadvpn.model.LatLng
import net.mullvad.mullvadvpn.model.Latitude
import net.mullvad.mullvadvpn.model.Longitude

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
                zoom = marker?.type.toZoom(),
                cameraLatLng = cameraLocation,
                locationMarker = marker,
                percent = percent
            )
        }
    Log.d("MullvadMap", "CameraLocation: ${mapViewState.cameraLatLng}")
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
    val duration = distance.toAnimationDuration()

    val longitudeAnimation = remember { Animatable(targetCameraLocation.longitude.value) }

    val latitudeAnimation = remember { Animatable(targetCameraLocation.latitude.value) }
    val secureZoomAnimation = remember {
        Animatable(if (marker?.type == MarkerType.SECURE) SECURE_ZOOM else UNSECURE_ZOOM)
    }
    val zoomOutMultiplier = remember { Animatable(1f) }

    LaunchedEffect(targetCameraLocation) {
        launch { latitudeAnimation.animateTo(targetCameraLocation.latitude.value, tween(duration)) }
        launch {
            // Unwind longitudeAnimation into a Longitude
            val currentLongitude = Longitude.fromFloat(longitudeAnimation.value)

            // Resolve a vector showing us the shortest path to the target longitude, e.g going
            // from 170 to -170 would result in 20 since we can wrap around the globe
            val shortestPathVector = currentLongitude.vectorTo(targetCameraLocation.longitude)

            // Animate to the new camera location using the shortest path vector
            longitudeAnimation.animateTo(
                longitudeAnimation.value + shortestPathVector.value,
                tween(duration),
            )

            // Current value animation value might be outside of range of a Longitude, so when the
            // animation is done we unwind the animation to the correct value
            longitudeAnimation.snapTo(targetCameraLocation.longitude.value)
        }
        launch {
            zoomOutMultiplier.animateTo(
                targetValue = 1f,
                animationSpec =
                    keyframes {
                        if (duration < SHORT_ANIMATION_MILLIS) {
                            durationMillis = duration
                            1f at duration using EaseInOut
                        } else {
                            durationMillis = duration
                            1.25f at duration / 3 using EaseInOut
                            1f at duration using EaseInOut
                        }
                    }
            )
        }
    }

    LaunchedEffect(marker?.type) {
        launch { secureZoomAnimation.animateTo(targetValue = marker?.type.toZoom(), tween(2000)) }
    }

    return MapViewState(
        zoom = secureZoomAnimation.value * zoomOutMultiplier.value * 0.9f,
        cameraLatLng =
            LatLng(
                Latitude(latitudeAnimation.value),
                Longitude.fromFloat(longitudeAnimation.value)
            ),
        locationMarker = marker,
        percent = percent
    )
}

fun MarkerType?.toZoom() =
    when (this) {
        MarkerType.SECURE -> SECURE_ZOOM
        MarkerType.UNSECURE,
        null -> UNSECURE_ZOOM
    }

fun Int.toAnimationDuration() =
    (this * DISTANCE_DURATION_SCALE_FACTOR).coerceIn(MIN_ANIMATION_MILLIS, MAX_ANIMATION_MILLIS)

const val SECURE_ZOOM = 1.25f
const val UNSECURE_ZOOM = 1.35f
const val DISTANCE_DURATION_SCALE_FACTOR = 20
const val SHORT_ANIMATION_MILLIS = 1700
const val MIN_ANIMATION_MILLIS = 1300
const val MAX_ANIMATION_MILLIS = 2500
