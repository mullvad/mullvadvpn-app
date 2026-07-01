package net.mullvad.mullvadvpn.lib.map

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LocalLifecycleOwner
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.GlobeViewState
import net.mullvad.mullvadvpn.lib.map.data.Hop
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.MapSurfaceView

@Composable
fun Map(
    cameraPosition: CameraPosition,
    modifier: Modifier = Modifier,
    markers: List<Marker> = emptyList(),
    hops: List<Hop> = emptyList(),
    globeColors: GlobeColors = GlobeColors.default(),
) {
    val globeViewState = GlobeViewState(cameraPosition, markers, hops, globeColors)
    Map(modifier = modifier, globeViewState = globeViewState)
}

@Composable
private fun Map(modifier: Modifier = Modifier, globeViewState: GlobeViewState) {
    val lifeCycleState = LocalLifecycleOwner.current.lifecycle

    AndroidView(
        modifier = modifier,
        factory = { MapSurfaceView(it) },
        update = { glSurfaceView ->
            glSurfaceView.lifecycle = lifeCycleState
            glSurfaceView.setData(globeViewState)
        },
        onRelease = { it.lifecycle = null },
    )
}

@Composable
fun Map(
    cameraPosition: CameraPosition,
    modifier: Modifier = Modifier,
    markers: List<Marker> = emptyList(),
    hops: List<Hop> = emptyList(),
    globeColors: GlobeColors = GlobeColors.default(),
    onMarkerClick: ((Marker) -> Unit) = {},
    onMarkerLongPress: (Offset, Marker) -> Unit = { _, _ -> },
) {
    val globeViewState = GlobeViewState(cameraPosition, markers, hops, globeColors)
    Map(modifier = modifier, globeViewState = globeViewState, onMarkerClick, onMarkerLongPress)
}

@Composable
private fun Map(
    modifier: Modifier = Modifier,
    globeViewState: GlobeViewState,
    onClick: (Marker) -> Unit = {},
    onLongClick: (Offset, Marker) -> Unit = { _, _ -> },
) {
    var view: MapSurfaceView? = remember { null }

    val lifeCycleState = LocalLifecycleOwner.current.lifecycle

    AndroidView(
        modifier =
            Modifier.pointerInput(lifeCycleState) {
                    detectTapGestures(
                        onTap = {
                            val result = view?.closestMarker(it) ?: return@detectTapGestures
                            onClick(result.first)
                        },
                        onLongPress = {
                            val result = view?.closestMarker(it) ?: return@detectTapGestures
                            onLongClick(result.second, result.first)
                        },
                    )
                }
                .then(modifier),
        factory = { MapSurfaceView(it) },
        update = { glSurfaceView ->
            glSurfaceView.lifecycle = lifeCycleState
            view = glSurfaceView
            glSurfaceView.setData(globeViewState)
        },
        onRelease = { it.lifecycle = null },
    )
}
