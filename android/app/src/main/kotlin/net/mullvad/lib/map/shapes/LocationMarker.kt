package net.mullvad.lib.map.shapes

import android.opengl.GLES31
import android.opengl.Matrix
import java.nio.ByteBuffer
import kotlin.math.cos
import kotlin.math.sin
import net.mullvad.lib.map.Coordinate

class LocationMarker(val color: FloatArray) {

    // The green color of the location marker when in the secured state
    val locationMarkerSecureColor = floatArrayOf(0.267f, 0.678f, 0.302f)
    // The red color of the location marker when in the unsecured state
    val locationMarkerUnsecureColor = floatArrayOf(0.89f, 0.251f, 0.224f)

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
                floatArrayOf(*color, 0.4f),
                floatArrayOf(*color, 0.4f)
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
                floatArrayOf(*color, 1f),
                floatArrayOf(*color, 1f),
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
        val positionArrayBuffer = rings.map { it.first }.flatten()
        val positionByteBuffer =
            ByteBuffer.allocate(positionArrayBuffer.size * 4).also { byteBuffer ->
                positionArrayBuffer.forEach(byteBuffer::putFloat)
                byteBuffer.position(0)
            }

        val colorArrayBuffer = rings.map { it.second }.flatten()
        val colorByteBuffer =
            ByteBuffer.allocate(colorArrayBuffer.size * 4).also { byteBuffer ->
                colorArrayBuffer.forEach(byteBuffer::putFloat)
                byteBuffer.position(0)
            }

        positionBuffer = GLHelper.initArrayBuffer(positionByteBuffer)
        colorBuffer = GLHelper.initArrayBuffer(colorByteBuffer)

        shaderProgram = GLHelper.initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(
                vertexPosition = GLES31.glGetAttribLocation(shaderProgram, "aVertexPosition"),
                vertexColor = GLES31.glGetAttribLocation(shaderProgram, "aVertexColor")
            )
        uniformLocation =
            UniformLocation(
                projectionMatrix = GLES31.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES31.glGetUniformLocation(shaderProgram, "uModelViewMatrix")
            )
    }

    fun draw(
        projectionMatrix: FloatArray,
        viewMatrix: FloatArray,
        coordinate: Coordinate,
        size: Float
    ) {
        val modelViewMatrix = viewMatrix.copyOf()

        GLES31.glUseProgram(shaderProgram)

        Matrix.rotateM(modelViewMatrix, 0, coordinate.lon, 0f, 1f, 0f)
        Matrix.rotateM(modelViewMatrix, 0, -coordinate.lat, 1f, 0f, 0f)

        Matrix.scaleM(modelViewMatrix, 0, size, size, 1f)
        Matrix.translateM(modelViewMatrix, 0, 0f, 0f, 1.0001f)
        //
        GLES31.glBindBuffer(GLES31.GL_ARRAY_BUFFER, positionBuffer)
        GLES31.glVertexAttribPointer(
            attribLocations.vertexPosition,
            3,
            GLES31.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES31.glEnableVertexAttribArray(attribLocations.vertexPosition)

        GLES31.glBindBuffer(GLES31.GL_ARRAY_BUFFER, colorBuffer)
        GLES31.glVertexAttribPointer(
            attribLocations.vertexColor,
            4,
            GLES31.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES31.glEnableVertexAttribArray(attribLocations.vertexColor)

        GLES31.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES31.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, modelViewMatrix, 0)

        var offset = 0
        for (ringSize in ringSizes) {
            val numVertices = ringSize / 3
            GLES31.glDrawArrays(GLES31.GL_TRIANGLE_FAN, offset, numVertices)
            offset += numVertices
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
            val angle = (i.toDouble() / numEdges.toDouble()) * 2.0 * Math.PI
            val x = offset[0] + radius * cos(angle)
            val y = offset[1] + radius * sin(angle)
            val z = offset[2]
            positions.add(x.toFloat())
            positions.add(y.toFloat())
            positions.add(z)
            colors.addAll(ringColor.toTypedArray())
        }
        return positions.toList() to colors.toList()
    }
}
