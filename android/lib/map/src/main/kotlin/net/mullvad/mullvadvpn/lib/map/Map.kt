package net.mullvad.mullvadvpn.lib.map

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LocalLifecycleOwner
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.MapGLSurfaceView
import net.mullvad.mullvadvpn.lib.model.LatLong

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
    globeColors: GlobeColors,
) {
    Map(
        modifier = modifier,
        cameraLocation =
            animatedCameraPosition(
                baseZoom = cameraBaseZoom,
                targetCameraLocation = cameraLocation,
                cameraVerticalBias = cameraVerticalBias,
            ),
        markers = markers,
        globeColors,
    )
}

@Composable
internal fun Map(modifier: Modifier = Modifier, mapViewState: MapViewState) {

    val lifeCycleState = LocalLifecycleOwner.current.lifecycle

    AndroidView(
        modifier = modifier,
        factory = { MapGLSurfaceView(it) },
        update = { glSurfaceView ->
            glSurfaceView.lifecycle = lifeCycleState
            glSurfaceView.setData(mapViewState)
        },
        onRelease = { it.lifecycle = null },
    )
}
