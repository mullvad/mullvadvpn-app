package net.mullvad.mullvadvpn.lib.map.internal

import android.content.Context
import android.opengl.GLSurfaceView
import androidx.compose.ui.geometry.Offset
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import net.mullvad.mullvadvpn.lib.map.data.GlobeViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.data.toLatLng
import net.mullvad.mullvadvpn.lib.model.LatLong

internal class MapSurfaceView(context: Context) : GLSurfaceView(context) {
    private val renderer: MapRenderer = MapRenderer(context.resources)
    var lifecycle: Lifecycle? = null
        set(value) {
            field?.removeObserver(observer)
            value?.addObserver(observer)
            field = value
        }

    private val observer = LifecycleEventObserver { source, event ->
        when (event) {
            Lifecycle.Event.ON_RESUME -> onResume()
            Lifecycle.Event.ON_PAUSE -> onPause()
            else -> {}
        }
    }

    init {
        // Create an OpenGL ES 2.0 context
        setEGLContextClientVersion(2)

        // Set the Renderer for drawing on the GLSurfaceView
        setRenderer(renderer)
        renderMode = RENDERMODE_WHEN_DIRTY
    }

    fun setData(viewState: GlobeViewState) {
        renderer.setViewState(viewState)
        requestRender()
    }

    fun getPosition(offset: Offset): LatLong? = renderer.calculateIntersection(offset)?.toLatLng()

    fun closestMarker(offset: Offset): Pair<Marker, Offset>? {
        val (marker, distance) = renderer.closestMarker(offset) ?: return null
        return if (distance < MIN_DISTANCE) {
            marker?.id?.let { marker to offset }
        } else {
            null
        }
    }

    companion object {
        private const val MIN_DISTANCE = 0.03f
    }
}
