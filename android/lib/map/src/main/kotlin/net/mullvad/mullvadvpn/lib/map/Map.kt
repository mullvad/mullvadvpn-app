package net.mullvad.mullvadvpn.lib.map

import android.util.Log
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
import net.mullvad.mullvadvpn.model.LatLng

@Composable
fun Map(
    modifier: Modifier,
    animateCameraMovement: Boolean,
    cameraLocation: LatLng,
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
            Log.d("mullvad", "AAA View Disposed ${view.hashCode()}")
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

const val SECURE_ZOOM = 1.25f
const val UNSECURE_ZOOM = 1.35f
const val DISTANCE_DURATION_SCALE_FACTOR = 20
const val SHORT_ANIMATION_MILLIS = 1700
const val MIN_ANIMATION_MILLIS = 1300
const val MAX_ANIMATION_MILLIS = 2500
