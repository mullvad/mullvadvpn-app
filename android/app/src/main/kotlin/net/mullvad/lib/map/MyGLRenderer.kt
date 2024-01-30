package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLES31
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import net.mullvad.lib.map.shapes.Globe
import net.mullvad.lib.map.shapes.LocationMarker

class MyGLRenderer(val context: Context) : GLSurfaceView.Renderer {
    private lateinit var globe: Globe
    private lateinit var locationMarker: LocationMarker
    private lateinit var locationMarker2: LocationMarker
    private lateinit var locationMarker3: LocationMarker


    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        // Set the background frame color
        // initialize a triangle
        globe = Globe(context)
        locationMarker = LocationMarker(floatArrayOf(1.0f, 1.0f, 0.0f))
        locationMarker2 = LocationMarker(floatArrayOf(1.0f, 0.0f, 0.0f))
        locationMarker3 = LocationMarker(floatArrayOf(0.0f, 0.0f, 1.0f))

        initGLOptions()
    }

    private fun initGLOptions() {
        GLES31.glEnable(GLES31.GL_CULL_FACE)
        GLES31.glCullFace(GLES31.GL_BACK)

        GLES31.glEnable(GLES31.GL_BLEND)
        GLES31.glBlendFunc(GLES31.GL_SRC_ALPHA, GLES31.GL_ONE_MINUS_SRC_ALPHA)
    }

    private val projectionMatrix = FloatArray(16).apply {
        Matrix.setIdentityM(this, 0)
    }

    private val gothenburgCoordinate = Coordinate(57.67f, 11.98f)
    private val helsinkiCoordinate = Coordinate(60.170834f, 24.9375f)
    private val sydneyCoordinate = Coordinate(-33.86f, 151.21f)
    private val losAngelesCoordinate = Coordinate(34.05f, -118.25f)
    private val newYorkCoordinate = Coordinate(40.73f, -73.93f)
    private val romeCoordinate = Coordinate(41.893f, 12.482f)
    private val poleCoordinate1 = Coordinate(88f, -90f)
    private val poleCoordinate2 = Coordinate(88f, 90f)
    private val antarcticaCoordinate = Coordinate(-85f, 0f)

    private val connectedZoom = 1.25f
    private val zoom = 3.35f

    override fun onDrawFrame(gl10: GL10) {
        // Clear function
        clear()

        val viewMatrix = FloatArray(16)
        Matrix.setIdentityM(viewMatrix, 0)

        val offsetY = 0.088f + (zoom - connectedZoom) * 0.3f

        Matrix.translateM(viewMatrix, 0, 0f, offsetY, -zoom)

        val coordinate = gothenburgCoordinate

        val time = System.currentTimeMillis() % 4000
        val angleInDegrees = 360.0f / 4000.0f * time.toInt()

        Matrix.rotateM(viewMatrix, 0, angleInDegrees, 1f, 0f, 0f)
        Matrix.rotateM(viewMatrix, 0, coordinate.lon, 0f, -1f, 0f)

        globe.draw(projectionMatrix.copyOf(), viewMatrix.copyOf())
        locationMarker2.draw(projectionMatrix.copyOf(), viewMatrix.copyOf(), gothenburgCoordinate, 0.13f)
//        locationMarker3.draw(projectionMatrix, viewMatrix.copyOf(), helsinkiCoordinate, 0.03f)
//        locationMarker.draw(projectionMatrix, viewMatrix.copyOf(), sydneyCoordinate, 0.02f)
    }

    private fun clear() {
        // Redraw background color
        // TODO Change to black
        GLES31.glClearColor(0.0f, 1.0f, 1.0f, 1.0f)
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
