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


    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        // Set the background frame color
        // initialize a triangle
        globe = Globe(context)

        // The green color of the location marker when in the secured state
        val locationMarkerSecureColor = floatArrayOf(0.267f, 0.678f, 0.302f)
        // The red color of the location marker when in the unsecured state
        val locationMarkerUnsecureColor = floatArrayOf(0.89f, 0.251f, 0.224f)
        locationMarker = LocationMarker(locationMarkerSecureColor)
        locationMarker2 = LocationMarker(locationMarkerUnsecureColor)

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

    private val stockholmCoordinate = Coordinate(59.3293f, 18.0686f)
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
    private val zoom = 1.35f

    override fun onDrawFrame(gl10: GL10) {
        // Clear function
        clear()

        val viewMatrix = FloatArray(16)
        Matrix.setIdentityM(viewMatrix, 0)

        val offsetY = 0.088f + (zoom - connectedZoom) * 0.3f

        Matrix.translateM(viewMatrix, 0, 0f, offsetY, -zoom)

        val coordinate = gothenburgCoordinate

        Matrix.rotateM(viewMatrix, 0, coordinate.lat, 1f, 0f, 0f)
        Matrix.rotateM(viewMatrix, 0, coordinate.lon, 0f, -1f, 0f)

        globe.draw(projectionMatrix.copyOf(), viewMatrix.copyOf())
        locationMarker.draw(projectionMatrix, viewMatrix.copyOf(), gothenburgCoordinate, 0.02f)
        locationMarker2.draw(projectionMatrix.copyOf(), viewMatrix.copyOf(), stockholmCoordinate, 0.03f)
//        locationMarker3.draw(projectionMatrix, viewMatrix.copyOf(), helsinkiCoordinate, 0.03f)
    }

    private fun clear() {
        // Redraw background color
        // TODO Change to black
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
