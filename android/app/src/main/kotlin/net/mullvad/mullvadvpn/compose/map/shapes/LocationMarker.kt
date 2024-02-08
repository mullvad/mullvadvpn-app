package net.mullvad.mullvadvpn.compose.map.shapes

import android.opengl.GLES20
import android.opengl.Matrix
import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.compose.map.internal.COLOR_COMPONENT_SIZE
import java.nio.FloatBuffer
import kotlin.math.cos
import kotlin.math.sin
import net.mullvad.mullvadvpn.compose.map.internal.VERTEX_COMPONENT_SIZE
import net.mullvad.mullvadvpn.compose.map.data.LatLng
import net.mullvad.mullvadvpn.compose.map.internal.initArrayBuffer
import net.mullvad.mullvadvpn.compose.map.internal.initShaderProgram
import net.mullvad.mullvadvpn.compose.map.internal.toFloatArrayWithoutAlpha

class LocationMarker(val color: Color) {

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

    private val white = floatArrayOf(1.0f, 1.0f, 1.0f)
    private val black = floatArrayOf(0.0f, 0.0f, 0.0f)
    private val rings =
        listOf(
            circleFanVertices(
                32,
                0.5f,
                floatArrayOf(0.0f, 0.0f, 0.0f),
                floatArrayOf(*color.toFloatArrayWithoutAlpha(), 0.4f),
                floatArrayOf(*color.toFloatArrayWithoutAlpha(), 0.4f)
            ), // Semi-transparent outer
            circleFanVertices(
                16,
                0.28f,
                floatArrayOf(0.0f, -0.05f, 0.00001f),
                floatArrayOf(*black, 0.55f),
                floatArrayOf(*black, 0.0f)
            ), // shadow
            circleFanVertices(
                32,
                0.185f,
                floatArrayOf(0.0f, 0.0f, 0.00002f),
                floatArrayOf(*white, 1f),
                floatArrayOf(*white, 1f)
            ), // white ring
            circleFanVertices(
                32,
                0.15f,
                floatArrayOf(0.0f, 0.0f, 0.00003f),
                floatArrayOf(*color.toFloatArrayWithoutAlpha(), 1f),
                floatArrayOf(*color.toFloatArrayWithoutAlpha(), 1f),
            ) // Center colored circle
        )

    private val shaderProgram: Int
    private val attribLocations: AttribLocations
    private val uniformLocation: UniformLocation

    data class AttribLocations(val vertexPosition: Int, val vertexColor: Int)

    data class UniformLocation(val projectionMatrix: Int, val modelViewMatrix: Int)

    private val positionBuffer: Int
    private val colorBuffer: Int
    private val ringSizes: List<Int> = rings.map { (positions, _) -> positions.size }

    init {
        val positionArrayBuffer = rings.flatMap { it.first }
        val positionByteBuffer = FloatBuffer.wrap(positionArrayBuffer.toFloatArray())

        val colorArrayBuffer = rings.flatMap { it.second }
        val colorByteBuffer = FloatBuffer.wrap(colorArrayBuffer.toFloatArray())

        positionBuffer = initArrayBuffer(positionByteBuffer)
        colorBuffer = initArrayBuffer(colorByteBuffer)

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

    fun draw(projectionMatrix: FloatArray, viewMatrix: FloatArray, latLng: LatLng, size: Float) {
        val modelViewMatrix = viewMatrix.copyOf()

        GLES20.glUseProgram(shaderProgram)

        Matrix.rotateM(modelViewMatrix, 0, latLng.longitude.value, 0f, 1f, 0f)
        Matrix.rotateM(modelViewMatrix, 0, -latLng.latitude.value, 1f, 0f, 0f)

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
        offset: FloatArray,
        centerColor: FloatArray,
        ringColor: FloatArray,
    ): Pair<List<Float>, List<Float>> {
        val positions = mutableListOf(*offset.toTypedArray())
        val colors = mutableListOf(*centerColor.toTypedArray())

        for (i in 0..numEdges) {

            val angle = (i.toFloat() / numEdges.toFloat()) * 2f * Math.PI
            val x = offset[0] + radius * cos(angle).toFloat()
            val y = offset[1] + radius * sin(angle).toFloat()
            val z = offset[2]
            positions.add(x)
            positions.add(y)
            positions.add(z)
            colors.addAll(ringColor.toTypedArray())
        }
        return positions.toList() to colors.toList()
    }

    companion object {
        private const val MARKER_TRANSLATE_Z_FACTOR = 1.0001f
    }
}
