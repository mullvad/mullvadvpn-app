package net.mullvad.mullvadvpn.compose.map

import android.util.Log
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.map.data.LatLng
import net.mullvad.mullvadvpn.compose.map.data.MapViewState
import net.mullvad.mullvadvpn.compose.map.data.Marker
import net.mullvad.mullvadvpn.compose.map.data.MarkerType
import net.mullvad.mullvadvpn.compose.map.internal.COMPLETE_ANGLE
import net.mullvad.mullvadvpn.compose.map.internal.MapGLShader
import net.mullvad.mullvadvpn.compose.util.rememberPrevious

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

    val longitudeAnimation = remember { Animatable(targetCameraLocation.longitude) }

    // Correct the value to be within -180 to 180
    val longitudeAnimationCorrected =
        remember(longitudeAnimation) {
            derivedStateOf {
                val value = longitudeAnimation.value.mod(COMPLETE_ANGLE)
                if (value > COMPLETE_ANGLE / 2) {
                    value - COMPLETE_ANGLE
                } else {
                    value
                }
            }
        }
    val latitudeAnimation = remember { Animatable(targetCameraLocation.latitude) }
    val secureZoomAnimation = remember {
        Animatable(if (marker?.type == MarkerType.SECURE) SECURE_ZOOM else UNSECURE_ZOOM)
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
                    COMPLETE_ANGLE
                )

            longitudeAnimation.animateTo(
                longitudeAnimation.value + shortestPathVector,
                tween(duration)
            )
            // Unwind longitudeAnimation (prevent us from winding up angle to infinity)
            longitudeAnimation.snapTo(longitudeAnimation.value.mod(COMPLETE_ANGLE))
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
        cameraLatLng = LatLng(latitudeAnimation.value, longitudeAnimationCorrected.value),
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

fun MarkerType?.toZoom() =
    when (this) {
        MarkerType.SECURE -> net.mullvad.mullvadvpn.compose.map.SECURE_ZOOM
        MarkerType.UNSECURE,
        null -> net.mullvad.mullvadvpn.compose.map.UNSECURE_ZOOM
    }

fun Int.toAnimationDuration() =
    (this * DISTANCE_DURATION_SCALE_FACTOR).coerceIn(MIN_ANIMATION_MILLIS, MAX_ANIMATION_MILLIS)

const val SECURE_ZOOM = 1.25f
const val UNSECURE_ZOOM = 1.35f
const val DISTANCE_DURATION_SCALE_FACTOR = 20
const val SHORT_ANIMATION_MILLIS = 1700
const val MIN_ANIMATION_MILLIS = 1300
const val MAX_ANIMATION_MILLIS = 2500
