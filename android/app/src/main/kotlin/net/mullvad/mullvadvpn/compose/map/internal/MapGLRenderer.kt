package net.mullvad.mullvadvpn.compose.map.internal

import android.content.res.Resources
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import androidx.compose.ui.graphics.Color
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.tan
import net.mullvad.mullvadvpn.compose.map.data.MapViewState
import net.mullvad.mullvadvpn.compose.map.data.MarkerType
import net.mullvad.mullvadvpn.compose.map.internal.shapes.Globe
import net.mullvad.mullvadvpn.compose.map.internal.shapes.LocationMarker

class MapGLRenderer(private val resources: Resources, private val mapConfig: MapConfig) :
    GLSurfaceView.Renderer {
    private lateinit var secureLocationMarker: LocationMarker
    private lateinit var unsecureLocationMarker: LocationMarker

    private lateinit var globe: Globe

    private lateinit var viewState: MapViewState

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        globe = Globe(resources)

        secureLocationMarker = LocationMarker(mapConfig.secureMarkerColor)
        unsecureLocationMarker = LocationMarker(mapConfig.unsecureMarkerColor)

        initGLOptions()
    }

    private fun initGLOptions() {
        GLES20.glEnable(GLES20.GL_CULL_FACE)
        GLES20.glCullFace(GLES20.GL_BACK)

        GLES20.glEnable(GLES20.GL_BLEND)
        GLES20.glBlendFunc(GLES20.GL_SRC_ALPHA, GLES20.GL_ONE_MINUS_SRC_ALPHA)
    }

    private val projectionMatrix = newIdentityMatrix()

    override fun onDrawFrame(gl10: GL10) {
        // Clear canvas
        clear()

        val viewMatrix = newIdentityMatrix()

        val percent = viewState.percent

        val z = viewState.zoom - 1f
        val planeSize = tan(FIELD_OF_VIEW.toRadians() / 2f) * z * 2f
        val offsetY = planeSize * (0.5f - percent)
        Matrix.translateM(viewMatrix, 0, 0f, offsetY, -viewState.zoom)
        Matrix.rotateM(viewMatrix, 0, viewState.cameraLatLng.latitude.value, 1f, 0f, 0f)
        Matrix.rotateM(viewMatrix, 0, viewState.cameraLatLng.longitude.value, 0f, -1f, 0f)


        val vp = viewMatrix.copyOf()
        globe.draw(projectionMatrix, vp, mapConfig.globeColors)

        viewState.locationMarker?.let {
            when (it.type) {
                MarkerType.SECURE ->
                    secureLocationMarker.draw(projectionMatrix, vp, it.latLng, 0.02f)
                MarkerType.UNSECURE ->
                    unsecureLocationMarker.draw(projectionMatrix, vp, it.latLng, 0.02f)
            }
        }
    }

    private fun Float.toRadians() = this * Math.PI.toFloat() / (COMPLETE_ANGLE / 2)

    private fun clear() {
        // Redraw background color
        GLES20.glClearColor(0.0f, 0.0f, 0.0f, 1.0f)
        GLES20.glClearDepthf(1.0f)
        GLES20.glEnable(GLES20.GL_DEPTH_TEST)
        GLES20.glDepthFunc(GLES20.GL_LEQUAL)

        GLES20.glClear(GLES20.GL_COLOR_BUFFER_BIT or GLES20.GL_DEPTH_BUFFER_BIT)
    }

    override fun onSurfaceChanged(unused: GL10, width: Int, height: Int) {
        GLES20.glViewport(0, 0, width, height)

        val ratio: Float = width.toFloat() / height.toFloat()
        Matrix.perspectiveM(
            projectionMatrix,
            0,
            FIELD_OF_VIEW,
            ratio,
            PERSPECTIVE_Z_NEAR,
            PERSPECTIVE_Z_FAR
        )
    }

    fun setViewState(viewState: MapViewState) {
        this.viewState = viewState
    }

    companion object {
        private const val PERSPECTIVE_Z_NEAR = 0.05f
        private const val PERSPECTIVE_Z_FAR = 10f
        private const val FIELD_OF_VIEW = 70f
    }
}

