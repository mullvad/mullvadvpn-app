package net.mullvad.mullvadvpn.lib.map.internal

import android.annotation.SuppressLint
import android.content.Context
import android.opengl.GLSurfaceView
import net.mullvad.mullvadvpn.lib.map.data.MapViewState

internal class MapGLSurfaceView(context: Context) : GLSurfaceView(context) {

    private val renderer: MapGLRenderer

    init {
        // Create an OpenGL ES 2.0 context
        setEGLContextClientVersion(2)

        debugFlags = DEBUG_CHECK_GL_ERROR or DEBUG_LOG_GL_CALLS
        renderer = MapGLRenderer(context.resources)

        // Set the Renderer for drawing on the GLSurfaceView
        setRenderer(renderer)
        renderMode = RENDERMODE_WHEN_DIRTY
    }

    fun setData(viewState: MapViewState) {
        renderer.setViewState(viewState)
        requestRender()
    }
}
