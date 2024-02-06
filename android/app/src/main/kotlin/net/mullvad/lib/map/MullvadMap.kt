package net.mullvad.lib.map

import androidx.compose.animation.core.AnimationVector2D
import androidx.compose.animation.core.EaseInOut
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.core.TwoWayConverter
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.animateValue
import androidx.compose.animation.core.animateValueAsState
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
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

    val cameraLatitude = remember { mutableFloatStateOf(cameraLocation.latitude) }

    val animationVector by
        remember(cameraLocation) {
            derivedStateOf { shortestPath(previousLocation, cameraLocation) }
        }

    val mutableTransitionState = remember { MutableTransitionState(animationVector) }

    val transition =
        updateTransition(transitionState = mutableTransitionState, label = "cameraLocation")

    transition.animateValue(
        typeConverter =
            TwoWayConverter(
                convertToVector = { AnimationVector2D(it.latitude, it.longitude) },
                convertFromVector = { vector: AnimationVector2D -> LatLng(vector.v1, vector.v2) }
            ),
        transitionSpec = { tween(duration) },
        targetValueByState = { state -> state },
        label = ""
    )


    val newLatLng =
        if (transition.isRunning) {
            cameraLocation + animationVector
        } else {
            cameraLocation
        }

    val animatedLatLng =
        animateValueAsState(
            targetValue = cameraLocation,
            typeConverter =
                TwoWayConverter(
                    convertToVector = { AnimationVector2D(it.latitude, it.longitude) },
                    convertFromVector = { LatLng(it.v1, it.v2) }
                ),
            animationSpec = tween(duration),
            finishedListener = { cameraLatitude.value = it.latitude }
        )

    //    val animatedLat by
    //        animateFloatAsState(
    //            targetValue = cameraLatitude.latitude,
    //            tween(durationMillis = duration),
    //            label = "latitude"
    //        )
    //
    //    val animatedLon =
    //        animateFloatAsState(
    //            targetValue = cameraLatitude.longitude,
    //            tween(durationMillis = duration),
    //            label = "longitude"
    //        )

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
                targetValue = if (previousLocation != cameraLatitude) 1f else 1.001f,
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
        cameraLatLng = transition.currentState,
        locationMarker = marker,
        percent = percent
    )
}

fun shortestPath(c1: LatLng, c2: LatLng): LatLng {
    var longDiff = c2.latitude - c1.latitude
    if (longDiff > 180) {
        longDiff -= 360
    } else if (longDiff < -180) {
        longDiff += 360
    }
    return LatLng(c2.longitude - c1.longitude, longDiff)
}
