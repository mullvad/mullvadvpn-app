package net.mullvad.mullvadvpn.lib.map.internal.shapes

import android.opengl.GLES20
import androidx.compose.ui.graphics.Color
import java.nio.FloatBuffer
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin
import net.mullvad.mullvadvpn.lib.map.data.Sphere
import net.mullvad.mullvadvpn.lib.map.data.Vector3
import net.mullvad.mullvadvpn.lib.map.internal.VERTEX_COMPONENT_SIZE
import net.mullvad.mullvadvpn.lib.map.internal.initGLArrayBuffer
import net.mullvad.mullvadvpn.lib.map.internal.initShaderProgram
import net.mullvad.mullvadvpn.lib.model.LatLong
import net.mullvad.mullvadvpn.lib.model.toRadians

class HopArc(
    val from: LatLong,
    val to: LatLong,
    val color: Color,
    private val segments: Int = DEFAULT_SEGMENT_SIZE,
) {
    private val shaderProgram: Int
    private val attribLocations: AttribLocations
    private val uniformLocation: UniformLocation
    private val positionBuffer: Int
    private val colorArray: FloatArray

    init {
        val start = from.toWorldVector3()
        val end = to.toWorldVector3()

        val vertices = buildArcVertices(start, end, segments)

        val positionFloatBuffer = FloatBuffer.wrap(vertices)
        positionBuffer = initGLArrayBuffer(positionFloatBuffer)

        shaderProgram = initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(
                vertexPosition = GLES20.glGetAttribLocation(shaderProgram, "aVertexPosition")
            )

        uniformLocation =
            UniformLocation(
                color = GLES20.glGetUniformLocation(shaderProgram, "uColor"),
                projectionMatrix = GLES20.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES20.glGetUniformLocation(shaderProgram, "uModelViewMatrix"),
            )

        colorArray = floatArrayOf(color.red, color.green, color.blue, color.alpha)
    }

    private fun buildArcVertices(
        start: Vector3,
        end: Vector3,
        segments: Int,
    ): FloatArray {
        val vertices = FloatArray((segments + 1) * VERTEX_COMPONENT_SIZE)

        // If the distance is short/zero we want a more drop like arc shape, where the curve goes
        // in the opposite direction in the beginning to avoid it becoming a straight line up and
        // down.
        val distance = start.distanceTo(end)
        val isShortArc = distance < SHORT_ARC_CUTOFF_DISTANCE
        val maxHeight = if (isShortArc) SHORT_ARC_MAX_HEIGHT else longArcMaxHeight(distance)
        // If it is a short arc we create a baseTangentVector that is used to offset in width later
        val shortArcTangentVector = if (isShortArc) baseTangentVector(start, end) else null

        val hopVector = end - start
        val baseHeight =
            Sphere.RADIUS + 0.00010f // Offset since each marker is hovering above the globe.

        for (i in 0..segments) {
            val progress = i.toFloat() / segments
            // We start drawing from start, and draw on part (t) until we reach then end.
            val point = start + hopVector * progress
            val unitVector = point.normalize()
            val height = baseHeight + maxHeight * progress * (1.0f - progress)

            var segmentPoint = unitVector * height
            // If we have a short vector we need to apply the tangent vector to ensure
            // we don't end up with a line that goes straight up and down. Here we add
            // the drop shape
            if (shortArcTangentVector != null) {
                val angle = (PI * progress).toFloat()
                val width = -SHORT_ARC_MAX_WIDTH * sin(angle) * cos(angle)
                segmentPoint += (shortArcTangentVector * width)
            }

            val index = i * VERTEX_COMPONENT_SIZE
            vertices[index] = segmentPoint.x
            vertices[index + 1] = segmentPoint.y
            vertices[index + 2] = segmentPoint.z
        }
        return vertices
    }

    /**
     * Returns a unit vector tangent to the sphere at [start], pointing towards [end].
     *
     * Used to give the short-arc drop loop a stable sideways direction to bulge into. When [start]
     * and [end] are the same point a tangent along the longitude lines are chosen.
     */
    private fun baseTangentVector(start: Vector3, end: Vector3): Vector3 {
        val normal = start.normalize()
        val diffVector = (end - start)
        val towardsEnd = diffVector - normal * diffVector.dot(normal)

        val tangent =
            if (start == end)
                // We don't care to handle if it is close to North/South Pole
                normal.cross(Vector3(0f, 1f, 0f))
            else towardsEnd

        return tangent.normalize()
    }

    fun draw(projectionMatrix: FloatArray, viewMatrix: FloatArray, lineWidth: Float = 4f) {
        GLES20.glUseProgram(shaderProgram)

        GLES20.glLineWidth(lineWidth)

        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, positionBuffer)
        GLES20.glVertexAttribPointer(
            attribLocations.vertexPosition,
            VERTEX_COMPONENT_SIZE,
            GLES20.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES20.glEnableVertexAttribArray(attribLocations.vertexPosition)

        GLES20.glUniform4fv(uniformLocation.color, 1, colorArray, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, viewMatrix, 0)

        GLES20.glDrawArrays(GLES20.GL_LINE_STRIP, 0, segments + 1)

        GLES20.glDisableVertexAttribArray(attribLocations.vertexPosition)
    }

    fun onRemove() {
        GLES20.glDeleteBuffers(1, intArrayOf(positionBuffer), 0)
        GLES20.glDeleteProgram(shaderProgram)
    }

    private fun LatLong.toWorldVector3(): Vector3 {
        val phi = this.latitude.value.toRadians()
        val theta = this.longitude.value.toRadians()

        // To match location markers precisely, we do NOT negate x.
        val x = cos(phi) * sin(theta)
        val y = sin(phi)
        val z = cos(phi) * cos(theta)

        return Vector3(x, y, z)
    }

    private fun longArcMaxHeight(distance: Float): Float =
        (LONG_ARC_DISTANCE_FACTOR * distance).coerceIn(LONG_ARC_MIN_HEIGHT, LONG_ARC_MAX_HEIGHT)

    private data class AttribLocations(val vertexPosition: Int)

    private data class UniformLocation(
        val color: Int,
        val projectionMatrix: Int,
        val modelViewMatrix: Int,
    )

    companion object {
        private val vertexShaderCode =
            """
            attribute vec3 aVertexPosition;

            uniform mat4 uModelViewMatrix;
            uniform mat4 uProjectionMatrix;

            void main(void) {
                gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aVertexPosition, 1.0);
            }
            """
                .trimIndent()

        private val fragmentShaderCode =
            """
            precision mediump float;
            uniform vec4 uColor;

            void main(void) {
                gl_FragColor = uColor;
            }
            """
                .trimIndent()

        private const val DEFAULT_SEGMENT_SIZE = 48
        private const val SHORT_ARC_CUTOFF_DISTANCE = 0.02f
        private const val SHORT_ARC_MAX_HEIGHT = 0.08f
        private const val SHORT_ARC_MAX_WIDTH = 0.03f

        private const val LONG_ARC_DISTANCE_FACTOR = 0.1f
        private const val LONG_ARC_MIN_HEIGHT = 0.08f
        private const val LONG_ARC_MAX_HEIGHT = 0.40f
    }
}
