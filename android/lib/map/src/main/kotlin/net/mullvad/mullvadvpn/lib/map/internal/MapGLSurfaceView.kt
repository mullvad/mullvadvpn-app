package net.mullvad.mullvadvpn.lib.map.internal

import android.annotation.SuppressLint
import android.content.Context
import android.opengl.GLSurfaceView
import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.internal.shapes.GlobeColors
import net.mullvad.mullvadvpn.lib.map.internal.shapes.LocationMarkerColors

@SuppressLint("ViewConstructor")
class MapGLSurfaceView(context: Context, mapConfig: MapConfig) : GLSurfaceView(context) {

    private val renderer: MapGLRenderer

    init {
        // Create an OpenGL ES 2.0 context
        setEGLContextClientVersion(2)

        debugFlags = DEBUG_CHECK_GL_ERROR or DEBUG_LOG_GL_CALLS
        renderer = MapGLRenderer(context.resources, mapConfig)
        // Set the Renderer for drawing on the GLSurfaceView
        setRenderer(renderer)
        renderMode = RENDERMODE_WHEN_DIRTY
    }

    fun setData(viewState: MapViewState) {
        renderer.setViewState(viewState)
        requestRender()
    }
}

data class MapConfig(
    val globeColors: GlobeColors =
        GlobeColors(
            landColor = Color(0.16f, 0.302f, 0.45f),
            oceanColor = Color(0.098f, 0.18f, 0.271f),
            contourColor = Color(0.098f, 0.18f, 0.271f)
        ),
    val secureMarkerColor: LocationMarkerColors =
        LocationMarkerColors(centerColor = Color(0.267f, 0.678f, 0.302f)),
    val unsecureMarkerColor: LocationMarkerColors =
        LocationMarkerColors(centerColor = Color(0.89f, 0.251f, 0.224f))
)
