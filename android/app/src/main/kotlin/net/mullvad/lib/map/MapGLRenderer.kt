package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLES31
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import android.util.Log
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.tan
import net.mullvad.lib.map.data.Coordinate
import net.mullvad.lib.map.data.MapViewState
import net.mullvad.lib.map.data.Marker
import net.mullvad.lib.map.data.MarkerType
import net.mullvad.lib.map.shapes.Globe
import net.mullvad.lib.map.shapes.LocationMarker

class MapGLRenderer(val context: Context) : GLSurfaceView.Renderer {
    private lateinit var globe: Globe
    private lateinit var secureLocationMarker: LocationMarker
    private lateinit var unsecureLocationMarker: LocationMarker

    private val gothenburgCoordinate = Coordinate(57.7089f, 11.9746f)
    private var viewState: MapViewState =
        MapViewState(
            2.75f,
            gothenburgCoordinate,
            Marker(gothenburgCoordinate, MarkerType.UNSECURE),
            0f,
            false,
            70f
        )

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        // Set the background frame color
        // initialize a triangle
        globe = Globe(context)

        // The green color of the location marker when in the secured state
        val locationMarkerSecureColor = floatArrayOf(0.267f, 0.678f, 0.302f)
        // The red color of the location marker when in the unsecured state
        val locationMarkerUnsecureColor = floatArrayOf(0.89f, 0.251f, 0.224f)

        secureLocationMarker = LocationMarker(locationMarkerSecureColor)
        unsecureLocationMarker = LocationMarker(locationMarkerUnsecureColor)

        initGLOptions()
    }

    private fun initGLOptions() {
        GLES31.glEnable(GLES31.GL_CULL_FACE)
        GLES31.glCullFace(GLES31.GL_BACK)

        GLES31.glEnable(GLES31.GL_BLEND)
        GLES31.glBlendFunc(GLES31.GL_SRC_ALPHA, GLES31.GL_ONE_MINUS_SRC_ALPHA)
    }

    private val projectionMatrix = FloatArray(16).apply { Matrix.setIdentityM(this, 0) }

    override fun onDrawFrame(gl10: GL10) {
        // Clear function
        clear()

        val viewMatrix = FloatArray(16)
        Matrix.setIdentityM(viewMatrix, 0)

        val percent = viewState.percent

        val offsetY =
            if (!viewState.mode) {
                val z = viewState.zoom - 1f
                val planeSize = tan(viewState.fov.toRadians() / 2f) * z * 2f
                planeSize * (0.5f - percent)
            } else {
                0f
            }
        Matrix.translateM(viewMatrix, 0, 0f, offsetY, -viewState.zoom)
        Matrix.rotateM(viewMatrix, 0, viewState.cameraCoordinate.lat, 1f, 0f, 0f)
        Matrix.rotateM(viewMatrix, 0, viewState.cameraCoordinate.lon, 0f, -1f, 0f)

        val vP = projectionMatrix.copyOf()

        if (viewState.mode) {

            val cameraTilt =
                (0.5f - percent) *
                    viewState
                        .fov // atan2(distanceFromCenter, z) * 180f/Math.PI.toFloat() - 90f + fov
            Matrix.rotateM(vP, 0, cameraTilt, 1f, 0f, 0f)

            //        val distanceFromTop = planeSize * percent
            //        val distanceFromCenter = abs(distanceFromTop - planeSize/2f)
            //
        }

        globe.draw(vP.copyOf(), viewMatrix.copyOf())

        when (viewState.locationMarker.type) {
            MarkerType.SECURE ->
                secureLocationMarker.draw(
                    vP,
                    viewMatrix.copyOf(),
                    viewState.locationMarker.coordinate,
                    0.02f
                )
            MarkerType.UNSECURE ->
                unsecureLocationMarker.draw(
                    vP,
                    viewMatrix.copyOf(),
                    viewState.locationMarker.coordinate,
                    0.02f
                )
        }
    }

    private fun Float.toRadians() = this * Math.PI.toFloat() / 180f

    private fun clear() {
        // Redraw background color
        // TODO Change to black
        GLES31.glClearColor(0.0f, 0.0f, 0.0f, 1.0f)
        GLES31.glClearDepthf(1.0f)
        GLES31.glEnable(GLES31.GL_DEPTH_TEST)
        GLES31.glDepthFunc(GLES31.GL_LEQUAL)

        GLES31.glClear(GLES31.GL_COLOR_BUFFER_BIT or GLES31.GL_DEPTH_BUFFER_BIT)
    }

    fun onFovChanged(fov: Float, width: Int, height: Int) {
        val ratio: Float = width.toFloat() / height.toFloat()
        Matrix.perspectiveM(projectionMatrix, 0, viewState.fov, ratio, 0.05f, 10f)
    }

    override fun onSurfaceChanged(unused: GL10, width: Int, height: Int) {
        GLES31.glViewport(0, 0, width, height)

        val ratio: Float = width.toFloat() / height.toFloat()

        // this projection matrix is applied to object coordinates
        // in the onDrawFrame() method
        Log.d("MullvadMap", "Ratio: $ratio")
        Matrix.perspectiveM(projectionMatrix, 0, viewState.fov, ratio, 0.05f, 10f)
    }

    fun setViewState(viewState: MapViewState) {
        this.viewState = viewState
    }
}
