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
import net.mullvad.mullvadvpn.lib.map.internal.shapes.Globe
import net.mullvad.mullvadvpn.lib.map.internal.shapes.LocationMarker
import net.mullvad.mullvadvpn.lib.model.toRadians

typealias Point3D = Triple<Float, Float, Float>

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

    private val projectionMatrix = newIdentityMatrix()

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
    }

    fun isOnGlobe(offset: Offset): Vector3? {
        val cameraz = -viewState.cameraPosition.zoom
        val camerax = 0f
        val cameray = 0f

        val camera = Vector3(camerax, cameray, cameraz)

        val sphere = Sphere(Vector3(0f, 0f, 0f), 1f)
        val ratio: Float = viewPortSize.width.toFloat() / viewPortSize.height.toFloat()

        val directionVector = calculateDirectionVector(viewState.cameraPosition.fov,
        ratio, viewPortSize.width, viewPortSize.height, offset.x, offset.y, nearPlaneDistance = PERSPECTIVE_Z_NEAR)

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
            val point1 =  ray.origin + ray.direction * t
            val point2 =  ray.origin + ray.direction * t2
            Logger.d("Intersection point1: $point1, point2: $point2")
            return ray.origin + ray.direction * t
        }
    }

    fun calculateDirectionVector(
        fovy: Float,
        aspectRatio: Float,
        viewportWidth: Float,
        viewportHeight: Float,
        tapScreenX: Float,
        tapScreenY: Float,
        nearPlaneDistance: Float = 1.0f
    ): Vector3 {
        val halfHeight = tan(fovy / 2.0f) * nearPlaneDistance
        val halfWidth = halfHeight * aspectRatio
        val x = (2.0f * tapScreenX / viewportWidth - 1.0f) * halfWidth
        val y = (1.0f - 2.0f * tapScreenY / viewportHeight) * halfHeight
        val z = -nearPlaneDistance
        return normalize(Vector3(x, y, z))
    }

    fun normalize(vector: Vector3): Vector3 {
        val length = sqrt(vector.x * vector.x + vector.y * vector.y + vector.z * vector.z)
        return Vector3(vector.x / length, vector.y / length, vector.z / length)
    }

    companion object {
        private const val PERSPECTIVE_Z_NEAR = 0.05f
        private const val PERSPECTIVE_Z_FAR = 10f
        private const val FIELD_OF_VIEW = 70f
    }
}


data class Vector3(val x: Float, val y: Float, val z: Float) {
    fun dot(other: Vector3): Float {
        return x * other.x + y * other.y + z * other.z
    }

    operator fun minus(other: Vector3): Vector3 {
        return Vector3(x - other.x, y - other.y, z - other.z)
    }

    operator fun times(scalar: Float): Vector3 {
        return Vector3(x * scalar, y * scalar, z * scalar)
    }

    operator fun plus(other: Vector3): Vector3 {
        return Vector3(x + other.x, y + other.y, z + other.z)
    }
}
data class Ray(val origin: Vector3, val direction: Vector3)
data class Sphere(val center: Vector3, val radius: Float)

