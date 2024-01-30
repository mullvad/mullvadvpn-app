package net.mullvad.lib.map.shapes

import android.content.Context
import android.opengl.GLES31
import android.opengl.Matrix
import java.nio.ByteBuffer
import net.mullvad.mullvadvpn.R

class Globe(context: Context) {
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

    data class AttribLocations(val vertexPosition: Int)

    data class UniformLocation(val color: Int, val projectionMatrix: Int, val modelViewMatrix: Int)


    private val landColor: FloatArray = floatArrayOf(0.16f, 0.302f, 0.45f, 1.0f)
    private val oceanColor: FloatArray = floatArrayOf(0.098f, 0.18f, 0.271f, 1.0f)
    private val contourColor: FloatArray = floatArrayOf(0.098f, 0.18f, 0.271f, 1.0f)
    private val landIndices: GLHelper.IndexBufferWithLength
    private val landContour: GLHelper.IndexBufferWithLength
    private val landVertexBuffer: Int

    private val oceanIndices: GLHelper.IndexBufferWithLength
    private val oceanVertexBuffer: Int

    init {
        val landPosStream = context.resources.openRawResource(R.raw.land_positions)
        val landVertByteArray = landPosStream.use { it.readBytes() }
        val landVertByteBuffer = ByteBuffer.wrap(landVertByteArray)
        landVertexBuffer = GLHelper.initArrayBuffer(landVertByteBuffer)

        val landTriangleIndicesStream =
            context.resources.openRawResource(R.raw.land_triangle_indices)
        val landTriangleIndicesByteArray = landTriangleIndicesStream.use { it.readBytes() }
        val landTriangleIndicesBuffer = ByteBuffer.wrap(landTriangleIndicesByteArray)
        landIndices = GLHelper.initIndexBuffer(landTriangleIndicesBuffer)

        val landContourIndicesStream = context.resources.openRawResource(R.raw.land_contour_indices)
        val landContourIndicesByteArray = landContourIndicesStream.use { it.readBytes() }
        val landContourIndicesBuffer = ByteBuffer.wrap(landContourIndicesByteArray)
        landContour = GLHelper.initIndexBuffer(landContourIndicesBuffer)

        val oceanPosStream = context.resources.openRawResource(R.raw.ocean_positions)
        val oceanVertByteArray = oceanPosStream.use { it.readBytes() }
        val oceanVertByteBuffer = ByteBuffer.wrap(oceanVertByteArray)
        oceanVertexBuffer = GLHelper.initArrayBuffer(oceanVertByteBuffer)

        val oceanTriangleIndicesStream = context.resources.openRawResource(R.raw.ocean_indices)
        val oceanTriangleIndicesByteArray = oceanTriangleIndicesStream.use { it.readBytes() }
        val oceanTriangleIndicesBuffer = ByteBuffer.wrap(oceanTriangleIndicesByteArray)
        oceanIndices = GLHelper.initIndexBuffer(oceanTriangleIndicesBuffer)

        // create empty OpenGL ES Program
        shaderProgram = GLHelper.initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(GLES31.glGetAttribLocation(shaderProgram, "aVertexPosition"))
        uniformLocation =
            UniformLocation(
                color = GLES31.glGetUniformLocation(shaderProgram, "uColor"),
                projectionMatrix = GLES31.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES31.glGetUniformLocation(shaderProgram, "uModelViewMatrix")
            )
    }

    fun draw(projectionMatrix: FloatArray, viewMatrix: FloatArray) {
        val globeViewMatrix = viewMatrix.copyOf()

        // Add program to OpenGL ES environment
        GLES31.glUseProgram(shaderProgram)

        // Set thickness of contour lines
        GLES31.glLineWidth(3f)
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landVertexBuffer,
            landContour,
            contourColor,
            GLES31.GL_LINES
        )

        Matrix.scaleM(globeViewMatrix, 0, 0.9999f, 0.9999f, 0.9999f)
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landVertexBuffer,
            landIndices,
            landColor,
            GLES31.GL_TRIANGLES,
        )

        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            oceanVertexBuffer,
            oceanIndices,
            oceanColor,
            GLES31.GL_TRIANGLES
        )
    }

    private fun drawBufferElements(
        projectionMatrix: FloatArray,
        modelViewMatrix: FloatArray,
        positionBuffer: Int,
        indexBuffer: GLHelper.IndexBufferWithLength,
        color: FloatArray,
        mode: Int,
    ) {
        GLES31.glBindBuffer(GLES31.GL_ARRAY_BUFFER, positionBuffer)
        GLES31.glVertexAttribPointer(
            attribLocations.vertexPosition,
            3, // Num components
            GLES31.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES31.glEnableVertexAttribArray(attribLocations.vertexPosition)

        GLES31.glBindBuffer(GLES31.GL_ELEMENT_ARRAY_BUFFER, indexBuffer.indexBuffer)
        GLES31.glUniform4fv(uniformLocation.color, 1, color, 0)
        GLES31.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES31.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, modelViewMatrix, 0)
        GLES31.glDrawElements(mode, indexBuffer.length, GLES31.GL_UNSIGNED_INT, 0)
        GLES31.glDisableVertexAttribArray(attribLocations.vertexPosition)
    }
}
