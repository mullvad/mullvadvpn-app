package net.mullvad.mullvadvpn.lib.map.internal

import android.content.res.Resources
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import android.util.Log
import androidx.collection.LruCache
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.tan
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.internal.shapes.Globe
import net.mullvad.mullvadvpn.lib.map.internal.shapes.LocationMarker
import net.mullvad.mullvadvpn.model.COMPLETE_ANGLE

internal class MapGLRenderer(private val resources: Resources) : GLSurfaceView.Renderer {

    private lateinit var globe: Globe

    // Due to location markers themselves containing colors we cache them to avoid recreating them
    // for every draw call.
    private val markerCache: LruCache<LocationMarkerColors, LocationMarker> =
        object : LruCache<LocationMarkerColors, LocationMarker>(100) {
            override fun entryRemoved(
                evicted: Boolean,
                key: LocationMarkerColors,
                oldValue: LocationMarker,
                newValue: LocationMarker?
            ) {
                Log.d("MullvadMap", "Remove marker! $oldValue")
                oldValue.onRemove()
            }
        }

    private lateinit var viewState: MapViewState

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        globe = Globe(resources)
        markerCache.evictAll()
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

        val yOffset = toOffsetY(viewState.cameraPosition)

        Matrix.translateM(viewMatrix, 0, 0f, yOffset, -viewState.cameraPosition.zoom)

        Matrix.rotateM(viewMatrix, 0, viewState.cameraPosition.latLong.latitude.value, 1f, 0f, 0f)
        Matrix.rotateM(viewMatrix, 0, viewState.cameraPosition.latLong.longitude.value, 0f, -1f, 0f)

        val vp = viewMatrix.copyOf()
        globe.draw(projectionMatrix, vp, viewState.globeColors)

        Log.d("MullvadMap", "viewState.locationMarker.size: ${viewState.locationMarker.size}")
        Log.d("MullvadMap", "MarkerCache: $markerCache")
        viewState.locationMarker.forEach {
            val marker =
                markerCache[it.colors]
                    ?: LocationMarker(it.colors).also {
                        Log.d("MullvadMap", "Cache marker! ${it.colors}")
                        markerCache.put(it.colors, it)
                    }
            //            val marker = LocationMarker(it.colors)

            Log.d("MullvadMap", "Draw marker! $marker ${it.latLong} ${it.size}")
            marker.draw(projectionMatrix.copyOf(), viewMatrix.copyOf(), it.latLong, it.size)
        }
    }

    private fun Float.toRadians() = this * Math.PI.toFloat() / (COMPLETE_ANGLE / 2)

    private fun toOffsetY(cameraPosition: CameraPosition): Float {
        val percent = cameraPosition.bias
        val z = cameraPosition.zoom - 1f
        // Calculate the size of the plane at the current z position
        val planeSize = tan(FIELD_OF_VIEW.toRadians() / 2f) * z * 2f
        return planeSize * (0.5f - percent)
    }

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
