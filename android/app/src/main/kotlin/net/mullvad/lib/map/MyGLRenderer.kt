package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import net.mullvad.lib.map.shapes.Globe
import net.mullvad.lib.map.shapes.Triangle

class MyGLRenderer(val context: Context) : GLSurfaceView.Renderer {
    private lateinit var mTriangle: Triangle
    private lateinit var globe: Globe

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        // Set the background frame color
        GLES20.glClearColor(0.0f, 0.0f, 0.0f, 1.0f)
        // initialize a triangle
        mTriangle = Triangle()
        globe = Globe(context)
    }

    override fun onDrawFrame(gl10: GL10) {
        // Redraw background color
        GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT)
        mTriangle.draw()
        globe.draw(
            floatArrayOf(1f, 1f, 1f),
            floatArrayOf(1f, 1f, 1f),
        )
    }

    override fun onSurfaceChanged(unused: GL10, width: Int, height: Int) {
        GLES20.glViewport(0, 0, width, height)
    }
}
