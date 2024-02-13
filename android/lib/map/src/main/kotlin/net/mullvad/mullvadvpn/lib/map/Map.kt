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
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.MapGLSurfaceView
import net.mullvad.mullvadvpn.model.LatLong

@Composable
fun Map(
    modifier: Modifier,
    cameraLocation: CameraPosition,
    markers: List<Marker>,
    globeColors: GlobeColors,
) {
    val mapViewState = MapViewState(cameraLocation, markers, globeColors)
    Map(modifier = modifier, mapViewState = mapViewState)
}

@Composable
fun AnimatedMap(
    modifier: Modifier,
    cameraLocation: LatLong,
    cameraBaseZoom: Float,
    cameraVerticalBias: Float,
    markers: List<Marker>,
    globeColors: GlobeColors
) {
    Map(
        modifier = modifier,
        cameraLocation =
            animatedCameraPosition(
                baseZoom = cameraBaseZoom,
                targetCameraLocation = cameraLocation,
                cameraVerticalBias = cameraVerticalBias
            ),
        markers = markers,
        globeColors
    )
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

    AndroidView(modifier = modifier, factory = { MapGLSurfaceView(it) }) { glSurfaceView ->
        view = glSurfaceView
        glSurfaceView.setData(mapViewState)
    }
}
