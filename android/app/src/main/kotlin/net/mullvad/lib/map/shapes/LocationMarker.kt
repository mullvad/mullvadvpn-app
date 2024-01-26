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
    // The red color of the location marken when in the unsecured state
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

    private val shaderProgram: Int
    private val attribLocations: AttribLocations
    private val uniformLocation: UniformLocation

    data class AttribLocations(val vertexPosition: Int, val vertexColor: Int)

    data class UniformLocation(val projectionMatrix: Int, val modelViewMatrix: Int)

    private val positionBuffer: Int
    private val colorBuffer: Int
    private val ringPositionCount: List<Int>

    init {

        val white = floatArrayOf(1.0f, 1.0f, 1.0f)
        val black = floatArrayOf(0.0f, 0.0f, 0.0f)
        val rings =
            listOf(
                circleFanVertices(
                    32,
                    0.5f,
                    floatArrayOf(0.0f, 0.0f, 0.0f),
                    color + 0.4f,
                    color + 0.4f
                ), // Semi-transparent outer
                circleFanVertices(
                    16,
                    0.28f,
                    floatArrayOf(0.0f, -0.05f, 0.00001f),
                    black + 0.55f,
                    black + 0f
                ), // shadow
                circleFanVertices(
                    32,
                    0.185f,
                    floatArrayOf(0.0f, 0.0f, 0.00002f),
                    white + 1f,
                    white + 1f
                ), // white ring
                circleFanVertices(
                    32,
                    0.15f,
                    floatArrayOf(0.0f, 0.0f, 0.00003f),
                    color + 1f,
                    color + 1f
                ) // Center colored circle
            )

        val positionArrayBuffer = rings.map { it.first.toList() }.flatten()
        val positionByteBuffer = ByteBuffer.allocate(positionArrayBuffer.size * 4)
        positionArrayBuffer.forEach { positionByteBuffer.putFloat(it) }

        val colorArrayBuffer = rings.map { it.second.toList() }.flatten()
        val colorByteBuffer = ByteBuffer.allocate(colorArrayBuffer.size * 4)
        colorArrayBuffer.forEach { colorByteBuffer.putFloat(it) }

        ringPositionCount = rings.map { it.first.size }

        positionBuffer = GLHelper.initArrayBuffer(ByteBuffer.wrap(positionByteBuffer.array()))
        colorBuffer = GLHelper.initArrayBuffer(ByteBuffer.wrap(colorByteBuffer.array()))

        shaderProgram = GLHelper.initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(
                GLES31.glGetAttribLocation(shaderProgram, "aVertexPosition"),
                GLES31.glGetAttribLocation(shaderProgram, "aVertexColor")
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
        val modelViewMatrix = viewMatrix.clone()

        GLES31.glUseProgram(shaderProgram)

        val (theta, phi) = coordinate2thetaphi(coordinate)
        Matrix.rotateM(modelViewMatrix, 0, theta, 0f, 1f, 0f)
        Matrix.rotateM(modelViewMatrix, 0, phi, 1f, 0f, 0f)
        Matrix.scaleM(modelViewMatrix, 0, size, size, 1f)
        Matrix.translateM(modelViewMatrix, 0, 0f, 0f, 1.0001f)

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
        for (element in ringPositionCount) {
            val numVertices = element / 3
            GLES31.glDrawArrays(GLES31.GL_TRIANGLE_FAN, offset, numVertices)
            offset += numVertices
        }
//        GLES31.glDisableVertexAttribArray(attribLocations.vertexPosition)
//        GLES31.glDisableVertexAttribArray(attribLocations.vertexColor)
    }

    private fun coordinate2thetaphi(coordinate: Coordinate): Pair<Float, Float> {
        val phi = coordinate.lat * (Math.PI / 180)
        val theta = coordinate.lon * (Math.PI / 180)
        return theta.toFloat() to phi.toFloat()
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
    ): Pair<FloatArray, FloatArray> {
        val positions = mutableListOf(*offset.toTypedArray())
        val colors = mutableListOf(*centerColor.toTypedArray())

        for (i in 0..numEdges) {
            val angle = (i / numEdges) * 2 * Math.PI
            val x = offset[0] + radius * cos(angle).toFloat()
            val y = offset[1] + radius * sin(angle).toFloat()
            val z = offset[2]
            positions.add(x)
            positions.add(y)
            positions.add(z)
            colors.addAll(ringColor.toTypedArray())
        }
        return positions.toFloatArray() to colors.toFloatArray()
    }
}
