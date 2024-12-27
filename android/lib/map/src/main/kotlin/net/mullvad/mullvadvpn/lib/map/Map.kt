package net.mullvad.mullvadvpn.lib.map

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.MapGLSurfaceView
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.RelayItemId

@Composable
fun Map(
    modifier: Modifier,
    cameraLocation: CameraPosition,
    markers: List<Marker>,
    globeColors: GlobeColors,
    onClickRelayItemId: (GeoLocationId) -> Unit,
    onLongClickRelayItemId: (Offset, GeoLocationId) -> Unit,
) {
    val mapViewState = MapViewState(cameraLocation, markers, globeColors)
    Map(modifier = modifier, mapViewState = mapViewState, onClickRelayItemId, onLongClickRelayItemId)
}

@Composable
fun AnimatedMap(
    modifier: Modifier,
    cameraLocation: LatLong,
    cameraBaseZoom: Float,
    cameraVerticalBias: Float,
    markers: List<Marker>,
    globeColors: GlobeColors,
    onClickRelayItemId: (RelayItemId) -> Unit,
    onLongClickRelayItemId: (Offset, RelayItemId) -> Unit
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
        onClickRelayItemId = onClickRelayItemId,
        onLongClickRelayItemId = onLongClickRelayItemId,
    )
}

@Composable
internal fun Map(
    modifier: Modifier = Modifier,
    mapViewState: MapViewState,
    onClickRelayItemId: (GeoLocationId) -> Unit,
    onLongClickRelayItemId: (Offset, GeoLocationId) -> Unit,
) {
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

    AndroidView(
        modifier =
        Modifier.pointerInput(lifeCycleState) {
                detectTapGestures(
                    onTap = {
                        Logger.i("Registered marker click: $it")
                        val result = view?.onMapClick(it) ?: return@detectTapGestures
                        onClickRelayItemId(result.first)
                    },
                    onLongPress = {
                        Logger.i("Registered marker long click")
                        val result = view?.onMapClick(it) ?: return@detectTapGestures

                        onLongClickRelayItemId(result.second, result.first)
                    },
                )
            }.then(modifier),
        factory = { MapGLSurfaceView(it) },
    ) { glSurfaceView ->
        view = glSurfaceView
        glSurfaceView.setData(mapViewState)
    }
}
