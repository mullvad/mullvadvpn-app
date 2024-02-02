package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLES10
import android.opengl.GLES31
import android.opengl.GLES32
import android.opengl.GLSurfaceView
import net.mullvad.lib.map.data.MapViewState
import javax.microedition.khronos.opengles.GL10

class MapGLSurfaceView(context: Context) : GLSurfaceView(context) {

    private val renderer: MapGLRenderer

    init {

        // Create an OpenGL ES 2.0 context
        setEGLContextClientVersion(2)

        debugFlags = DEBUG_CHECK_GL_ERROR or DEBUG_LOG_GL_CALLS
        renderer = MapGLRenderer(context)
        // Set the Renderer for drawing on the GLSurfaceView
        setRenderer(renderer)
        renderMode = RENDERMODE_WHEN_DIRTY
    }

    fun setData(viewState: MapViewState) {
        renderer.setViewState(viewState)
        renderer.onFovChanged(viewState.fov, width = width, height = height)
        requestRender()
    }
}
