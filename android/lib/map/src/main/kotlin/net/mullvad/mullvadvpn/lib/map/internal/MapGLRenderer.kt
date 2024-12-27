package net.mullvad.mullvadvpn.lib.map.internal

import android.content.res.Resources
import android.opengl.GLES20
import android.opengl.GLSurfaceView
import android.opengl.Matrix
import androidx.collection.LruCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import co.touchlab.kermit.Logger
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.pow
import kotlin.math.sqrt
import kotlin.math.tan
import net.mullvad.mullvadvpn.lib.map.data.CameraPosition
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.MapViewState
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.map.internal.shapes.Globe
import net.mullvad.mullvadvpn.lib.map.internal.shapes.LocationMarker
import net.mullvad.mullvadvpn.lib.model.map.Ray
import net.mullvad.mullvadvpn.lib.model.map.Sphere
import net.mullvad.mullvadvpn.lib.model.map.Vector3
import net.mullvad.mullvadvpn.lib.model.map.rotateAroundX
import net.mullvad.mullvadvpn.lib.model.map.rotateAroundY
import net.mullvad.mullvadvpn.lib.model.map.toLatLng
import net.mullvad.mullvadvpn.lib.model.map.toVector3
import net.mullvad.mullvadvpn.lib.model.toRadians

internal class MapGLRenderer(private val resources: Resources) : GLSurfaceView.Renderer {

    private lateinit var globe: Globe
    private var viewPortSize: Size = Size(0f, 0f)
    private val radius: Float = 1f

    // Due to location markers themselves containing colors we cache them to avoid recreating them
    // for every draw call.
    private val markerCache: LruCache<LocationMarkerColors, LocationMarker> =
        object : LruCache<LocationMarkerColors, LocationMarker>(100) {
            override fun entryRemoved(
                evicted: Boolean,
                key: LocationMarkerColors,
                oldValue: LocationMarker,
                newValue: LocationMarker?,
            ) {
                oldValue.onRemove()
            }
        }

    private lateinit var viewState: MapViewState
    private val projectionMatrix = newIdentityMatrix()
    private var globalViewMatrix = newIdentityMatrix()

    override fun onSurfaceCreated(unused: GL10, config: EGLConfig) {
        globe = Globe(resources)
        markerCache.evictAll()
        initGLOptions()
    }

    private fun initGLOptions() {
        // Enable cull face (To not draw the backside of triangles)
        GLES20.glEnable(GLES20.GL_CULL_FACE)
        GLES20.glCullFace(GLES20.GL_BACK)

        // Enable blend
        GLES20.glEnable(GLES20.GL_BLEND)
        GLES20.glBlendFunc(GLES20.GL_SRC_ALPHA, GLES20.GL_ONE_MINUS_SRC_ALPHA)
    }

    override fun onDrawFrame(gl10: GL10) {
        // Clear canvas
        clear()

        val viewMatrix = newIdentityMatrix()

        // Adjust zoom & vertical bias
        val yOffset = toOffsetY(viewState.cameraPosition)
        Matrix.translateM(viewMatrix, 0, 0f, yOffset, -viewState.cameraPosition.zoom)

        // Rotate to match the camera position
        Matrix.rotateM(viewMatrix, 0, viewState.cameraPosition.latLong.latitude.value, 1f, 0f, 0f)
        Matrix.rotateM(viewMatrix, 0, viewState.cameraPosition.latLong.longitude.value, 0f, -1f, 0f)

        globalViewMatrix = viewMatrix.copyOf()
        globe.draw(projectionMatrix, viewMatrix, viewState.globeColors)

        // Draw location markers
        viewState.locationMarker.forEach {
            val marker =
                markerCache[it.colors]
                    ?: LocationMarker(it.colors).also { markerCache.put(it.colors, it) }

            marker.draw(projectionMatrix, viewMatrix, it.latLong, it.size)
        }
    }

