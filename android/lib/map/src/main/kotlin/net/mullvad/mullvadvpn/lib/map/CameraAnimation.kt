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
import net.mullvad.mullvadvpn.lib.map.internal.DISTANCE_DURATION_SCALE_FACTOR
import net.mullvad.mullvadvpn.lib.map.internal.FAR_ANIMATION_MAX_ZOOM_MULTIPLIER
import net.mullvad.mullvadvpn.lib.map.internal.MAX_ANIMATION_MILLIS
import net.mullvad.mullvadvpn.lib.map.internal.MAX_MULTIPLIER_PEAK_TIMING
import net.mullvad.mullvadvpn.lib.map.internal.MIN_ANIMATION_MILLIS
import net.mullvad.mullvadvpn.lib.map.internal.SHORT_ANIMATION_CUTOFF_MILLIS
import net.mullvad.mullvadvpn.model.LatLong
import net.mullvad.mullvadvpn.model.Latitude
import net.mullvad.mullvadvpn.model.Longitude

@Composable
fun animatedCameraPosition(
    baseZoom: Float,
    targetCameraLocation: LatLong,
    cameraVerticalBias: Float,
): CameraPosition {
    val previousLocation =
        rememberPrevious(
            current = targetCameraLocation,
            shouldUpdate = { prev, curr -> prev != curr }
        ) ?: targetCameraLocation

    val distance =
        remember(targetCameraLocation) { targetCameraLocation.distanceTo(previousLocation).toInt() }
    val duration = distance.toAnimationDuration()

    val longitudeAnimation = remember { Animatable(targetCameraLocation.longitude.value) }

    val latitudeAnimation = remember { Animatable(targetCameraLocation.latitude.value) }
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
                        if (duration < SHORT_ANIMATION_CUTOFF_MILLIS) {
                            durationMillis = duration
                            1f at duration using EaseInOut
                        } else {
                            durationMillis = duration
                            FAR_ANIMATION_MAX_ZOOM_MULTIPLIER at
                                (duration * MAX_MULTIPLIER_PEAK_TIMING).toInt() using
                                EaseInOut
                            1f at duration using EaseInOut
                        }
                    }
            )
        }
    }

    return CameraPosition(
        zoom = baseZoom * zoomOutMultiplier.value,
        latLong =
            LatLong(
                Latitude(latitudeAnimation.value),
                Longitude.fromFloat(longitudeAnimation.value)
            ),
        bias = cameraVerticalBias
    )
}

private fun Int.toAnimationDuration() =
    (this * DISTANCE_DURATION_SCALE_FACTOR).coerceIn(MIN_ANIMATION_MILLIS, MAX_ANIMATION_MILLIS)
