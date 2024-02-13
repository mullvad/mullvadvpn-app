package net.mullvad.mullvadvpn.lib.map

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.MapConfig
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.data.MarkerType
import net.mullvad.mullvadvpn.lib.map.internal.MapGLSurfaceView
import net.mullvad.mullvadvpn.model.LatLong

@Composable
fun Map(
    modifier: Modifier,
    animateCameraMovement: Boolean,
    cameraLocation: LatLong,
    marker: Marker?,
    percent: Float,
) {
    val mapViewState =
        if (animateCameraMovement) {
            MapViewState(marker, animatedCameraPosition(cameraLocation, marker, percent))
        } else {
            MapViewState(marker, CameraPosition(cameraLocation, marker?.type.toZoom(), percent))
        }
    Map(modifier = modifier, mapViewState = mapViewState)
}

@Composable
internal fun Map(modifier: Modifier = Modifier, mapViewState: MapViewState) {

    var view: MapGLSurfaceView? = remember { null }

    val lifeCycleState = LocalLifecycleOwner.current.lifecycle

    DisposableEffect(key1 = lifeCycleState) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_RESUME -> {
                    view?.onResume()
                }
                Lifecycle.Event.ON_PAUSE -> {
                    view?.onPause()
                }
                else -> {}
            }
        }
        lifeCycleState.addObserver(observer)

        onDispose {
            lifeCycleState.removeObserver(observer)
            view?.onPause()
            view = null
        }
    }

    // TODO how to handle mapConfig changes? Can we recreate the view? make them recomposable?
    AndroidView(modifier = modifier, factory = { MapGLSurfaceView(it, mapConfig = MapConfig()) }) {
        glSurfaceView ->
        view = glSurfaceView
        glSurfaceView.setData(mapViewState)
    }
}

fun MarkerType?.toZoom() =
    when (this) {
        MarkerType.SECURE -> SECURE_ZOOM
        MarkerType.UNSECURE,
        null -> UNSECURE_ZOOM
    }

fun Int.toAnimationDuration() =
    (this * DISTANCE_DURATION_SCALE_FACTOR).coerceIn(MIN_ANIMATION_MILLIS, MAX_ANIMATION_MILLIS)

// Distance to marker when secure/unsecure
const val SECURE_ZOOM = 1.15f
const val UNSECURE_ZOOM = 1.20f
const val SECURE_ZOOM_ANIMATION_MILLIS = 2000

// Constant what will talk the distance in LatLng multiply it to determine the animation duration,
// the result is then confined to the MIN_ANIMATION_MILLIS and MAX_ANIMATION_MILLIS
const val DISTANCE_DURATION_SCALE_FACTOR = 20
const val MIN_ANIMATION_MILLIS = 1300
const val MAX_ANIMATION_MILLIS = 2500
// The cut off where we go from a short animation (camera pans) to a far animation (camera pans +
// zoom out)
const val SHORT_ANIMATION_CUTOFF_MILLIS = 1700

// Multiplier for the zoom out animation
const val FAR_ANIMATION_MAX_ZOOM_MULTIPLIER = 1.30f
// When in the far animation we reach the MAX_ZOOM_MULTIPLIER, value is between 0 and 1
const val MAX_MULTIPLIER_PEAK_TIMING = .35f
