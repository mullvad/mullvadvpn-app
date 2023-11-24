package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLES31
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import android.os.SystemClock
import android.renderscript.Matrix4f
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import net.mullvad.lib.map.shapes.Globe

class MyGLRenderer(val context: Context) : GLSurfaceView.Renderer {
    private lateinit var globe: Globe

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        // Set the background frame color
        GLES31.glClearColor(0.0f, 0.0f, 0.0f, 1.0f)
        // initialize a triangle
        globe = Globe(context)
    }


    private val vPMatrix = FloatArray(16)
    private val projectionMatrix = FloatArray(16)
    private val viewMatrix = FloatArray(16)
    private val rotationMatrix = FloatArray(16)


    override fun onDrawFrame(gl10: GL10) {
        // Redraw background color
        GLES31.glClear(GLES31.GL_COLOR_BUFFER_BIT)

        val scratch = FloatArray(16)

        Matrix.setLookAtM(viewMatrix, 0, 0f, 0f, 3f, 0f, 0f, 0f, 0f, 1.0f, 0.0f)
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
        Matrix.setRotateM(rotationMatrix, 0, angle, 0f, 0f, -1.0f)

        // Combine the rotation matrix with the projection and camera view
        // Note that the vPMatrix factor *must be first* in order
        // for the matrix multiplication product to be correct.
        Matrix.multiplyMM(scratch, 0, vPMatrix, 0, rotationMatrix, 0)

        // Draw triangle

        globe.draw(projectionMatrix, viewMatrix)


    }

    override fun onSurfaceChanged(unused: GL10, width: Int, height: Int) {
        GLES31.glViewport(0, 0, width, height)


        val ratio: Float = width.toFloat() / height.toFloat()

        // this projection matrix is applied to object coordinates
        // in the onDrawFrame() method
        Matrix.perspectiveM(projectionMatrix, 0, 70f, ratio, 0.1f, 10f)

    }
}
