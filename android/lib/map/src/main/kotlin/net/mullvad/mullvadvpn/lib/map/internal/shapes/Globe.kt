package net.mullvad.mullvadvpn.lib.map.internal.shapes

import android.content.res.Resources
import android.opengl.GLES20
import android.opengl.Matrix
import androidx.compose.ui.graphics.Color
import java.nio.ByteBuffer
import net.mullvad.mullvadvpn.lib.map.internal.IndexBufferWithLength
import net.mullvad.mullvadvpn.lib.map.internal.VERTEX_COMPONENT_SIZE
import net.mullvad.mullvadvpn.lib.map.internal.initArrayBuffer
import net.mullvad.mullvadvpn.lib.map.internal.initIndexBuffer
import net.mullvad.mullvadvpn.lib.map.internal.initShaderProgram
import net.mullvad.mullvadvpn.lib.map.internal.toFloatArray
import net.mullvad.mullvadvpn.lib.map.R

data class GlobeColors(
    val landColor: Color,
    val oceanColor: Color,
    val contourColor: Color,
) {
    val landColorArray = landColor.toFloatArray()
    val oceanColorArray = oceanColor.toFloatArray()
    val contourColorArray = contourColor.toFloatArray()
}

class Globe(resources: Resources) {
    private val vertexShaderCode =
        """
    attribute vec3 aVertexPosition;
    uniform vec4 uColor;
    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;
    varying lowp vec4 vColor;
    void main(void) {
      gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aVertexPosition, 1.0);
      vColor = uColor;
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

    private data class AttribLocations(val vertexPosition: Int)
    private data class UniformLocation(val color: Int, val projectionMatrix: Int, val modelViewMatrix: Int)

    private val landIndices: IndexBufferWithLength
    private val landContour: IndexBufferWithLength
    private val landVertexBuffer: Int

    private val oceanIndices: IndexBufferWithLength
    private val oceanVertexBuffer: Int

    init {
        val landPosStream = resources.openRawResource(R.raw.land_positions)
        val landVertByteArray = landPosStream.use { it.readBytes() }
        val landVertByteBuffer = ByteBuffer.wrap(landVertByteArray)
        landVertexBuffer = initArrayBuffer(landVertByteBuffer)

        val landTriangleIndicesStream =
            resources.openRawResource(R.raw.land_triangle_indices)
        val landTriangleIndicesByteArray = landTriangleIndicesStream.use { it.readBytes() }
        val landTriangleIndicesBuffer = ByteBuffer.wrap(landTriangleIndicesByteArray)
        landIndices = initIndexBuffer(landTriangleIndicesBuffer)

        val landContourIndicesStream = resources.openRawResource(R.raw.land_contour_indices)
        val landContourIndicesByteArray = landContourIndicesStream.use { it.readBytes() }
        val landContourIndicesBuffer = ByteBuffer.wrap(landContourIndicesByteArray)
        landContour = initIndexBuffer(landContourIndicesBuffer)

        val oceanPosStream = resources.openRawResource(R.raw.ocean_positions)
        val oceanVertByteArray = oceanPosStream.use { it.readBytes() }
        val oceanVertByteBuffer = ByteBuffer.wrap(oceanVertByteArray)
        oceanVertexBuffer = initArrayBuffer(oceanVertByteBuffer)

        val oceanTriangleIndicesStream = resources.openRawResource(R.raw.ocean_indices)
        val oceanTriangleIndicesByteArray = oceanTriangleIndicesStream.use { it.readBytes() }
        val oceanTriangleIndicesBuffer = ByteBuffer.wrap(oceanTriangleIndicesByteArray)
        oceanIndices = initIndexBuffer(oceanTriangleIndicesBuffer)

        // create empty OpenGL ES Program
        shaderProgram = initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(GLES20.glGetAttribLocation(shaderProgram, "aVertexPosition"))
        uniformLocation =
            UniformLocation(
                color = GLES20.glGetUniformLocation(shaderProgram, "uColor"),
                projectionMatrix = GLES20.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES20.glGetUniformLocation(shaderProgram, "uModelViewMatrix")
            )
    }

    fun draw(
        projectionMatrix: FloatArray,
        viewMatrix: FloatArray,
        colors: GlobeColors,
        contourWidth: Float = 3f
    ) {
        val globeViewMatrix = viewMatrix.copyOf()

        // Add program to OpenGL ES environment
        GLES20.glUseProgram(shaderProgram)

        // Set thickness of contour lines
        GLES20.glLineWidth(contourWidth)
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landVertexBuffer,
            landContour,
            colors.contourColorArray,
            GLES20.GL_LINES
        )

        Matrix.scaleM(
            globeViewMatrix,
            0,
            LAND_OCEAN_SCALE_FACTOR,
            LAND_OCEAN_SCALE_FACTOR,
            LAND_OCEAN_SCALE_FACTOR
        )
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landVertexBuffer,
            landIndices,
            colors.landColorArray,
            GLES20.GL_TRIANGLES,
        )

        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            oceanVertexBuffer,
            oceanIndices,
            colors.oceanColorArray,
            GLES20.GL_TRIANGLES
        )
    }

    private fun drawBufferElements(
        projectionMatrix: FloatArray,
        modelViewMatrix: FloatArray,
        positionBuffer: Int,
        indexBuffer: IndexBufferWithLength,
        color: FloatArray,
        mode: Int,
    ) {
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

        GLES20.glBindBuffer(GLES20.GL_ELEMENT_ARRAY_BUFFER, indexBuffer.indexBuffer)
        GLES20.glUniform4fv(uniformLocation.color, 1, color, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, modelViewMatrix, 0)
        GLES20.glDrawElements(mode, indexBuffer.length, GLES20.GL_UNSIGNED_INT, 0)
        GLES20.glDisableVertexAttribArray(attribLocations.vertexPosition)
    }

    companion object {
        private const val LAND_OCEAN_SCALE_FACTOR = 0.999f
    }
}
