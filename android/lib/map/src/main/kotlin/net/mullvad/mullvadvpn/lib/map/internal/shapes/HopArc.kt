package net.mullvad.mullvadvpn.lib.map.internal.shapes

import android.opengl.GLES20
import androidx.compose.ui.graphics.Color
import java.nio.FloatBuffer
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
        val distance = start.distanceTo(end)
        val maxHeight = longArcMaxHeight(distance)

        val hopVector = end - start
        val baseHeight =
            Sphere.RADIUS + 0.00010f // Offset since each marker is hovering above the globe.

        for (i in 0..segments) {
            val progress = i.toFloat() / segments
            // We start drawing from start, and draw on part (t) until we reach then end.
            val point = start + hopVector * progress
            val unitVector = point.normalize()
            val height = baseHeight + maxHeight * progress * (1.0f - progress)

            val segmentPoint = unitVector * height

            val index = i * VERTEX_COMPONENT_SIZE
            vertices[index] = segmentPoint.x
            vertices[index + 1] = segmentPoint.y
            vertices[index + 2] = segmentPoint.z
        }
        return vertices
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
        (ARC_DISTANCE_FACTOR * distance).coerceIn(ARC_MIN_HEIGHT, ARC_MAX_HEIGHT)

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
        private const val ARC_DISTANCE_FACTOR = 0.1f
        private const val ARC_MIN_HEIGHT = 0.04f
        private const val ARC_MAX_HEIGHT = 0.40f
    }
}
