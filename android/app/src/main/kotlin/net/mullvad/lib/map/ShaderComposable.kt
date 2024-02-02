package net.mullvad.lib.map

import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import net.mullvad.lib.map.data.Coordinate
import net.mullvad.lib.map.data.MapViewState
import net.mullvad.lib.map.data.Marker
import net.mullvad.lib.map.data.MarkerType

@Composable
fun MapGLShader(modifier: Modifier = Modifier, coordinate: Coordinate, zoom: Float, percent: Float) {
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

    //    animateFloatAsState(targetValue = )
    val gothenburg = Coordinate(57.7089f, 11.9746f)
    val mapViewState =
        MapViewState(
            zoom = zoom,
            cameraCoordinate = gothenburg,
            locationMarker = Marker(gothenburg, MarkerType.SECURE),
            percent = percent
        )

    AndroidView(modifier = modifier, factory = { MapGLSurfaceView(it) }) { glSurfaceView ->
        view = glSurfaceView
        glSurfaceView.setData(mapViewState)
    }
}
