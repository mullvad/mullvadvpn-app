package net.mullvad.mullvadvpn.lib.map.internal.shapes

import android.opengl.GLES20
import android.opengl.Matrix
import androidx.compose.ui.graphics.Color
import java.nio.FloatBuffer
import kotlin.math.cos
import kotlin.math.sin
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.internal.COLOR_COMPONENT_SIZE
import net.mullvad.mullvadvpn.lib.map.internal.VERTEX_COMPONENT_SIZE
import net.mullvad.mullvadvpn.lib.map.internal.initArrayBuffer
import net.mullvad.mullvadvpn.lib.map.internal.initShaderProgram
import net.mullvad.mullvadvpn.lib.map.internal.toFloatArray
import net.mullvad.mullvadvpn.model.LatLong

internal class LocationMarker(val colors: LocationMarkerColors) {

    private val shaderProgram: Int
    private val attribLocations: AttribLocations
    private val uniformLocation: UniformLocation
    private val positionBuffer: Int
    private val colorBuffer: Int
    private val ringSizes: List<Int>

    init {
        val rings = createRings()
        ringSizes = rings.map { (positions, _) -> positions.size }

        val positionFloatArray = joinMultipleArrays(rings.map { it.vertices })
        val positionFloatBuffer = FloatBuffer.wrap(positionFloatArray)

        val colorFloatArray = joinMultipleArrays(rings.map { it.verticesColor })
        val colorFloatBuffer = FloatBuffer.wrap(colorFloatArray)

        positionBuffer = initArrayBuffer(positionFloatBuffer)
        colorBuffer = initArrayBuffer(colorFloatBuffer)

        shaderProgram = initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(
                vertexPosition = GLES20.glGetAttribLocation(shaderProgram, "aVertexPosition"),
                vertexColor = GLES20.glGetAttribLocation(shaderProgram, "aVertexColor")
            )
        uniformLocation =
            UniformLocation(
                projectionMatrix = GLES20.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES20.glGetUniformLocation(shaderProgram, "uModelViewMatrix")
            )
    }

    fun draw(projectionMatrix: FloatArray, viewMatrix: FloatArray, latLong: LatLong, size: Float) {
        val modelViewMatrix = viewMatrix.copyOf()

        GLES20.glUseProgram(shaderProgram)

        Matrix.rotateM(modelViewMatrix, 0, latLong.longitude.value, 0f, 1f, 0f)
        Matrix.rotateM(modelViewMatrix, 0, latLong.latitude.value, -1f, 0f, 0f)

        Matrix.scaleM(modelViewMatrix, 0, size, size, 1f)

        // Translate marker to put it above the globe
        Matrix.translateM(modelViewMatrix, 0, 0f, 0f, MARKER_TRANSLATE_Z_FACTOR)

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

        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, colorBuffer)
        GLES20.glVertexAttribPointer(
            attribLocations.vertexColor,
            COLOR_COMPONENT_SIZE,
            GLES20.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES20.glEnableVertexAttribArray(attribLocations.vertexColor)

        GLES20.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, modelViewMatrix, 0)

        var offset = 0
        for (ringSize in ringSizes) {
            GLES20.glDrawArrays(GLES20.GL_TRIANGLE_FAN, offset, ringSize)
            // Add number off vertices in the ring to the offset
            offset += ringSize / VERTEX_COMPONENT_SIZE
        }
    }

    // Returns vertex positions and color values for a circle.
    // `offset` is a vector of x, y and z values determining how much to offset the circle
    // position from origo
    private fun circleFanVertices(
        numEdges: Int,
        radius: Float,
        offset: FloatArray = floatArrayOf(0.0f, 0.0f, 0.0f),
        centerColor: Color,
        ringColor: Color,
    ): Ring {
        require(numEdges > 2) { "Number of edges must be greater than 2" }

        // Edges + center + first point
        val points = numEdges + 2

        val positions = FloatArray(points * VERTEX_COMPONENT_SIZE)
        val positionsColor = FloatArray(points * COLOR_COMPONENT_SIZE)

        // Start adding the center the center point
        offset.forEachIndexed { index, value -> positions[index] = value }
        centerColor.toFloatArray().forEachIndexed { index, value -> positionsColor[index] = value }

        val ringColorArray = ringColor.toFloatArray()

        for (i in 1 until points) {

            val angle = (i.toFloat() / numEdges) * 2f * Math.PI
            val posIndex = i * VERTEX_COMPONENT_SIZE
            positions[posIndex] = offset[0] + radius * cos(angle).toFloat()
            positions[posIndex + 1] = offset[1] + radius * sin(angle).toFloat()
            positions[posIndex + 2] = offset[2]

            val colorIndex = i * COLOR_COMPONENT_SIZE
            ringColorArray.forEachIndexed { index, value ->
                positionsColor[colorIndex + index] = value
            }
        }

        return Ring(positions, positionsColor)
    }

    private fun joinMultipleArrays(arrays: List<FloatArray>): FloatArray {
        val result = FloatArray(arrays.sumOf { it.size })
        var offset = 0
        for (array in arrays) {
            array.copyInto(result, offset)
            offset += array.size
        }
        return result
    }

    @Suppress("MagicNumber")
    private fun createRings(): List<Ring> =
        listOf(
            circleFanVertices(
                32,
                0.5f,
                floatArrayOf(0.0f, 0.0f, 0.0f),
                colors.perimeterColors,
                colors.perimeterColors,
            ), // Semi-transparent outer
            circleFanVertices(
                16,
                0.28f,
                floatArrayOf(0.0f, -0.05f, 0.00001f),
                colors.shadowColor,
                colors.shadowColor.copy(alpha = 0.0f),
            ), // Shadow
            circleFanVertices(
                32,
                0.185f,
                floatArrayOf(0.0f, 0.0f, 0.00002f),
                colors.ringBorderColor,
                colors.ringBorderColor,
            ), // White ring
            circleFanVertices(
                32,
                0.15f,
                floatArrayOf(0.0f, 0.0f, 0.00003f),
                colors.centerColor,
                colors.centerColor,
            ) // Center colored circle
        )

    fun onRemove() {
        GLES20.glDeleteBuffers(2, intArrayOf(positionBuffer, colorBuffer), 0)
        GLES20.glDeleteProgram(shaderProgram)
    }

    private data class Ring(val vertices: FloatArray, val verticesColor: FloatArray)

    private data class AttribLocations(val vertexPosition: Int, val vertexColor: Int)

    private data class UniformLocation(val projectionMatrix: Int, val modelViewMatrix: Int)

    companion object {
        private const val MARKER_TRANSLATE_Z_FACTOR = 1.0001f

        // Vertex, and fragment shader code is taken from Mullvad Desktop 3dmap.ts
        private val vertexShaderCode =
            """
    attribute vec3 aVertexPosition;
    attribute vec4 aVertexColor;

    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;

    varying lowp vec4 vColor;

    void main(void) {
        gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aVertexPosition, 1.0);
        vColor = aVertexColor;
    }
        """
                .trimIndent()
        private val fragmentShaderCode =
            """
    varying lowp vec4 vColor;

    void main(void) {
        gl_FragColor = vColor;
    }
        """
                .trimIndent()
    }
}
