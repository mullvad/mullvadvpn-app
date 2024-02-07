package net.mullvad.lib.map

import android.annotation.SuppressLint
import android.content.Context
import android.opengl.GLSurfaceView
import android.util.Log
import androidx.compose.ui.graphics.Color
import net.mullvad.lib.map.data.MapViewState
import net.mullvad.lib.map.shapes.GlobeColors
import javax.microedition.khronos.egl.EGL10
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.egl.EGLContext
import javax.microedition.khronos.egl.EGLDisplay

@SuppressLint("ViewConstructor")
class MapGLSurfaceView(context: Context, mapConfig: MapConfig) : GLSurfaceView(context) {

    private val renderer: MapGLRenderer

    init {
        // Create an OpenGL ES 2.0 context
        setEGLContextClientVersion(2)

        debugFlags = DEBUG_CHECK_GL_ERROR or DEBUG_LOG_GL_CALLS
        renderer = MapGLRenderer(context, mapConfig)
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
    val globeColors: GlobeColors = GlobeColors(
        landColor = Color(0.16f, 0.302f, 0.45f),
        oceanColor = Color(0.098f, 0.18f, 0.271f),
        contourColor = Color(0.098f, 0.18f, 0.271f)

    ),
    val secureMarkerColor: Color = Color(0.267f, 0.678f, 0.302f),
    val unsecureMarkerColor: Color = Color(0.89f, 0.251f, 0.224f)
)
