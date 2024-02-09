package net.mullvad.mullvadvpn.lib.map

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.data.MarkerType
import net.mullvad.mullvadvpn.model.LatLng
import net.mullvad.mullvadvpn.model.Latitude
import net.mullvad.mullvadvpn.model.Longitude

@Composable
fun animatedCameraPosition(
    targetCameraLocation: LatLng,
    marker: Marker?,
    percent: Float,
): CameraPosition {
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

    return CameraPosition(
        zoom = secureZoomAnimation.value * zoomOutMultiplier.value * 0.9f,
        latLng =
            LatLng(
                Latitude(latitudeAnimation.value),
                Longitude.fromFloat(longitudeAnimation.value)
            ),
        bias = percent
    )
}