    private fun toOffsetY(cameraPosition: CameraPosition): Float {
        //        val percent = cameraPosition.verticalBias
        val z = cameraPosition.zoom - 1f

        // Calculate the size of the plane at the current z position
        val planeSizeY = tan(cameraPosition.fov.toRadians() / 2f) * z * 2f

        // Calculate the start of the plane
        val planeStartY = planeSizeY / 2f

        // Return offset based on the bias
        return 0f // planeStartY - planeSizeY * percent
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

        viewPortSize = Size(width.toFloat(), height.toFloat())

        val ratio: Float = width.toFloat() / height.toFloat()

        if (ratio.isFinite()) {
            Matrix.perspectiveM(
                projectionMatrix,
                0,
                FIELD_OF_VIEW,
                ratio,
                PERSPECTIVE_Z_NEAR,
                PERSPECTIVE_Z_FAR,
            )
        }
    }

    fun setViewState(viewState: MapViewState) {
        this.viewState = viewState
        markerVector = viewState.locationMarker.map { it.latLong.toVector3() to it }.toMap()
    }

    var markerVector = mapOf<Vector3, Marker>()

    fun closestMarker(offset: Offset): Pair<Marker?, Float>? {
        val cameraz = -viewState.cameraPosition.zoom
        val camerax = 0f
        val cameray = 0f

        val camera = Vector3(camerax, cameray, cameraz)

        val sphere = Sphere(Vector3(0f, 0f, 0f), 1f)
        val ratio: Float = viewPortSize.width.toFloat() / viewPortSize.height.toFloat()

        val directionVector =
            calculateDirectionVector(
                viewState.cameraPosition.fov,
                ratio,
                viewPortSize.width,
                viewPortSize.height,
                offset.x,
                offset.y,
                nearPlaneDistance = PERSPECTIVE_Z_NEAR,
            )

        val ray = Ray(camera, directionVector)

        val oc = ray.origin - sphere.center // Vector from ray origin to sphere center
        val a = ray.direction.dot(ray.direction)
        val b = 2f * oc.dot(ray.direction)
        val c = oc.dot(oc) - sphere.radius.pow(2f)
        val discriminant = b.pow(2f) - 4f * a * c

        if (discriminant < 0f) {
            return null // No intersection
        } else {
            val t = (-b - sqrt(discriminant)) / (2f * a) // Closest intersection point
            val t2 = (-b + sqrt(discriminant)) / (2f * a) // Closest intersection point
            Logger.d("Intersection t1: $t, t2: $t2")
            val point2 = ray.origin + ray.direction * t2

            val newPosition =
                point2
                    .rotateAroundX(-viewState.cameraPosition.latLong.latitude.value)
                    .rotateAroundY(viewState.cameraPosition.latLong.longitude.value)

            Logger.d("Intersection point2: $point2")
            Logger.d("Intersection real vector: $newPosition")
            Logger.d("Clicked lat lng: ${newPosition.toLatLng()}")

            val closestMarker = markerVector.minByOrNull { it.key.distanceTo(newPosition) }

            if (closestMarker != null) {
                return closestMarker.value to closestMarker.key.distanceTo(newPosition)
            }

            return null
        }
    }

    fun calculateDirectionVector(
        fovy: Float,
        aspectRatio: Float,
        viewportWidth: Float,
        viewportHeight: Float,
        tapScreenX: Float,
        tapScreenY: Float,
        nearPlaneDistance: Float = 1.0f,
    ): Vector3 {
        val halfHeight = tan(fovy.toRadians() / 2.0f) * nearPlaneDistance
        val halfWidth = halfHeight * aspectRatio
        val x = (2.0f * tapScreenX / viewportWidth - 1.0f) * halfWidth
        val y = (1.0f - 2.0f * tapScreenY / viewportHeight) * halfHeight
        val z = -nearPlaneDistance
        return Vector3(x, y, z).normalize()
    }

    companion object {
        private const val PERSPECTIVE_Z_NEAR = 0.05f
        private const val PERSPECTIVE_Z_FAR = 10f
        private const val FIELD_OF_VIEW = 70f
    }
}
