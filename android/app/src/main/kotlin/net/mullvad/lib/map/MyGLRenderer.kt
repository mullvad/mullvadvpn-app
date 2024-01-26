package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLES31
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import android.os.SystemClock
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import net.mullvad.lib.map.shapes.Globe
import net.mullvad.lib.map.shapes.LocationMarker

class MyGLRenderer(val context: Context) : GLSurfaceView.Renderer {
    private lateinit var globe: Globe
    private lateinit var locationMarker: LocationMarker

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        // Set the background frame color
        // initialize a triangle
        globe = Globe(context)
        locationMarker = LocationMarker(floatArrayOf(1.0f, 1.0f, 0.0f, 1.0f))

        initGLOptions()
    }

    private fun initGLOptions(){
        GLES31.glEnable(GLES31.GL_CULL_FACE)
        GLES31.glCullFace(GLES31.GL_BACK)

        GLES31.glEnable(GLES31.GL_BLEND)
        GLES31.glBlendFunc(GLES31.GL_SRC_ALPHA, GLES31.GL_ONE_MINUS_SRC_ALPHA)
    }

    private val vPMatrix = FloatArray(16)
    private val projectionMatrix = FloatArray(16)
    private val viewMatrix = FloatArray(16)
    private val rotationMatrix = FloatArray(16)

    val connectedZoom = 1.25f
    private val zoom = 0.7f
    override fun onDrawFrame(gl10: GL10) {
        // Clear function
        clear()

        val scratch = FloatArray(16)


        val offsetY = 0.088 + (zoom - connectedZoom) * 0.3

        Matrix.setLookAtM(viewMatrix, 0, 0f, offsetY.toFloat(), 1f, 0f, 0f, 0f, 0f, 1.0f, 0.0f)
        Matrix.multiplyMM(vPMatrix, 0, projectionMatrix, 0, viewMatrix, 0)

        // mTriangle.draw(vPMatrix)
        // Set the camera position (View matrix)

        //        globe.draw(
        //            projectionMatrix,
        //            viewMatrix,
        //        )

        // Create a rotation transformation for the triangle
        val time = SystemClock.uptimeMillis() % 4000L
        val angle = 0.090f * time.toInt()
        Matrix.setRotateM(rotationMatrix, 0, angle, 0.0f, 0.0f, 1.0f)

        // Combine the rotation matrix with the projection and camera view
        // Note that the vPMatrix factor *must be first* in order
        // for the matrix multiplication product to be correct.
        Matrix.multiplyMM(scratch, 0, vPMatrix, 0, rotationMatrix, 0)

        // Draw triangle
        locationMarker.draw(scratch, viewMatrix, Coordinate(90f, 90f), 0.03f * 200f)
        locationMarker.draw(scratch, viewMatrix, Coordinate(0f, 0f), 0.03f * 10200f)
        locationMarker.draw(scratch, viewMatrix, Coordinate(180f, 180f), 0.03f * 10f)
        locationMarker.draw(scratch, viewMatrix, Coordinate(270f, 270f), 0.03f * 1f)
        //globe.draw(scratch, viewMatrix)

    }

    private fun clear() {
        // Redraw background color
        GLES31.glClearColor(0.0f, 0.0f, 0.0f, 1.0f)
        GLES31.glClearDepthf(1.0f)
        GLES31.glEnable(GLES31.GL_DEPTH_TEST)
        GLES31.glDepthFunc(GLES31.GL_LEQUAL)

        GLES31.glClear(GLES31.GL_COLOR_BUFFER_BIT or GLES31.GL_DEPTH_BUFFER_BIT)
    }

    override fun onSurfaceChanged(unused: GL10, width: Int, height: Int) {
        GLES31.glViewport(0, 0, width, height)

        val ratio: Float = width.toFloat() / height.toFloat()

        // this projection matrix is applied to object coordinates
        // in the onDrawFrame() method
        Matrix.perspectiveM(projectionMatrix, 0, 70f, ratio, 0.1f, 10f)
    }
}
