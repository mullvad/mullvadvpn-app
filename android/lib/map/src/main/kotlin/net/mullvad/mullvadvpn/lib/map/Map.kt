package net.mullvad.mullvadvpn.lib.map

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LocalLifecycleOwner
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.MapGLSurfaceView
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.Latitude
import net.mullvad.mullvadvpn.lib.model.Longitude

@Preview
@Composable
fun MapPreview() {
    val infinite = rememberInfiniteTransition()
    val spin =
        infinite.animateFloat(
            0f,
            360f,
            infiniteRepeatable(animation = tween(30000, easing = LinearEasing)),
        )

    Map(
        modifier = Modifier,
        cameraLocation =
            CameraPosition(
                LatLong(Latitude(0f), Longitude.fromFloat(spin.value)),
                2f,
                verticalBias = 0.5f,
            ),
        markers = emptyList(),
        globeColors =
            GlobeColors(
                // Green
                landColor = Color(0xFF26513C),
                // Blue
                oceanColor = Color(0xFF161E50),
                // Darker green
                contourColor = Color(0xFF1B3626),
            ),
    )
}

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
